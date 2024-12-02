//! The global session terminal command

use clap::Args;
use serde::{Deserialize, Serialize};
use srv_mod_entity::{
    entities::agent,
    sea_orm::{prelude::*, Condition},
};
use tracing::{debug, instrument};

use crate::{command_handler::CommandHandlerArguments, post_process_result::PostProcessResult};

/// Terminal session arguments for the global session terminal
#[derive(Args, Debug, PartialEq, Eq, Serialize)]
pub struct GlobalSessionTerminalSessionsArguments {
    /// List of session hostnames to open terminal sessions for
    pub ids: Option<Vec<String>>,
}

/// The record of a terminal session
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionOpeningRecordUnparsed {
    /// The identifier of the session
    pub id:       String,
    /// The hostname of the session
    pub hostname: String,
    /// The current working directory of the session
    pub cwd:      String,
}

/// The record of a terminal session
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionOpeningRecord {
    /// The hostname of the session
    pub hostname: String,
    /// The current working directory of the session
    pub cwd:      String,
    /// The arguments of the session
    pub args:     Vec<String>,
}

impl From<agent::Model> for SessionOpeningRecord {
    fn from(record: agent::Model) -> Self {
        Self {
            hostname: record.hostname,
            cwd:      record.cwd,
            args:     vec![record.id],
        }
    }
}

/// Chains the hostname condition for the query
///
/// # Arguments
///
/// * `ids` - The list of hostnames to query
///
/// # Returns
///
/// The condition for the query
fn make_hostname_condition_from_ids(ids: Vec<String>) -> Condition {
    let mut condition = Condition::any();

    for id in ids.iter() {
        condition = condition.add(agent::Column::Hostname.eq(id));
    }

    condition
}

/// Handle the sessions command
///
/// This command lists all the terminal sessions in the database
///
/// # Arguments
///
/// * `config` - The command handler arguments
/// * `args` - The terminal session arguments
///
/// # Returns
///
/// The serialized result of the command
#[instrument]
pub async fn handle(
    config: CommandHandlerArguments,
    args: &GlobalSessionTerminalSessionsArguments,
) -> Result<String, String> {
    debug!("Terminal command received");

    let connection = config.db_pool.clone();

    // If the ids are provided, return the terminal emulator internal handle open sessions command
    if args.ids.is_some() {
        let agents = agent::Entity::find()
            .filter(make_hostname_condition_from_ids(args.ids.clone().unwrap()))
            .all(&connection)
            .await
            .map_err(|e| e.to_string())?;

        let results = agents
            .into_iter()
            .map(SessionOpeningRecord::from)
            .collect::<Vec<_>>();

        return Ok(format!(
            "__TERMINAL_EMULATOR_INTERNAL_HANDLE_OPEN_SESSIONS__{}",
            serde_json::to_string(&results).map_err(|e| e.to_string())?
        ));
    }

    // list all the agents (sessions) in the database
    let result = agent::Entity::find()
        .all(&connection)
        .await
        .map_err(|e| e.to_string())?;

    debug!("Found agents: {:?}", result);

    // Serialize the result
    serde_json::to_string(&PostProcessResult {
        r#type: "sessions".to_owned(),
        data:   result,
    })
    .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use chrono::Utc;
    use kageshirei_communication_protocol::{NetworkInterface, NetworkInterfaceArray};
    use srv_mod_entity::{
        active_enums::AgentIntegrity,
        entities::logs,
        sea_orm::{ActiveValue, Database, DatabaseConnection, DbErr, TransactionTrait},
    };
    use tokio::sync::broadcast;

    use super::*;
    use crate::command_handler::{HandleArguments, HandleArgumentsSession, HandleArgumentsUser};

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

        agent::Entity::insert_many(vec![
            agent::ActiveModel {
                id:                 ActiveValue::Set("agent1".to_owned()),
                pid:                ActiveValue::Set(1),
                secret:             ActiveValue::Set("test".to_owned()),
                cwd:                ActiveValue::Set("test".to_owned()),
                server_secret:      ActiveValue::Set("test".to_owned()),
                operating_system:   ActiveValue::Set("test".to_owned()),
                integrity:          ActiveValue::Set(AgentIntegrity::Medium),
                updated_at:         ActiveValue::Set(Utc::now().naive_utc()),
                domain:             ActiveValue::Set(Some("test".to_owned())),
                hostname:           ActiveValue::Set("host1".to_owned()),
                network_interfaces: ActiveValue::Set(NetworkInterfaceArray {
                    network_interfaces: vec![NetworkInterface {
                        name:        Some("test".to_owned()),
                        dhcp_server: Some("test".to_owned()),
                        address:     Some("test".to_owned()),
                    }],
                }),
                ppid:               ActiveValue::Set(1),
                username:           ActiveValue::Set("test".to_owned()),
                process_name:       ActiveValue::Set("test".to_owned()),
                signature:          ActiveValue::Set("test1".to_owned()),
                terminated_at:      ActiveValue::Set(None),
                created_at:         ActiveValue::Set(Utc::now().naive_utc()),
            },
            agent::ActiveModel {
                id:                 ActiveValue::Set("agent2".to_owned()),
                pid:                ActiveValue::Set(1),
                secret:             ActiveValue::Set("test".to_owned()),
                cwd:                ActiveValue::Set("test".to_owned()),
                server_secret:      ActiveValue::Set("test".to_owned()),
                operating_system:   ActiveValue::Set("test".to_owned()),
                integrity:          ActiveValue::Set(AgentIntegrity::Medium),
                updated_at:         ActiveValue::Set(Utc::now().naive_utc()),
                domain:             ActiveValue::Set(Some("test".to_owned())),
                hostname:           ActiveValue::Set("host2".to_owned()),
                network_interfaces: ActiveValue::Set(NetworkInterfaceArray {
                    network_interfaces: vec![NetworkInterface {
                        name:        Some("test".to_owned()),
                        dhcp_server: Some("test".to_owned()),
                        address:     Some("test".to_owned()),
                    }],
                }),
                ppid:               ActiveValue::Set(1),
                username:           ActiveValue::Set("test".to_owned()),
                process_name:       ActiveValue::Set("test".to_owned()),
                signature:          ActiveValue::Set("test2".to_owned()),
                terminated_at:      ActiveValue::Set(None),
                created_at:         ActiveValue::Set(Utc::now().naive_utc()),
            },
        ])
        .exec(&db_pool)
        .await
        .unwrap();

        db_pool
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_handle_with_ids() {
        // Mock database setup with specific agent records
        let db = init().await;

        // Mock broadcast channel
        let (sender, mut receiver) = broadcast::channel(1);

        // Create command handler arguments
        let config = Arc::new(HandleArguments {
            session:          HandleArgumentsSession {
                session_id: "test".to_owned(),
                hostname:   "test".to_owned(),
            },
            user:             HandleArgumentsUser {
                user_id:  "test".to_owned(),
                username: "test".to_owned(),
            },
            db_pool:          db,
            broadcast_sender: sender,
        });

        // Define input arguments with specific IDs
        let args = GlobalSessionTerminalSessionsArguments {
            ids: Some(vec!["host1".to_owned(), "host2".to_owned()]),
        };

        // Call the function
        let result = handle(config, &args).await;

        // Assert success
        assert!(result.is_ok());

        // Verify the result string contains expected session information
        let output = result.unwrap();
        assert!(output.contains("__TERMINAL_EMULATOR_INTERNAL_HANDLE_OPEN_SESSIONS__"));
        assert!(output.contains("host1"));
        assert!(output.contains("host2"));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_handle_without_ids() {
        // Mock database setup with specific agent records
        let db = init().await;

        // Mock broadcast channel
        let (sender, mut receiver) = broadcast::channel(1);

        // Create command handler arguments
        let config = Arc::new(HandleArguments {
            session:          HandleArgumentsSession {
                session_id: "test".to_owned(),
                hostname:   "test".to_owned(),
            },
            user:             HandleArgumentsUser {
                user_id:  "test".to_owned(),
                username: "test".to_owned(),
            },
            db_pool:          db,
            broadcast_sender: sender,
        });

        // Define input arguments without IDs
        let args = GlobalSessionTerminalSessionsArguments {
            ids: None,
        };

        // Call the function
        let result = handle(config, &args).await;

        // Assert success
        assert!(result.is_ok());

        // Verify the result string contains serialized session information
        let output = result.unwrap();
        assert!(output.contains(r#""type":"sessions""#));
        assert!(output.contains("host1"));
    }

    #[test]
    fn test_make_hostname_condition_from_ids() {
        use srv_mod_entity::{
            entities::agent,
            sea_orm::{ColumnTrait, Condition},
        };

        // Define test input
        let ids = vec!["host1".to_owned(), "host2".to_owned()];

        // Generate condition
        let condition = make_hostname_condition_from_ids(ids);

        // Expected condition
        let expected_condition = Condition::any()
            .add(agent::Column::Hostname.eq("host1"))
            .add(agent::Column::Hostname.eq("host2"));

        // Assert the conditions match
        assert_eq!(
            format!("{:?}", condition),
            format!("{:?}", expected_condition)
        );
    }
}
