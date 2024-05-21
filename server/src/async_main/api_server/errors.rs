use axum::{
    http::StatusCode,
    Json,
    response::{IntoResponse, Response},
};
use serde_json::json;

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
}
