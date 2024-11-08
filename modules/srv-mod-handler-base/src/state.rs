use std::sync::Arc;

use srv_mod_config::handlers::Config;
use srv_mod_entity::sea_orm::DatabaseConnection;

#[expect(clippy::module_name_repetitions, reason = "The name is descriptive")]
pub type HandlerSharedState = Arc<HandlerState>;

/// The shared state for the API server
#[derive(Debug, Clone)]
#[expect(clippy::module_name_repetitions, reason = "The name is descriptive")]
pub struct HandlerState {
    /// The shared configuration
    pub config:  Arc<Config>,
    /// The database connection pool
    pub db_pool: DatabaseConnection,
}
