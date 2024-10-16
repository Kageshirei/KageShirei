use alloc::string::{String, ToString};
use core::slice;

use kageshirei_win32::ntdef::{OSVersionInfo, RtlUserProcessParameters};
use mod_agentcore::instance;

use crate::utils::unicodestring_to_string;

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
                let image_path_slice =
                    slice::from_raw_parts(image_path_name.buffer, image_path_name.length as usize / 2);
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
/// * `lp_version_information`: A mutable reference to an `OSVersionInfo` struct that will be filled
///   with the version information of the operating system.
///
/// # Returns
/// A status code indicating success or failure. If the provided structure size is invalid,
/// `STATUS_INVALID_PARAMETER` is returned. Otherwise, the function returns `STATUS_SUCCESS`.
pub unsafe fn nt_rtl_get_version(lp_version_information: &mut OSVersionInfo) -> i32 {
    // Get the pointer to the PEB
    let peb = instance().teb.as_ref().unwrap().process_environment_block;

    if lp_version_information.dw_os_version_info_size != core::mem::size_of::<OSVersionInfo>() as u32 {
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

/// Retrieves the value of a specified environment variable by accessing the PEB (Process
/// Environment Block).
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

pub unsafe fn print_all_environment_variables() {
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
                    // let env_slice = slice::from_raw_parts(env_ptr, len as usize);
                    // let env_string = String::from_utf16_lossy(env_slice);
                    // libc_println!("{}", env_string);
                    env_ptr = env_ptr.add(len + 1);
                }
            }
        }
    }
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
pub unsafe fn get_username() -> String { get_environment_variable("USERNAME=") }

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
pub unsafe fn get_os() -> String { get_environment_variable("OS=") }

/// Retrieves the computer name of the current process by accessing the PEB (Process Environment
/// Block).
///
/// # Safety
/// This function performs unsafe operations, such as dereferencing raw pointers obtained
/// from the PEB. It is intended to be used in contexts where the caller ensures the safety
/// of accessing these pointers.
///
/// # Returns
/// A `String` containing the computer name of the current process. If the computer name cannot be
/// determined, an empty string is returned.
pub unsafe fn get_computer_name() -> String { get_environment_variable("COMPUTERNAME=") }

/// Retrieves the user domain of the current process by accessing the PEB (Process Environment
/// Block).
///
/// # Safety
/// This function performs unsafe operations, such as dereferencing raw pointers obtained
/// from the PEB. It is intended to be used in contexts where the caller ensures the safety
/// of accessing these pointers.
///
/// # Returns
/// A `String` containing the user domain of the current process. If the user domain cannot be
/// determined, an empty string is returned.
pub unsafe fn get_user_domain() -> String { get_environment_variable("USERDOMAIN=") }

/// Combines OS name and version information into a single string.
///
/// # Safety
/// This function performs unsafe operations, such as dereferencing raw pointers obtained
/// from the PEB. It is intended to be used in contexts where the caller ensures the safety
/// of accessing these pointers.
///
/// # Returns
/// A `String` containing the combined OS name and version information.
pub unsafe fn get_os_version_info() -> Result<OSVersionInfo, i32> {
    // Initialize version information
    let mut version_info = OSVersionInfo::new();

    // Get the version information
    let status = nt_rtl_get_version(&mut version_info);

    if status < 0 {
        return Err(status);
    }
    Ok(version_info)
}

/// Get the full path of the current executable.
///
/// This function accesses the Thread Environment Block (TEB) to retrieve the Process Environment
/// Block (PEB), and from there it obtains the process parameters which include the image path name.
/// It converts the image path name from a UTF-16 encoded string to a Rust `String`.
///
/// # Safety
///
/// This function performs raw pointer dereferencing and should be used with caution.
///
/// # Returns
///
/// Returns an `Option<String>` containing the full path of the current executable if successful,
/// or `None` if any step of the process fails.
pub unsafe fn get_image_path_name() -> String {
    // Get the pointer to the TEB
    let peb = instance().teb.as_ref().unwrap().process_environment_block;

    if !peb.is_null() {
        // Get the process parameters from the PEB
        let process_parameters = (*peb).process_parameters as *mut RtlUserProcessParameters;
        if !process_parameters.is_null() {
            // Get the ImagePathName from the process parameters
            let image_path_name = &(*process_parameters).image_path_name;
            if !image_path_name.buffer.is_null() {
                // Convert the ImagePathName to a Rust String
                let length = (image_path_name.length / 2) as usize;
                let buffer = core::slice::from_raw_parts(image_path_name.buffer, length);
                return alloc::string::String::from_utf16_lossy(buffer);
            }
        }
    }

    String::new()
}

/// Retrieves the current directory as a string.
///
/// This helper function accesses the Process Environment Block (PEB) to obtain the
/// current directory path. It converts the DOS path stored in the PEB to a Rust string
/// and returns it. If the current directory cannot be retrieved, it returns an empty string.
///
/// # Returns
/// - `String`: The current directory path as a Rust string, or an empty string if the path cannot
///   be retrieved.
///
/// # Details
/// This function is useful for debugging and for any situation where the current directory
/// needs to be displayed or logged. It interacts with the PEB to get the current directory path.
pub fn get_current_directory() -> String {
    unsafe {
        // Retrieve the Process Environment Block (PEB)
        let peb = instance().teb.as_ref().unwrap().process_environment_block;
        if !peb.is_null() {
            let process_parameters = (*peb).process_parameters as *mut RtlUserProcessParameters;
            let cur_dir = &mut process_parameters.as_mut().unwrap().current_directory;

            // Convert the current directory's DOS path to a Rust string
            return unicodestring_to_string(&(*cur_dir).dos_path).unwrap_or_default();
        }
    }
    String::new()
}

#[cfg(test)]
mod tests {
    use libc_print::libc_println;

    use super::*;

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
