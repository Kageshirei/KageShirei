use std::sync::Arc;

use srv_mod_config::handlers::HandlerConfig;
use srv_mod_database::Pool;

pub type HttpHandlerSharedState = Arc<HttpHandlerState>;

/// The shared state for the API server
#[derive(Debug, Clone)]
pub struct HttpHandlerState {
	/// The shared configuration
	pub config: Arc<HandlerConfig>,
	/// The database connection pool
	pub db_pool: Pool,
}
