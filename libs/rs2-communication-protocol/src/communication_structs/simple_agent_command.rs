use serde::{Deserialize, Serialize};

use crate::communication_structs::agent_commands::AgentCommands;
use crate::metadata::Metadata;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SimpleAgentCommand {
	/// The command to be executed
	pub op: AgentCommands,
	/// The command metadata
	pub metadata: Metadata,
}