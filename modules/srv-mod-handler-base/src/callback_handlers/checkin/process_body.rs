//! Processes the body of a checkin request, this is the entrypoint to the complex checking
//! procedures

use kageshirei_communication_protocol::{
    communication::{Checkin, CheckinResponse},
    Format,
};
use srv_mod_entity::{entities::agent as agent_entity, sea_orm::DatabaseConnection};
use tracing::instrument;

use super::{agent, agent_profiles::apply_filters};
use crate::error;

/// Persist the checkin data into the database as an agent
async fn persist(data: Checkin, db_pool: DatabaseConnection) -> Result<agent_entity::Model, error::CommandHandling> {
    let create_agent_instance = agent::prepare(data)?;

    let db = db_pool.clone();

    agent::create_or_update(create_agent_instance, &db).await
}

#[instrument(skip(db_pool, format))]
pub async fn handle_checkin<F>(
    data: Checkin,
    db_pool: DatabaseConnection,
    mut format: F,
) -> Result<Vec<u8>, error::CommandHandling>
where
    F: Format,
{
    let agent = persist(data, db_pool.clone()).await?;

    // apply filters to the agent
    let config = apply_filters(&agent, db_pool.clone()).await;

    format
        .write::<CheckinResponse, &str>(config, None)
        .map_err(error::CommandHandling::Format)
}

// #[cfg(test)]
// mod test {
// use std::sync::Arc;
//
// use axum::{body::Bytes, http::StatusCode};
// use bytes::{BufMut, BytesMut};
// use kageshirei_communication_protocol::{
// communication::checkin::{Checkin, PartialCheckin},
// magic_numbers,
// };
// use serial_test::serial;
// use srv_mod_config::{
// handlers,
// handlers::{EncryptionScheme, HandlerConfig, HandlerSecurityConfig, HandlerType},
// };
// use srv_mod_database::{
// bb8,
// diesel::{Connection, PgConnection},
// diesel_async::{pooled_connection::AsyncDieselConnectionManager, AsyncPgConnection},
// diesel_migrations::MigrationHarness,
// migration::MIGRATIONS,
// Pool,
// };
// use srv_mod_handler_base::state::HttpHandlerState;
//
// fn make_config() -> HandlerConfig {
// let config = HandlerConfig {
// enabled:   true,
// r#type:    HandlerType::Http,
// protocols: vec![handlers::Protocol::Json],
// port:      8081,
// host:      "127.0.0.1".to_string(),
// tls:       None,
// security:  HandlerSecurityConfig {
// encryption_scheme: EncryptionScheme::Plain,
// algorithm:         None,
// encoder:           None,
// },
// };
//
// config
// }
//
// async fn drop_database(url: String) {
// let mut connection = PgConnection::establish(url.as_str()).unwrap();
//
// connection.revert_all_migrations(MIGRATIONS).unwrap();
// connection.run_pending_migrations(MIGRATIONS).unwrap();
// }
//
// async fn make_pool(url: String) -> Pool {
// let connection_manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(url);
// Arc::new(
// bb8::Pool::builder()
// .max_size(1u32)
// .build(connection_manager)
// .await
// .unwrap(),
// )
// }
//
// #[test]
// fn test_ensure_is_not_empty() {
// use axum::{body::Bytes, http::StatusCode};
//
// use crate::routes::public::checkin::process_body::ensure_is_not_empty;
//
// let empty_body = Bytes::new();
// let response = ensure_is_not_empty(empty_body);
// assert_eq!(response.is_some(), true);
//
// let unwrapped_response = response.unwrap();
// assert_eq!(unwrapped_response.status(), StatusCode::OK);
// }
//
// #[test]
// fn test_match_magic_numbers() {
// use axum::body::Bytes;
//
// use crate::routes::public::checkin::process_body::match_magic_numbers;
//
// let json_magic_numbers = Bytes::from(magic_numbers::JSON.to_vec());
// let result = match_magic_numbers(json_magic_numbers);
// assert_eq!(result.is_ok(), true);
// assert_eq!(result.unwrap(), handlers::Protocol::Json);
//
// let unknown_magic_numbers = Bytes::from("unknown".as_bytes());
// let result = match_magic_numbers(unknown_magic_numbers);
// assert_eq!(result.is_err(), true);
// }
//
// #[test]
// fn test_process_json() {
// let obj_checkin = Checkin::new(PartialCheckin {
// operative_system:  "Windows".to_string(),
// hostname:          "DESKTOP-PC".to_string(),
// domain:            "WORKGROUP".to_string(),
// username:          "user".to_string(),
// ip:                "10.2.123.45".to_string(),
// process_id:        1234,
// parent_process_id: 5678,
// process_name:      "agent.exe".to_string(),
// elevated:          true,
// });
// let checkin = serde_json::to_string(&obj_checkin).unwrap();
//
// let mut bytes = BytesMut::with_capacity(checkin.len() + magic_numbers::JSON.len());
// for b in magic_numbers::JSON.iter() {
// bytes.put_u8(*b);
// }
// for b in checkin.as_bytes() {
// bytes.put_u8(*b);
// }
//
// let result = super::process_json(bytes.freeze());
// assert_eq!(result.is_ok(), true);
// let processed_checkin = result.unwrap();
// assert_eq!(processed_checkin, obj_checkin);
// }
//
// #[tokio::test]
// #[serial]
// async fn test_persist() {
// let obj_checkin = Checkin::new(PartialCheckin {
// operative_system:  "Windows".to_string(),
// hostname:          "DESKTOP-PC".to_string(),
// domain:            "WORKGROUP".to_string(),
// username:          "user".to_string(),
// ip:                "10.2.123.45".to_string(),
// process_id:        1234,
// parent_process_id: 5678,
// process_name:      "agent.exe".to_string(),
// elevated:          true,
// });
//
// let shared_config = make_config();
// let connection_string = "postgresql://kageshirei:kageshirei@localhost/kageshirei".to_string();
//
// Ensure the database is clean
// drop_database(connection_string.clone()).await;
// let pool = make_pool(connection_string.clone()).await;
//
// let route_state = Arc::new(HttpHandlerState {
// config:  Arc::new(shared_config),
// db_pool: pool,
// });
//
// let agent = super::persist(Ok(obj_checkin), &route_state).await;
//
// assert_eq!(agent.operative_system, "Windows");
// assert_eq!(agent.hostname, "DESKTOP-PC");
// assert_eq!(agent.domain, "WORKGROUP");
// assert_eq!(agent.username, "user");
// assert_eq!(agent.ip, "10.2.123.45");
// assert_eq!(agent.process_id, 1234);
// assert_eq!(agent.parent_process_id, 5678);
// assert_eq!(agent.process_name, "agent.exe");
// assert_eq!(agent.elevated, true);
// assert_ne!(agent.server_secret_key, "");
// assert_ne!(agent.secret_key, "");
// assert_eq!(
// agent.signature,
// "YdkxtuNA9_78BiX7Oe_445oEr_Rktlcve1k73kBQ9pvoq_04qXVVcRfenXjy5Sc6947p9dn_YSiLGFw6YVXp0g"
// );
//
// Ensure the database is clean
// drop_database(connection_string.clone()).await;
// }
//
// #[tokio::test]
// #[serial]
// async fn test_process_body() {
// let obj_checkin = Checkin::new(PartialCheckin {
// operative_system:  "Windows".to_string(),
// hostname:          "DESKTOP-PC".to_string(),
// domain:            "WORKGROUP".to_string(),
// username:          "user".to_string(),
// ip:                "10.2.123.45".to_string(),
// process_id:        1234,
// parent_process_id: 5678,
// process_name:      "agent.exe".to_string(),
// elevated:          true,
// });
// let checkin = serde_json::to_string(&obj_checkin).unwrap();
//
// let mut bytes = BytesMut::with_capacity(checkin.len() + magic_numbers::JSON.len());
// for b in magic_numbers::JSON.iter() {
// bytes.put_u8(*b);
// }
// for b in checkin.as_bytes() {
// bytes.put_u8(*b);
// }
//
// let shared_config = make_config();
// let connection_string = "postgresql://kageshirei:kageshirei@localhost/kageshirei".to_string();
//
// Ensure the database is clean
// drop_database(connection_string.clone()).await;
// let pool = make_pool(connection_string.clone()).await;
//
// let route_state = Arc::new(HttpHandlerState {
// config:  Arc::new(shared_config),
// db_pool: pool,
// });
//
// let response = super::process_body(route_state, bytes.freeze()).await;
// assert_eq!(response.status(), StatusCode::OK);
//
// let body = axum::body::to_bytes(response.into_body(), usize::MAX)
// .await
// .unwrap();
// assert_eq!(body.is_empty(), false);
// }
//
// #[tokio::test]
// #[serial]
// async fn test_process_body_with_invalid_magic_numbers() {
// let obj_checkin = Checkin::new(PartialCheckin {
// operative_system:  "Windows".to_string(),
// hostname:          "DESKTOP-PC".to_string(),
// domain:            "WORKGROUP".to_string(),
// username:          "user".to_string(),
// ip:                "10.2.123.45".to_string(),
// process_id:        1234,
// parent_process_id: 5678,
// process_name:      "agent.exe".to_string(),
// elevated:          true,
// });
// let checkin = serde_json::to_string(&obj_checkin).unwrap();
//
// let mut bytes = BytesMut::with_capacity(checkin.len());
// for b in checkin.as_bytes() {
// bytes.put_u8(*b);
// }
//
// let shared_config = make_config();
// let connection_string = "postgresql://kageshirei:kageshirei@localhost/kageshirei".to_string();
//
// Ensure the database is clean
// drop_database(connection_string.clone()).await;
// let pool = make_pool(connection_string.clone()).await;
//
// let route_state = Arc::new(HttpHandlerState {
// config:  Arc::new(shared_config),
// db_pool: pool,
// });
//
// let response = super::process_body(route_state, bytes.freeze()).await;
// assert_eq!(response.status(), StatusCode::OK);
//
// let body = axum::body::to_bytes(response.into_body(), usize::MAX)
// .await
// .unwrap();
// assert_eq!(body.is_empty(), true);
// }
// }
