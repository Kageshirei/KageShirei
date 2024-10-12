use tokio_util::sync::CancellationToken;
use tracing::error;

use srv_mod_config::SharedConfig;
use srv_mod_entity::sea_orm::DatabaseConnection;
use srv_mod_operator_api::start;

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
pub fn spawn(config: SharedConfig, cancellation_token: CancellationToken, db: DatabaseConnection) -> tokio::task::JoinHandle<()> {
    let api_server_task = start(
        config.clone(),
        cancellation_token.clone(),
        db,
    );

    let api_server_thread = tokio::spawn(async move {
        let exit_status = api_server_task.await;

        if exit_status.is_err() {
            error!("Api server died with error: {}", exit_status.err().unwrap())
        }
    });

    api_server_thread
}