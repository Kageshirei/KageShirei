use diesel::{AsChangeset, Insertable, Queryable, Selectable};
use serde::Deserialize;
use serde_json::Value;

use crate::schema_extension::AgentCommandStatus;

#[derive(Debug, Queryable, Selectable, Clone, PartialEq)]
#[diesel(table_name = crate::schema::agents_command_requests)]
pub struct AgentCommandRequest {
	/// The unique identifier for the command (cuid2)
	pub id: String,
	/// The agent that ran the command
	pub agent_id: String,
	/// The parsed command, this must be a common structure serializable to any protocol format
	pub command: Value,
	/// Raw unparsed command output coming from the agent
	pub output: Option<String>,
	/// The command's status
	pub status: AgentCommandStatus,
	/// When the command was retrieved by the agent
	pub retrieved_at: Option<chrono::DateTime<chrono::Utc>>,
	/// When the command was completed successfully by the agent
	pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
	/// When the command failed (in agent context)
	pub failed_at: Option<chrono::DateTime<chrono::Utc>>,
	/// The command's creation date
	pub created_at: chrono::DateTime<chrono::Utc>,
	/// The command's last update date
	pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::agents_command_requests)]
pub struct CreateAgentCommandRequest {
	/// The unique identifier for the command (cuid2)
	pub id: String,
	/// The agent that ran the command
	pub agent_id: String,
	/// The parsed command, this must be a common structure serializable to any protocol format
	pub command: Value,
	/// Raw unparsed command output coming from the agent
	pub output: Option<String>,
	/// The command's status
	pub status: AgentCommandStatus,
	/// When the command was retrieved by the agent
	pub retrieved_at: Option<chrono::DateTime<chrono::Utc>>,
	/// When the command was completed successfully by the agent
	pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
	/// When the command failed (in agent context)
	pub failed_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl CreateAgentCommandRequest {
	pub fn new(id: String, agent_id: String, command: Value) -> Self {
		Self {
			id,
			agent_id,
			command,
			output: None,
			status: AgentCommandStatus::Pending,
			retrieved_at: None,
			completed_at: None,
			failed_at: None,
		}
	}
}
