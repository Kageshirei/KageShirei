use std::sync::Arc;

use srv_mod_config::handlers::HandlerConfig;
use srv_mod_entity::sea_orm::DatabaseConnection;
use tokio_util::sync::CancellationToken;
use tracing::error;

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
pub fn spawn(
    config: Arc<HandlerConfig>,
    cancellation_token: CancellationToken,
    db: DatabaseConnection,
) -> tokio::task::JoinHandle<()> {
    let api_server_task = srv_mod_handler_http::start(config.clone(), cancellation_token.clone(), db);

    let api_server_thread = tokio::spawn(async move {
        let exit_status = api_server_task.await;

        if exit_status.is_err() {
            error!("Api server died with error: {}", exit_status.err().unwrap())
        }
    });

    api_server_thread
}
