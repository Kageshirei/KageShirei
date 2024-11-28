//! Parse the response from the base handler and return a response that can be sent to the client

use axum::{
    body::Body,
    http::{Response, StatusCode},
    response::IntoResponse as _,
};
use kageshirei_communication_protocol::Format as _;
use kageshirei_format_json::FormatJson;
use srv_mod_config::handlers::Format as FormatConfig;
use srv_mod_handler_base::response::BaseHandlerResponse;
use tracing::warn;

/// Parse the response from the base handler and return a response that can be sent to the client
///
/// # Arguments
///
/// * `response` - The response from the base handler
///
/// # Returns
///
/// A response that can be sent to the client
pub fn parse_base_handler_response(response: Result<BaseHandlerResponse, BaseHandlerResponse>) -> Response<Body> {
    let response = response.unwrap_or_else(|e| e);

    let body = if let Some(formatter) = response.formatter {
        match formatter {
            FormatConfig::Json => FormatJson.write::<_, &str>(response.body, None),
        }
    }
    else {
        Ok(response.body)
    };

    if body.is_err() {
        warn!("HTTP handler failed to format the response");
        return (
            StatusCode::from_u16(response.status.get()).unwrap_or(StatusCode::OK),
            b"",
        )
            .into_response();
    }
    let body = body.unwrap();

    (
        StatusCode::from_u16(response.status.get()).unwrap_or(StatusCode::OK),
        body,
    )
        .into_response()
}
