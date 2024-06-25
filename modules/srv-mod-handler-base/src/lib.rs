use axum::body::Body;
use axum::response::Response;
use bytes::Bytes;

use crate::state::HandlerSharedState;

mod decode_or_fail;
pub mod state;

/// Handle the extraction of pending commands for a given id marking all the retrieved ones as running
pub async fn handle_command_retrieval() {
	todo!("Retrieve commands for the provided id");
}

/// Handle the result of the execution of a given command marking it as completed or failed depending on the result
pub async fn handle_command_result(state: HandlerSharedState, mut body: Bytes) -> Response<Body> {
	// Decode the body if an encoder is provided
	if state.config.security.encoder.is_some() {
		let encoder = state.config.security.encoder.as_ref().unwrap();
		body = decode_or_fail::decode_or_fail_response(encoder, body)?;
	}

	todo!("Execute the provided command");
}
