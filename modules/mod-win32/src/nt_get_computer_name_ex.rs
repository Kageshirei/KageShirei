use core::{ffi::c_void, ops::Div as _, ptr};

extern crate alloc;

use alloc::{vec, vec::Vec};

use kageshirei_win32::{
    ntdef::{KeyValuePartialInformation, UnicodeString, REG_SZ, ULONG},
    ntstatus::{STATUS_BUFFER_OVERFLOW, STATUS_BUFFER_TOO_SMALL},
};
use libc_print::libc_println;
use mod_agentcore::instance;

use crate::{nt_reg_api::nt_open_key, utils::NT_STATUS};

/// Retrieves the computer name from the registry.
///
/// # Arguments
///
/// * `registry_key` - A string slice that holds the path to the registry key.
/// * `value_name_str` - A string slice that holds the name of the value to retrieve.
/// * `lp_buffer` - A mutable reference to a vector where the retrieved data will be stored.
/// * `n_size` - A mutable reference to an unsigned long that will hold the size of the retrieved
///   data.
///
/// # Safety
/// This function performs unsafe operations, including:
/// * Dereferencing raw pointers (`ptr::null_mut`, `key_info.as_ptr`, etc.).
/// * Interacting with the Windows Registry and accessing system-level APIs that require strict
///   adherence to correct usage.
/// * Modifying the contents of `lp_buffer` and `n_size`, which must be valid, mutable references
///   provided by the caller.
///
/// The caller must ensure that:
/// * The registry key and value names provided are valid and accessible.
/// * The `lp_buffer` and `n_size` parameters point to valid memory.
/// * The data retrieved from the registry is of the expected type and size.
///
/// Misuse or incorrect assumptions about the registry's structure or state may lead to undefined
/// behavior or memory corruption.
///
/// # Returns
///
/// * `true` if the operation was successful, `false` otherwise.
///
/// This function queries the registry for a specific value and stores the result in a buffer.
/// It first determines the required buffer size, allocates the buffer, and then retrieves the data.
pub unsafe fn get_computer_name_from_registry(
    registry_key: &str,
    value_name_str: &str,
    lp_buffer: &mut Vec<u16>,
    n_size: &mut ULONG,
) -> bool {
    // Check if the registry key is empty
    if registry_key.is_empty() {
        return false;
    }

    // Open the registry key and obtain a handle
    let key_handle = match nt_open_key(registry_key) {
        Ok(handle) => handle,
        Err(_) => {
            libc_println!("[!] NtOpenKey failed handle is null");
            return false;
        },
    };

    // Initialize the Unicode string for the registry value name
    let value_utf16_string: Vec<u16> = value_name_str.encode_utf16().collect();
    let mut value_name = UnicodeString::new();
    value_name.init(value_utf16_string.as_ptr());

    // Query the registry value to get the required buffer size
    let mut result_length: ULONG = 0;
    let status = instance().ntdll.nt_query_value_key.run(
        key_handle,
        &value_name,
        2,
        ptr::null_mut::<c_void>(),
        0,
        &mut result_length,
    );

    // Check if the query resulted in a buffer overflow or buffer too small error, which is expected
    if status != STATUS_BUFFER_OVERFLOW && status != STATUS_BUFFER_TOO_SMALL {
        libc_println!("[!] NtQueryValueKey failed: {}", NT_STATUS(status));
        instance().ntdll.nt_close.run(key_handle);
        return false;
    }

    if result_length == 0 {
        libc_println!("[!] NtQueryValue result lenght is 0");
        instance().ntdll.nt_close.run(key_handle);
        return false;
    }

    // Allocate the buffer for the registry data
    let key_info_size = result_length;
    let mut key_info = vec![0u8; key_info_size as usize];

    // Query the registry value to get the actual data
    let status = instance().ntdll.nt_query_value_key.run(
        key_handle,
        &value_name,
        2,
        key_info.as_mut_ptr() as *mut c_void,
        key_info_size as ULONG,
        &mut result_length,
    );

    // Check if the query was successful
    if status < 0 {
        libc_println!("[!] Second NtQueryValueKey failed: {}", NT_STATUS(status));
        instance().ntdll.nt_close.run(key_handle);
        return false;
    }

    // Process the key_info data
    let key_info_ptr = key_info.as_ptr() as *const KeyValuePartialInformation;
    let key_info_ref = &*key_info_ptr;

    // Ensure the data type is REG_SZ (string)
    if key_info_ref.data_type != REG_SZ {
        libc_println!(
            "[!] Key Info data type is wrong: {}",
            key_info_ref.data_type
        );
        instance().ntdll.nt_close.run(key_handle);
        return false;
    }

    // Update the size and buffer with the registry data
    *n_size = key_info_ref.data_length.div(2);
    lp_buffer.clear();
    lp_buffer.extend_from_slice(core::slice::from_raw_parts(
        key_info_ref.data.as_ptr() as *const u16,
        *n_size as usize,
    ));

    // Close the registry key
    instance().ntdll.nt_close.run(key_handle) == 0
}

/// Enum representing the different formats of computer names that can be retrieved.
#[derive(Debug)]
pub enum ComputerNameFormat {
    /// The NetBIOS name of the computer.
    ComputerNameNetBIOS,
    /// The DNS domain name of the computer.
    ComputerNameDnsDomain,
    /// The DNS hostname of the computer.
    ComputerNameDnsHostname,
    /// The physical DNS domain name of the computer.
    ComputerNamePhysicalDnsDomain,
    /// The physical DNS hostname of the computer.
    ComputerNamePhysicalDnsHostname,
}

/// Retrieves the computer name based on the specified format.
///
/// This function queries the Windows registry to obtain the computer name in the requested format.
/// It supports various formats, including NetBIOS, DNS domain, DNS hostname, physical DNS domain,
/// and physical DNS hostname. The result is stored in the provided buffer.
///
/// # Arguments
///
/// * `name_type` - The format in which the computer name is requested. This can be one of:
///     - `ComputerNameFormat::ComputerNameNetBIOS`: The NetBIOS name of the computer.
///     - `ComputerNameFormat::ComputerNameDnsDomain`: The DNS domain name of the computer.
///     - `ComputerNameFormat::ComputerNameDnsHostname`: The DNS hostname of the computer.
///     - `ComputerNameFormat::ComputerNamePhysicalDnsDomain`: The physical DNS domain name of the
///       computer.
///     - `ComputerNameFormat::ComputerNamePhysicalDnsHostname`: The physical DNS hostname of the
///       computer.
/// * `lp_buffer` - A mutable reference to a vector where the retrieved name will be stored. The
///   buffer will be resized as needed to accommodate the name.
/// * `n_size` - A mutable reference to an unsigned long that will hold the size of the retrieved
///   data. On input, it specifies the size of the buffer. On output, it receives the number of
///   characters stored in the buffer, excluding the null terminator.
///
/// # Safety
/// This function performs unsafe operations, such as dereferencing raw pointers and interacting
/// with Windows Registry APIs. The caller must ensure that:
/// * `lp_buffer` is a valid and writable buffer.
/// * `n_size` points to valid memory.
/// * The provided `name_type` corresponds to a valid registry key.
///
/// # Returns
///
/// * `true` if the operation was successful, `false` otherwise. If the function fails, the buffer
///   and size are not modified.
pub unsafe fn get_computer_name_ex(name_type: ComputerNameFormat, lp_buffer: &mut Vec<u16>, n_size: &mut u32) -> bool {
    // Check if the buffer is empty and the requested size is greater than 0
    if lp_buffer.is_empty() && *n_size > 0 {
        return false;
    }

    // Match on the requested computer name format and retrieve the appropriate value from the registry
    match name_type {
        ComputerNameFormat::ComputerNameNetBIOS => {
            let mut ret = get_computer_name_from_registry(
                "\\Registry\\Machine\\System\\CurrentControlSet\\Control\\ComputerName\\ActiveComputerName\0",
                "ComputerName\0",
                lp_buffer,
                n_size,
            );
            if !ret {
                ret = get_computer_name_from_registry(
                    "\\Registry\\Machine\\System\\CurrentControlSet\\Control\\ComputerName\\ComputerName\0",
                    "ComputerName\0",
                    lp_buffer,
                    n_size,
                );
            }
            ret
        },
        ComputerNameFormat::ComputerNameDnsDomain => {
            get_computer_name_from_registry(
                "\\Registry\\Machine\\System\\CurrentControlSet\\Services\\Tcpip\\Parameters\0",
                "Domain\0",
                lp_buffer,
                n_size,
            )
        },
        ComputerNameFormat::ComputerNameDnsHostname => {
            get_computer_name_from_registry(
                "\\Registry\\Machine\\System\\CurrentControlSet\\Services\\Tcpip\\Parameters\0",
                "Hostname\0",
                lp_buffer,
                n_size,
            )
        },
        ComputerNameFormat::ComputerNamePhysicalDnsDomain => {
            get_computer_name_from_registry(
                "\\Registry\\Machine\\System\\CurrentControlSet\\Services\\Tcpip\\Parameters\0",
                "NV Domain\0",
                lp_buffer,
                n_size,
            )
        },
        ComputerNameFormat::ComputerNamePhysicalDnsHostname => {
            get_computer_name_from_registry(
                "\\Registry\\Machine\\System\\CurrentControlSet\\Services\\Tcpip\\Parameters\0",
                "NV Hostname\0",
                lp_buffer,
                n_size,
            )
        },
    }
}

// #[cfg(test)]
// mod tests {
// use alloc::string::String;
//
// use libc_print::libc_println;
//
// use super::*;
//
// #[test]
// fn test_get_computer_name_from_registry() {
// unsafe {
// let registry_key =
// "\\Registry\\Machine\\System\\CurrentControlSet\\Control\\ComputerName\\ActiveComputerName\0";
// let value_name_str = "ComputerName\0";
// let mut lp_buffer = Vec::new();
// let mut n_size: ULONG = 0;
// let success = get_computer_name_from_registry(registry_key, value_name_str, &mut lp_buffer, &mut
// n_size);
//
// libc_println!("Success: {:?}", success);
// libc_println!("Computer Name: {:?}", String::from_utf16_lossy(&lp_buffer));
// libc_println!("Size: {:?}", n_size);
// }
// }
//
// #[test]
// fn test_get_computer_name_netbios() {
// unsafe {
// let mut buffer = Vec::new();
// let mut size: u32 = 0;
// let success = get_computer_name_ex(
// ComputerNameFormat::ComputerNameNetBIOS,
// &mut buffer,
// &mut size,
// );
//
// assert!(success, "Failed to get ComputerNameNetBIOS");
// libc_println!(
// "ComputerNameNetBIOS: {:?}, Size: {}",
// String::from_utf16_lossy(&buffer),
// size
// );
// }
// }
//
// #[test]
// fn test_get_computer_name_dns_domain() {
// unsafe {
// let mut buffer = Vec::new();
// let mut size: u32 = 0;
// let success = get_computer_name_ex(
// ComputerNameFormat::ComputerNameDnsDomain,
// &mut buffer,
// &mut size,
// );
//
// assert!(success, "Failed to get ComputerNameDnsDomain");
// libc_println!(
// "ComputerNameDnsDomain: {:?}, Size: {}",
// String::from_utf16_lossy(&buffer),
// size
// );
// }
// }
//
// #[test]
// fn test_get_computer_name_dns_hostname() {
// unsafe {
// let mut buffer = Vec::new();
// let mut size: u32 = 0;
// let success = get_computer_name_ex(
// ComputerNameFormat::ComputerNameDnsHostname,
// &mut buffer,
// &mut size,
// );
//
// assert!(success, "Failed to get ComputerNameDnsHostname");
// libc_println!(
// "ComputerNameDnsHostname: {:?}, Size: {}",
// String::from_utf16_lossy(&buffer),
// size
// );
// }
// }
//
// #[test]
// fn test_get_computer_name_physical_dns_domain() {
//     unsafe {
//         let mut buffer = Vec::new();
//         let mut size: u32 = 0;
//         let success = nt_get_computer_name_ex(
//             ComputerNameFormat::ComputerNamePhysicalDnsDomain,
//             &mut buffer,
//             &mut size,
//         );
//
//         assert!(success, "Failed to get ComputerNamePhysicalDnsDomain");
//         libc_println!(
//             "ComputerNamePhysicalDnsDomain: {:?}, Size: {}",
//             String::from_utf16_lossy(&buffer),
//             size
//         );
//     }
// }
//
// #[test]
// fn test_get_computer_name_physical_dns_hostname() {
// unsafe {
// let mut buffer = Vec::new();
// let mut size: u32 = 0;
// let success = get_computer_name_ex(
// ComputerNameFormat::ComputerNamePhysicalDnsHostname,
// &mut buffer,
// &mut size,
// );
//
// assert!(success, "Failed to get ComputerNamePhysicalDnsHostname");
// libc_println!(
// "ComputerNamePhysicalDnsHostname: {:?}, Size: {}",
// String::from_utf16_lossy(&buffer),
// size
// );
// }
// }
// }
