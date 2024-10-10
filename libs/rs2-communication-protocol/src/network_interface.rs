use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NetworkInterface {
    /// The name of the network interface
    pub name: String,
    /// The IP address of the network interface
    pub address: String,
    /// The DHCP server of the network interface
    pub dhcp_server: String,
}
