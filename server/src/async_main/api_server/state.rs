use std::sync::Arc;

use crate::config::config::SharedConfig;

pub type ApiServerSharedState = Arc<ApiServerState>;

/// The shared state for the API server
#[derive(Debug)]
pub struct ApiServerState {
	/// The shared configuration
	pub config: SharedConfig,

}