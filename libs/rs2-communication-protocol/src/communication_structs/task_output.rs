use crate::metadata::{Metadata, WithMetadata};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug, Clone)]
/// TaskOutput structure to represent task-related information, including optional fields for:
/// - output: the result of the task as an Option<String>
/// - started_at: the timestamp when the task started as an Option<i64>
/// - ended_at: the timestamp when the task ended as an Option<i64>
/// - exit_code: the task's exit code as an Option<i32>
/// - metadata: an Arc-wrapped Metadata object, allowing shared ownership and thread safety
pub struct TaskOutput {
    pub output: Option<String>,          // Optional task output as a String
    pub started_at: Option<i64>,         // Optional timestamp for when the task started
    pub ended_at: Option<i64>,           // Optional timestamp for when the task ended
    pub exit_code: Option<i32>,          // Optional exit code of the task
    pub metadata: Option<Arc<Metadata>>, // Optional metadata associated with the task
}

impl TaskOutput {
    // Constructor for TaskOutput that initializes all fields to None.
    pub fn new() -> Self {
        TaskOutput {
            output: None,
            started_at: None,
            ended_at: None,
            exit_code: None,
            metadata: None,
        }
    }

    // Adds metadata to the TaskOutput by wrapping it in an Arc for thread-safe shared ownership.
    // Returns a mutable reference to the updated TaskOutput instance.
    pub fn with_metadata(&mut self, metadata: Metadata) -> &mut Self {
        self.metadata = Some(Arc::new(metadata));
        self
    }
}

// Implementation of the WithMetadata trait for TaskOutput, which provides a method
// to retrieve the metadata as an Arc<Metadata>.
impl WithMetadata for TaskOutput {
    fn get_metadata(&self) -> Arc<Metadata> {
        // Returns a clone of the Arc-wrapped metadata. Assumes that metadata is always present.
        self.metadata.as_ref().unwrap().clone()
    }
}
