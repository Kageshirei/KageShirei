//! Basic agent response
//!
//! This struct is minimal and should be extended as needed for more complex responses coming from
//! the agent. Should be noted that this struct must be used as a first step of the 2-step response
//! parsing.
//!
//! Response parsing flow:
//! 1. Parse the BasicAgentResponse
//!     1. Based on the Metadata.command_id, parse the rest of the response to the appropriate type
//! 2. Parse the rest of the response to the extended response derived at step 1.1

use serde::{Deserialize, Serialize};

use crate::metadata::Metadata;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "server", derive(Debug))]
pub struct BasicAgentResponse {
    /// The command metadata
    pub metadata: Metadata,
}
