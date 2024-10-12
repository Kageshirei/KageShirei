use std::fmt::Display;

use serde::{Deserialize, Serialize};

/// The type of events that can be emitted
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum EventType {
	#[serde(rename = "log")]
	Log,
	#[serde(rename = "command_output")]
	CommandOutput,
}

impl Display for EventType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			EventType::Log => write!(f, "log"),
			EventType::CommandOutput => write!(f, "command_output"),
		}
	}
}

/// The SSE event type, this is the type that will be sent to the client (split into data, event and id)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SseEvent {
	pub data: String,
	pub event: EventType,
	pub id: Option<String>,
}
