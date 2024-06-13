use axum::Router;

use crate::state::ApiServerSharedState;

mod refresh_token;
mod terminal;
mod sse;
mod logs;
mod notifications;

pub fn make_routes(state: ApiServerSharedState) -> Router<ApiServerSharedState> {
	Router::new()
		.merge(refresh_token::route(state.clone()))
		.merge(terminal::route(state.clone()))
		.merge(sse::route(state.clone()))
        .merge(logs::route(state.clone()))
        .merge(notifications::route(state.clone()))
		.with_state(state)
}
