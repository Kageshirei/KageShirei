use axum::{body::Body, http::HeaderMap, response::Response};
use bytes::Bytes;
use srv_mod_config::handlers::EncryptionScheme;

use crate::state::HandlerSharedState;

pub(crate) mod callback_handlers;
mod decode_or_fail;
mod decrypt_asymmetric_or_fail;
mod decrypt_symmetric_or_fail;
mod process_body;
pub mod state;

/// Handle the extraction of pending commands for a given id marking all the retrieved ones as running
pub async fn handle_command_retrieval() {
    todo!("Retrieve commands for the provided id");
}

/// Handle the result of the execution of a given command marking it as completed or failed depending on the result
pub async fn handle_command_result(
    state: HandlerSharedState,
    mut body: Bytes,
    headers: HeaderMap,
    cmd_request_id: String,
) -> Response<Body> {
    // Decode the body if an encoder is provided
    if state.config.security.encoder.is_some() {
        let encoder = state.config.security.encoder.as_ref().unwrap();
        body = decode_or_fail::decode_or_fail_response(encoder, body).unwrap();
    }

    // Decrypt the body if an encryption scheme is provided
    body = match state.config.security.encryption_scheme {
        EncryptionScheme::Plain => body,
        EncryptionScheme::Symmetric => {
            decrypt_symmetric_or_fail::decrypt_symmetric_or_fail(state.config.security.algorithm.as_ref(), body)
                .unwrap()
        },
        EncryptionScheme::Asymmetric => {
            decrypt_asymmetric_or_fail::decrypt_asymmetric_or_fail(state.config.security.algorithm.as_ref(), body)
                .unwrap()
        },
    };

    process_body::process_body(state.db_pool.clone(), body, headers, cmd_request_id).await
}
