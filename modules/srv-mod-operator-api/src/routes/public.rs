use axum::Router;

use crate::state::ApiServerSharedState;

pub mod authenticate;

/// Create the public routes for the API server
pub fn make_routes(state: ApiServerSharedState) -> Router<ApiServerSharedState> {
	Router::new()
		.merge(authenticate::route(state.clone()))
		.with_state(state)
}
