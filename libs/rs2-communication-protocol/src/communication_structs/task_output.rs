use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::metadata::{Metadata, WithMetadata};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TaskOutput {
    pub output: Option<String>,
    pub started_at: Option<i64>, //timestamp
    pub ended_at: Option<i64>,   //timestamp
    pub exit_code: Option<u8>,
    pub metadata: Option<Arc<Metadata>>,
}

impl TaskOutput {
    pub fn new() -> Self {
        TaskOutput {
            output: None,
            started_at: None,
            ended_at: None,
            exit_code: None,
            metadata: None,
        }
    }

    pub fn with_metadata(&mut self, metadata: Metadata) -> &mut Self {
        self.metadata = Some(Arc::new(metadata));
        self
    }
}

impl WithMetadata for TaskOutput {
    fn get_metadata(&self) -> Arc<Metadata> {
        self.metadata.as_ref().unwrap().clone()
    }
}
