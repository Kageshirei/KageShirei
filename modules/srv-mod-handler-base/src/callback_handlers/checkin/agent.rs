use anyhow::{anyhow, Result};
use axum::http::StatusCode;
use tracing::{info, warn};

use rs2_communication_protocol::communication_structs::checkin::Checkin;
use rs2_crypt::encoder::base64::Base64Encoder;
use rs2_crypt::encoder::Encoder;
use rs2_crypt::encryption_algorithm::asymmetric_algorithm::AsymmetricAlgorithm;
use rs2_crypt::encryption_algorithm::ident_algorithm::IdentEncryptor;
use srv_mod_database::diesel;
use srv_mod_database::diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use srv_mod_database::diesel_async::{AsyncPgConnection, RunQueryDsl};
use srv_mod_database::models::agent::{Agent, CreateAgent};

use super::signature::make_signature;

/// Ensure that the checkin data is valid
pub fn ensure_checkin_is_valid(
	data: Result<Checkin>,
) -> Result<Checkin> {
	// if the data is not a checkin struct, drop the request
	if data.is_err() {
		warn!(
            "Failed to parse checkin data, request refused: {:?}",
            data.err()
        );
		warn!("Internal status code: {}", StatusCode::UNPROCESSABLE_ENTITY);

		// always return OK to avoid leaking information
		return Err(anyhow!("Failed to parse checkin data"));
	}

	// return the checkin data
	data
}

/// Prepare the agent for insertion into the database
pub fn prepare(data: Checkin) -> CreateAgent {
	let agent_signature = make_signature(&data);

	let encoder = Base64Encoder;
	// the usage of the IdentEncryptor hardcoded here does not force it as it is used only to specialize the type
	// not to encrypt anything
	let agent_secret_key = AsymmetricAlgorithm::<IdentEncryptor>::make_temporary_secret_key();
	let agent_secret_key = encoder.encode(agent_secret_key);

	// the usage of the IdentEncryptor hardcoded here does not force it as it is used only to specialize the type
	// not to encrypt anything
	let server_secret = AsymmetricAlgorithm::<IdentEncryptor>::make_temporary_secret_key();
	let server_secret = encoder.encode(server_secret);

	// prepare the agent object for insertion
	let mut agent = CreateAgent::from(data);
	// update the agent with the new server/agent secret key and signature
	agent.server_secret_key = server_secret;
	agent.secret_key = agent_secret_key;
	agent.signature = agent_signature;

	agent
}

pub async fn create_or_update(agent: CreateAgent, connection: &mut AsyncPgConnection) -> Agent {
	use srv_mod_database::schema::agents::dsl::*;

	// check if the agent already exists
	let agent_exists = agents
		.filter(signature.eq(&agent.signature))
		.first::<Agent>(connection)
		.await;

	if agent_exists.is_ok() {
		info!("Existing agent detected, updating ...");

		let agent = diesel::update(agents.filter(signature.eq(&agent.signature)))
			.set(&agent)
			.returning(Agent::as_returning())
			.get_result::<Agent>(connection)
			.await
			.unwrap();

		info!("Agent data updated (id: {})", agent.id);

		// return the updated object
		agent
	} else {
		info!("New agent detected, inserting ...");

		let agent = diesel::insert_into(agents)
			.values(&agent)
			.returning(Agent::as_returning())
			.get_result::<Agent>(connection)
			.await
			.unwrap();

		info!("New agent recorded (id: {})", agent.id);

		// return the inserted object
		agent
	}
}

#[cfg(test)]
mod tests {
	use std::sync::Arc;

	use anyhow::anyhow;

	use rs2_communication_protocol::communication_structs::checkin::{Checkin, PartialCheckin};
	use srv_mod_database::{bb8, Pool};
	use srv_mod_database::diesel::{Connection, PgConnection};
	use srv_mod_database::diesel::ExpressionMethods;
	use srv_mod_database::diesel::QueryDsl;
	use srv_mod_database::diesel_async::{AsyncPgConnection, RunQueryDsl};
	use srv_mod_database::diesel_async::pooled_connection::AsyncDieselConnectionManager;
	use srv_mod_database::diesel_migrations::MigrationHarness;
	use srv_mod_database::migration::MIGRATIONS;

	use crate::routes::public::checkin::agent::ensure_checkin_is_valid;

	async fn drop_database(url: String) {
		let mut connection = PgConnection::establish(url.as_str()).unwrap();

		connection.revert_all_migrations(MIGRATIONS).unwrap();
		connection.run_pending_migrations(MIGRATIONS).unwrap();
	}

	async fn make_pool(url: String) -> Pool {
		let connection_manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(url);
		Arc::new(
			bb8::Pool::builder()
				.max_size(1u32)
				.build(connection_manager)
				.await
				.unwrap(),
		)
	}

	#[test]
	fn ensure_checkin_is_valid_returns_error_when_data_is_err() {
		let data = Err(anyhow!("Failed to parse checkin data"));
		let result = ensure_checkin_is_valid(data);
		assert!(result.is_err());
	}

	#[test]
	fn ensure_checkin_is_valid_returns_ok_when_data_is_ok() {
		let data = Checkin::new(PartialCheckin {
			operative_system: "Windows".to_string(),
			hostname: "DESKTOP-PC".to_string(),
			domain: "WORKGROUP".to_string(),
			username: "user".to_string(),
			ip: "10.2.123.45".to_string(),
			process_id: 1234,
			parent_process_id: 5678,
			process_name: "agent.exe".to_string(),
			elevated: true,
		});
		let result = ensure_checkin_is_valid(Ok(data.clone()));
		assert_eq!(result.unwrap(), data);
	}

	#[test]
	fn prepare_returns_create_agent() {
		let data = Checkin::new(PartialCheckin {
			operative_system: "Windows".to_string(),
			hostname: "DESKTOP-PC".to_string(),
			domain: "WORKGROUP".to_string(),
			username: "user".to_string(),
			ip: "10.2.123.45".to_string(),
			process_id: 1234,
			parent_process_id: 5678,
			process_name: "agent.exe".to_string(),
			elevated: true,
		});
		let result = super::prepare(data);
		assert_eq!(result.operative_system, "Windows");
		assert_eq!(result.hostname, "DESKTOP-PC");
		assert_eq!(result.domain, "WORKGROUP");
		assert_eq!(result.username, "user");
		assert_eq!(result.ip, "10.2.123.45");
		assert_eq!(result.process_id, 1234);
		assert_eq!(result.parent_process_id, 5678);
		assert_eq!(result.process_name, "agent.exe");
		assert_eq!(result.elevated, true);
		assert_ne!(result.server_secret_key, "");
		assert_ne!(result.secret_key, "");
		assert_ne!(result.signature, "");
		assert_ne!(result.id, "");
	}

	#[tokio::test]
	async fn create_or_update_returns_agent() {
		let data = Checkin::new(PartialCheckin {
			operative_system: "Windows".to_string(),
			hostname: "DESKTOP-PC".to_string(),
			domain: "WORKGROUP".to_string(),
			username: "user".to_string(),
			ip: "10.2.123.45".to_string(),
			process_id: 1234,
			parent_process_id: 5678,
			process_name: "agent.exe".to_string(),
			elevated: true,
		});

		let connection_string = "postgresql://rs2:rs2@localhost/rs2".to_string();

		// Ensure the database is clean
		drop_database(connection_string.clone()).await;
		let pool = make_pool(connection_string.clone()).await;

		let mut connection = pool.get().await.unwrap();
		let agent = super::prepare(data.clone());
		let result = super::create_or_update(agent, &mut connection).await;

		assert_eq!(result.operative_system, "Windows");
		assert_eq!(result.hostname, "DESKTOP-PC");
		assert_eq!(result.domain, "WORKGROUP");
		assert_eq!(result.username, "user");
		assert_eq!(result.ip, "10.2.123.45");
		assert_eq!(result.process_id, 1234);
		assert_eq!(result.parent_process_id, 5678);
		assert_eq!(result.process_name, "agent.exe");
		assert_eq!(result.elevated, true);
		assert_ne!(result.server_secret_key, "");
		assert_ne!(result.secret_key, "");
		assert_ne!(result.signature, "");
		assert_ne!(result.id, "");

		// check if the agent already exists
		let agent_exists = srv_mod_database::schema::agents::dsl::agents
			.filter(srv_mod_database::schema::agents::dsl::signature.eq(&result.signature))
			.first::<srv_mod_database::models::agent::Agent>(&mut connection)
			.await;

		assert!(agent_exists.is_ok());
		assert_eq!(agent_exists.unwrap().id, result.id);

		// update the agent with the new server/agent secret key and signature
		let agent = super::prepare(data);
		let new_result = super::create_or_update(agent, &mut connection).await;

		assert_eq!(new_result.operative_system, "Windows");
		assert_eq!(new_result.hostname, "DESKTOP-PC");
		assert_eq!(new_result.domain, "WORKGROUP");
		assert_eq!(new_result.username, "user");
		assert_eq!(new_result.ip, "10.2.123.45");
		assert_eq!(new_result.process_id, 1234);
		assert_eq!(new_result.parent_process_id, 5678);
		assert_eq!(new_result.process_name, "agent.exe");
		assert_eq!(new_result.elevated, true);
		assert_ne!(new_result.server_secret_key, result.server_secret_key);
		assert_ne!(new_result.secret_key, result.secret_key);
		assert_eq!(new_result.signature, result.signature);

		// Ensure the database is clean
		drop_database(connection_string.clone()).await;
	}
}