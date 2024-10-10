use crate::metadata::{Metadata, WithMetadata};
use crate::network_interface::NetworkInterface;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub struct PartialCheckin {
    /// The OS name
    pub operative_system: String,
    /// The victim hostname
    pub hostname: String,
    /// The domain of the victim
    pub domain: String,
    /// The username of whose runs the agent
    pub username: String,
    /// The internal IP(s) of the victim (multiple network interfaces)
    pub network_interfaces: Vec<NetworkInterface>,
    /// The process ID of the agent
    pub process_id: i64,
    /// The parent process ID of the agent
    pub parent_process_id: i64,
    /// The process name of the agent
    pub process_name: String,
    /// The integrity level of the agent
    pub integrity_level: i16,
    /// The current working directory of the agent
    pub cwd: String,
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
    /// The internal IP(s) of the victim (multiple network interfaces)
    pub network_interfaces: Vec<NetworkInterface>,
    /// The process ID of the agent
    pub process_id: i64,
    /// The parent process ID of the agent
    pub parent_process_id: i64,
    /// The process name of the agent
    pub process_name: String,
    /// The integrity level of the agent
    pub integrity_level: i16,
    /// The current working directory of the agent
    pub cwd: String,
    /// The metadata of the struct
    metadata: Option<Arc<Metadata>>,
}

impl Checkin {
    pub fn new(partial: PartialCheckin) -> Self {
        Self {
            operative_system: partial.operative_system,
            hostname: partial.hostname,
            domain: partial.domain,
            username: partial.username,
            network_interfaces: partial.network_interfaces,
            process_id: partial.process_id,
            parent_process_id: partial.parent_process_id,
            process_name: partial.process_name,
            integrity_level: partial.integrity_level,
            cwd: partial.cwd,
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
        self.metadata.as_ref().unwrap().clone()
    }
}

/// The checkin response struct
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct CheckinResponse {
    /// The unique identifier of the agent, required to poll for tasks
    pub id: String,
    /// The unix timestamp of the kill date, if any
    pub kill_date: Option<i64>,
    /// The working hours of for the current day (unix timestamp), if any
    pub working_hours: Option<Vec<Option<i64>>>,
    /// The agent polling interval in milliseconds
    pub polling_interval: i64,
    /// The agent polling jitter in milliseconds
    pub polling_jitter: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checkin() {
        let mut checkin = Checkin::new(PartialCheckin {
            operative_system: "Windows".to_string(),
            hostname: "DESKTOP-PC".to_string(),
            domain: "WORKGROUP".to_string(),
            username: "user".to_string(),
            network_interfaces: Vec::new(),
            process_id: 1234,
            parent_process_id: 5678,
            process_name: "agent.exe".to_string(),
            integrity_level: 0,
            cwd: "C:\\Users\\Public\\rs2-agent.exe".to_string(),
        });

        let metadata = Metadata {
            request_id: "request_id".to_string(),
            command_id: "command_id".to_string(),
            agent_id: "agent_id".to_string(),
            path: None,
        };

        checkin.with_metadata(metadata);

        println!("{}", serde_json::to_string_pretty(&checkin).unwrap());
    }
}
