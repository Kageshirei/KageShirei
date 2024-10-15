use chrono::Utc;
use clap::Args;
use serde::Serialize;
use srv_mod_config::sse::common_server_state::{EventType, SseEvent};
use srv_mod_entity::{
    active_enums::LogLevel,
    entities::{logs, terminal_history},
    sea_orm::{prelude::*, ActiveValue::Set},
};
use tracing::{debug, instrument};

use crate::command_handler::CommandHandlerArguments;

/// Terminal session arguments for the global session terminal
#[derive(Args, Debug, PartialEq, Eq, Serialize)]
pub struct TerminalSessionClearArguments {
    /// Delete the command permanently, removing it from the database.
    ///
    /// This is a hard delete and cannot be undone.
    #[arg(short, long)]
    pub permanent: bool,
}

/// Handle the clear command
#[instrument]
pub async fn handle(config: CommandHandlerArguments, args: &TerminalSessionClearArguments) -> Result<String, String> {
    debug!("Terminal command received");

    let db = config.db_pool.clone();

    let log: logs::Model;

    if !args.permanent {
        // clear commands marking them as deleted (soft delete)
        let pending_log = logs::ActiveModel {
            level: Set(LogLevel::Warning),
            title: Set("Soft clean".to_owned()),
            message: Set(Some("Commands have been soft cleaned.".to_owned())),
            extra: Set(Some(serde_json::json!({
                "session": config.session.hostname,
                "ran_by": config.user.username,
            }))),
            ..Default::default()
        };

        let (update, log_insertion) = tokio::join!(
            terminal_history::Entity::update_many()
                .filter(terminal_history::Column::SessionId.eq(&config.session.session_id))
                .col_expr(terminal_history::Column::DeletedAt, Expr::value(Utc::now()))
                .col_expr(
                    terminal_history::Column::RestoredAt,
                    Expr::value(None::<DateTime>)
                )
                .exec(&db),
            pending_log.insert(&db)
        );

        update.map_err(|e| e.to_string())?;
        log = log_insertion.map_err(|e| e.to_string())?;
    }
    else {
        // clear commands permanently
        let pending_log = logs::ActiveModel {
            level: Set(LogLevel::Warning),
            title: Set("Permanent clean".to_owned()),
            message: Set(Some("Commands have been permanently cleaned.".to_owned())),
            extra: Set(Some(serde_json::json!({
                "session": config.session.hostname,
                "ran_by": config.user.username,
            }))),
            ..Default::default()
        };
        let (delete, log_insertion) = tokio::join!(
            terminal_history::Entity::delete_many()
                .filter(terminal_history::Column::SessionId.eq(&config.session.session_id))
                .exec(&db),
            pending_log.insert(&db)
        );

        delete.map_err(|e| e.to_string())?;
        log = log_insertion.map_err(|e| e.to_string())?;
    }

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
    Ok("__TERMINAL_EMULATOR_INTERNAL_HANDLE_CLEAR__".to_owned())
}

#[cfg(test)]
mod tests {
    use kageshirei_srv_test_helper::tests::*;
    use serial_test::serial;
    use srv_mod_database::models::command::CreateCommand;

    use super::*;
    use crate::session_terminal_emulator::clear::TerminalSessionClearArguments;

    #[tokio::test]
    #[serial]
    async fn test_handle_soft_delete() {
        drop_database().await;
        let db_pool = make_pool().await;

        let user = generate_test_user(db_pool.clone()).await;

        let session_id_v = "global";
        let args = TerminalSessionClearArguments {
            permanent: false,
        };

        let binding = db_pool.clone();

        // open a scope to automatically drop the connection once exited
        {
            let mut connection = binding.get().await.unwrap();

            // Insert a dummy command
            let inserted_command_0 = diesel::insert_into(commands)
                .values(&CreateCommand::new(
                    user.id.clone(),
                    session_id_v.to_string(),
                ))
                .returning(Command::as_select())
                .get_result(&mut connection)
                .await
                .unwrap();

            assert_eq!(inserted_command_0.deleted_at, None);
            assert_eq!(inserted_command_0.restored_at, None);

            let inserted_command_1 = diesel::insert_into(commands)
                .values(&CreateCommand::new(
                    user.id.clone(),
                    session_id_v.to_string(),
                ))
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
        let retrieved_commands = commands
            .select(Command::as_select())
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
        let args = TerminalSessionClearArguments {
            permanent: true,
        };

        let binding = db_pool.clone();

        // open a scope to automatically drop the connection once exited
        {
            let mut connection = binding.get().await.unwrap();

            // Insert a dummy command
            let inserted_command_0 = diesel::insert_into(commands)
                .values(&CreateCommand::new(
                    user.id.clone(),
                    session_id_v.to_string(),
                ))
                .returning(Command::as_select())
                .get_result(&mut connection)
                .await
                .unwrap();

            assert_eq!(inserted_command_0.deleted_at, None);
            assert_eq!(inserted_command_0.restored_at, None);

            let inserted_command_1 = diesel::insert_into(commands)
                .values(&CreateCommand::new(
                    user.id.clone(),
                    session_id_v.to_string(),
                ))
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
        let retrieved_commands = commands
            .select(Command::as_select())
            .filter(session_id.eq(session_id_v))
            .get_results(&mut connection)
            .await
            .unwrap();

        assert_eq!(retrieved_commands.len(), 0);

        drop_database().await;
    }
}
