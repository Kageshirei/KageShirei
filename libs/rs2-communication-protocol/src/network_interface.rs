use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NetworkInterface {
    /// The name of the network interface
    pub name: Option<String>,
    /// The IP address of the network interface
    pub address: Option<String>,
    /// The DHCP server of the network interface
    pub dhcp_server: Option<String>,
}

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
    pub fn new(name: Option<String>, address: Option<String>, dhcp_server: Option<String>) -> Self {
        NetworkInterface {
            name,
            address,
            dhcp_server,
        }
    }

    /// Converts a Vec of tuples (name, address, dhcp_server) into a Vec of NetworkInterface instances.
    ///
    /// # Arguments
    /// * `data` - A Vec of tuples where each tuple contains three strings: name, address, and dhcp_server.
    ///
    /// # Returns
    /// A Vec of `NetworkInterface` instances created from the input data.
    pub fn from_tuples(data: Vec<(String, String, String)>) -> Vec<NetworkInterface> {
        data.into_iter()
            .map(|(name, address, dhcp_server)| NetworkInterface {
                name: Some(name),
                address: Some(address),
                dhcp_server: Some(dhcp_server),
            })
            .collect()
    }
}
