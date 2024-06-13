use clap::Args;
use serde::{Deserialize, Serialize};
use tracing::{debug, instrument};

use srv_mod_database::{diesel, Pool};
use srv_mod_database::diesel::{ExpressionMethods, Queryable, QueryDsl, SelectableHelper};
use srv_mod_database::diesel::internal::derives::multiconnection::chrono;
use srv_mod_database::diesel_async::RunQueryDsl;
use srv_mod_database::schema::agents;

use crate::command_handler::CommandHandlerArguments;
use crate::post_process_result::PostProcessResult;

/// Terminal session arguments for the global session terminal
#[derive(Args, Debug, PartialEq, Serialize)]
pub struct GlobalSessionTerminalSessionsArguments {
	/// List of session hostnames to open terminal sessions for
	pub ids: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Queryable)]
pub struct SessionRecord {
	pub id: String,
	pub hostname: String,
	pub domain: String,
	pub username: String,
	pub ip: String,
	pub integrity_level: i16,
	pub operative_system: String,
}

/// Handle the sessions command
#[instrument]
pub async fn handle(config: CommandHandlerArguments, args: &GlobalSessionTerminalSessionsArguments) -> anyhow::Result<String> {
	debug!("Terminal command received");

	let mut connection = config.db_pool
	                           .get()
	                           .await
	                           .map_err(|_| anyhow::anyhow!("Failed to get a connection from the pool"))?;

	// If the ids are provided, return the terminal emulator internal handle open sessions command
	if args.ids.is_some() {
		return Ok("__TERMINAL_EMULATOR_INTERNAL_HANDLE_OPEN_SESSIONS__".to_string());
	}

	// list all the agents (sessions) in the database
	let result = agents::table
		.select((
			agents::id,
			agents::hostname,
			agents::domain,
			agents::username,
			agents::ip,
			agents::integrity_level,
			agents::operative_system,
		))
		.get_results::<SessionRecord>(&mut connection)
		.await
		.map_err(|e| anyhow::anyhow!(e))?;

	// Serialize the result
	Ok(
		serde_json::to_string(
			&PostProcessResult {
				r#type: "sessions".to_string(),
				data: result,
			}
		).map_err(|e| anyhow::anyhow!(e))?
	)
}

#[cfg(test)]
mod tests {
	use serial_test::serial;

	use rs2_communication_protocol::communication_structs::checkin::{Checkin, PartialCheckin};
	use rs2_srv_test_helper::tests::{drop_database, generate_test_user, make_pool};
	use srv_mod_database::models::agent::CreateAgent;

	use super::*;

	#[tokio::test]
	#[serial]
	async fn test_handle_history_display() {
		drop_database().await;
		let db_pool = make_pool().await;

		let user = generate_test_user(db_pool.clone()).await;

		let session_id_v = "global";

		let binding = db_pool.clone();

		// open a scope to automatically drop the connection once exited
		{
			let mut connection = binding.get().await.unwrap();

			let mut agent = CreateAgent::from(Checkin::new(PartialCheckin {
				operative_system: "Windows".to_string(),
				hostname: "DESKTOP-PC".to_string(),
				domain: "WORKGROUP".to_string(),
				username: "user".to_string(),
				ip: "10.2.123.45".to_string(),
				process_id: 1234,
				parent_process_id: 5678,
				process_name: "agent.exe".to_string(),
				elevated: false,
			}));

			agent.signature = "random-signature-0".to_string();

			// Insert a dummy agent
			let inserted_agent_0 = diesel::insert_into(agents::table)
				.values(&agent)
				.execute(&mut connection)
				.await
				.unwrap();

			let mut agent = CreateAgent::from(Checkin::new(PartialCheckin {
				operative_system: "Windows".to_string(),
				hostname: "NICE-DC".to_string(),
				domain: "NICE-DOMAIN".to_string(),
				username: "guest".to_string(),
				ip: "10.2.123.56".to_string(),
				process_id: 1234,
				parent_process_id: 5678,
				process_name: "agent.exe".to_string(),
				elevated: true,
			}));

			agent.signature = "random-signature-1".to_string();

			// Insert a dummy agent
			let inserted_agent_1 = diesel::insert_into(agents::table)
				.values(&agent)
				.execute(&mut connection)
				.await
				.unwrap();
		}

		let args = GlobalSessionTerminalSessionsArguments { ids: None };
		let result = handle(db_pool.clone(), &args).await;

		assert!(result.is_ok());
		let result = result.unwrap();
		let deserialized = serde_json::from_str::<Vec<SessionRecord>>(result.as_str()).unwrap();

		assert_eq!(deserialized.len(), 2);
		assert_eq!(deserialized[0].domain, "WORKGROUP");
		assert_eq!(deserialized[1].domain, "NICE-DOMAIN");

		drop_database().await;
	}
}