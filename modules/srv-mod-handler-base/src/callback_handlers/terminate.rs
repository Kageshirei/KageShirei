use anyhow::Result;
use bytes::Bytes;
use chrono::Utc;
use srv_mod_database::bb8::PooledConnection;
use srv_mod_database::diesel::ExpressionMethods;
use srv_mod_database::diesel::QueryDsl;
use srv_mod_database::diesel_async::pooled_connection::AsyncDieselConnectionManager;
use srv_mod_database::diesel_async::{AsyncPgConnection, RunQueryDsl};
use srv_mod_database::models::log::CreateLog;
use srv_mod_database::schema::{agents, agents_command_requests};
use srv_mod_database::schema_extension::AgentCommandStatus;
use srv_mod_database::{diesel, Pool};
use tracing::instrument;

struct AgentSpecs {
    agent_id: String,
    hostname: String,
}

/// Get the agent specs from the database
async fn get_agent_specs(
    cmd_request_id: &str,
    conn: &mut PooledConnection<'_, AsyncDieselConnectionManager<AsyncPgConnection>>,
) -> Result<AgentSpecs> {
    let (agent_id, hostname) = agents_command_requests::table
        .inner_join(agents::table)
        .select((agents::id, agents::hostname))
        .filter(agents_command_requests::id.eq(cmd_request_id))
        .first::<(String, String)>(conn)
        .await?;

    Ok(AgentSpecs { agent_id, hostname })
}

/// Handle the termination of the agent
#[instrument]
pub async fn handle_terminate(db_pool: Pool, cmd_request_id: String) -> Result<Bytes> {
    let mut conn = db_pool.get().await?;
    let agent_specs = get_agent_specs(&cmd_request_id, &mut conn).await?;

    let log = CreateLog::new(srv_mod_database::schema_extension::LogLevel::WARN)
        .with_title("Agent terminated")
        .with_message("Agent terminated in response to a `terminate` command request")
        .with_extra_value(serde_json::json!({
            "hostname": agent_specs.hostname,
        }));
    tokio::join!(
        // Create a log entry for agent termination
        log.save(&mut conn),
        // Update the command request status to completed
        diesel::update(agents_command_requests::table)
            .filter(agents_command_requests::id.eq(cmd_request_id))
            .set((
                agents_command_requests::status.eq(AgentCommandStatus::Completed),
                agents_command_requests::output.eq("Agent terminated"),
                agents_command_requests::completed_at.eq(Utc::now()),
            ))
            .execute(&mut *db_pool.get().await?),
        // Update the agent status to terminated
        diesel::update(agents::table)
            .filter(agents::id.eq(agent_specs.agent_id))
            .set((agents::terminated_at.eq(Utc::now())))
            .execute(&mut *db_pool.get().await?),
    );

    // Return an empty response
    Ok(Bytes::new())
}
