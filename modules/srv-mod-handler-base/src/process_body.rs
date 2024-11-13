//! This module contains the logic to process the body of the request, this is a protocol agnostic
//! module that will try to match the magic numbers of the body to the appropriate format and then
//! handle the command by executing it and returning the response if any

use std::num::NonZeroU16;

use axum::http::{HeaderMap, StatusCode};
use kageshirei_communication_protocol::{
    communication::{AgentCommands, BasicAgentResponse, Checkin as CheckinStruct},
    magic_numbers,
    Format,
};
use kageshirei_format_json::FormatJson;
use kageshirei_utils::duration_extension::DurationExt as _;
use serde::Deserialize;
use srv_mod_config::handlers;
use srv_mod_entity::sea_orm::DatabaseConnection;
use tracing::{instrument, warn};

use crate::{callback_handlers, error, response::BaseHandlerResponse};

/// Ensure that the body is not empty by returning a response if it is
#[instrument(skip_all)]
pub fn ensure_is_not_empty(body: Vec<u8>) -> Option<BaseHandlerResponse> {
    if body.is_empty() {
        warn!("Empty checking request received, request refused");
        warn!("Internal status code: {}", StatusCode::BAD_REQUEST);

        // always return OK to avoid leaking information
        return Some(BaseHandlerResponse {
            status:    NonZeroU16::try_from(StatusCode::OK.as_u16()).unwrap_or(NonZeroU16::new(200).unwrap()),
            body:      vec![],
            formatter: None,
        });
    }

    None
}

/// Match the magic numbers of the body to the appropriate protocol
#[instrument(skip_all)]
fn match_magic_numbers(body: &[u8]) -> Result<handlers::Format, String> {
    if body.len() >= magic_numbers::JSON.len() &&
        body.get(.. magic_numbers::JSON.len())
            .eq(&Some(&magic_numbers::JSON))
    {
        return Ok(handlers::Format::Json);
    }

    Err("Unknown format".to_owned())
}

/// Handle the command by executing it and returning the response if any
#[instrument(skip(raw_body, format))]
async fn handle_command<F>(
    db_pool: DatabaseConnection,
    basic_response: BasicAgentResponse,
    format: F,
    raw_body: Vec<u8>,
    headers: HeaderMap,
    cmd_request_id: String,
) -> Result<Vec<u8>, error::CommandHandling>
where
    F: Format + Send,
{
    match AgentCommands::from(basic_response.metadata.command_id) {
        AgentCommands::Terminate => callback_handlers::terminate::handle_terminate(db_pool, cmd_request_id).await,
        AgentCommands::Checkin => {
            let checkin = format
                .read::<CheckinStruct, &str>(raw_body.as_slice(), None)
                .map_err(error::CommandHandling::Format)?;
            callback_handlers::checkin::process_body::handle_checkin(checkin, db_pool, format).await
        },
        AgentCommands::INVALID => {
            // if the command is not recognized, return an empty response
            warn!("Unknown command, request refused");
            warn!("Internal status code: {}", StatusCode::BAD_REQUEST);

            Ok(Vec::<u8>::new())
        },
    }
}

/// Process the body by matching the protocol and handling the command
#[instrument(skip_all)]
pub async fn process_body(
    db_pool: DatabaseConnection,
    body: Vec<u8>,
    headers: HeaderMap,
    cmd_request_id: String,
) -> BaseHandlerResponse {
    // ensure that the body is not empty or return a response
    let is_empty = ensure_is_not_empty(body.clone());
    if is_empty.is_some() {
        return is_empty.unwrap();
    }

    match match_magic_numbers(body.as_slice()) {
        Ok(format) => {
            match format {
                handlers::Format::Json => {
                    let data = process_json(body.as_slice()).unwrap();
                    let response = handle_command(db_pool, data, FormatJson, body, headers, cmd_request_id)
                        .await
                        .unwrap_or(Vec::<u8>::new());

                    BaseHandlerResponse {
                        status:    NonZeroU16::try_from(StatusCode::OK.as_u16())
                            .unwrap_or(NonZeroU16::new(200).unwrap()),
                        body:      response,
                        formatter: Some(handlers::Format::Json),
                    }
                },
            }
        },
        Err(_) => {
            // if no protocol matches, drop the request
            warn!(
                "Unknown format, request refused. Internal status code: {}",
                StatusCode::BAD_REQUEST
            );

            // always return OK to avoid leaking information
            BaseHandlerResponse {
                status:    NonZeroU16::try_from(StatusCode::OK.as_u16()).unwrap_or(NonZeroU16::new(200).unwrap()),
                body:      vec![],
                formatter: None,
            }
        },
    }
}

/// Process the body as a JSON protocol
#[instrument(name = "JSON protocol", skip(body), fields(latency = tracing::field::Empty))]
fn process_json<T>(body: &[u8]) -> Result<T, kageshirei_communication_protocol::error::Format>
where
    T: for<'a> Deserialize<'a>,
{
    let now = std::time::Instant::now();

    // try to read the body as a checkin struct
    let result = FormatJson.read::<T, &str>(body, None);

    // record the latency of the operation
    let latency = now.elapsed();
    tracing::Span::current().record(
        "latency",
        humantime::format_duration(latency.round()).to_string(),
    );

    result
}

#[cfg(test)]
mod test {
    use axum::http::StatusCode;
    use kageshirei_communication_protocol::magic_numbers;
    use serde::Serialize;
    use srv_mod_config::handlers;

    use super::*;

    #[test]
    fn test_ensure_is_not_empty() {
        let empty_body = Vec::<u8>::new();
        let response = ensure_is_not_empty(empty_body);
        assert_eq!(response.is_some(), true);

        let unwrapped_response = response.unwrap();
        assert_eq!(
            unwrapped_response.status,
            NonZeroU16::try_from(StatusCode::OK.as_u16()).unwrap_or(NonZeroU16::new(200).unwrap())
        );
        assert_eq!(unwrapped_response.body, Vec::<u8>::new());
        assert_eq!(unwrapped_response.formatter, None);
    }

    #[test]
    fn test_match_magic_numbers() {
        let json_magic_numbers = magic_numbers::JSON.to_vec();
        let result = match_magic_numbers(json_magic_numbers.as_slice());
        assert_eq!(result.is_ok(), true);
        assert_eq!(result.unwrap(), handlers::Format::Json);
    }

    #[test]
    fn test_do_not_match_magic_numbers() {
        let unknown_magic_numbers = "unknown".as_bytes();
        let result = match_magic_numbers(unknown_magic_numbers);
        assert_eq!(result.is_err(), true);
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Sample {
        val: u8,
    }

    #[test]
    fn test_process_json() {
        let sample = Sample {
            val: 42,
        };
        let mut serialized_sample = magic_numbers::JSON.to_vec();
        let json = serde_json::to_string(&sample).unwrap();

        serialized_sample.resize(serialized_sample.len() + json.len(), 0);
        for (i, b) in json.as_bytes().iter().enumerate() {
            serialized_sample[magic_numbers::JSON.len() + i] = *b;
        }

        let result = process_json::<Sample>(serialized_sample.as_slice());
        assert_eq!(result.is_ok(), true);

        let processed_checkin = result.unwrap();
        assert_eq!(processed_checkin, sample);
    }
}
