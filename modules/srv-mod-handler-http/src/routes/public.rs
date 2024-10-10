use axum::Router;

use srv_mod_handler_base::state::HandlerSharedState;

mod checkin;
mod heuristic_handler;

/// Create the public routes for the API server
pub fn make_routes(state: HandlerSharedState) -> Router<HandlerSharedState> {
	Router::new()
		.merge(checkin::route(state.clone()))
		.merge(heuristic_handler::route(state.clone()))
		.with_state(state)
}
