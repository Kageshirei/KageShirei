//! Define the metadata struct and trait responsible for holding metadata about some struct state
//! used during the communication.

use alloc::{string::String, sync::Arc};

use serde::{Deserialize, Serialize};

/// Define the metadata struct responsible for holding metadata about a struct used during the
/// communication.
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "server", derive(Debug))]
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

// Safety: The Metadata struct is safe to be sent between threads
unsafe impl Send for Metadata {}
// Safety: The Metadata struct is safe to be shared between threads
unsafe impl Sync for Metadata {}

/// Define the metadata trait responsible for providing metadata about a type.
#[allow(
    clippy::module_name_repetitions,
    reason = "The trait is named `WithMetadata` because it provides metadata for a type."
)]
pub trait WithMetadata {
    /// Get the metadata for the type.
    fn get_metadata(&self) -> Option<Arc<Metadata>>;
}
