use std::iter::once;
use std::sync::Arc;
use std::time::Duration;

use axum::extract::MatchedPath;
use axum::http::header::AUTHORIZATION;
use axum::http::Request;
use axum::response::Response;
use axum::Router;
use axum::routing::{get, post};
use tokio::{select, signal};
use tokio_util::bytes::Bytes;
use tokio_util::sync::CancellationToken;
use tower_http::body::UnsyncBoxBody;
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::compression::CompressionLayer;
use tower_http::normalize_path::NormalizePathLayer;
use tower_http::sensitive_headers::SetSensitiveHeadersLayer;
use tower_http::trace::TraceLayer;
use tower_http::validate_request::ValidateRequestHeaderLayer;
use tracing::{debug, error, info, info_span, instrument, Level, span, Span, warn};

use rs2_utils::duration_extension::DurationExt;

use crate::async_main::api_server::jwt_keys::{API_SERVER_JWT_KEYS, Keys};
use crate::config::config::SharedConfig;
use crate::database::Pool;

mod claims;
mod jwt_keys;
mod state;

pub async fn start(
	config: SharedConfig,
	cancellation_token: CancellationToken,
	pool: Pool,
) -> anyhow::Result<()> {
	let readonly_config = config.read().await;

	API_SERVER_JWT_KEYS.get_or_init(|| {
		Keys::new(readonly_config.jwt.secret.as_bytes())
	});
	debug!(readonly_config.jwt.secret, "JWT keys initialized successfully!");

	let shared_state = Arc::new(state::ApiServerState {
		config: config.clone(),
		db_pool: pool
	});

	let app = Router::new()
		.route("/protected", get(|| async {
			tokio::time::sleep(Duration::from_secs(2)).await;
			"Protected data"
		}))
		.with_state(shared_state)
		.layer((
			// add log tracing
			TraceLayer::new_for_http()
				.make_span_with(|request: &Request<_>| {
					let matched_path = if let Some(path) = request
						.extensions()
						.get::<MatchedPath>()
						.map(MatchedPath::as_str) {
						path
					} else {
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
				.on_request(|_request: &Request<_>, _span: &Span| {
					info!("Started processing request")
				})
				.on_response(|response: &Response<_>, latency: Duration, span: &Span| {
					let status = response.status();

					span.record("latency", humantime::format_duration(latency.round()).to_string());
					span.record("status", format!("{} {}", status.as_str(), status.canonical_reason().unwrap_or("")));

					info!("Request processed")
				}),
			// catch panics (if any) most likely from external crates, just to avoid crashing the server
			CatchPanicLayer::new(),
			// add compression support
			CompressionLayer::new(),
			// normalize paths before routing trimming trailing slashes
			NormalizePathLayer::trim_trailing_slash(),
			// validate request headers for content type accepting only json and form data (subtypes are allowed)
			ValidateRequestHeaderLayer::accept("application/json"),
			ValidateRequestHeaderLayer::accept("multipart/form-data"),
			// set sensitive headers to be removed from logs
			SetSensitiveHeadersLayer::new(once(AUTHORIZATION))
		));

	// let mut make_service = app.into_make_service_with_connect_info::<SocketAddr>();

	// start listening on the provided address
	let listener = tokio::net::TcpListener::bind(
		format!("{}:{}", readonly_config.api_server.host, readonly_config.api_server.port)
	).await?;
	info!(address = %listener.local_addr().unwrap(), "Api server listening");

	axum::serve(listener, app)
		.with_graceful_shutdown(handle_graceful_shutdown(cancellation_token))
		.await
		.unwrap();

	Ok(())
}

async fn handle_graceful_shutdown(cancellation_token: CancellationToken) {
	cancellation_token.cancelled().await;
	warn!("Api server shutting down");
}