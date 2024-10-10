use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{debug, instrument};

use srv_mod_config::sse::common_server_state::{EventType, SseEvent};
use srv_mod_database::{diesel, Pool};
use srv_mod_database::diesel::{BoolExpressionMethods, ExpressionMethods, NullableExpressionMethods, Queryable, QueryDsl, SelectableHelper};
use srv_mod_database::diesel::query_builder::BoxedSelectStatement;
use srv_mod_database::diesel::sql_types::Varchar;
use srv_mod_database::diesel_async::RunQueryDsl;
use srv_mod_database::models::command::{Command, HistoryRecord};
use srv_mod_database::models::notification::{CreateNotification, Notification};
use srv_mod_database::schema::commands;
use srv_mod_database::schema::notifications;
use srv_mod_database::schema::users;
use srv_mod_database::schema_extension::LogLevel;

use crate::command_handler::CommandHandlerArguments;
use crate::post_process_result::PostProcessResult;

/// Terminal session arguments for the global session terminal
#[derive(Args, Debug, PartialEq, Serialize)]
pub struct TerminalSessionMakeNotificationArguments {
	/// The level of the notification to send
	#[arg(short, long)]
	pub level: LogLevel,
	/// The title of the notification
	#[arg(short, long)]
	pub title: String,
	/// The message of the notification
	#[arg(short, long)]
	pub message: String,
}

/// Handle the history command
#[instrument]
pub async fn handle(config: CommandHandlerArguments, args: &TerminalSessionMakeNotificationArguments) -> anyhow::Result<String> {
	debug!("Terminal command received");

	let mut connection = config.db_pool.get().await?;

	let inserted_notification = diesel::insert_into(notifications::table)
		.values(
			CreateNotification::new(args.level.clone())
				.with_title(args.title.as_str())
				.with_message(args.message.as_str())
		)
		.returning(Notification::as_select())
		.get_result(&mut connection)
		.await?;

	config.broadcast_sender.send(SseEvent {
		event: EventType::Notification,
		id: Some(inserted_notification.id.clone()),
		data: serde_json::to_string(&inserted_notification)?,
	})?;

	Ok("Notification sent".to_string())
}

#[cfg(test)]
mod tests {
	use serial_test::serial;

	use rs2_srv_test_helper::tests::{drop_database, generate_test_user, make_pool};
	use srv_mod_database::models::command::CreateCommand;

	use crate::session_terminal_emulator::history::TerminalSessionHistoryArguments;

	use super::*;

	#[tokio::test]
	#[serial]
	async fn test_handle_history_display() {
		drop_database().await;
		let db_pool = make_pool().await;

		let user = generate_test_user(db_pool.clone()).await;

		let session_id_v = "global";
		let args = TerminalSessionHistoryArguments { command: None };

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