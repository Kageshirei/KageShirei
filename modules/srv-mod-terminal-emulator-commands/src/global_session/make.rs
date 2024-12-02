use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{debug, instrument};

use crate::{
    command_handler::CommandHandlerArguments,
    global_session::make::notification::TerminalSessionMakeNotificationArguments,
    post_process_result::PostProcessResult,
};

mod notification;

/// Terminal session arguments for the global session terminal
#[derive(Args, Debug, PartialEq, Serialize)]
pub struct TerminalSessionMakeArguments {
    #[command(subcommand)]
    pub command: MakeSubcommands,
}

#[derive(Subcommand, Debug, PartialEq, Serialize)]
pub enum MakeSubcommands {
    /// Make a new notification and broadcast it to all connected clients
    #[serde(rename = "notification")]
    Notification(TerminalSessionMakeNotificationArguments),
}

/// Handle the history command
#[instrument]
pub async fn handle(config: CommandHandlerArguments, args: &TerminalSessionMakeArguments) -> Result<String, String> {
    debug!("Terminal command received");

    match &args.command {
        MakeSubcommands::Notification(args) => notification::handle(config.clone(), args).await,
    }
}

#[cfg(test)]
mod tests {
    use kageshirei_srv_test_helper::tests::{drop_database, generate_test_user, make_pool};
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
