//! The shared state for the API server

use std::sync::Arc;

use srv_mod_config::{sse::common_server_state::SseEvent, SharedConfig};
use srv_mod_entity::sea_orm::DatabaseConnection;

/// The shared state for the API server
pub type ApiServerSharedState = Arc<ApiServerState>;

/// The shared state for the API server
#[allow(
    dead_code,
    reason = "The config field is not directly used into this module but may be in other depending on the operator api"
)]
#[derive(Debug, Clone)]
pub struct ApiServerState {
    /// The shared configuration
    pub config:           SharedConfig,
    /// The database connection pool
    pub db_pool:          DatabaseConnection,
    /// The broadcast sender for the API server
    pub broadcast_sender: tokio::sync::broadcast::Sender<SseEvent>,
}
