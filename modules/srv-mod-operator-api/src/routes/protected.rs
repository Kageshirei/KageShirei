use axum::Router;

use crate::state::ApiServerSharedState;

mod refresh_token;

pub fn make_routes(state: ApiServerSharedState) -> Router<ApiServerSharedState> {
	Router::new()
		.merge(refresh_token::route(state.clone()))
		.with_state(state)
}
