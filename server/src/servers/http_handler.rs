use std::sync::Arc;

use tokio_util::sync::CancellationToken;
use tracing::error;

use srv_mod_config::handlers::HandlerConfig;
use srv_mod_database::Pool;

/// Spawns the API server task.
///
/// # Parameters
///
/// - `config` - The handler configuration.
/// - `cancellation_token` - The cancellation token.
/// - `pool` - The database connection pool.
///
/// # Returns
///
/// The join handle for the API server task.
pub fn spawn(config: Arc<HandlerConfig>, cancellation_token: CancellationToken, pool: Pool) -> tokio::task::JoinHandle<()> {
	let api_server_task = srv_mod_handler_http::start(config.clone(), cancellation_token.clone(), pool.clone());
	let api_server_thread = tokio::spawn(async move {
		let exit_status = api_server_task.await;

		if exit_status.is_err() {
			error!("Api server died with error: {}", exit_status.err().unwrap())
		}
	});

	api_server_thread
}