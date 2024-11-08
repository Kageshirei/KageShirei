//! This module contains the logic to process the body of the request, this is a protocol agnostic
//! module that will try to match the magic numbers of the body to the appropriate format and then
//! handle the command by executing it and returning the response if any

use axum::{
    body::Body,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse as _, Response},
    Json,
};
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

use crate::{callback_handlers, error};

/// Ensure that the body is not empty by returning a response if it is
#[instrument(skip_all)]
pub fn ensure_is_not_empty(body: Vec<u8>) -> Option<Response<Body>> {
    if body.is_empty() {
        warn!("Empty checking request received, request refused");
        warn!("Internal status code: {}", StatusCode::BAD_REQUEST);

        // always return OK to avoid leaking information
        return Some((StatusCode::OK, "").into_response());
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
) -> Response<Body> {
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

                    Json(response).into_response()
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
            (StatusCode::OK, "").into_response()
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
    use std::sync::Arc;

    use axum::{body::Bytes, http::StatusCode};
    use bytes::{BufMut, BytesMut};
    use kageshirei_communication_protocol::{
        communication::checkin::{Checkin, PartialCheckin},
        magic_numbers,
    };
    use serial_test::serial;
    use srv_mod_config::{
        handlers,
        handlers::{EncryptionScheme, HandlerConfig, HandlerSecurityConfig, HandlerType},
    };
    use srv_mod_database::{
        bb8,
        diesel::{Connection, PgConnection},
        diesel_async::{pooled_connection::AsyncDieselConnectionManager, AsyncPgConnection},
        diesel_migrations::MigrationHarness,
        migration::MIGRATIONS,
        Pool,
    };
    use srv_mod_handler_base::state::HttpHandlerState;

    fn make_config() -> HandlerConfig {
        let config = HandlerConfig {
            enabled:   true,
            r#type:    HandlerType::Http,
            protocols: vec![handlers::Protocol::Json],
            port:      8081,
            host:      "127.0.0.1".to_string(),
            tls:       None,
            security:  HandlerSecurityConfig {
                encryption_scheme: EncryptionScheme::Plain,
                algorithm:         None,
                encoder:           None,
            },
        };

        config
    }

    async fn drop_database(url: String) {
        let mut connection = PgConnection::establish(url.as_str()).unwrap();

        connection.revert_all_migrations(MIGRATIONS).unwrap();
        connection.run_pending_migrations(MIGRATIONS).unwrap();
    }

    async fn make_pool(url: String) -> Pool {
        let connection_manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(url);
        Arc::new(
            bb8::Pool::builder()
                .max_size(1u32)
                .build(connection_manager)
                .await
                .unwrap(),
        )
    }

    #[test]
    fn test_ensure_is_not_empty() {
        use axum::{body::Bytes, http::StatusCode};

        use crate::routes::public::checkin::process_body::ensure_is_not_empty;

        let empty_body = Bytes::new();
        let response = ensure_is_not_empty(empty_body);
        assert_eq!(response.is_some(), true);

        let unwrapped_response = response.unwrap();
        assert_eq!(unwrapped_response.status(), StatusCode::OK);
    }

    #[test]
    fn test_match_magic_numbers() {
        use axum::body::Bytes;

        use crate::routes::public::checkin::process_body::match_magic_numbers;

        let json_magic_numbers = Bytes::from(magic_numbers::JSON.to_vec());
        let result = match_magic_numbers(json_magic_numbers);
        assert_eq!(result.is_ok(), true);
        assert_eq!(result.unwrap(), handlers::Protocol::Json);

        let unknown_magic_numbers = Bytes::from("unknown".as_bytes());
        let result = match_magic_numbers(unknown_magic_numbers);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn test_process_json() {
        let obj_checkin = Checkin::new(PartialCheckin {
            operative_system:  "Windows".to_string(),
            hostname:          "DESKTOP-PC".to_string(),
            domain:            "WORKGROUP".to_string(),
            username:          "user".to_string(),
            ip:                "10.2.123.45".to_string(),
            process_id:        1234,
            parent_process_id: 5678,
            process_name:      "agent.exe".to_string(),
            elevated:          true,
        });
        let checkin = serde_json::to_string(&obj_checkin).unwrap();

        let mut bytes = BytesMut::with_capacity(checkin.len() + magic_numbers::JSON.len());
        for b in magic_numbers::JSON.iter() {
            bytes.put_u8(*b);
        }
        for b in checkin.as_bytes() {
            bytes.put_u8(*b);
        }

        let result = super::process_json(bytes.freeze());
        assert_eq!(result.is_ok(), true);
        let processed_checkin = result.unwrap();
        assert_eq!(processed_checkin, obj_checkin);
    }

    #[tokio::test]
    #[serial]
    async fn test_persist() {
        let obj_checkin = Checkin::new(PartialCheckin {
            operative_system:  "Windows".to_string(),
            hostname:          "DESKTOP-PC".to_string(),
            domain:            "WORKGROUP".to_string(),
            username:          "user".to_string(),
            ip:                "10.2.123.45".to_string(),
            process_id:        1234,
            parent_process_id: 5678,
            process_name:      "agent.exe".to_string(),
            elevated:          true,
        });

        let shared_config = make_config();
        let connection_string = "postgresql://kageshirei:kageshirei@localhost/kageshirei".to_string();

        // Ensure the database is clean
        drop_database(connection_string.clone()).await;
        let pool = make_pool(connection_string.clone()).await;

        let route_state = Arc::new(HttpHandlerState {
            config:  Arc::new(shared_config),
            db_pool: pool,
        });

        let agent = super::persist(Ok(obj_checkin), &route_state).await;

        assert_eq!(agent.operative_system, "Windows");
        assert_eq!(agent.hostname, "DESKTOP-PC");
        assert_eq!(agent.domain, "WORKGROUP");
        assert_eq!(agent.username, "user");
        assert_eq!(agent.ip, "10.2.123.45");
        assert_eq!(agent.process_id, 1234);
        assert_eq!(agent.parent_process_id, 5678);
        assert_eq!(agent.process_name, "agent.exe");
        assert_eq!(agent.elevated, true);
        assert_ne!(agent.server_secret_key, "");
        assert_ne!(agent.secret_key, "");
        assert_eq!(
            agent.signature,
            "YdkxtuNA9_78BiX7Oe_445oEr_Rktlcve1k73kBQ9pvoq_04qXVVcRfenXjy5Sc6947p9dn_YSiLGFw6YVXp0g"
        );

        // Ensure the database is clean
        drop_database(connection_string.clone()).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_process_body() {
        let obj_checkin = Checkin::new(PartialCheckin {
            operative_system:  "Windows".to_string(),
            hostname:          "DESKTOP-PC".to_string(),
            domain:            "WORKGROUP".to_string(),
            username:          "user".to_string(),
            ip:                "10.2.123.45".to_string(),
            process_id:        1234,
            parent_process_id: 5678,
            process_name:      "agent.exe".to_string(),
            elevated:          true,
        });
        let checkin = serde_json::to_string(&obj_checkin).unwrap();

        let mut bytes = BytesMut::with_capacity(checkin.len() + magic_numbers::JSON.len());
        for b in magic_numbers::JSON.iter() {
            bytes.put_u8(*b);
        }
        for b in checkin.as_bytes() {
            bytes.put_u8(*b);
        }

        let shared_config = make_config();
        let connection_string = "postgresql://kageshirei:kageshirei@localhost/kageshirei".to_string();

        // Ensure the database is clean
        drop_database(connection_string.clone()).await;
        let pool = make_pool(connection_string.clone()).await;

        let route_state = Arc::new(HttpHandlerState {
            config:  Arc::new(shared_config),
            db_pool: pool,
        });

        let response = super::process_body(route_state, bytes.freeze()).await;
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(body.is_empty(), false);
    }

    #[tokio::test]
    #[serial]
    async fn test_process_body_with_invalid_magic_numbers() {
        let obj_checkin = Checkin::new(PartialCheckin {
            operative_system:  "Windows".to_string(),
            hostname:          "DESKTOP-PC".to_string(),
            domain:            "WORKGROUP".to_string(),
            username:          "user".to_string(),
            ip:                "10.2.123.45".to_string(),
            process_id:        1234,
            parent_process_id: 5678,
            process_name:      "agent.exe".to_string(),
            elevated:          true,
        });
        let checkin = serde_json::to_string(&obj_checkin).unwrap();

        let mut bytes = BytesMut::with_capacity(checkin.len());
        for b in checkin.as_bytes() {
            bytes.put_u8(*b);
        }

        let shared_config = make_config();
        let connection_string = "postgresql://kageshirei:kageshirei@localhost/kageshirei".to_string();

        // Ensure the database is clean
        drop_database(connection_string.clone()).await;
        let pool = make_pool(connection_string.clone()).await;

        let route_state = Arc::new(HttpHandlerState {
            config:  Arc::new(shared_config),
            db_pool: pool,
        });

        let response = super::process_body(route_state, bytes.freeze()).await;
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(body.is_empty(), true);
    }
}
