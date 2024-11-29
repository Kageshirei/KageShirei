//! The error module for the API server

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

/// The error enum for the API server
macro_rules! define_error_enum {
    ($enum_name:ident, $($variant:ident = ($status:expr, $msg:expr)),* $(,)?) => {
        #[derive(Debug)]
        pub enum $enum_name {
            $($variant),*
        }

        impl IntoResponse for $enum_name {
            fn into_response(self) -> Response {
                let (status, error_message) = match self {
                    $(Self::$variant => ($status, $msg)),*
                };

                let body = Json(json!({
                    "error": error_message,
                }));

                (status, body).into_response()
            }
        }
    };
}

define_error_enum! {
    ApiServerError,
    WrongCredentials = (StatusCode::UNAUTHORIZED, "Invalid credentials"),
    MissingCredentials = (StatusCode::BAD_REQUEST, "Username or password missing"),
    TokenCreation = (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong while creating the token"),
    InvalidToken = (StatusCode::BAD_REQUEST, "Invalid token"),
    InternalServerError = (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong"),
    // TerminalEmulatorError = (StatusCode::INTERNAL_SERVER_ERROR, ""), // Created via make_terminal_emulator_error
}

impl ApiServerError {
    /// Create a new `ApiServerError::TerminalEmulatorError` with the given error message
    pub fn make_terminal_emulator_error(session_id: &str, command: &str, error: &str) -> Response {
        let body = Json(json!({
            "session_id": session_id,
            "command": command,
            "response": error,
        }));
        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}
