use kageshirei_communication_protocol::NetworkInterfaceArray;
use sea_orm::{prelude::DateTime, DerivePartialModel, FromQueryResult};
use serde::{Deserialize, Serialize};

use crate::{active_enums::AgentIntegrity, entities::prelude::Agent};

#[derive(DerivePartialModel, FromQueryResult, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[sea_orm(entity = "Agent")]
pub struct FullSessionRecord {
    /// The unique identifier for the agent (cuid2)
    pub id:                 String,
    /// The OS name
    pub operating_system:   String,
    /// The victim hostname
    pub hostname:           String,
    /// The domain of the victim
    pub domain:             Option<String>,
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
    pub integrity:          AgentIntegrity,
    /// The current working directory of the agent
    pub cwd:                String,
    /// The agent's creation date
    pub created_at:         DateTime,
}
