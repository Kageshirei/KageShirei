use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};
use serde_json::json;
use srv_mod_entity::{
    entities::{terminal_history, user},
    sea_orm::{prelude::*, sea_query::Alias, Condition, IntoIdentity, IntoSimpleExpr, Order, QueryOrder},
};
use tracing::{debug, instrument};

use crate::{
    command_handler::CommandHandlerArguments,
    post_process_result::PostProcessResult,
    session_terminal_emulator::history::restore::TerminalSessionHistoryRestoreArguments,
};

mod restore;

/// Terminal session arguments for the global session terminal
#[derive(Args, Debug, PartialEq, Serialize)]
pub struct TerminalSessionHistoryArguments {
    /// Display the full history including the commands hidden using `clear`
    #[arg(short, long)]
    pub full:    bool,
    #[command(subcommand)]
    pub command: Option<HistorySubcommands>,
}

#[derive(Subcommand, Debug, PartialEq, Serialize)]
pub enum HistorySubcommands {
    /// Restore a list of commands from the history
    #[serde(rename = "restore")]
    Restore(TerminalSessionHistoryRestoreArguments),
}

/// Handle the history command
#[instrument]
pub async fn handle(config: CommandHandlerArguments, args: &TerminalSessionHistoryArguments) -> Result<String, String> {
    debug!("Terminal command received");

    if let Some(subcommand) = &args.command {
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
            r#type: "history".to_string(),
            data:   history,
        })
        .map_err(|e| e.to_string())?)
    }
}

#[cfg(test)]
mod tests {
    use rs2_srv_test_helper::tests::{drop_database, generate_test_user, make_pool};
    use serial_test::serial;
    use srv_mod_database::models::command::CreateCommand;

    use super::*;
    use crate::session_terminal_emulator::history::TerminalSessionHistoryArguments;

    #[tokio::test]
    #[serial]
    async fn test_handle_history_display() {
        drop_database().await;
        let db_pool = make_pool().await;

        let user = generate_test_user(db_pool.clone()).await;

        let session_id_v = "global";
        let args = TerminalSessionHistoryArguments {
            command: None,
        };

        let binding = db_pool.clone();

        // open a scope to automatically drop the connection once exited
        {
            let mut connection = binding.get().await.unwrap();

            let mut cmd = CreateCommand::new(user.id.clone(), session_id_v.to_string());
            cmd.command = "ls".to_string();

            // Insert a dummy command
            let inserted_command_0 = diesel::insert_into(commands::table)
                .values(&cmd)
                .returning(Command::as_select())
                .get_result(&mut connection)
                .await
                .unwrap();

            assert_eq!(inserted_command_0.deleted_at, None);
            assert_eq!(inserted_command_0.restored_at, None);

            let mut cmd = CreateCommand::new(user.id.clone(), session_id_v.to_string());
            cmd.command = "pwd".to_string();

            let inserted_command_1 = diesel::insert_into(commands::table)
                .values(&cmd)
                .returning(Command::as_select())
                .get_result(&mut connection)
                .await
                .unwrap();

            assert_eq!(inserted_command_1.deleted_at, None);
            assert_eq!(inserted_command_1.restored_at, None);
        }

        let result = handle(session_id_v, db_pool, &args).await;

        assert!(result.is_ok());
        let result = result.unwrap();
        let deserialized = serde_json::from_str::<Vec<HistoryRecord>>(result.as_str()).unwrap();

        assert_eq!(deserialized.len(), 2);
        assert_eq!(deserialized[0].command, "ls");
        assert_eq!(deserialized[1].command, "pwd");

        drop_database().await;
    }
}
