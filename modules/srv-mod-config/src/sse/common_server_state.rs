use std::fmt::Display;

use serde::{Deserialize, Serialize};

/// The type of events that can be emitted
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub enum EventType {
    #[serde(rename = "log")]
    Log,
    #[serde(rename = "command_output")]
    CommandOutput,
}

impl Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[expect(
            clippy::pattern_type_mismatch,
            reason = "Cannot dereference into the Display trait implementation"
        )]
        match self {
            Self::Log => write!(f, "log"),
            Self::CommandOutput => write!(f, "command_output"),
        }
    }
}

/// The SSE event type, this is the type that will be sent to the client (split into data, event and
/// id)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SseEvent {
    pub data:  String,
    pub event: EventType,
    pub id:    Option<String>,
}
