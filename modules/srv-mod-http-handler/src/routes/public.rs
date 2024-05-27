use axum::Router;

use crate::state::HttpHandlerSharedState;

mod authenticate;

/// Create the public routes for the API server
pub fn make_routes(state: HttpHandlerSharedState) -> Router<HttpHandlerSharedState> {
	Router::new()
		.merge(authenticate::route(state.clone()))
		.with_state(state)
}
