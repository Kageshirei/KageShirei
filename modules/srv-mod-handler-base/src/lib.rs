//! This module contains the base handler for all the handler specialization.
//! It contains the logic to handle the retrieval of commands and the processing of the results.

use std::num::NonZeroU16;

use axum::http::{HeaderMap, StatusCode};
use kageshirei_communication_protocol::Format as _;
use kageshirei_format_json::FormatJson;
use srv_mod_config::handlers::EncryptionScheme;
use srv_mod_entity::{
    active_enums::CommandStatus,
    entities::agent_command,
    sea_orm::{prelude::*, QuerySelect as _, TransactionTrait as _},
};

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
pub async fn handle_command_retrieval(state: HandlerSharedState, agent_id: String) -> BaseHandlerResponse {
    let db_connection = state.db_pool.clone();

    let commands = db_connection
        .transaction::<_, _, DbErr>(|txn| {
            Box::pin(async move {
                let commands = agent_command::Entity::find()
                    .select_only()
                    .column(agent_command::Column::Command)
                    .filter(agent_command::Column::AgentId.eq(agent_id.clone()))
                    .filter(agent_command::Column::Status.eq(CommandStatus::Pending))
                    .all(txn)
                    .await?;

                agent_command::Entity::update_many()
                    .filter(agent_command::Column::AgentId.eq(agent_id))
                    .filter(agent_command::Column::Status.eq(CommandStatus::Pending))
                    .col_expr(
                        agent_command::Column::Status,
                        Expr::value(CommandStatus::Running),
                    )
                    .exec(txn)
                    .await?;

                Ok(commands)
            })
        })
        .await;

    if commands.is_err() {
        // Return an empty response if the transaction failed
        return BaseHandlerResponse {
            status:    NonZeroU16::try_from(StatusCode::OK.as_u16()).unwrap_or(NonZeroU16::new(200).unwrap()),
            body:      Vec::new(),
            formatter: None,
        };
    }
    let commands = commands.unwrap();

    // Format the commands as JSON, this may require a custom formatter in the future
    // TODO: Implement a custom formatter for the commands, maybe a configuration option?
    let body = FormatJson
        .write::<_, &str>(commands, None)
        .unwrap_or_default();

    BaseHandlerResponse {
        status: NonZeroU16::try_from(StatusCode::OK.as_u16()).unwrap_or(NonZeroU16::new(200).unwrap()),
        body,
        formatter: None,
    }
}

/// Handle the result of the execution of a given command marking it as completed or failed
/// depending on the result
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
