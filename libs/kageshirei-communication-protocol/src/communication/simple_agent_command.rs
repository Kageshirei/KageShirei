use serde::{Deserialize, Serialize};

use crate::{communication::agent_commands::AgentCommands, metadata::Metadata};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "server", derive(Debug))]
pub struct SimpleAgentCommand {
    /// The command to be executed
    pub op:       AgentCommands,
    /// The command metadata
    pub metadata: Metadata,
}
