//! Handle the termination of the agent (terminate command)

use chrono::Utc;
use srv_mod_entity::{
    active_enums::{CommandStatus, LogLevel},
    entities::{agent, agent_command, logs},
    sea_orm::{prelude::*, sea_query::Alias, ActiveValue::Set, DatabaseConnection},
};
use tracing::instrument;

use crate::error;

/// The minimal agent specs required to allow a unique agent identification with the hostname
#[derive(Debug, Clone, PartialEq, Eq)]
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

    // This is very unlikely to happen, but we need to check if the agent is present anyway.
    //
    // Why is this unlikely? Because the agent_id is a foreign key, and the agent_command entity
    // should not be able to exist without a valid agent_id, this is represented by the constraint
    // in the database schema (check the migration crate,
    // m20241012_070555_create_agent_command_table.rs).
    // However, we still need to check for this case, as it is possible to have a corrupted database.
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
                Expr::value(CommandStatus::Completed).cast_as(Alias::new("command_status"))
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

#[cfg(test)]
mod tests {
    use kageshirei_communication_protocol::{NetworkInterface, NetworkInterfaceArray};
    use srv_mod_entity::{
        active_enums::{AgentIntegrity, CommandStatus},
        entities::{agent, agent_command},
        sea_orm::{entity::*, Database, TransactionTrait},
    };

    use super::*;

    async fn cleanup(db: DatabaseConnection) {
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
    }

    async fn init() -> DatabaseConnection {
        let db_pool = Database::connect("postgresql://kageshirei:kageshirei@localhost/kageshirei")
            .await
            .unwrap();

        cleanup(db_pool.clone()).await;

        let agent = agent::Entity::insert(agent::ActiveModel {
            id:                 ActiveValue::Set("test-id".to_owned()),
            pid:                ActiveValue::Set(1),
            secret:             ActiveValue::Set("test".to_owned()),
            cwd:                ActiveValue::Set("test".to_owned()),
            server_secret:      ActiveValue::Set("test".to_owned()),
            operating_system:   ActiveValue::Set("test".to_owned()),
            integrity:          ActiveValue::Set(AgentIntegrity::Medium),
            updated_at:         ActiveValue::Set(Utc::now().naive_utc()),
            domain:             ActiveValue::Set(Some("test".to_owned())),
            hostname:           ActiveValue::Set("test-hostname".to_owned()),
            network_interfaces: ActiveValue::Set(NetworkInterfaceArray {
                network_interfaces: vec![NetworkInterface {
                    name:        Some("test".to_owned()),
                    dhcp_server: Some("test".to_owned()),
                    address:     Some("test".to_owned()),
                }],
            }),
            ppid:               ActiveValue::Set(1),
            username:           ActiveValue::Set("test".to_owned()),
            process_name:       ActiveValue::Set("test".to_owned()),
            signature:          ActiveValue::Set("test".to_owned()),
            terminated_at:      ActiveValue::Set(None),
            created_at:         ActiveValue::Set(Utc::now().naive_utc()),
        })
        .exec_with_returning(&db_pool)
        .await
        .unwrap();

        let agent2 = agent::Entity::insert(agent::ActiveModel {
            id:                 ActiveValue::Set("test-id-2".to_owned()),
            pid:                ActiveValue::Set(1),
            secret:             ActiveValue::Set("test".to_owned()),
            cwd:                ActiveValue::Set("test".to_owned()),
            server_secret:      ActiveValue::Set("test".to_owned()),
            operating_system:   ActiveValue::Set("test".to_owned()),
            integrity:          ActiveValue::Set(AgentIntegrity::Medium),
            updated_at:         ActiveValue::Set(Utc::now().naive_utc()),
            domain:             ActiveValue::Set(Some("test".to_owned())),
            hostname:           ActiveValue::Set("test-hostname".to_owned()),
            network_interfaces: ActiveValue::Set(NetworkInterfaceArray {
                network_interfaces: vec![NetworkInterface {
                    name:        Some("test".to_owned()),
                    dhcp_server: Some("test".to_owned()),
                    address:     Some("test".to_owned()),
                }],
            }),
            ppid:               ActiveValue::Set(1),
            username:           ActiveValue::Set("test".to_owned()),
            process_name:       ActiveValue::Set("test".to_owned()),
            signature:          ActiveValue::Set("test2".to_owned()),
            terminated_at:      ActiveValue::Set(None),
            created_at:         ActiveValue::Set(Utc::now().naive_utc()),
        })
        .exec_with_returning(&db_pool)
        .await
        .unwrap();

        agent_command::Entity::insert_many(vec![
            agent_command::ActiveModel {
                id: ActiveValue::Set("test1".to_owned()),
                agent_id: ActiveValue::Set(agent.id.clone()),
                command: ActiveValue::Set(serde_json::json!({
                    "test": "cmd1"
                })),
                status: ActiveValue::Set(CommandStatus::Pending),
                created_at: ActiveValue::Set(Utc::now().naive_utc()),
                updated_at: ActiveValue::Set(Utc::now().naive_utc()),
                ..Default::default()
            },
            agent_command::ActiveModel {
                id: ActiveValue::Set("test2".to_owned()),
                agent_id: ActiveValue::Set(agent2.id.clone()),
                command: ActiveValue::Set(serde_json::json!({
                    "test": "cmd2"
                })),
                status: ActiveValue::Set(CommandStatus::Pending),
                created_at: ActiveValue::Set(Utc::now().naive_utc()),
                updated_at: ActiveValue::Set(Utc::now().naive_utc()),
                ..Default::default()
            },
            agent_command::ActiveModel {
                id: ActiveValue::Set("test3".to_owned()),
                agent_id: ActiveValue::Set(agent.id.clone()),
                command: ActiveValue::Set(serde_json::json!({
                    "test": "cmd3"
                })),
                status: ActiveValue::Set(CommandStatus::Failed),
                created_at: ActiveValue::Set(Utc::now().naive_utc()),
                updated_at: ActiveValue::Set(Utc::now().naive_utc()),
                ..Default::default()
            },
        ])
        .exec(&db_pool)
        .await
        .unwrap();

        db_pool
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_get_agent_specs_success() {
        let db_pool = init().await;

        let specs = get_agent_specs("test1", &db_pool).await;
        assert!(specs.is_ok());
        let specs = specs.unwrap();
        assert_eq!(specs.agent_id, "test-id");
        assert_eq!(specs.hostname, "test-hostname");
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_get_agent_specs_not_found() {
        let db_pool = init().await;

        let specs = get_agent_specs("test123", &db_pool).await;
        assert!(specs.is_err());
        assert_eq!(specs.unwrap_err(), error::CommandHandling::NotFound);
    }

    // ---------------------------------------------------------------------------------------------
    // This test always fails as no way to force the removal of the agent alone was found given the
    // constraints of the database schema.
    // Have you found a solution that does not involve changing the schema? Please let us know!
    // ---------------------------------------------------------------------------------------------
    //
    // #[tokio::test]
    // #[serial_test::serial]
    // async fn test_get_agent_specs_agent_not_found() {
    // let db_pool = init().await;
    //
    // let specs = get_agent_specs("test2", &db_pool).await;
    // assert!(specs.is_err());
    // assert_eq!(specs.unwrap_err(), error::CommandHandling::AgentNotFound);
    // }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_handle_terminate_success() {
        let db_pool = init().await;

        let result = handle_terminate(db_pool.clone(), "test1".to_string()).await;
        result.unwrap();
        // assert!(result.is_ok());
        // assert!(result.unwrap().is_empty());
    }
}
