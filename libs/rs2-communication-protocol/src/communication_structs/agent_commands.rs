use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
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
    /// Print the current working directory
    ///
    /// This command is used to print the agent's current working directory
    ///
    /// # Type mapping
    ///
    /// This command maps to the `SimpleAgentCommand` struct
    #[serde(rename = "pwd")]
    PrintWorkingDirectory,
}

impl Display for AgentCommands {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Terminate => write!(f, "terminate"),
            Self::Checkin => write!(f, "checkin"),
            Self::PrintWorkingDirectory => write!(f, "pwd"),
            Self::INVALID => write!(f, "invalid"),
        }
    }
}

impl From<String> for AgentCommands {
    fn from(s: String) -> Self {
        match s.as_str() {
            "terminate" => Self::Terminate,
            "checkin" => Self::Checkin,
            "pwd" => Self::PrintWorkingDirectory,
            _ => Self::INVALID,
        }
    }
}
