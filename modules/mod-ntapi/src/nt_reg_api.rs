extern crate alloc;
extern crate libc_print;

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::ptr::null_mut;
use core::slice;
use rs2_winapi::ntregapi::{KeyBasicInformation, KeyValuePartialInformation};
use rs2_winapi::{
    ntdef::{ObjectAttributes, UnicodeString, HANDLE, OBJ_CASE_INSENSITIVE},
    ntregapi::{KEY_ENUMERATE_SUB_KEYS, KEY_READ},
    ntstatus::{STATUS_BUFFER_OVERFLOW, STATUS_BUFFER_TOO_SMALL, STATUS_OBJECT_NAME_NOT_FOUND},
};

use crate::{nt_close, nt_enumerate_key, nt_open_key, nt_query_value_key};

/// Opens a registry key and returns the handle.
///
/// This function initializes a `UnicodeString` and `ObjectAttributes` structure,
/// and then calls the `NtOpenKey` syscall to open the registry key.
///
/// # Arguments
///
/// * `key` - A string slice that holds the path to the registry key.
///
/// # Returns
///
/// * `Result<HANDLE, i32>` - A result containing the handle to the registry key if successful, otherwise an error code.
///
/// # Safety
///
/// This function is unsafe because it interacts with raw pointers and low-level system calls.
pub unsafe fn open_key(key: &str) -> Result<HANDLE, i32> {
    let mut key_handle: HANDLE = null_mut();

    // Initialize the Unicode string for the registry key
    let mut key_name = UnicodeString::new();
    let utf16_string: Vec<u16> = key.encode_utf16().chain(Some(0)).collect();
    key_name.init(utf16_string.as_ptr());

    // Initialize the object attributes for the registry key
    let mut object_attributes = ObjectAttributes::new();
    ObjectAttributes::initialize(
        &mut object_attributes,
        &mut key_name,
        OBJ_CASE_INSENSITIVE, // 0x40
        null_mut(),
        null_mut(),
    );

    // Open the registry key with NtOpenKey
    let ntstatus = nt_open_key(
        &mut key_handle,
        KEY_READ | KEY_ENUMERATE_SUB_KEYS,
        &mut object_attributes,
    );

    if ntstatus != 0 {
        return Err(ntstatus);
    }

    Ok(key_handle)
}

/// Reads a registry value and returns its content as a string.
///
/// This function initializes a `UnicodeString` for the value name,
/// and then calls the `NtQueryValueKey` syscall to retrieve the value data.
/// If the buffer size is not sufficient, it reallocates the buffer based on the required length
/// and retries the call until it succeeds or fails with a different error.
///
/// # Arguments
///
/// * `key_handle` - The handle to the open registry key.
/// * `value_name` - A string slice that holds the name of the value to be read.
///
/// # Returns
///
/// * `Result<String, i32>` - A result containing the value content as a string if successful, otherwise an error code.
///
/// # Safety
///
/// This function is unsafe because it interacts with raw pointers and low-level system calls.
pub unsafe fn query_value(key_handle: HANDLE, value_name: &str) -> Result<String, i32> {
    let value_utf16_string: Vec<u16> = value_name.encode_utf16().chain(Some(0)).collect();
    let mut value_unicode = UnicodeString::new();
    value_unicode.init(value_utf16_string.as_ptr());

    let mut value_result_length: u32 = 0;
    let mut value_info: Vec<u8> = Vec::with_capacity(64);
    let mut ntstatus;

    loop {
        ntstatus = nt_query_value_key(
            key_handle,
            &value_unicode,
            2,
            value_info.as_mut_ptr() as *mut _,
            value_info.capacity() as u32,
            &mut value_result_length,
        );

        if ntstatus == STATUS_OBJECT_NAME_NOT_FOUND {
            return Err(STATUS_OBJECT_NAME_NOT_FOUND);
        } else if ntstatus == STATUS_BUFFER_OVERFLOW || ntstatus == STATUS_BUFFER_TOO_SMALL {
            value_info.reserve(value_result_length as usize);
            continue;
        } else if ntstatus != 0 {
            return Err(ntstatus);
        } else {
            break;
        }
    }

    let value_info_ptr = value_info.as_ptr() as *const KeyValuePartialInformation;
    let value_info_ref = &*value_info_ptr;
    let data_length = value_info_ref.data_length as usize;

    let data_slice =
        slice::from_raw_parts(value_info_ref.data.as_ptr() as *const u16, data_length / 2);

    let value = String::from_utf16_lossy(&data_slice)
        .trim_end_matches('\0')
        .to_string();
    Ok(value)
}

/// Enumerates sub-keys of a given registry key.
///
/// This function enumerates the sub-keys of the specified registry key
/// and returns them as a vector of strings.
///
/// # Arguments
///
/// * `key` - A string slice that holds the path to the registry key.
///
/// # Returns
///
/// * `Result<Vec<String>, i32>` - A result containing a vector of sub-key names if successful, otherwise an error code.
///
/// # Safety
///
/// This function is unsafe because it interacts with raw pointers and low-level system calls.
pub unsafe fn enumerate_sub_keys(key: &str) -> Result<Vec<String>, i32> {
    let key_handle = open_key(key)?;
    let mut sub_keys = Vec::new();

    let mut index = 0;
    let mut result_buffer: [u16; 256] = [0; 256];
    loop {
        let mut result_length: u32 = 0;

        let status = nt_enumerate_key(
            key_handle,
            index,
            0,
            result_buffer.as_mut_ptr() as *mut _,
            result_buffer.len() as u32 * 2,
            &mut result_length,
        );

        if status != 0 {
            if index == 0 {
                nt_close(key_handle);
                return Err(status);
            } else {
                break;
            }
        }

        let key_info_ptr = result_buffer.as_ptr() as *const KeyBasicInformation;
        let key_info_ref = &*key_info_ptr;

        let name_length = key_info_ref.name_length as usize;
        let name_slice = slice::from_raw_parts(key_info_ref.name.as_ptr(), name_length / 2);
        let sub_key_name: String = String::from_utf16_lossy(name_slice);

        sub_keys.push(sub_key_name);
        index += 1;
    }

    nt_close(key_handle);
    Ok(sub_keys)
}

#[cfg(test)]
mod tests {
    use crate::nt_close;

    use super::*;
    use libc_print::libc_println;
    use rs2_winapi::ntstatus::NT_SUCCESS;

    #[test]
    fn test_open_key() {
        unsafe {
            // Try to open a well-known registry key
            let registry_key = r"\Registry\Machine\Software\Microsoft\Windows\CurrentVersion";
            match open_key(registry_key) {
                Ok(handle) => {
                    libc_println!("Successfully opened registry key: {}\n", registry_key);
                    nt_close(handle); // Don't forget to close the handle after the test
                }
                Err(status) => {
                    libc_println!(
                        "Failed to open registry key: {}. NTSTATUS: {:#X}",
                        registry_key,
                        status
                    );
                    assert!(
                        NT_SUCCESS(status),
                        "Expected success, but got NTSTATUS: {:#X}",
                        status
                    );
                }
            }
        }
    }

    #[test]
    fn test_query_value() {
        unsafe {
            // First, open a well-known registry key
            let registry_key = r"\Registry\Machine\Software\Microsoft\Windows\CurrentVersion";
            let key_handle = match open_key(registry_key) {
                Ok(handle) => handle,
                Err(status) => {
                    libc_println!(
                        "Failed to open registry key: {}. NTSTATUS: {:#X}",
                        registry_key,
                        status
                    );
                    return;
                }
            };

            // Query a well-known value from the opened registry key
            let value_name = "ProgramFilesDir";
            match query_value(key_handle, value_name) {
                Ok(value) => {
                    libc_println!("Successfully queried value: {} = {}\n", value_name, value);
                }
                Err(status) => {
                    libc_println!(
                        "Failed to query value: {}. NTSTATUS: {:#X}",
                        value_name,
                        status
                    );
                    assert!(
                        NT_SUCCESS(status),
                        "Expected success, but got NTSTATUS: {:#X}",
                        status
                    );
                }
            }

            nt_close(key_handle); // Don't forget to close the handle after the test
        }
    }

    #[test]
    fn test_enumerate_sub_keys() {
        unsafe {
            let registry_key =
                "\\Registry\\Machine\\System\\CurrentControlSet\\Services\\Tcpip\\Parameters\\Interfaces";
            match enumerate_sub_keys(registry_key) {
                Ok(sub_keys) => {
                    for sub_key in sub_keys {
                        libc_println!("Sub-key: {}\\{}", registry_key, sub_key);
                    }
                }
                Err(status) => {
                    libc_println!("Failed to enumerate sub-keys. NT STATUS: {:#X}", status);
                }
            }
        }
    }
}
