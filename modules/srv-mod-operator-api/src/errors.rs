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
        #[derive(Debug, Clone, PartialEq, Eq)]
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

#[cfg(test)]
mod tests {
    use axum::{http::StatusCode, response::Response, Json};

    use super::*;

    #[derive(serde::Deserialize)]
    struct Err {
        error: String,
    }

    #[derive(serde::Deserialize)]
    struct TerminalEmulatorErr {
        session_id: String,
        command:    String,
        response:   String,
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_api_server_error_into_response() {
        // Test each variant of the ApiServerError enum
        let test_cases = vec![
            (
                ApiServerError::WrongCredentials,
                StatusCode::UNAUTHORIZED,
                "Invalid credentials",
            ),
            (
                ApiServerError::MissingCredentials,
                StatusCode::BAD_REQUEST,
                "Username or password missing",
            ),
            (
                ApiServerError::TokenCreation,
                StatusCode::INTERNAL_SERVER_ERROR,
                "Something went wrong while creating the token",
            ),
            (
                ApiServerError::InvalidToken,
                StatusCode::BAD_REQUEST,
                "Invalid token",
            ),
            (
                ApiServerError::InternalServerError,
                StatusCode::INTERNAL_SERVER_ERROR,
                "Something went wrong",
            ),
        ];

        for (error_variant, expected_status, expected_message) in test_cases {
            let response: Response = error_variant.into_response();

            // Check the status code
            assert_eq!(response.status(), expected_status);

            // Check the response body
            let body: Err = serde_json::from_slice(
                axum::body::to_bytes(response.into_body(), usize::MAX)
                    .await
                    .unwrap()
                    .iter()
                    .as_slice(),
            )
            .unwrap();

            assert_eq!(body.error, expected_message);
        }
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_make_terminal_emulator_error() {
        let session_id = "session_123";
        let command = "clear";
        let error = "Some error occurred";

        let response = ApiServerError::make_terminal_emulator_error(session_id, command, error);

        // Check the status code
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        // Check the response body
        let body: TerminalEmulatorErr = serde_json::from_slice(
            axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap()
                .iter()
                .as_slice(),
        )
        .unwrap();

        assert_eq!(body.session_id, session_id);
        assert_eq!(body.command, command);
        assert_eq!(body.response, error);
    }
}
