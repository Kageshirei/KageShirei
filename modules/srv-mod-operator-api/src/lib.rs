#![feature(let_chains)]

use std::{iter::once, sync::Arc, time::Duration};

use axum::{
    extract::{DefaultBodyLimit, Host, MatchedPath},
    handler::HandlerWithoutStateExt,
    http::{header::AUTHORIZATION, Method, Request, StatusCode, Uri},
    response::{Redirect, Response},
    routing::post,
    Router,
};
use axum_server::tls_rustls::RustlsConfig;
use jwt_keys::{Keys, API_SERVER_JWT_KEYS};
use rs2_utils::{duration_extension::DurationExt, unrecoverable_error::unrecoverable_error};
use srv_mod_config::SharedConfig;
use srv_mod_database::Pool;
use state::ApiServerSharedState;
use tokio::select;
use tokio_util::sync::CancellationToken;
use tower_http::{
    catch_panic::CatchPanicLayer,
    compression::{predicate::NotForContentType, CompressionLayer, DefaultPredicate, Predicate},
    cors::{Any, Cors, CorsLayer},
    limit::RequestBodyLimitLayer,
    normalize_path::NormalizePathLayer,
    sensitive_headers::SetSensitiveHeadersLayer,
    trace::TraceLayer,
    validate_request::ValidateRequestHeaderLayer,
};
use tracing::{debug, error, info, info_span, warn, Span};

mod claims;
mod errors;
mod jwt_keys;
mod request_body_from_content_type;
mod routes;
mod state;

pub async fn start(config: SharedConfig, cancellation_token: CancellationToken, pool: Pool) -> anyhow::Result<()> {
    let readonly_config = config.read().await;

    // initialize the JWT keys
    API_SERVER_JWT_KEYS.get_or_init(|| Keys::new(readonly_config.jwt.secret.as_bytes()));
    debug!(
        readonly_config.jwt.secret,
        "JWT keys initialized successfully!"
    );

    // create a broadcast channel for the server this is where events will be broadcasted to and retrieved by the sse
    // endpoint
    let (broadcast_sender, _) = tokio::sync::broadcast::channel(128);

    // create a shared state for the server
    let shared_state: ApiServerSharedState = Arc::new(state::ApiServerState {
        config: config.clone(),
        db_pool: pool,
        broadcast_sender,
    });

    // init the router
    let app = Router::new()
        .merge(routes::public::make_routes(shared_state.clone()))
        .merge(routes::protected::make_routes(shared_state.clone()))
        .with_state(shared_state)
        .layer((
            // add log tracing
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<_>| {
                    let matched_path = if let Some(path) = request
                        .extensions()
                        .get::<MatchedPath>()
                        .map(MatchedPath::as_str)
                    {
                        path
                    }
                    else {
                        "None"
                    };

                    info_span!(
                        "http_request",
                        method = %request.method(),
                        path = %request.uri().path(),
                        matched_path,
                        latency = tracing::field::Empty,
                        status = tracing::field::Empty,
                    )
                })
                .on_request(|_request: &Request<_>, _span: &Span| info!("Started processing request"))
                .on_response(|response: &Response<_>, latency: Duration, span: &Span| {
                    let status = response.status();

                    span.record(
                        "latency",
                        humantime::format_duration(latency.round()).to_string(),
                    );
                    span.record(
                        "status",
                        format!(
                            "{} {}",
                            status.as_str(),
                            status.canonical_reason().unwrap_or("")
                        ),
                    );

                    info!("Request processed")
                }),
            // catch panics (if any) most likely from external crates, just to avoid crashing the server
            CatchPanicLayer::new(),
            // add compression support for all responses except for text/event-stream
            CompressionLayer::new()
                .compress_when(DefaultPredicate::new().and(NotForContentType::new("text/event-stream"))),
            // normalize paths before routing trimming trailing slashes
            NormalizePathLayer::trim_trailing_slash(),
            // limit request body size to 50mb
            DefaultBodyLimit::disable(),
            RequestBodyLimitLayer::new(0x3200000 /* 50mb = (50 * 1024 * 1024) */),
            // set sensitive headers to be removed from logs
            SetSensitiveHeadersLayer::new(once(AUTHORIZATION)),
            CorsLayer::very_permissive(),
        ));

    if let Some(tls_config) = readonly_config.api_server.tls.clone() &&
        tls_config.enabled
    {
        info!("Starting API server with TLS support");
        warn!("Plain http server will be automatically redirected to https");

        tokio::spawn(redirect_http_to_https(
            readonly_config.api_server.host.clone(),
            readonly_config.api_server.port,
            tls_config.port,
            cancellation_token.clone(),
        ));

        let rustls_config = RustlsConfig::from_pem_file(tls_config.cert, tls_config.key).await?;

        let listener = tokio::net::TcpListener::bind(format!(
            "{}:{}",
            if let Some(tls_host) = tls_config.host {
                tls_host
            }
            else {
                readonly_config.api_server.host.clone()
            },
            tls_config.port
        ))
        .await;

        let listener = unwrap_listener_or_fail(
            readonly_config.api_server.host.clone(),
            tls_config.port,
            listener,
        );

        info!(address = %listener.local_addr().unwrap(), "HTTPS api server listening");

        select! {
            _ = axum_server::from_tcp_rustls(listener.into_std()?, rustls_config).serve(app.into_make_service()) => {},
            _ = handle_graceful_shutdown("HTTPS", cancellation_token) => {},
        }

        return Ok(());
    }

    // start listening on the provided address
    let listener = tokio::net::TcpListener::bind(format!(
        "{}:{}",
        readonly_config.api_server.host, readonly_config.api_server.port
    ))
    .await;

    let listener = unwrap_listener_or_fail(
        readonly_config.api_server.host.clone(),
        readonly_config.api_server.port,
        listener,
    );

    info!(address = %listener.local_addr().unwrap(), "Api server listening");

    // start serving requests
    axum::serve(listener, app)
        .with_graceful_shutdown(handle_graceful_shutdown("HTTP", cancellation_token))
        .await?;

    Ok(())
}

/// Unwraps the listener or fails with an unrecoverable error
fn unwrap_listener_or_fail(
    host: String,
    port: u16,
    listener: std::io::Result<tokio::net::TcpListener>,
) -> tokio::net::TcpListener {
    if listener.is_err() {
        error!("Cannot bind to {} at port {}", host, port);
        unrecoverable_error().unwrap();
    }

    listener.unwrap()
}

/// Handle the shutdown signal gracefully closing all connections and waiting for all requests to complete
async fn handle_graceful_shutdown(context: &str, cancellation_token: CancellationToken) {
    cancellation_token.cancelled().await;
    warn!("{context} api server shutting down");
}

/// Redirects all http requests to https
async fn redirect_http_to_https(
    host: String,
    http_port: u16,
    https_port: u16,
    cancellation_token: CancellationToken,
) -> anyhow::Result<()> {
    let redirect = move |Host(host): Host, uri: Uri| {
        async move {
            match make_https(host, uri.clone(), http_port, https_port) {
                Ok(uri) => Ok(Redirect::permanent(&uri.to_string())),
                Err(error) => {
                    warn!(%error, %uri, "Failed to convert URI to HTTPS");
                    Err(StatusCode::BAD_REQUEST)
                },
            }
        }
    };

    let app = Router::new()
        .route(
            "/*key",
            post(redirect).get(redirect).put(redirect).delete(redirect),
        )
        .layer((
            // normalize paths before routing trimming trailing slashes
            NormalizePathLayer::trim_trailing_slash(),
            // limit request body size to 50mb
            DefaultBodyLimit::disable(),
            CorsLayer::very_permissive(),
        ));

    // start listening on the provided address
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", host, http_port)).await;

    let listener = unwrap_listener_or_fail(host.clone(), http_port, listener);

    info!(address = %listener.local_addr().unwrap(), "HTTP api server listening");

    axum::serve(listener, app)
        .with_graceful_shutdown(handle_graceful_shutdown("HTTP", cancellation_token))
        .await
        .unwrap();

    Ok(())
}

/// Converts a http uri to https
fn make_https(host: String, uri: Uri, http_port: u16, https_port: u16) -> anyhow::Result<Uri> {
    let mut parts = uri.into_parts();

    parts.scheme = Some(axum::http::uri::Scheme::HTTPS);

    if parts.path_and_query.is_none() {
        parts.path_and_query = Some("/".parse().unwrap());
    }

    let https_host = host.replace(&http_port.to_string(), &https_port.to_string());
    parts.authority = Some(https_host.parse()?);

    Ok(Uri::from_parts(parts)?)
}
