use std::convert::Infallible;

use axum::{debug_handler, Router};
use axum::extract::State;
use axum::response::Sse;
use axum::response::sse::Event;
use axum::routing::get;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::BroadcastStream;
use tracing::instrument;

use crate::claims::JwtClaims;
use crate::state::ApiServerSharedState;

/// The handler for the public authentication route
#[debug_handler]
#[instrument(name = "GET /sse", skip(state))]
async fn get_handler(
	State(state): State<ApiServerSharedState>,
	_jwt_claims: JwtClaims,
) -> Sse<impl futures_util::Stream<Item = Result<Event, Infallible>>> {
	let rx = state.broadcast_sender.subscribe();
	let stream = BroadcastStream::new(rx).filter_map(|result| {
		match result {
			Ok(event) => Some(Ok(
				Event::default()
					.data(event.data)
					.event(event.event.to_string())
					.id(event.id.unwrap_or_default())
			)),
			Err(_) => None, // Ignore errors
		}
	});

	Sse::new(stream)
}

/// Creates the public authentication routes
pub fn route(state: ApiServerSharedState) -> Router<ApiServerSharedState> {
	Router::new()
		.route("/sse", get(get_handler))
		.with_state(state)
}