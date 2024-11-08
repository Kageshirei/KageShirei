//! Handle the termination of the agent (terminate command)

use chrono::Utc;
use srv_mod_entity::{
    active_enums::{CommandStatus, LogLevel},
    entities::{agent, agent_command, logs},
    sea_orm::{prelude::*, ActiveValue::Set, DatabaseConnection},
};
use tracing::instrument;

use crate::error;

/// The minimal agent specs required to allow a unique agent identification with the hostname
struct AgentSpecs {
    /// The id of the agent
    agent_id: String,
    /// The hostname of the agent
    hostname: String,
}

/// Get the agent specs from the database
async fn get_agent_specs(
    cmd_request_id: &str,
    conn: &DatabaseConnection,
) -> Result<AgentSpecs, error::CommandHandling> {
    let command_with_agent = agent_command::Entity::find()
        .find_also_related(agent::Entity)
        .filter(agent_command::Column::Id.eq(cmd_request_id))
        .one(conn)
        .await
        .map_err(|e| error::CommandHandling::Database("Failed to get agent specs".to_owned(), e))?;

    if command_with_agent.is_none() {
        return Err(error::CommandHandling::NotFound);
    }
    let command_with_agent = command_with_agent.unwrap();

    let agent = command_with_agent.1;
    if agent.is_none() {
        return Err(error::CommandHandling::AgentNotFound);
    }
    let agent = agent.unwrap();

    Ok(AgentSpecs {
        agent_id: agent.id,
        hostname: agent.hostname,
    })
}

/// Handle the termination of the agent
#[instrument]
pub async fn handle_terminate(
    db_pool: DatabaseConnection,
    cmd_request_id: String,
) -> Result<Vec<u8>, error::CommandHandling> {
    let db = db_pool.clone();
    let agent_specs = get_agent_specs(&cmd_request_id, &db).await?;

    let log = logs::ActiveModel {
        level: Set(LogLevel::Warning),
        title: Set("Agent terminated".to_owned()),
        message: Set(Some(
            "Agent terminated in response to a `terminate` command request".to_owned(),
        )),
        extra: Set(Some(serde_json::json!({
            "hostname": agent_specs.hostname,
        }))),
        ..Default::default()
    };

    let responses = tokio::join!(
        // Create a log entry for agent termination
        log.insert(&db),
        // Update the command request status to completed
        agent_command::Entity::update_many()
            .filter(agent_command::Column::Id.eq(cmd_request_id))
            .col_expr(
                agent_command::Column::Status,
                Expr::value(CommandStatus::Completed)
            )
            .col_expr(
                agent_command::Column::Output,
                Expr::value("Agent terminated")
            )
            .col_expr(
                agent_command::Column::CompletedAt,
                Expr::value(Utc::now().naive_utc())
            )
            .exec(&db),
        // Update the agent status to terminated
        agent::Entity::update_many()
            .filter(agent::Column::Id.eq(agent_specs.agent_id))
            .col_expr(
                agent::Column::TerminatedAt,
                Expr::value(Utc::now().naive_utc())
            )
            .exec(&db),
    );

    // Check if any of the operations failed
    if responses.0.is_err() || responses.1.is_err() || responses.2.is_err() {
        return Err(error::CommandHandling::Database(
            "Failed to terminate agent".to_owned(),
            if responses.0.is_err() {
                responses.0.err().unwrap()
            }
            else if responses.1.is_err() {
                responses.1.err().unwrap()
            }
            else {
                responses.2.err().unwrap()
            },
        ));
    }

    // Return an empty response
    Ok(Vec::<u8>::new())
}
