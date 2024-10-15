use std::sync::Arc;

use serde::{Deserialize, Serialize};

/// Define the metadata struct responsible for holding metadata about a struct used during the communication.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Metadata {
    /// The request identifier (Cuid2)
    pub request_id: String,
    /// The command identifier (Cuid2)
    pub command_id: String,
    /// The agent identifier (Cuid2)
    pub agent_id:   String,
    /// An optional path for path-based protocols (e.g. HTTP) where the request should be sent
    pub path:       Option<String>,
}

/// Define the metadata trait responsible for providing metadata about a type.
pub trait WithMetadata {
    /// Get the metadata for the type.
    fn get_metadata(&self) -> Arc<Metadata>;
}
