use clap::Args;
use serde::Serialize;
use tracing::{debug, instrument};

use crate::command_handler::CommandHandlerArguments;
use rs2_communication_protocol::communication_structs::agent_commands::AgentCommands;
use rs2_communication_protocol::communication_structs::simple_agent_command::SimpleAgentCommand;
use rs2_communication_protocol::metadata::Metadata;
use srv_mod_config::sse::common_server_state::{EventType, SseEvent};
use srv_mod_entity::active_enums::LogLevel;
use srv_mod_entity::entities::{agent_command, logs};
use srv_mod_entity::sea_orm::prelude::*;
use srv_mod_entity::sea_orm::ActiveValue::Set;

/// Handle the terminate command
#[instrument]
pub async fn handle(config: CommandHandlerArguments) -> Result<String, String> {
    debug!("Terminal command received");

    let db = config.db_pool.clone();

    let pending_log = logs::ActiveModel {
        level: Set(LogLevel::Warning),
        title: Set("Terminate issued".to_string()),
        message: Set(Some("Agent termination requested".to_string())),
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
    agent_command.command = Set(
        serde_json::to_value(SimpleAgentCommand {
            op: AgentCommands::Terminate,
            metadata: Metadata {
                request_id: agent_command.id.clone().unwrap(),
                command_id: AgentCommands::Terminate.to_string(),
                agent_id: config.session.session_id.clone(),
                path: None,
            },
        }).map_err(|e| e.to_string())?,
    );

    let (command_request, log_insertion) = tokio::join!(
		agent_command.insert(&db),
		pending_log.insert(&db)
	);

    command_request.map_err(|e| e.to_string())?;
    let log = log_insertion.map_err(|e| e.to_string())?;

    // broadcast the log
    config.broadcast_sender.send(SseEvent {
        data: serde_json::to_string(&log).map_err(|e| e.to_string())?,
        event: EventType::Log,
        id: Some(log.id),
    }).map_err(|e| e.to_string())?;

    // Signal the frontend terminal emulator to clear the terminal screen
    Ok("Command issued successfully".to_string())
}

#[cfg(test)]
mod tests {
    use serial_test::serial;

    use rs2_srv_test_helper::tests::*;
    use srv_mod_database::models::command::CreateCommand;

    use crate::session_terminal_emulator::clear::TerminalSessionClearArguments;

    use super::*;

    #[tokio::test]
    #[serial]
    async fn test_handle_soft_delete() {
        drop_database().await;
        let db_pool = make_pool().await;

        let user = generate_test_user(db_pool.clone()).await;

        let session_id_v = "global";
        let args = TerminalSessionClearArguments { permanent: false };

        let binding = db_pool.clone();

        // open a scope to automatically drop the connection once exited
        {
            let mut connection = binding.get().await.unwrap();

            // Insert a dummy command
            let inserted_command_0 = diesel::insert_into(commands)
                .values(&CreateCommand::new(user.id.clone(), session_id_v.to_string()))
                .returning(Command::as_select())
                .get_result(&mut connection)
                .await
                .unwrap();

            assert_eq!(inserted_command_0.deleted_at, None);
            assert_eq!(inserted_command_0.restored_at, None);

            let inserted_command_1 = diesel::insert_into(commands)
                .values(&CreateCommand::new(user.id.clone(), session_id_v.to_string()))
                .returning(Command::as_select())
                .get_result(&mut connection)
                .await
                .unwrap();

            assert_eq!(inserted_command_1.deleted_at, None);
            assert_eq!(inserted_command_1.restored_at, None);
        }

        let result = handle(session_id_v, db_pool, &args).await;
        assert!(result.is_ok());

        let mut connection = binding.get().await.unwrap();
        let retrieved_commands = commands.select(Command::as_select())
                                         .filter(session_id.eq(session_id_v))
                                         .get_results(&mut connection)
                                         .await
                                         .unwrap();

        assert_eq!(retrieved_commands.len(), 2);
        assert!(retrieved_commands.iter().all(|c| c.deleted_at.is_some()));
        assert!(retrieved_commands.iter().all(|c| c.restored_at.is_none()));

        drop_database().await;
    }

    #[tokio::test]
    #[serial]
    async fn test_handle_hard_delete() {
        drop_database().await;
        let db_pool = make_pool().await;

        let user = generate_test_user(db_pool.clone()).await;

        let session_id_v = "global";
        let args = TerminalSessionClearArguments { permanent: true };

        let binding = db_pool.clone();

        // open a scope to automatically drop the connection once exited
        {
            let mut connection = binding.get().await.unwrap();

            // Insert a dummy command
            let inserted_command_0 = diesel::insert_into(commands)
                .values(&CreateCommand::new(user.id.clone(), session_id_v.to_string()))
                .returning(Command::as_select())
                .get_result(&mut connection)
                .await
                .unwrap();

            assert_eq!(inserted_command_0.deleted_at, None);
            assert_eq!(inserted_command_0.restored_at, None);

            let inserted_command_1 = diesel::insert_into(commands)
                .values(&CreateCommand::new(user.id.clone(), session_id_v.to_string()))
                .returning(Command::as_select())
                .get_result(&mut connection)
                .await
                .unwrap();

            assert_eq!(inserted_command_1.deleted_at, None);
            assert_eq!(inserted_command_1.restored_at, None);
        }

        let result = handle(session_id_v, db_pool, &args).await;
        assert!(result.is_ok());

        let mut connection = binding.get().await.unwrap();
        let retrieved_commands = commands.select(Command::as_select())
                                         .filter(session_id.eq(session_id_v))
                                         .get_results(&mut connection)
                                         .await
                                         .unwrap();

        assert_eq!(retrieved_commands.len(), 0);

        drop_database().await;
    }
}