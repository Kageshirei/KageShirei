use tracing::{debug, instrument};

use srv_mod_database::{diesel, Pool};
use srv_mod_database::diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use srv_mod_database::diesel::internal::derives::multiconnection::chrono;
use srv_mod_database::diesel_async::RunQueryDsl;
use srv_mod_database::models::command::Command;
use srv_mod_database::schema::commands::dsl::*;

/// Handle the clear command
#[instrument]
pub async fn handle(session_id_v: &str, db_pool: Pool) -> anyhow::Result<String> {
	debug!("Terminal command received");

	let mut connection = db_pool
		.get()
		.await
		.map_err(|_| anyhow::anyhow!("Failed to get a connection from the pool"))?;

	// clear commands hiding marking them as deleted (soft delete)
	diesel::update(commands)
		.filter(session_id.eq(session_id_v))
		.set((
			deleted_at.eq(chrono::Utc::now()),
			restored_at.eq(None),
		))
		.execute(&mut connection)
		.await
		.map_err(|e| Err(anyhow::anyhow!(e)))?;

	// Signal the frontend terminal emulator to clear the terminal screen
	Ok("__TERMINAL_EMULATOR_INTERNAL_HANDLE_CLEAR__".to_string())
}