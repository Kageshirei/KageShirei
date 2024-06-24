use core::slice;

use alloc::string::{String, ToString};
use rs2_win32::ntdef::{OSVersionInfo, RtlUserProcessParameters};

use mod_agentcore::instance;

/// Retrieves the name of the current process by accessing the PEB (Process Environment Block).
///
/// # Safety
/// This function performs unsafe operations, such as dereferencing raw pointers obtained
/// from the PEB. It is intended to be used in contexts where the caller ensures the safety
/// of accessing these pointers.
///
/// # Returns
/// A `String` containing the name of the current process. If the process name cannot be
/// determined, an empty string is returned.
pub unsafe fn get_process_name() -> String {
    // Get the pointer to the PEB
    let peb = instance().teb.as_ref().unwrap().process_environment_block;
    if !peb.is_null() {
        // Get the process parameters from the PEB
        let process_parameters = (*peb).process_parameters as *mut RtlUserProcessParameters;
        if !process_parameters.is_null() {
            // Get the image path name from the process parameters
            let image_path_name = &(*process_parameters).image_path_name;
            if !image_path_name.buffer.is_null() {
                // Convert the UTF-16 buffer to a Rust slice
                let image_path_slice = slice::from_raw_parts(
                    image_path_name.buffer,
                    image_path_name.length as usize / 2,
                );
                // Convert the UTF-16 slice to a Rust string
                let full_path = String::from_utf16_lossy(image_path_slice);
                // Extract the process name from the full path
                return full_path.split('\\').last().unwrap_or("").to_string();
            }
        }
    }
    // Return an empty string if the process name could not be determined
    String::new()
}

/// Retrieves the version information of the current operating system by accessing the PEB.
///
/// # Safety
/// This function performs unsafe operations, such as dereferencing raw pointers obtained
/// from the PEB. It is intended to be used in contexts where the caller ensures the safety
/// of accessing these pointers.
///
/// # Parameters
/// * `lp_version_information`: A mutable reference to an `OSVersionInfo` struct that will
///   be filled with the version information of the operating system.
///
/// # Returns
/// A status code indicating success or failure. If the provided structure size is invalid,
/// `STATUS_INVALID_PARAMETER` is returned. Otherwise, the function returns `STATUS_SUCCESS`.
pub unsafe fn nt_rtl_get_version(lp_version_information: &mut OSVersionInfo) -> i32 {
    // Get the pointer to the PEB
    let peb = instance().teb.as_ref().unwrap().process_environment_block;

    if lp_version_information.dw_os_version_info_size
        != core::mem::size_of::<OSVersionInfo>() as u32
    {
        return -1;
    }

    // Fill in the version information from the PEB
    lp_version_information.dw_major_version = (*peb).os_major_version;
    lp_version_information.dw_minor_version = (*peb).os_minor_version;
    lp_version_information.dw_build_number = (*peb).os_build_number;
    lp_version_information.dw_platform_id = (*peb).os_platform_id;
    lp_version_information.sz_csd_version.fill(0);

    // Copy the CSD version string if it exists
    // if (*peb).csd_version.length > 0
    //     && !(*peb).csd_version.buffer.is_null()
    //     && *(*peb).csd_version.buffer != 0
    // {
    //     let length = lstrlenW((*peb).csd_version.buffer);
    //     let length = core::cmp::min(
    //         length as usize,
    //         lp_version_information.sz_csd_version.len() - 1,
    //     );
    //     ptr::copy_nonoverlapping(
    //         (*peb).csd_version.buffer,
    //         lp_version_information.sz_csd_version.as_mut_ptr(),
    //         length,
    //     );
    // }

    0 // STATUS_SUCCESS
}

/// Retrieves the value of a specified environment variable by accessing the PEB (Process Environment Block).
///
/// # Safety
/// This function performs unsafe operations, such as dereferencing raw pointers obtained
/// from the PEB. It is intended to be used in contexts where the caller ensures the safety
/// of accessing these pointers.
///
/// # Parameters
/// * `variable_name`: The name of the environment variable to retrieve.
///
/// # Returns
/// A `String` containing the value of the specified environment variable. If the variable cannot be
/// determined, an empty string is returned.
unsafe fn get_environment_variable(variable_name: &str) -> String {
    // Get the pointer to the PEB
    let peb = instance().teb.as_ref().unwrap().process_environment_block;
    if !peb.is_null() {
        // Get the process parameters from the PEB
        let process_parameters = (*peb).process_parameters as *mut RtlUserProcessParameters;
        if !process_parameters.is_null() {
            // Get the environment block from the process parameters
            let environment = (*process_parameters).environment;
            if !environment.is_null() {
                // Convert the environment block to a Rust slice of UTF-16 strings
                let mut env_ptr = environment as *const u16;
                while *env_ptr != 0 {
                    let mut len = 0;
                    while *env_ptr.add(len) != 0 {
                        len += 1;
                    }
                    let env_slice = slice::from_raw_parts(env_ptr, len as usize);
                    let env_string = String::from_utf16_lossy(env_slice);
                    // Check if the environment string starts with the specified variable name
                    if let Some(value) = env_string.strip_prefix(variable_name) {
                        return value.to_string();
                    }
                    env_ptr = env_ptr.add(len + 1);
                }
            }
        }
    }
    // Return an empty string if the variable cannot be determined
    String::new()
}

/// Retrieves the username of the current process by accessing the PEB (Process Environment Block).
///
/// # Safety
/// This function performs unsafe operations, such as dereferencing raw pointers obtained
/// from the PEB. It is intended to be used in contexts where the caller ensures the safety
/// of accessing these pointers.
///
/// # Returns
/// A `String` containing the username of the current process. If the username cannot be
/// determined, an empty string is returned.
pub unsafe fn get_username() -> String {
    get_environment_variable("USERNAME=")
}

/// Retrieves the OS of the current process by accessing the PEB (Process Environment Block).
///
/// # Safety
/// This function performs unsafe operations, such as dereferencing raw pointers obtained
/// from the PEB. It is intended to be used in contexts where the caller ensures the safety
/// of accessing these pointers.
///
/// # Returns
/// A `String` containing the OS of the current process. If the OS cannot be
/// determined, an empty string is returned.
pub unsafe fn get_os() -> String {
    get_environment_variable("OS=")
}

/// Retrieves the computer name of the current process by accessing the PEB (Process Environment Block).
///
/// # Safety
/// This function performs unsafe operations, such as dereferencing raw pointers obtained
/// from the PEB. It is intended to be used in contexts where the caller ensures the safety
/// of accessing these pointers.
///
/// # Returns
/// A `String` containing the computer name of the current process. If the computer name cannot be
/// determined, an empty string is returned.
pub unsafe fn get_computer_name() -> String {
    get_environment_variable("COMPUTERNAME=")
}

/// Retrieves the user domain of the current process by accessing the PEB (Process Environment Block).
///
/// # Safety
/// This function performs unsafe operations, such as dereferencing raw pointers obtained
/// from the PEB. It is intended to be used in contexts where the caller ensures the safety
/// of accessing these pointers.
///
/// # Returns
/// A `String` containing the user domain of the current process. If the user domain cannot be
/// determined, an empty string is returned.
pub unsafe fn get_user_domain() -> String {
    get_environment_variable("USERDOMAIN=")
}

/// Combines OS name and version information into a single string.
///
/// # Safety
/// This function performs unsafe operations, such as dereferencing raw pointers obtained
/// from the PEB. It is intended to be used in contexts where the caller ensures the safety
/// of accessing these pointers.
///
/// # Returns
/// A `String` containing the combined OS name and version information.
pub unsafe fn get_os_version_info() -> OSVersionInfo {
    // Initialize version information
    let mut version_info = OSVersionInfo::new();

    // Get the version information
    let status = nt_rtl_get_version(&mut version_info);

    version_info
}

#[cfg(test)]
mod tests {

    use super::*;
    use libc_print::libc_println;

    #[test]
    fn test_get_process_name() {
        let process_name = unsafe { get_process_name() };
        libc_println!("Process Name: {:?}", process_name);
        assert!(!process_name.is_empty(), "Process name should not be empty");
    }

    #[test]
    fn test_nt_rtl_get_version() {
        let mut version_info = OSVersionInfo::new();

        let status = unsafe { nt_rtl_get_version(&mut version_info) };
        libc_println!("Major version: {:?}", version_info.dw_major_version);
        libc_println!("Minor version: {:?}", version_info.dw_minor_version);
        libc_println!("Build number: {:?}", version_info.dw_build_number);
        libc_println!("Platform id: {:?}", version_info.dw_platform_id);
        assert_eq!(status, 0, "Status should be STATUS_SUCCESS");
    }

    #[test]
    fn test_get_user_name() {
        let user_name = unsafe { get_username() };
        libc_println!("User Name: {:?}", user_name);
        assert!(!user_name.is_empty(), "User name should not be empty");
    }

    #[test]
    fn test_get_os() {
        let os = unsafe { get_os() };
        libc_println!("OS: {:?}", os);
        assert!(!os.is_empty(), "OS should not be empty");
    }

    #[test]
    fn test_get_computer_name() {
        let computer_name = unsafe { get_computer_name() };
        libc_println!("Computer Name: {:?}", computer_name);
        assert!(
            !computer_name.is_empty(),
            "Computer name should not be empty"
        );
    }

    #[test]
    fn test_get_user_domain() {
        let user_domain = unsafe { get_user_domain() };
        libc_println!("User Domain: {:?}", user_domain);
        assert!(!user_domain.is_empty(), "User domain should not be empty");
    }
}
