use log::info;
use srv_mod_config::ReadOnlyConfig;
use srv_mod_entity::sea_orm::{ConnectOptions, Database, DatabaseConnection};
use srv_mod_migration::{Migrator, MigratorTrait};
use tracing::error;

/// Runs the database migrations and returns a connection to the database.
///
/// # Parameters
/// - `database_url` - The URL of the database.
/// - `readonly_config` - The read-only configuration.
///
/// # Returns
/// The database connection.
pub async fn run(database_url: &str, readonly_config: &ReadOnlyConfig) -> Result<DatabaseConnection, String> {
    let mut connect_options = ConnectOptions::new(database_url);
    connect_options.max_connections(readonly_config.database.pool_size as u32);

    info!("Connecting to the database");
    let db = Database::connect(connect_options).await.map_err(|e| {
        error!("Failed to connect to the database: {}", e);
        e.to_string()
    })?;

    info!("Updating database schema");
    Migrator::up(&db, None).await.map_err(|e| {
        error!("Failed to run migrations: {}", e);
        e.to_string()
    })?;

    info!("Database schema updated successfully");

    Ok(db)
}
