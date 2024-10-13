use axum::Router;

use crate::state::ApiServerSharedState;

mod logs;
mod notifications;
mod refresh_token;
mod sessions;
mod sse;
mod terminal;

pub fn make_routes(state: ApiServerSharedState) -> Router<ApiServerSharedState> {
    Router::new()
        .merge(refresh_token::route(state.clone()))
        .merge(terminal::route(state.clone()))
        .merge(sse::route(state.clone()))
        .merge(logs::route(state.clone()))
        .merge(notifications::route(state.clone()))
        .merge(sessions::route(state.clone()))
        .with_state(state)
}
