//! Prepare the agent and operates on its middle stages in order to persist it into the database

use kageshirei_communication_protocol::communication::Checkin;
use kageshirei_crypt::{
    encoder::{
        base64::{Encoder, Variant},
        Encoder as _,
    },
    encryption_algorithm::{ident_algorithm::IdentEncryptor, AsymmetricAlgorithm},
};
use srv_mod_entity::{
    active_enums::AgentIntegrity,
    entities::agent,
    sea_orm::{prelude::*, ActiveValue::Set, DatabaseConnection},
};
use tracing::info;

use super::signature::make_signature;
use crate::error;

/// Prepare the agent for insertion into the database
pub fn prepare(data: Checkin) -> Result<agent::ActiveModel, error::CommandHandling> {
    let agent_signature = make_signature(&data).map_err(error::CommandHandling::Crypt)?;

    let encoder = Encoder::new(Variant::UrlUnpadded);

    // the usage of the IdentEncryptor hardcoded here does not force it as it is used only to specialize
    // the type not to encrypt anything
    let agent_secret_key = AsymmetricAlgorithm::<IdentEncryptor>::make_temporary_secret_key();
    let agent_secret_key = encoder
        .encode(agent_secret_key.as_slice())
        .map_err(error::CommandHandling::Crypt)?;

    // the usage of the IdentEncryptor hardcoded here does not force it as it is used only to specialize
    // the type not to encrypt anything
    let server_secret = AsymmetricAlgorithm::<IdentEncryptor>::make_temporary_secret_key();
    let server_secret = encoder
        .encode(server_secret.as_slice())
        .map_err(error::CommandHandling::Crypt)?;

    // prepare the agent object for insertion
    Ok(agent::ActiveModel {
        operating_system: Set(data.operative_system),
        hostname: Set(data.hostname),
        domain: Set(Some(data.domain)),
        username: Set(data.username),
        network_interfaces: Set(data.network_interfaces),
        pid: Set(data.pid),
        ppid: Set(data.ppid),
        process_name: Set(data.process_name),
        integrity: Set(AgentIntegrity::from(data.integrity_level)),
        cwd: Set(data.cwd),
        server_secret: Set(server_secret),
        secret: Set(agent_secret_key),
        signature: Set(agent_signature),
        ..Default::default()
    })
}

/// Creates or updates an agent in the database based on its signature
pub async fn create_or_update(
    agent: agent::ActiveModel,
    connection: &DatabaseConnection,
) -> Result<agent::Model, error::CommandHandling> {
    // check if the agent already exists
    let agent_exists = agent::Entity::find()
        .filter(agent::Column::Signature.eq(agent.signature.clone().unwrap()))
        .one(connection)
        .await;

    if agent_exists.is_ok() {
        info!("Existing agent detected, updating ...");

        let agent = agent::Entity::update_many()
            .filter(agent::Column::Signature.eq(agent.signature.clone().unwrap()))
            .set(agent)
            .exec_with_returning(connection)
            .await
            .map_err(|e| error::CommandHandling::Database("Failed to update agent".to_owned(), e))?;

        let agent = agent
            .first()
            // TOC/TOU inconsistency detected, this is generally really difficult to achieve as
            // there are only a few instructions between the initial select and the update, anyway
            // there is still a very small change that in highly parallelized environments with lots
            // of agents and operators working concurrently this happens, so we need to handle it
            // gracefully to avoid any possibility for the server to crash
            .ok_or(error::CommandHandling::Generic(
                "Failed to update the agent, TOC/TOU inconsistency detected".to_owned(),
            ))?
            .to_owned();

        info!("Agent data updated (id: {})", agent.id);

        // return the updated object
        Ok(agent)
    }
    else {
        info!("New agent detected, inserting ...");

        let agent = agent
            .insert(connection)
            .await
            .map_err(|e| error::CommandHandling::Database("Failed to insert agent".to_owned(), e))?;

        info!("New agent recorded (id: {})", agent.id);

        // return the inserted object
        Ok(agent)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use anyhow::anyhow;
    use kageshirei_communication_protocol::communication::checkin::{Checkin, PartialCheckin};
    use srv_mod_database::{
        bb8,
        diesel::{Connection, ExpressionMethods, PgConnection, QueryDsl},
        diesel_async::{pooled_connection::AsyncDieselConnectionManager, AsyncPgConnection, RunQueryDsl},
        diesel_migrations::MigrationHarness,
        migration::MIGRATIONS,
        Pool,
    };

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
            operative_system:  "Windows".to_string(),
            hostname:          "DESKTOP-PC".to_string(),
            domain:            "WORKGROUP".to_string(),
            username:          "user".to_string(),
            ip:                "10.2.123.45".to_string(),
            process_id:        1234,
            parent_process_id: 5678,
            process_name:      "agent.exe".to_string(),
            elevated:          true,
        });
        let result = ensure_checkin_is_valid(Ok(data.clone()));
        assert_eq!(result.unwrap(), data);
    }

    #[test]
    fn prepare_returns_create_agent() {
        let data = Checkin::new(PartialCheckin {
            operative_system:  "Windows".to_string(),
            hostname:          "DESKTOP-PC".to_string(),
            domain:            "WORKGROUP".to_string(),
            username:          "user".to_string(),
            ip:                "10.2.123.45".to_string(),
            process_id:        1234,
            parent_process_id: 5678,
            process_name:      "agent.exe".to_string(),
            elevated:          true,
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
            operative_system:  "Windows".to_string(),
            hostname:          "DESKTOP-PC".to_string(),
            domain:            "WORKGROUP".to_string(),
            username:          "user".to_string(),
            ip:                "10.2.123.45".to_string(),
            process_id:        1234,
            parent_process_id: 5678,
            process_name:      "agent.exe".to_string(),
            elevated:          true,
        });

        let connection_string = "postgresql://kageshirei:kageshirei@localhost/kageshirei".to_string();

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
