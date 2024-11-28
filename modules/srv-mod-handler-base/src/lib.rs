//! This module contains the base handler for all the handler specialization.
//! It contains the logic to handle the retrieval of commands and the processing of the results.
#![feature(async_closure)]

use std::num::NonZeroU16;

use axum::http::{HeaderMap, StatusCode};
use kageshirei_communication_protocol::Format as _;
use kageshirei_format_json::FormatJson;
use srv_mod_config::handlers::EncryptionScheme;
use srv_mod_entity::{
    active_enums::CommandStatus,
    entities::agent_command,
    sea_orm::{prelude::*, sea_query::Alias, QuerySelect as _, TransactionTrait as _},
};
use tracing::{error, instrument};

use crate::{response::BaseHandlerResponse, state::HandlerSharedState};

pub(crate) mod callback_handlers;
mod decode_or_fail;
mod decrypt_asymmetric_or_fail;
mod decrypt_symmetric_or_fail;
mod error;
mod process_body;
pub mod response;
pub mod state;

/// Handle the extraction of pending commands for a given id marking all the retrieved ones as
/// running
#[instrument]
pub async fn handle_command_retrieval(
    state: HandlerSharedState,
    agent_id: String,
) -> Result<BaseHandlerResponse, BaseHandlerResponse> {
    let db_connection = state.db_pool.clone();

    let commands = db_connection
        .transaction::<_, _, DbErr>(|txn| {
            Box::pin(async move {
                let commands: Vec<serde_json::Value> = agent_command::Entity::find()
                    .select_only()
                    .column(agent_command::Column::Command)
                    .filter(agent_command::Column::AgentId.eq(agent_id.clone()))
                    .filter(agent_command::Column::Status.eq(CommandStatus::Pending))
                    .into_tuple()
                    .all(txn)
                    .await?;

                agent_command::Entity::update_many()
                    .col_expr(
                        agent_command::Column::Status,
                        Expr::value(CommandStatus::Running).cast_as(Alias::new("command_status")),
                    )
                    .filter(agent_command::Column::AgentId.eq(agent_id.clone()))
                    .filter(agent_command::Column::Status.eq(CommandStatus::Pending))
                    .exec(txn)
                    .await?;

                Ok(commands)
            })
        })
        .await;

    if commands.is_err() {
        error!(
            "Command retrieval transaction failed with error: {}",
            commands.err().unwrap()
        );
        // Return an empty response if the transaction failed
        return Err(BaseHandlerResponse {
            status:    NonZeroU16::try_from(StatusCode::OK.as_u16()).unwrap_or(NonZeroU16::new(200).unwrap()),
            body:      Vec::new(),
            formatter: None,
        });
    }
    let commands = commands.unwrap();

    // Format the commands as JSON, this may require a custom formatter in the future
    // TODO: Implement a custom formatter for the commands, maybe a configuration option?
    let body = FormatJson
        .write::<_, &str>(commands, None)
        .unwrap_or_default();

    Ok(BaseHandlerResponse {
        status: NonZeroU16::try_from(StatusCode::OK.as_u16()).unwrap_or(NonZeroU16::new(200).unwrap()),
        body,
        formatter: None,
    })
}

/// Handle the result of the execution of a given command marking it as completed or failed
/// depending on the result
#[instrument]
pub async fn handle_command_result(
    state: HandlerSharedState,
    body: Vec<u8>,
    headers: HeaderMap,
    cmd_request_id: String,
) -> Result<BaseHandlerResponse, BaseHandlerResponse> {
    // Decode the body if an encoder is provided
    let body = if state.config.security.encoder.is_some() {
        let encoder = state.config.security.encoder.as_ref().unwrap();
        decode_or_fail::decode_or_fail_response(encoder, body)
    }
    else {
        Ok(body)
    }?;

    // Decrypt the body if an encryption scheme is provided
    let body = match state.config.security.encryption_scheme {
        EncryptionScheme::Plain => Ok(body),
        EncryptionScheme::Symmetric => {
            decrypt_symmetric_or_fail::decrypt_symmetric_or_fail(state.config.security.algorithm.as_ref(), body)
        },
        EncryptionScheme::Asymmetric => {
            decrypt_asymmetric_or_fail::decrypt_asymmetric_or_fail(state.config.security.algorithm.as_ref(), body)
        },
    }?;

    Ok(process_body::process_body(state.db_pool.clone(), body, headers, cmd_request_id).await)
}

#[cfg(test)]
mod test {
    use std::{num::NonZeroU16, sync::Arc};

    use axum::http::StatusCode;
    use kageshirei_communication_protocol::{Format as _, NetworkInterface, NetworkInterfaceArray};
    use kageshirei_format_json::FormatJson;
    use srv_mod_entity::{
        active_enums::AgentIntegrity,
        entities::agent,
        sea_orm::{ActiveValue::Set, Database},
    };

    use super::*;
    use crate::state::HandlerState;

    #[tokio::test]
    async fn test_handle_command_retrieval() {
        let db_pool = Database::connect("postgresql://kageshirei:kageshirei@localhost/kageshirei")
            .await
            .unwrap();

        let cleanup = async |db: DatabaseConnection| {
            db.transaction::<_, (), DbErr>(|txn| {
                Box::pin(async move {
                    agent::Entity::delete_many().exec(txn).await.unwrap();
                    agent_command::Entity::delete_many()
                        .exec(txn)
                        .await
                        .unwrap();
                    Ok(())
                })
            })
            .await
            .unwrap();
        };
        cleanup(db_pool.clone()).await;

        let state = Arc::new(HandlerState {
            db_pool,
            config: Default::default(),
        });

        let agent = agent::Entity::insert(agent::ActiveModel {
            id:                 Set("test".to_owned()),
            pid:                Set(1),
            secret:             Set("test".to_owned()),
            cwd:                Set("test".to_owned()),
            server_secret:      Set("test".to_owned()),
            operating_system:   Set("test".to_owned()),
            integrity:          Set(AgentIntegrity::Medium),
            updated_at:         Set(chrono::Utc::now().naive_utc()),
            domain:             Set(Some("test".to_owned())),
            hostname:           Set("test".to_owned()),
            network_interfaces: Set(NetworkInterfaceArray {
                network_interfaces: vec![NetworkInterface {
                    name:        Some("test".to_owned()),
                    dhcp_server: Some("test".to_owned()),
                    address:     Some("test".to_owned()),
                }],
            }),
            ppid:               Set(1),
            username:           Set("test".to_owned()),
            process_name:       Set("test".to_owned()),
            signature:          Set("test".to_owned()),
            terminated_at:      Set(None),
            created_at:         Set(chrono::Utc::now().naive_utc()),
        })
        .exec_with_returning(&state.db_pool)
        .await
        .unwrap();

        agent_command::Entity::insert_many(vec![
            agent_command::ActiveModel {
                id: Set("test1".to_owned()),
                agent_id: Set(agent.id.clone()),
                command: Set(serde_json::json!({
                    "test": "cmd1"
                })),
                status: Set(CommandStatus::Pending),
                created_at: Set(chrono::Utc::now().naive_utc()),
                updated_at: Set(chrono::Utc::now().naive_utc()),
                ..Default::default()
            },
            agent_command::ActiveModel {
                id: Set("test2".to_owned()),
                agent_id: Set(agent.id.clone()),
                command: Set(serde_json::json!({
                    "test": "cmd2"
                })),
                status: Set(CommandStatus::Pending),
                created_at: Set(chrono::Utc::now().naive_utc()),
                updated_at: Set(chrono::Utc::now().naive_utc()),
                ..Default::default()
            },
            agent_command::ActiveModel {
                id: Set("test3".to_owned()),
                agent_id: Set(agent.id.clone()),
                command: Set(serde_json::json!({
                    "test": "cmd3"
                })),
                status: Set(CommandStatus::Failed),
                created_at: Set(chrono::Utc::now().naive_utc()),
                updated_at: Set(chrono::Utc::now().naive_utc()),
                ..Default::default()
            },
        ])
        .exec(&state.db_pool)
        .await
        .unwrap();

        let response = handle_command_retrieval(state.clone(), agent.id.clone()).await;

        assert_eq!(
            response.status,
            NonZeroU16::try_from(StatusCode::OK.as_u16()).unwrap_or(NonZeroU16::new(200).unwrap())
        );

        let body = FormatJson
            .read::<Vec<serde_json::Value>, &str>(response.body.as_slice(), None)
            .unwrap();

        assert_eq!(body.len(), 2);
        assert_eq!(body[0].get("test").unwrap(), "cmd1");
        assert_eq!(body[1].get("test").unwrap(), "cmd2");

        cleanup(state.db_pool.clone()).await;
    }
}
