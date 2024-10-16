//! Agent commands
//!
//! This module contains the agent commands that can be sent to the agent.

use alloc::string::String;
use core::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "server", derive(Debug))]
pub enum AgentCommands {
    /// Invalid command
    INVALID,
    /// Terminate the agent
    ///
    /// This command is used to terminate the agent
    ///
    /// # Type mapping
    ///
    /// This command maps to the `SimpleAgentCommand` struct
    #[serde(rename = "terminate")]
    Terminate,
    /// Checkin the agent
    ///
    /// This command is used to check in the agent
    ///
    /// # Type mapping
    ///
    /// This command maps to the `SimpleAgentCommand` struct
    #[serde(rename = "checkin")]
    Checkin,
}

impl Display for AgentCommands {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        #[expect(
            clippy::pattern_type_mismatch,
            reason = "Cannot dereference into the Display trait implementation"
        )]
        match self {
            Self::Terminate => write!(f, "terminate"),
            Self::Checkin => write!(f, "checkin"),
            Self::INVALID => write!(f, "invalid"),
        }
    }
}

impl From<String> for AgentCommands {
    fn from(s: String) -> Self {
        match s.as_str() {
            "terminate" => Self::Terminate,
            "checkin" => Self::Checkin,
            _ => Self::INVALID,
        }
    }
}
