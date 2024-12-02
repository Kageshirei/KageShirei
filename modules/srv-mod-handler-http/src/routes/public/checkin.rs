//! The public checkin route for the API server

use axum::{
    body::{Body, Bytes},
    extract::State,
    http::HeaderMap,
    response::Response,
    routing::post,
    Router,
};
use srv_mod_handler_base::{handle_command_result, state::HandlerSharedState};
use tracing::instrument;

use crate::parse_base_handler_response::parse_base_handler_response;

/// The handler for the agent checking operation
#[instrument(name = "POST /checkin", skip_all)]
async fn post_handler(State(state): State<HandlerSharedState>, headers: HeaderMap, body: Bytes) -> Response<Body> {
    parse_base_handler_response(handle_command_result(state, body.to_vec(), headers, String::new()).await)
}

/// Creates the public authentication routes
pub fn route(state: HandlerSharedState) -> Router<HandlerSharedState> {
    Router::new()
        .route("/checkin", post(post_handler))
        .with_state(state)
}
