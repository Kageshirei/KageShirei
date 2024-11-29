//! Handle the terminate command

use kageshirei_communication_protocol::{
    communication::{AgentCommands, SimpleAgentCommand},
    Metadata,
};
use srv_mod_config::sse::common_server_state::{EventType, SseEvent};
use srv_mod_entity::{
    active_enums::LogLevel,
    entities::{agent_command, logs},
    sea_orm::{prelude::*, ActiveValue::Set},
};
use tracing::{debug, instrument};

use crate::command_handler::CommandHandlerArguments;

/// Handle the terminate command
#[instrument]
pub async fn handle(config: CommandHandlerArguments) -> Result<String, String> {
    debug!("Terminal command received");

    let db = config.db_pool.clone();

    let pending_log = logs::ActiveModel {
        level: Set(LogLevel::Warning),
        title: Set("Terminate issued".to_owned()),
        message: Set(Some("Agent termination requested".to_owned())),
        extra: Set(Some(serde_json::json!({
            "session": config.session.hostname,
            "ran_by": config.user.username,
        }))),
        ..Default::default()
    };

    // create the command id so we can reference it in the metadata
    let mut agent_command = agent_command::ActiveModel {
        agent_id: Set(config.session.session_id.clone()),
        ..Default::default()
    };
    agent_command.command = Set(serde_json::to_value(SimpleAgentCommand {
        op:       AgentCommands::Terminate,
        metadata: Metadata {
            request_id: agent_command.id.clone().unwrap(),
            command_id: AgentCommands::Terminate.to_string(),
            agent_id:   config.session.session_id.clone(),
            path:       None,
        },
    })
    .map_err(|e| e.to_string())?);

    let (command_request, log_insertion) = tokio::join!(agent_command.insert(&db), pending_log.insert(&db));

    command_request.map_err(|e| e.to_string())?;
    let log = log_insertion.map_err(|e| e.to_string())?;

    // broadcast the log
    config
        .broadcast_sender
        .send(SseEvent {
            data:  serde_json::to_string(&log).map_err(|e| e.to_string())?,
            event: EventType::Log,
            id:    Some(log.id),
        })
        .map_err(|e| e.to_string())?;

    // Signal the frontend terminal emulator to clear the terminal screen
    Ok("Command issued successfully".to_owned())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use chrono::Utc;
    use kageshirei_communication_protocol::{
        communication::{AgentCommands, SimpleAgentCommand},
        Metadata,
        NetworkInterface,
        NetworkInterfaceArray,
    };
    use srv_mod_entity::{
        active_enums::AgentIntegrity,
        entities::{agent, agent_command, logs},
        sea_orm::{ActiveValue, Database, DatabaseConnection, TransactionTrait},
    };
    use tokio::sync::{broadcast, mpsc};

    use super::*;
    use crate::command_handler::{HandleArguments, HandleArgumentsSession, HandleArgumentsUser};

    async fn cleanup(db: DatabaseConnection) {
        db.transaction::<_, (), DbErr>(|txn| {
            Box::pin(async move {
                agent::Entity::delete_many().exec(txn).await.unwrap();
                agent_command::Entity::delete_many()
                    .exec(txn)
                    .await
                    .unwrap();
                logs::Entity::delete_many().exec(txn).await.unwrap();

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
            id:                 Set("agent1".to_owned()),
            pid:                Set(1),
            secret:             Set("test".to_owned()),
            cwd:                Set("test".to_owned()),
            server_secret:      Set("test".to_owned()),
            operating_system:   Set("test".to_owned()),
            integrity:          Set(AgentIntegrity::Medium),
            updated_at:         Set(Utc::now().naive_utc()),
            domain:             Set(Some("test".to_owned())),
            hostname:           Set("test-hostname".to_owned()),
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
            created_at:         Set(Utc::now().naive_utc()),
        })
        .exec_with_returning(&db_pool)
        .await
        .unwrap();

        db_pool
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_handle_terminate_command() {
        // Mock database setup
        let db = init().await;

        // Mock broadcast channel
        let (sender, mut receiver) = broadcast::channel(1);

        // Create command handler arguments
        let config = Arc::new(HandleArguments {
            session:          HandleArgumentsSession {
                session_id: "agent1".to_owned(),
                hostname:   "test-hostname".to_owned(),
            },
            user:             HandleArgumentsUser {
                user_id:  "test".to_owned(),
                username: "test".to_owned(),
            },
            db_pool:          db.clone(),
            broadcast_sender: sender,
        });

        // Call the handle function
        let result = handle(config.clone()).await;

        // Verify result
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Command issued successfully");

        // Verify agent_command was inserted
        let agent_commands = agent_command::Entity::find()
            .filter(agent_command::Column::AgentId.eq("agent1"))
            .all(&db)
            .await
            .unwrap();
        assert_eq!(agent_commands.len(), 1);
        let inserted_command = &agent_commands[0];
        let command: SimpleAgentCommand = serde_json::from_value(inserted_command.command.clone()).unwrap();
        assert_eq!(command.op, AgentCommands::Terminate);

        // Verify logs were inserted
        let logs = logs::Entity::find().all(&db).await.unwrap();
        assert_eq!(logs.len(), 1);
        let inserted_log = &logs[0];
        assert_eq!(inserted_log.title, "Terminate issued");

        // Verify broadcast message was sent
        if let Ok(sent_event) = receiver.recv().await {
            assert_eq!(sent_event.event, EventType::Log);
            assert!(sent_event.data.contains("Terminate issued"));
        }
        else {
            panic!("No broadcast message was sent");
        }
    }
}
