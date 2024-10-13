use clap::Args;
use serde::Serialize;
use serde_json::json;
use tracing::{debug, instrument};

use srv_mod_config::sse::common_server_state::{EventType, SseEvent};

use crate::command_handler::CommandHandlerArguments;
use srv_mod_entity::active_enums::LogLevel;
use srv_mod_entity::entities::{logs, terminal_history};
use srv_mod_entity::sea_orm::prelude::*;

/// Terminal session arguments for the global session terminal
#[derive(Args, Debug, PartialEq, Serialize)]
pub struct TerminalSessionHistoryRestoreArguments {
	/// The list of command ids to restore
	pub command_ids: Vec<i64>,
}

/// Handle the clear command
#[instrument]
pub async fn handle(config: CommandHandlerArguments, args: &TerminalSessionHistoryRestoreArguments) -> Result<String, String> {
	debug!("Terminal command received");

	let db = config.db_pool.clone();

	// clear commands marking them as deleted (soft delete)
	let result = terminal_history::Entity::update_many()
		.filter(terminal_history::Column::SessionId.eq(&config.session.session_id))
		.filter(terminal_history::Column::SequenceCounter.eq_any(&args.command_ids))
		.col_expr(terminal_history::Column::RestoredAt, Expr::value(chrono::Utc::now()))
		.exec(&db)
		.await
		.map_err(|e| e.to_string())?;

	let message = format!("Restored {} command(s)", result.rows_affected);

	// create a log entry and save it
	let log = logs::ActiveModel {
		level: LogLevel::Info,
		title: "Command(s) restored".to_string(),
		message: Some(message.clone()),
		extra: Some(json!({
			"session": config.session.hostname,
			"ran_by": config.user.username,
		})),
		..Default::default()
	}
		.insert(&db)
		.await
		.map_err(|e| e.to_string())?;

	// broadcast the log
	config.broadcast_sender.send(SseEvent {
		data: serde_json::to_string(&log)?,
		event: EventType::Log,
		id: Some(log.id),
	})?;

	// Signal the frontend terminal emulator to clear the terminal screen
	Ok(message)
}

#[cfg(test)]
mod tests {
	use chrono::SubsecRound;
	use serial_test::serial;

	use rs2_srv_test_helper::tests::{drop_database, generate_test_user, make_pool};
	use srv_mod_database::models::command::CreateCommand;

	use crate::session_terminal_emulator::history::{HistoryRecord, TerminalSessionHistoryArguments};

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

		let mut to_restore = vec![];

		// open a scope to automatically drop the connection once exited
		{
			let mut connection = binding.get().await.unwrap();

			let mut cmd = CreateCommand::new(user.id.clone(), session_id_v.to_string());
			cmd.command = "ls".to_string();
			cmd.deleted_at = Some(chrono::Utc::now().trunc_subsecs(6));
			to_restore.push(cmd.id.clone());

			// Insert a dummy command
			let inserted_command_0 = diesel::insert_into(commands::table)
				.values(&cmd)
				.returning(Command::as_select())
				.get_result(&mut connection)
				.await
				.unwrap();

			assert_eq!(inserted_command_0.deleted_at, cmd.deleted_at);
			assert_eq!(inserted_command_0.restored_at, None);

			let mut cmd = CreateCommand::new(user.id.clone(), session_id_v.to_string());
			cmd.command = "pwd".to_string();
			cmd.deleted_at = Some(chrono::Utc::now().trunc_subsecs(6));
			to_restore.push(cmd.id.clone());

			let inserted_command_1 = diesel::insert_into(commands::table)
				.values(&cmd)
				.returning(Command::as_select())
				.get_result(&mut connection)
				.await
				.unwrap();

			assert_eq!(inserted_command_1.deleted_at, cmd.deleted_at);
			assert_eq!(inserted_command_1.restored_at, None);

			let mut cmd = CreateCommand::new(user.id.clone(), session_id_v.to_string());
			cmd.command = "cd ..".to_string();

			let inserted_command_2 = diesel::insert_into(commands::table)
				.values(&cmd)
				.returning(Command::as_select())
				.get_result(&mut connection)
				.await
				.unwrap();

			assert_eq!(inserted_command_2.deleted_at, None);
			assert_eq!(inserted_command_2.restored_at, None);
		}

		assert_eq!(to_restore.len(), 2);

		let result = crate::session_terminal_emulator::history::handle(session_id_v, db_pool.clone(), &args).await;

		assert!(result.is_ok());
		let result = result.unwrap();
		let deserialized = serde_json::from_str::<Vec<HistoryRecord>>(result.as_str()).unwrap();

		assert_eq!(deserialized.len(), 1);
		assert_eq!(deserialized[0].command, "cd ..");

		let args = TerminalSessionHistoryRestoreArguments { command_ids: to_restore };
		let restored = handle(session_id_v, db_pool.clone(), &args).await;

		assert!(restored.is_ok());
		let restored = restored.unwrap();
		assert_eq!(restored, "Restored 2 commands");

		let args = TerminalSessionHistoryArguments { command: None };
		let result = crate::session_terminal_emulator::history::handle(session_id_v, db_pool.clone(), &args).await;

		assert!(result.is_ok());
		let result = result.unwrap();
		let deserialized = serde_json::from_str::<Vec<HistoryRecord>>(result.as_str()).unwrap();

		assert_eq!(deserialized.len(), 3);
		assert_eq!(deserialized[0].command, "ls");
		assert_eq!(deserialized[1].command, "pwd");
		assert_eq!(deserialized[2].command, "cd ..");

		drop_database().await;
	}
}