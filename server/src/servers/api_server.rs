//! The API server module.

use srv_mod_config::SharedConfig;
use srv_mod_entity::sea_orm::DatabaseConnection;
use srv_mod_operator_api::start;
use tokio_util::sync::CancellationToken;
use tracing::error;

/// Spawns the API server task.
///
/// # Parameters
///
/// - `config` - The shared configuration.
/// - `cancellation_token` - The cancellation token.
/// - `pool` - The database connection pool.
///
/// # Returns
///
/// The join handle for the API server task.
pub fn spawn(
    config: SharedConfig,
    cancellation_token: CancellationToken,
    db: DatabaseConnection,
) -> tokio::task::JoinHandle<()> {
    let api_server_task = start(config, cancellation_token, db);

    tokio::spawn(async move {
        let exit_status = api_server_task.await;

        if exit_status.is_err() {
            error!("Api server died with error: {}", exit_status.err().unwrap())
        }
    })
}
