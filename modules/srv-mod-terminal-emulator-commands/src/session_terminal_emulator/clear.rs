//! Handle the clear command for the terminal emulator

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

    let log: logs::Model = if !args.permanent {
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
        log_insertion.map_err(|e| e.to_string())?
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
        log_insertion.map_err(|e| e.to_string())?
    };

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
    use std::sync::Arc;

    use chrono::Utc;
    use srv_mod_config::sse::common_server_state::{EventType, SseEvent};
    use srv_mod_entity::sea_orm::{
        ActiveValue::Set,
        Database,
        DatabaseConnection,
        EntityTrait,
        QueryFilter,
        TransactionTrait,
    };
    use tokio::sync::{broadcast, mpsc};

    use super::*;
    use crate::command_handler::{HandleArguments, HandleArgumentsSession, HandleArgumentsUser};

    async fn cleanup(db: DatabaseConnection) {
        db.transaction::<_, (), DbErr>(|txn| {
            Box::pin(async move {
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

        db_pool
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_handle_soft_delete() {
        // Mock database setup
        let db = init().await;

        // Mock broadcast channel
        let (sender, mut receiver) = broadcast::channel(1);

        // Create command handler arguments
        let config = Arc::new(HandleArguments {
            session:          HandleArgumentsSession {
                session_id: "test".to_owned(),
                hostname:   "test".to_owned(),
            },
            user:             HandleArgumentsUser {
                user_id:  "test".to_owned(),
                username: "test".to_owned(),
            },
            db_pool:          db,
            broadcast_sender: sender,
        });

        let args = TerminalSessionClearArguments {
            permanent: false,
        };

        let result = handle(config, &args).await;
        assert!(result.is_ok());
        let message = result.unwrap();
        assert_eq!(message, "__TERMINAL_EMULATOR_INTERNAL_HANDLE_CLEAR__");

        if let Ok(SseEvent {
            event,
            data,
            id: _,
        }) = receiver.recv().await
        {
            assert_eq!(event, EventType::Log);
            assert!(data.contains("Soft clean"));
        }
        else {
            panic!("Expected SSE event not received");
        }
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_handle_permanent_delete() {
        // Mock database setup
        let db = init().await;

        // Mock broadcast channel
        let (sender, mut receiver) = broadcast::channel(1);

        // Create command handler arguments
        let config = Arc::new(HandleArguments {
            session:          HandleArgumentsSession {
                session_id: "test".to_owned(),
                hostname:   "test".to_owned(),
            },
            user:             HandleArgumentsUser {
                user_id:  "test".to_owned(),
                username: "test".to_owned(),
            },
            db_pool:          db,
            broadcast_sender: sender,
        });

        let args = TerminalSessionClearArguments {
            permanent: true,
        };

        let result = handle(config, &args).await;
        assert!(result.is_ok());
        let message = result.unwrap();
        assert_eq!(message, "__TERMINAL_EMULATOR_INTERNAL_HANDLE_CLEAR__");

        if let Ok(SseEvent {
            event,
            data,
            id: _,
        }) = receiver.recv().await
        {
            assert_eq!(event, EventType::Log);
            assert!(data.contains("Permanent clean"));
        }
        else {
            panic!("Expected SSE event not received");
        }
    }
}
