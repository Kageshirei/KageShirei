use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};
use tracing::{debug, instrument};

use srv_mod_database::{diesel, Pool};
use srv_mod_database::diesel::{BoolExpressionMethods, ExpressionMethods, NullableExpressionMethods, Queryable, QueryDsl, SelectableHelper};
use srv_mod_database::diesel_async::RunQueryDsl;
use srv_mod_database::models::command::Command;
use srv_mod_database::schema::commands;
use srv_mod_database::schema::users;

use crate::session_terminal_emulator::history::restore::TerminalSessionHistoryRestoreArguments;

mod restore;

/// Terminal session arguments for the global session terminal
#[derive(Args, Debug, PartialEq, Serialize)]
pub struct TerminalSessionHistoryArguments {
	#[command(subcommand)]
	pub command: Option<HistorySubcommands>,
}

#[derive(Subcommand, Debug, PartialEq, Serialize)]
pub enum HistorySubcommands {
	/// Restore a list of commands from the history
	#[serde(rename = "restore")]
	Restore(TerminalSessionHistoryRestoreArguments),
}

#[derive(Debug, Serialize, Deserialize, Queryable)]
pub struct HistoryRecord {
	id: String,
	command: String,
	exit_code: Option<i32>,
	ran_by: String,
	created_at: chrono::DateTime<chrono::Utc>,
}

/// Handle the history command
#[instrument]
pub async fn handle(session_id_v: &str, db_pool: Pool, args: &TerminalSessionHistoryArguments) -> anyhow::Result<String> {
	debug!("Terminal command received");

	let mut connection = db_pool
		.get()
		.await
		.map_err(|_| anyhow::anyhow!("Failed to get a connection from the pool"))?;

	if let Some(subcommand) = &args.command {
		match subcommand {
			HistorySubcommands::Restore(args) => {
				restore::handle(session_id_v, db_pool.clone(), args).await
			}
		}
	} else {
		let history = commands::table.inner_join(users::table)
		                             .select((
			                             commands::id,
			                             commands::command,
			                             commands::exit_code.nullable(),
			                             users::username,
			                             commands::created_at
		                             ))
		                             .filter(commands::session_id.eq(session_id_v))
		                             .filter(
			                             // Select only commands that are not deleted or have been restored after deletion
			                             // deleted_at == null || (restored_at != null && restored_at > deleted_at)
			                             commands::deleted_at.is_null()
			                                                 .or(
				                                                 commands::restored_at.is_not_null()
				                                                                      .and(commands::restored_at.gt(commands::deleted_at))
			                                                 )
		                             )
		                             .order_by(commands::created_at.asc())
		                             .get_results::<HistoryRecord>(&mut connection)
		                             .await
		                             .map_err(|e| anyhow::anyhow!(e))?;

		Ok(
			serde_json::to_string(&history)
				.map_err(|e| anyhow::anyhow!(e))?
		)
	}
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