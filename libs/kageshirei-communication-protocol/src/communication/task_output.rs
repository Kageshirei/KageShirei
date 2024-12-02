//! TaskOutput structure to represent task-related information, including
//! optional fields for:
//! - output: the result of the task as an Option<String>
//! - started_at: the timestamp when the task started as an Option<i64>
//! - ended_at: the timestamp when the task ended as an Option<i64>
//! - exit_code: the task's exit code as an Option<i32>
//! - metadata: an Arc-wrapped Metadata object, allowing shared ownership and thread safety

use alloc::{string::String, sync::Arc};

use serde::{Deserialize, Serialize};

use crate::metadata::{Metadata, WithMetadata};

/// TaskOutput structure to represent task-related information, including
/// optional fields for:
/// - output: the result of the task as an Option<String>
/// - started_at: the timestamp when the task started as an Option<i64>
/// - ended_at: the timestamp when the task ended as an Option<i64>
/// - exit_code: the task's exit code as an Option<i32>
/// - metadata: an Arc-wrapped Metadata object, allowing shared ownership and thread safety
#[derive(Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "server", derive(Debug))]
pub struct TaskOutput {
    /// Optional task output as a String
    pub output:     Option<String>,
    /// Optional timestamp for when the task started
    pub started_at: Option<i64>,
    /// Optional timestamp for when the task ended
    pub ended_at:   Option<i64>,
    /// Optional exit code of the task
    pub exit_code:  Option<i32>,
    /// Optional metadata associated with the task
    pub metadata:   Option<Arc<Metadata>>,
}

impl Default for TaskOutput {
    fn default() -> Self { Self::new() }
}

impl TaskOutput {
    // Constructor for TaskOutput that initializes all fields to None.
    pub const fn new() -> Self {
        Self {
            output:     None,
            started_at: None,
            ended_at:   None,
            exit_code:  None,
            metadata:   None,
        }
    }
}

/// Implementation of the WithMetadata trait for TaskOutput, which provides a
/// method to retrieve the metadata as an Arc<Metadata>.
impl WithMetadata for TaskOutput {
    fn get_metadata(&self) -> Option<Arc<Metadata>> { self.metadata.clone() }
}
