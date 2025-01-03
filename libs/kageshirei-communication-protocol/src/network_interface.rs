//! Create a NetworkInterface struct to store information about a network
//! interface. The struct has three fields: name, address, and dhcp_server.

use alloc::{string::String, vec::Vec};

#[cfg(feature = "server")]
use sea_orm::FromJsonQueryResult;
use serde::{Deserialize, Serialize};

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(FromJsonQueryResult, Debug))]
pub struct NetworkInterface {
    /// The name of the network interface
    pub name:        Option<String>,
    /// The IP address of the network interface
    pub address:     Option<String>,
    /// The DHCP server of the network interface
    pub dhcp_server: Option<String>,
}

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(FromJsonQueryResult, Debug))]
#[allow(
    clippy::module_name_repetitions,
    reason = "The name repetition clarifies the struct's purpose as an array of NetworkInterface objects."
)]
pub struct NetworkInterfaceArray {
    pub network_interfaces: Vec<NetworkInterface>,
}

// Safety: The struct is safe to send and share between threads
unsafe impl Send for NetworkInterface {}
// Safety: The struct is safe to send and share between threads
unsafe impl Sync for NetworkInterface {}

impl NetworkInterface {
    /// Creates a new NetworkInterface instance with optional parameters.
    ///
    /// # Arguments
    /// * `name` - The name of the network interface (optional).
    /// * `address` - The IP address of the network interface (optional).
    /// * `dhcp_server` - The DHCP server of the network interface (optional).
    ///
    /// # Returns
    /// A new instance of `NetworkInterface`.
    pub const fn new(name: Option<String>, address: Option<String>, dhcp_server: Option<String>) -> Self {
        Self {
            name,
            address,
            dhcp_server,
        }
    }

    /// Converts a Vec of tuples (name, address, dhcp_server) into a Vec of
    /// NetworkInterface instances.
    ///
    /// # Arguments
    /// * `data` - A Vec of tuples where each tuple contains three strings: name, address, and
    ///   dhcp_server.
    ///
    /// # Returns
    /// A Vec of `NetworkInterface` instances created from the input data.
    pub fn from_tuples(data: Vec<(String, String, String)>) -> Vec<Self> {
        data.into_iter()
            .map(|(name, address, dhcp_server)| {
                Self {
                    name:        Some(name),
                    address:     Some(address),
                    dhcp_server: Some(dhcp_server),
                }
            })
            .collect()
    }
}
