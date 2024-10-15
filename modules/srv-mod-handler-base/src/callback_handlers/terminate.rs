use bytes::Bytes;
use chrono::Utc;
use srv_mod_entity::{
    active_enums::{CommandStatus, LogLevel},
    entities::{agent, agent_command, logs},
    sea_orm::{prelude::*, ActiveValue::Set, DatabaseConnection},
};
use tracing::instrument;

struct AgentSpecs {
    agent_id: String,
    hostname: String,
}

/// Get the agent specs from the database
async fn get_agent_specs(cmd_request_id: &str, conn: &DatabaseConnection) -> Result<AgentSpecs, String> {
    let command_with_agent = agent_command::Entity::find()
        .find_also_related(agent::Entity)
        .filter(agent_command::Column::Id.eq(cmd_request_id))
        .one(conn)
        .await
        .map_err(|e| format!("Failed to get agent specs: {}", e))?;
    if command_with_agent.is_none() {
        return Err("Command not found".to_owned());
    }
    let command_with_agent = command_with_agent.unwrap();

    let agent = command_with_agent.1;
    if agent.is_none() {
        return Err("Agent not found".to_owned());
    }
    let agent = agent.unwrap();

    Ok(AgentSpecs {
        agent_id: agent.id,
        hostname: agent.hostname,
    })
}

/// Handle the termination of the agent
#[instrument]
pub async fn handle_terminate(db_pool: DatabaseConnection, cmd_request_id: String) -> Result<Bytes, String> {
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

    tokio::join!(
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

    // Return an empty response
    Ok(Bytes::new())
}
