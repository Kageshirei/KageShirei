extern crate alloc;

use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use core::{ptr::null_mut, slice};

use mod_agentcore::instance;
use rs2_win32::{
    ntdef::{
        KeyBasicInformation,
        KeyValuePartialInformation,
        ObjectAttributes,
        UnicodeString,
        HANDLE,
        KEY_ENUMERATE_SUB_KEYS,
        KEY_READ,
        OBJ_CASE_INSENSITIVE,
    },
    ntstatus::{STATUS_BUFFER_OVERFLOW, STATUS_BUFFER_TOO_SMALL, STATUS_OBJECT_NAME_NOT_FOUND},
};

/// Opens a registry key and returns the handle.
///
/// This function initializes a `UnicodeString` and an `ObjectAttributes` structure,
/// and then calls the `NtOpenKey` syscall to open the specified registry key. The function
/// supports reading and enumerating subkeys.
///
/// # Parameters
/// - `key`: A string slice containing the path to the registry key that needs to be opened.
///
/// # Returns
/// - `Result<HANDLE, i32>`: A result containing the handle to the opened registry key if successful, otherwise an error
///   code (`NTSTATUS`) indicating the reason for failure.
///
/// # Details
/// This function uses the following NT API function:
/// - `NtOpenKey`: To open the registry key specified by the provided path.
///
/// # Safety
/// This function is marked as unsafe because it interacts with raw pointers and makes low-level
/// system calls that could lead to undefined behavior if not used correctly. Ensure that the input
/// string is a valid registry path, and that the function is called within a safe context.
pub unsafe fn nt_open_key(key: &str) -> Result<HANDLE, i32> {
    let mut key_handle: HANDLE = null_mut(); // Initialize the handle to null

    // Initialize the Unicode string for the registry key path.
    let mut key_name = UnicodeString::new();
    let utf16_string: Vec<u16> = key.encode_utf16().chain(Some(0)).collect(); // Convert the key string to UTF-16
    key_name.init(utf16_string.as_ptr()); // Initialize the UnicodeString structure

    // Initialize the object attributes for the registry key.
    let mut object_attributes = ObjectAttributes::new();
    ObjectAttributes::initialize(
        &mut object_attributes,
        &mut key_name,
        OBJ_CASE_INSENSITIVE, // Use case-insensitive name matching (0x40)
        null_mut(),           // No root directory
        null_mut(),           // No security descriptor
    );

    // Call NtOpenKey to open the registry key with the desired access rights.
    let ntstatus = instance().ntdll.nt_open_key.run(
        &mut key_handle,                   // Pointer to receive the key handle
        KEY_READ | KEY_ENUMERATE_SUB_KEYS, // Desired access: read and enumerate subkeys
        &mut object_attributes,            // Provide the object attributes for the key
    );

    // Check if the operation was successful
    if ntstatus != 0 {
        return Err(ntstatus); // Return the NTSTATUS error code if opening the key failed
    }

    Ok(key_handle) // Return the handle to the opened registry key
}

/// Reads a registry value and returns its content as a string.
///
/// This function initializes a `UnicodeString` for the value name,
/// and then calls the `NtQueryValueKey` syscall to retrieve the value data.
/// If the initial buffer size is insufficient, the function reallocates the buffer based on the required length
/// and retries the call until it either succeeds or fails with a different error.
///
/// # Parameters
/// - `key_handle`: The handle to the open registry key from which the value will be read.
/// - `value_name`: A string slice that specifies the name of the registry value to be read.
///
/// # Returns
/// - `Result<String, i32>`: A result containing the value content as a string if successful, or an error code
///   (`NTSTATUS`) if the operation fails.
///
/// # Safety
/// This function is marked as unsafe because it directly interacts with raw pointers and performs low-level
/// system calls, which can result in undefined behavior if not handled correctly.
pub unsafe fn nt_query_value_key(key_handle: HANDLE, value_name: &str) -> Result<String, i32> {
    // Convert the value name to a UTF-16 encoded string
    let value_utf16_string: Vec<u16> = value_name.encode_utf16().chain(Some(0)).collect();

    // Initialize the UnicodeString structure for the value name
    let mut value_unicode = UnicodeString::new();
    value_unicode.init(value_utf16_string.as_ptr());

    let mut value_result_length: u32 = 0; // Variable to store the length of the value data
    let mut value_info: Vec<u8> = Vec::with_capacity(64); // Initial buffer to store value information
    let mut ntstatus;

    loop {
        // Call NtQueryValueKey to retrieve the value data
        ntstatus = instance().ntdll.nt_query_value_key.run(
            key_handle,
            &value_unicode,
            2,                                 // Query type: KeyValuePartialInformation
            value_info.as_mut_ptr() as *mut _, // Pointer to the buffer for the value data
            value_info.capacity() as u32,      // Current buffer capacity
            &mut value_result_length,          // Variable to store the required buffer length
        );

        // Handle the different status codes
        if ntstatus == STATUS_OBJECT_NAME_NOT_FOUND {
            // The specified value name was not found
            return Err(STATUS_OBJECT_NAME_NOT_FOUND);
        } else if ntstatus == STATUS_BUFFER_OVERFLOW || ntstatus == STATUS_BUFFER_TOO_SMALL {
            // The buffer was too small; resize it and retry
            value_info.reserve(value_result_length as usize);
            continue;
        } else if ntstatus != 0 {
            // Other errors
            return Err(ntstatus);
        } else {
            break; // Successfully retrieved the value data
        }
    }

    // Interpret the retrieved data as a KeyValuePartialInformation structure
    let value_info_ptr = value_info.as_ptr() as *const KeyValuePartialInformation;
    let value_info_ref = &*value_info_ptr;
    let data_length = value_info_ref.data_length as usize;

    // Extract the data as a UTF-16 string
    let data_slice = slice::from_raw_parts(value_info_ref.data.as_ptr() as *const u16, data_length / 2);

    // Convert the UTF-16 string to a Rust string and remove trailing null characters
    let value = String::from_utf16_lossy(&data_slice)
        .trim_end_matches('\0')
        .to_string();

    Ok(value) // Return the value as a string
}

/// Enumerates sub-keys of a given registry key.
///
/// This function opens a specified registry key and enumerates its sub-keys,
/// returning them as a vector of strings. The function utilizes the `NtEnumerateKey` syscall
/// to retrieve the names of the sub-keys.
///
/// # Parameters
/// - `key`: A string slice that specifies the path to the registry key to be enumerated.
///
/// # Returns
/// - `Result<Vec<String>, i32>`: A result containing a vector of sub-key names if successful, or an error code
///   (`NTSTATUS`) if the operation fails.
///
/// # Safety
/// This function is marked as unsafe because it directly interacts with raw pointers and performs low-level
/// system calls, which can result in undefined behavior if not handled correctly.
pub unsafe fn nt_enumerate_key(key: &str) -> Result<Vec<String>, i32> {
    // Open the registry key
    let key_handle = nt_open_key(key)?;
    let mut sub_keys = Vec::new(); // Vector to store the sub-key names

    let mut index = 0; // Index for enumerating sub-keys
    let mut result_buffer: [u16; 256] = [0; 256]; // Buffer to store the name of each sub-key

    loop {
        let mut result_length: u32 = 0; // Variable to store the length of the result

        // Call NtEnumerateKey to retrieve the next sub-key name
        let status = instance().ntdll.nt_enumerate_key.run(
            key_handle,
            index,                                // Index of the sub-key to enumerate
            0,                                    // Key information class (0 for KeyBasicInformation)
            result_buffer.as_mut_ptr() as *mut _, // Buffer to receive the sub-key information
            result_buffer.len() as u32 * 2,       // Buffer length in bytes
            &mut result_length,                   // Variable to receive the length of the result
        );

        // Check the status of the operation
        if status != 0 {
            if index == 0 {
                // If the first enumeration fails, close the key handle and return the error
                instance().ntdll.nt_close.run(key_handle);
                return Err(status);
            } else {
                // If there are no more sub-keys, break the loop
                break;
            }
        }

        // Interpret the result as KeyBasicInformation
        let key_info_ptr = result_buffer.as_ptr() as *const KeyBasicInformation;
        let key_info_ref = &*key_info_ptr;

        // Extract the name of the sub-key
        let name_length = key_info_ref.name_length as usize;
        let name_slice = slice::from_raw_parts(key_info_ref.name.as_ptr(), name_length / 2);
        let sub_key_name: String = String::from_utf16_lossy(name_slice);

        // Store the sub-key name in the vector
        sub_keys.push(sub_key_name);
        index += 1; // Increment the index to enumerate the next sub-key
    }

    // Close the registry key handle
    instance().ntdll.nt_close.run(key_handle);
    Ok(sub_keys) // Return the vector of sub-key names
}

#[cfg(test)]
mod tests {
    use libc_print::libc_println;
    use rs2_win32::ntstatus::NT_SUCCESS;

    use super::*;
    use crate::utils::NT_STATUS;

    #[test]
    fn test_nt_open_key() {
        unsafe {
            // Try to open a well-known registry key
            let registry_key = r"\Registry\Machine\Software\Microsoft\Windows\CurrentVersion";
            match nt_open_key(registry_key) {
                Ok(handle) => {
                    libc_println!("Successfully opened registry key: {}\n", registry_key);
                    instance().ntdll.nt_close.run(handle);
                },
                Err(status) => {
                    libc_println!(
                        "Failed to open registry key: {}. NTSTATUS: {}",
                        registry_key,
                        NT_STATUS(status)
                    );
                    assert!(
                        NT_SUCCESS(status),
                        "Expected success, but got NTSTATUS: {}",
                        NT_STATUS(status)
                    );
                },
            }
        }
    }

    #[test]
    fn test_nt_query_value_key() {
        unsafe {
            // First, open a well-known registry key
            let registry_key = r"\Registry\Machine\Software\Microsoft\Windows\CurrentVersion";
            let key_handle = match nt_open_key(registry_key) {
                Ok(handle) => handle,
                Err(status) => {
                    libc_println!(
                        "Failed to open registry key: {}. NTSTATUS: {}",
                        registry_key,
                        NT_STATUS(status)
                    );
                    return;
                },
            };

            // Query a well-known value from the opened registry key
            let value_name = "ProgramFilesDir";
            match nt_query_value_key(key_handle, value_name) {
                Ok(value) => {
                    libc_println!("Successfully queried value: {} = {}\n", value_name, value);
                },
                Err(status) => {
                    libc_println!(
                        "Failed to query value: {}. NTSTATUS: {}",
                        value_name,
                        NT_STATUS(status)
                    );
                    assert!(
                        NT_SUCCESS(status),
                        "Expected success, but got NTSTATUS: {}",
                        NT_STATUS(status)
                    );
                },
            }

            instance().ntdll.nt_close.run(key_handle);
        }
    }

    #[test]
    fn test_nt_enumerate_key() {
        unsafe {
            let registry_key =
                "\\Registry\\Machine\\System\\CurrentControlSet\\Services\\Tcpip\\Parameters\\Interfaces";
            match nt_enumerate_key(registry_key) {
                Ok(sub_keys) => {
                    for sub_key in sub_keys {
                        libc_println!("Sub-key: {}\\{}", registry_key, sub_key);
                    }
                },
                Err(status) => {
                    libc_println!(
                        "Failed to enumerate sub-keys. NT STATUS: {}",
                        NT_STATUS(status)
                    );
                },
            }
        }
    }
}
