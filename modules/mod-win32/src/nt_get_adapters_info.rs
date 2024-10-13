use core::slice;

extern crate alloc;

use alloc::{format, string::String, vec::Vec};

use mod_agentcore::instance;
use rs2_win32::ntdef::KeyBasicInformation;

use crate::nt_reg_api::{nt_open_key, nt_query_value_key};

/// Main function to get the primary active IP addresses.
///
/// This function enumerates the registry keys under the given TCP/IP interfaces path,
/// reads the IP address and DHCP server for each interface, and collects them into a vector.
///
/// # Returns
///
/// * `Result<Vec<(String, String, String)>, i32>` - A result containing a vector of tuples with the interface name, IP
///   address, and DHCP server.
///
/// # Safety
///
/// This function is unsafe because it interacts with raw pointers and low-level system calls.
pub unsafe fn get_adapters_info() -> Result<Vec<(String, String, String)>, i32> {
    // Path to the registry key containing network interface information
    let registry_key = "\\Registry\\Machine\\System\\CurrentControlSet\\Services\\Tcpip\\Parameters\\Interfaces";

    // Open the registry key and obtain a handle
    let key_handle = match nt_open_key(registry_key) {
        Ok(handle) => handle,
        Err(status) => return Err(status),
    };

    let mut ip_addresses = Vec::new();

    let mut index = 0;
    let mut result_buffer: [u16; 256] = [0; 256]; // Buffer to hold the key information
    loop {
        let mut result_length: u32 = 0;

        // Enumerate the subkeys of the opened registry key
        let status = instance().ntdll.nt_enumerate_key.run(
            key_handle,
            index,
            0,
            result_buffer.as_mut_ptr() as *mut _,
            result_buffer.len() as u32 * 2,
            &mut result_length,
        );

        // If the enumeration fails, check if it's the first key or break the loop
        if status != 0 {
            if index == 0 {
                instance().ntdll.nt_close.run(key_handle);
                return Err(status);
            }
            else {
                break;
            }
        }

        // Interpret the result buffer as a KeyBasicInformation structure
        let key_info_ptr = result_buffer.as_ptr() as *const KeyBasicInformation;
        let key_info_ref = &*key_info_ptr;

        // Extract the name of the subkey
        let name_length = key_info_ref.name_length as usize;
        let name_slice = slice::from_raw_parts(key_info_ref.name.as_ptr(), name_length / 2);
        let key_name_str: String = String::from_utf16_lossy(name_slice);
        let sub_key_path = format!("{}\\{}", registry_key, key_name_str);

        // Open the subkey to access its values
        let sub_key_handle = match nt_open_key(&sub_key_path) {
            Ok(handle) => handle,
            Err(_) => {
                index += 1;
                continue;
            },
        };

        let mut name = String::new();
        let mut dhcp_server = String::new();
        let mut ip_address = String::new();

        // Check if both DhcpIPAddress and DhcpServer values exist
        if let Ok(ip_address_value) = nt_query_value_key(sub_key_handle, "DhcpIPAddress\0") {
            if let Ok(dhcp_server_value) = nt_query_value_key(sub_key_handle, "DhcpServer\0") {
                ip_address = ip_address_value;
                dhcp_server = dhcp_server_value;
            }
        }
        else if let Ok(ip_address_value) = nt_query_value_key(sub_key_handle, "IPAddress\0") {
            ip_address = ip_address_value;
        }

        // If no IP address is found, skip to the next key
        if ip_address.is_empty() {
            index += 1;
            instance().ntdll.nt_close.run(sub_key_handle);
            continue;
        }

        // Construct the path to the key that contains the interface name
        let name_key_path = format!(
            "\\Registry\\Machine\\System\\CurrentControlSet\\Control\\Network\\\
             {{4D36E972-E325-11CE-BFC1-08002BE10318}}\\{}\\Connection",
            key_name_str
        );

        // Open the key to read the interface name
        let name_key_handle = match nt_open_key(&name_key_path) {
            Ok(handle) => handle,
            Err(_) => {
                index += 1;
                continue;
            },
        };

        // Query the "Name" value from the connection key
        if let Ok(name_value) = nt_query_value_key(name_key_handle, "Name\0") {
            name = name_value;
        }

        // Store the gathered information in the vector
        ip_addresses.push((name, ip_address, dhcp_server));

        // Close the handles to the subkey and name key
        instance().ntdll.nt_close.run(sub_key_handle);
        instance().ntdll.nt_close.run(name_key_handle);
        index += 1;
    }

    // Close the handle to the main registry key
    instance().ntdll.nt_close.run(key_handle);
    Ok(ip_addresses)
}

#[cfg(test)]
mod tests {
    use libc_print::libc_println;

    use super::*;

    #[test]
    fn test_get_adapters_info() {
        unsafe {
            let ip_addresses = match get_adapters_info() {
                Ok(ip_addresses) => ip_addresses,
                Err(status) => {
                    libc_println!("NtOpenKey failed with NT STATUS: {:#X}", status);
                    return;
                },
            };

            let test_one = &ip_addresses[0].1;

            libc_println!("test_one: {}", test_one);
            for (name, ip, dhcp) in ip_addresses {
                libc_println!("Name: {}, IP Address: {}, Dhcp Server: {}", name, ip, dhcp);
            }
        }
    }
}
