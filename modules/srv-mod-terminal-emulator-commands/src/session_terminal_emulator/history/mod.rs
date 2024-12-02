//! Module for managing the terminal session history

use clap::{Args, Subcommand};
use serde::Serialize;
use srv_mod_entity::{
    entities::{terminal_history, user},
    sea_orm::{prelude::*, Condition, Order, QueryOrder as _},
};
use tracing::{debug, instrument};

use crate::{
    command_handler::CommandHandlerArguments,
    post_process_result::PostProcessResult,
    session_terminal_emulator::history::restore::TerminalSessionHistoryRestoreArguments,
};

mod restore;

/// Terminal session arguments for the global session terminal
#[derive(Args, Debug, PartialEq, Eq, Serialize)]
pub struct TerminalSessionHistoryArguments {
    /// Display the full history including the commands hidden using `clear`
    #[arg(short, long)]
    pub full:    bool,
    /// The subcommand to run if any
    #[command(subcommand)]
    pub command: Option<HistorySubcommands>,
}

#[derive(Subcommand, Debug, PartialEq, Eq, Serialize)]
pub enum HistorySubcommands {
    /// Restore a list of commands from the history
    #[serde(rename = "restore")]
    Restore(TerminalSessionHistoryRestoreArguments),
}

/// Handle the history command
#[instrument]
pub async fn handle(config: CommandHandlerArguments, args: &TerminalSessionHistoryArguments) -> Result<String, String> {
    debug!("Terminal command received");

    if let Some(ref subcommand) = args.command {
        #[expect(clippy::pattern_type_mismatch, reason = "Cannot move out of self")]
        match subcommand {
            HistorySubcommands::Restore(args) => restore::handle(config.clone(), args).await,
        }
    }
    else {
        let db = config.db_pool.clone();

        let mut conditions = Condition::all().add(terminal_history::Column::SessionId.eq(&config.session.session_id));

        // If the full flag is not set, filter out the commands that have been deleted
        if !args.full {
            // Select only commands that are not deleted or have been restored after deletion
            // deleted_at == null || (restored_at != null && restored_at > deleted_at)
            conditions = conditions.add(
                Condition::any()
                    .add(terminal_history::Column::DeletedAt.is_null())
                    .add(
                        Condition::all()
                            .add(terminal_history::Column::RestoredAt.is_not_null())
                            .add(
                                Expr::col(terminal_history::Column::RestoredAt)
                                    .gt(Expr::col(terminal_history::Column::DeletedAt)),
                            ),
                    ),
            );
        }

        let history = terminal_history::Entity::find()
            .find_also_related(user::Entity)
            .filter(conditions)
            .order_by(terminal_history::Column::CreatedAt, Order::Asc)
            .all(&db)
            .await
            .map_err(|e| e.to_string())?;

        Ok(serde_json::to_string(&PostProcessResult {
            r#type: "history".to_owned(),
            data:   history,
        })
        .map_err(|e| e.to_string())?)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use srv_mod_entity::{
        entities::logs,
        sea_orm::{prelude::*, Database, DatabaseConnection, EntityTrait, TransactionTrait},
    };
    use tokio::{self, sync::broadcast};

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
    async fn test_handle_history_full() {
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

        let args = TerminalSessionHistoryArguments {
            full:    true,
            command: None,
        };

        let result = handle(config, &args).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("\"type\":\"history\""));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_handle_history_filtered() {
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
        let args = TerminalSessionHistoryArguments {
            full:    false,
            command: None,
        };

        let result = handle(config, &args).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("\"type\":\"history\""));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_handle_restore_subcommand() {
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
        let restore_args = TerminalSessionHistoryRestoreArguments {
            command_ids: vec![1, 2],
        };

        let args = TerminalSessionHistoryArguments {
            full:    false,
            command: Some(HistorySubcommands::Restore(restore_args)),
        };

        let result = handle(config, &args).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Restored"));
    }
}
