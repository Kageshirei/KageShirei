use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::metadata::{Metadata, WithMetadata};

pub struct PartialCheckin {
	/// The OS name
	pub operative_system: String,
	/// The victim hostname
	pub hostname: String,
	/// The domain of the victim
	pub domain: String,
	/// The username of whose runs the agent
	pub username: String,
	/// The internal IP of the victim
	pub ip: String,
	/// The process ID of the agent
	pub process_id: i64,
	/// The parent process ID of the agent
	pub parent_process_id: i64,
	/// The process name of the agent
	pub process_name: String,
	/// Whether the agent is running as elevated
	pub elevated: bool,
}

/// The checkin struct used to check in the agent
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Checkin {
	/// The OS name
	pub operative_system: String,
	/// The victim hostname
	pub hostname: String,
	/// The domain of the victim
	pub domain: String,
	/// The username of whose runs the agent
	pub username: String,
	/// The internal IP of the victim
	pub ip: String,
	/// The process ID of the agent
	pub process_id: i64,
	/// The parent process ID of the agent
	pub parent_process_id: i64,
	/// The process name of the agent
	pub process_name: String,
	/// Whether the agent is running as elevated
	pub elevated: bool,
	/// The metadata of the struct
	#[serde(skip_serializing, skip_deserializing)]
	metadata: Option<Arc<Metadata>>,
}

impl Checkin {
	pub fn new(partial: PartialCheckin) -> Self {
		Self {
			operative_system: partial.operative_system,
			hostname: partial.hostname,
			domain: partial.domain,
			username: partial.username,
			ip: partial.ip,
			process_id: partial.process_id,
			parent_process_id: partial.parent_process_id,
			process_name: partial.process_name,
			elevated: partial.elevated,
			metadata: None,
		}
	}

	pub fn with_metadata(&mut self, metadata: Metadata) -> &mut Self {
		self.metadata = Some(Arc::new(metadata));
		self
	}
}

impl WithMetadata for Checkin {
	fn get_metadata(&self) -> Arc<Metadata> {
		self.metadata.clone().unwrap().clone()
	}
}

/// The checkin response struct
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct CheckinResponse {
	/// The unique identifier of the agent, required to poll for tasks
	pub id: String,
	/// The unix timestamp of the kill date, if any
	pub kill_date: Option<u64>,
	/// The working hours of for the current day (unix timestamp), if any
	pub working_hours: Option<u64>,
	/// The agent polling interval in milliseconds
	pub polling_interval: u64,
	/// The agent polling jitter in milliseconds
	pub polling_jitter: u64,
}