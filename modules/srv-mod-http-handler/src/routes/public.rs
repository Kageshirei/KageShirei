use axum::Router;

use crate::state::HttpHandlerSharedState;

mod checkin;

/// Create the public routes for the API server
pub fn make_routes(state: HttpHandlerSharedState) -> Router<HttpHandlerSharedState> {
	Router::new()
		.merge(checkin::route(state.clone()))
		.with_state(state)
}
