use clap::Args;
use serde::{Deserialize, Serialize};
use srv_mod_entity::{
    entities::agent,
    sea_orm::{prelude::*, Condition},
};
use tracing::{debug, instrument};

use crate::{command_handler::CommandHandlerArguments, post_process_result::PostProcessResult};

/// Terminal session arguments for the global session terminal
#[derive(Args, Debug, PartialEq, Serialize)]
pub struct GlobalSessionTerminalSessionsArguments {
    /// List of session hostnames to open terminal sessions for
    pub ids: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(
    clippy::module_name_repetitions,
    reason = "Repetition in the name emphasizes the purpose of this struct, which holds unparsed details about \
              session opening."
)]
pub struct SessionOpeningRecordUnparsed {
    pub id:       String,
    pub hostname: String,
    pub cwd:      String,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(
    clippy::module_name_repetitions,
    reason = "The repetition in the name clarifies that this struct encapsulates the details of a terminal session \
              record."
)]
pub struct SessionOpeningRecord {
    pub hostname: String,
    pub cwd:      String,
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
            .map(|record| SessionOpeningRecord::from(record))
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

    // Serialize the result
    Ok(serde_json::to_string(&PostProcessResult {
        r#type: "sessions".to_string(),
        data:   result,
    })
    .map_err(|e| e.to_string())?)
}

#[cfg(test)]
mod tests {
    use kageshirei_communication_protocol::communication_structs::checkin::{Checkin, PartialCheckin};
    use kageshirei_srv_test_helper::tests::{drop_database, generate_test_user, make_pool};
    use serial_test::serial;
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
                operative_system:  "Windows".to_string(),
                hostname:          "DESKTOP-PC".to_string(),
                domain:            "WORKGROUP".to_string(),
                username:          "user".to_string(),
                ip:                "10.2.123.45".to_string(),
                process_id:        1234,
                parent_process_id: 5678,
                process_name:      "agent.exe".to_string(),
                elevated:          false,
            }));

            agent.signature = "random-signature-0".to_string();

            // Insert a dummy agent
            let inserted_agent_0 = diesel::insert_into(agents::table)
                .values(&agent)
                .execute(&mut connection)
                .await
                .unwrap();

            let mut agent = CreateAgent::from(Checkin::new(PartialCheckin {
                operative_system:  "Windows".to_string(),
                hostname:          "NICE-DC".to_string(),
                domain:            "NICE-DOMAIN".to_string(),
                username:          "guest".to_string(),
                ip:                "10.2.123.56".to_string(),
                process_id:        1234,
                parent_process_id: 5678,
                process_name:      "agent.exe".to_string(),
                elevated:          true,
            }));

            agent.signature = "random-signature-1".to_string();

            // Insert a dummy agent
            let inserted_agent_1 = diesel::insert_into(agents::table)
                .values(&agent)
                .execute(&mut connection)
                .await
                .unwrap();
        }

        let args = GlobalSessionTerminalSessionsArguments {
            ids: None,
        };
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
