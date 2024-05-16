use std::sync::Arc;

use axum::Router;
use axum::routing::{get, post};
use tower_http::trace::TraceLayer;
use tracing::{debug, info, info_span, instrument, Level, span};

use crate::async_main::api_server::jwt_keys::{API_SERVER_JWT_KEYS, Keys};
use crate::config::config::SharedConfig;

mod claims;
mod jwt_keys;
mod state;

pub async fn start(config: SharedConfig) -> anyhow::Result<()> {
	let readonly_config = config.read().await;

	API_SERVER_JWT_KEYS.get_or_init(|| {
		Keys::new(readonly_config.jwt.secret.as_bytes())
	});
	debug!(readonly_config.jwt.secret, "JWT keys initialized successfully!");

	let shared_state = Arc::new(state::ApiServerState {
		config: config.clone(),
	});

	let app = Router::new()
		.route("/protected", get(|| async {
			"Protected data"
		}))
		// .route("/authorize", post(authorize))
		.with_state(shared_state)
		.layer(TraceLayer::new_for_http());

	let listener = tokio::net::TcpListener::bind(
		format!("{}:{}", readonly_config.api_server.host, readonly_config.api_server.port)
	).await?;

	info!(address = %listener.local_addr().unwrap(), "Api server listening");
	axum::serve(listener, app).await.unwrap();

	Ok(())
}