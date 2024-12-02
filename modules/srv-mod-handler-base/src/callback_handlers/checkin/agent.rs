//! Prepare the agent and operates on its middle stages in order to persist it into the database

use std::mem;

use kageshirei_communication_protocol::{communication::Checkin, NetworkInterfaceArray};
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
        network_interfaces: Set(NetworkInterfaceArray {
            network_interfaces: data.network_interfaces,
        }),
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

    if agent_exists.is_ok() && agent_exists.unwrap().is_some() {
        info!("Existing agent detected, updating ...");

        let agents = agent::Entity::update_many()
            .filter(agent::Column::Signature.eq(agent.signature.clone().unwrap()))
            .set(agent)
            .exec_with_returning(connection)
            .await
            .map_err(|e| error::CommandHandling::Database("Failed to update agent".to_owned(), e))?;

        let agent = agents
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
    use kageshirei_communication_protocol::communication::Checkin;
    use srv_mod_entity::{
        entities::agent_command,
        sea_orm::{ActiveValue, Database, TransactionTrait},
    };

    use super::*;

    /// Helper function to create a mock `Checkin` object
    fn mock_checkin() -> Checkin {
        Checkin {
            operative_system:   "Linux".to_string(),
            hostname:           "test-host".to_string(),
            domain:             "test-domain".to_string(),
            username:           "test-user".to_string(),
            network_interfaces: vec![],
            pid:                1234,
            ppid:               5678,
            process_name:       "test-process".to_string(),
            integrity_level:    1,
            cwd:                "/test/path".to_string(),
            metadata:           None,
        }
    }

    async fn cleanup(db: DatabaseConnection) {
        db.transaction::<_, (), DbErr>(|txn| {
            Box::pin(async move {
                agent::Entity::delete_many().exec(txn).await.unwrap();

                Ok(())
            })
        })
        .await
        .unwrap();
    }

    async fn init() -> DatabaseConnection {
        let db_pool = Database::connect("postgresql://kageshirei:kageshirei@localhost/kageshirei")
            .await
            .unwrap();

        cleanup(db_pool.clone()).await;

        db_pool
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_prepare() {
        let checkin_data = mock_checkin();

        // Test prepare function
        let prepared_agent = prepare(checkin_data.clone()).expect("Failed to prepare agent");

        assert_eq!(
            prepared_agent.operating_system.unwrap(),
            checkin_data.operative_system
        );
        assert_eq!(prepared_agent.hostname.unwrap(), checkin_data.hostname);
        assert_eq!(prepared_agent.username.unwrap(), checkin_data.username);
        assert_eq!(prepared_agent.pid.unwrap(), checkin_data.pid);
        assert_eq!(prepared_agent.ppid.unwrap(), checkin_data.ppid);
        assert_eq!(
            prepared_agent.process_name.unwrap(),
            checkin_data.process_name
        );
        assert_eq!(prepared_agent.cwd.unwrap(), checkin_data.cwd);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_create_or_update_insert() {
        let db: DatabaseConnection = init().await;

        let checkin_data = mock_checkin();
        let prepared_agent = prepare(checkin_data).expect("Failed to prepare agent");

        // Insert a new agent
        let inserted_agent = create_or_update(prepared_agent, &db)
            .await
            .expect("Failed to create or update agent");

        assert!(inserted_agent.id.len() > 0);
        assert_eq!(inserted_agent.hostname, "test-host");
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_create_or_update_update() {
        let db = init().await;

        let checkin_data = mock_checkin();
        let mut prepared_agent = prepare(checkin_data.clone()).expect("Failed to prepare agent");

        // Insert a new agent
        let inserted_agent = create_or_update(prepared_agent.clone(), &db)
            .await
            .expect("Failed to create or update agent");

        prepared_agent.hostname = Set("updated-host".to_owned());

        // Update the same agent
        let updated_agent = create_or_update(prepared_agent, &db)
            .await
            .expect("Failed to update agent");

        assert_eq!(inserted_agent.id, updated_agent.id);
        assert_eq!(updated_agent.hostname, "updated-host");
    }
}
