use serde::{Deserialize, Serialize};

use crate::metadata::Metadata;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BasicAgentResponse {
    /// The command metadata
    pub metadata: Metadata,
}
