use clap::Args;
use serde::Serialize;
use tracing::{debug, instrument};

use srv_mod_database::{diesel, Pool};
use srv_mod_database::diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use srv_mod_database::diesel::internal::derives::multiconnection::chrono;
use srv_mod_database::diesel_async::RunQueryDsl;
use srv_mod_database::models::command::Command;
use srv_mod_database::schema::commands::dsl::*;

/// Terminal session arguments for the global session terminal
#[derive(Args, Debug, PartialEq, Serialize)]
pub struct TerminalSessionClearArguments {
	/// Delete the command permanently, removing it from the database.
	///
	/// This is a hard delete and cannot be undone.
	#[arg(short, long)]
	pub permanent: bool,
}

/// Handle the clear command
#[instrument]
pub async fn handle(session_id_v: &str, db_pool: Pool, args: &TerminalSessionClearArguments) -> anyhow::Result<String> {
	debug!("Terminal command received");

	let mut connection = db_pool
		.get()
		.await
		.map_err(|_| anyhow::anyhow!("Failed to get a connection from the pool"))?;

	if !args.permanent {
		// clear commands marking them as deleted (soft delete)
		diesel::update(commands)
			.filter(session_id.eq(session_id_v))
			.set((
				deleted_at.eq(chrono::Utc::now()),
				restored_at.eq(None::<chrono::DateTime<chrono::Utc>>),
			))
			.execute(&mut connection)
			.await
			.map_err(|e| anyhow::anyhow!(e))?;
	} else {
		// clear commands permanently
		diesel::delete(commands)
			.filter(session_id.eq(session_id_v))
			.execute(&mut connection)
			.await
			.map_err(|e| anyhow::anyhow!(e))?;
	}

	// Signal the frontend terminal emulator to clear the terminal screen
	Ok("__TERMINAL_EMULATOR_INTERNAL_HANDLE_CLEAR__".to_string())
}

#[cfg(test)]
mod tests {
	use std::sync::Arc;

	use serial_test::serial;

	use srv_mod_database::bb8;
	use srv_mod_database::diesel::{Connection, PgConnection};
	use srv_mod_database::diesel_async::AsyncPgConnection;
	use srv_mod_database::diesel_async::pooled_connection::AsyncDieselConnectionManager;
	use srv_mod_database::diesel_migrations::MigrationHarness;
	use srv_mod_database::migration::MIGRATIONS;
	use srv_mod_database::models::command::CreateCommand;
	use srv_mod_database::models::user::{CreateUser, User};
	use srv_mod_database::schema::users;

	use crate::session_terminal_emulator::clear::TerminalSessionClearArguments;

	use super::*;

	async fn drop_database() {
		let mut connection = PgConnection::establish("postgresql://rs2:rs2@localhost/rs2").unwrap();

		connection.revert_all_migrations(MIGRATIONS).unwrap();
		connection.run_pending_migrations(MIGRATIONS).unwrap();
	}

	async fn make_pool() -> Pool {
		let connection_manager =
			AsyncDieselConnectionManager::<AsyncPgConnection>::new("postgresql://rs2:rs2@localhost/rs2");

		Arc::new(
			bb8::Pool::builder()
				.max_size(1u32)
				.build(connection_manager)
				.await
				.unwrap(),
		)
	}

	async fn generate_test_user(pool: Pool) -> User {
		let mut connection = pool.get().await.unwrap();
		diesel::insert_into(users::table)
			.values(CreateUser::new("test".to_string(), "test".to_string()))
			.returning(User::as_select())
			.get_result(&mut connection)
			.await
			.unwrap()
	}

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