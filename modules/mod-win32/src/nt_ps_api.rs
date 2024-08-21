use core::{
    ffi::c_void,
    mem::{self, size_of},
    ptr::{null, null_mut},
};

use alloc::vec::Vec;
use libc_print::libc_println;
use mod_agentcore::instance;
use rs2_win32::{
    ntdef::{
        AccessMask, ClientId, ObjectAttributes, ProcessBasicInformation, PsAttributeList,
        PsCreateInfo, PsCreateInfoUnion, PsCreateInitialFlags, PsCreateInitialState, PsCreateState,
        RtlUserProcessParameters, TokenMandatoryLabel, UnicodeString, HANDLE, OBJ_CASE_INSENSITIVE,
        PROCESS_ALL_ACCESS,
        PROCESS_CREATION_MITIGATION_POLICY_BLOCK_NON_MICROSOFT_BINARIES_ALWAYS_ON,
        PS_ATTRIBUTE_IMAGE_NAME, PS_ATTRIBUTE_MITIGATION_OPTIONS, PS_ATTRIBUTE_PARENT_PROCESS,
        RTL_USER_PROC_PARAMS_NORMALIZED, THREAD_ALL_ACCESS, TOKEN_INTEGRITY_LEVEL, TOKEN_QUERY,
        ULONG,
    },
    ntstatus::{STATUS_BUFFER_OVERFLOW, STATUS_BUFFER_TOO_SMALL},
};

/// Retrieves the PID (Process ID) and PPID (Parent Process ID) of the current process using NT API.
///
/// # Safety
/// This function performs unsafe operations, such as making system calls to retrieve process information.
///
/// # Returns
/// A tuple `(u32, u32)` containing the PID and PPID of the current process. If the operation fails,
/// both values in the tuple will be `0`.
pub unsafe fn get_pid_and_ppid() -> (u32, u32) {
    let mut pbi: ProcessBasicInformation = core::mem::zeroed();
    let mut return_length: u32 = 0;
    let status = instance().ntdll.nt_query_information_process.run(
        -1isize as HANDLE, // Use the handle for the current process
        0,                 // ProcessBasicInformation
        &mut pbi as *mut _ as *mut _,
        size_of::<ProcessBasicInformation>() as u32,
        &mut return_length as *mut u32,
    );

    if status == 0 {
        return (
            pbi.unique_process_id as u32,
            pbi.inherited_from_unique_process_id as u32,
        );
    }

    (0, 0)
}

/// Retrieves the PID (Process ID) of the current process using NT API.
///
/// # Safety
/// This function performs unsafe operations, such as making system calls to retrieve process information.
///
/// # Returns
/// A u32 containing the PID of the current process. If the operation fails,
/// return `0`.
pub unsafe fn get_pid() -> u32 {
    let mut pbi: ProcessBasicInformation = core::mem::zeroed();
    let mut return_length: u32 = 0;
    let status = instance().ntdll.nt_query_information_process.run(
        -1isize as HANDLE, // Use the handle for the current process
        0,                 // ProcessBasicInformation
        &mut pbi as *mut _ as *mut _,
        size_of::<ProcessBasicInformation>() as u32,
        &mut return_length as *mut u32,
    );

    if status == 0 {
        return pbi.unique_process_id as u32;
    }

    0
}

/// Retrieves a handle to a process with the specified PID and desired access rights.
///
/// # Safety
///
/// This function involves unsafe operations and raw pointer dereferencing.
///
/// # Parameters
///
/// - `pid`: The process ID of the target process.
/// - `desired_access`: The desired access rights for the process handle.
///
/// # Returns
///
/// A handle to the process if successful, otherwise `null_mut()`.
pub unsafe fn get_proc_handle(pid: i32, desired_access: AccessMask) -> HANDLE {
    let mut process_handle: HANDLE = null_mut();

    // Initialize object attributes for the process
    let mut object_attributes = ObjectAttributes::new();

    ObjectAttributes::initialize(
        &mut object_attributes,
        null_mut(),
        OBJ_CASE_INSENSITIVE, // 0x40
        null_mut(),
        null_mut(),
    );

    // Initialize client ID for the target process
    let mut client_id = ClientId::new();
    client_id.unique_process = pid as _;

    // Open the process with the specified access rights
    instance().ntdll.nt_open_process.run(
        &mut process_handle,
        desired_access,
        &mut object_attributes,
        &mut client_id as *mut _ as *mut c_void,
    );

    process_handle
}

/// Opens the process token for a given process handle.
///
/// # Safety
///
/// This function involves unsafe operations and raw pointer dereferencing.
///
/// # Parameters
///
/// - `process_handle`: A handle to the process for which the token is to be opened.
///
/// # Returns
///
/// A handle to the process token if successful, otherwise `null_mut()`.
pub unsafe fn get_process_token(process_handle: HANDLE) -> HANDLE {
    let mut token_handle: HANDLE = null_mut();
    instance()
        .ntdll
        .nt_open_process_token
        .run(process_handle, TOKEN_QUERY, &mut token_handle);
    token_handle
}

/// Retrieves the integrity level of a specified process.
///
/// # Safety
///
/// This function involves unsafe operations and raw pointer dereferencing.
///
/// # Parameters
///
/// - `process_handle`: A handle to the process for which the integrity level is to be retrieved.
///
/// # Returns
///
/// The integrity level as an `i32` representing the RID if successful, otherwise `-1`.
pub unsafe fn nt_get_integrity_level(process_handle: HANDLE) -> i32 {
    // Get the process token
    let token_handle = get_process_token(process_handle);

    if token_handle.is_null() {
        return -1;
    }

    let mut label: TokenMandatoryLabel = core::mem::zeroed();
    let mut return_length: ULONG = 0;
    let mut size = 0;

    // Query the token for integrity level information
    let ntstatus = instance().ntdll.nt_query_information_token.run(
        token_handle,
        TOKEN_INTEGRITY_LEVEL as ULONG,
        &mut label as *mut _ as *mut c_void,
        size,
        &mut return_length as *mut ULONG,
    );

    if ntstatus != STATUS_BUFFER_OVERFLOW && ntstatus != STATUS_BUFFER_TOO_SMALL {
        instance().ntdll.nt_close.run(token_handle);
        return -1;
    }

    if return_length == 0 {
        instance().ntdll.nt_close.run(token_handle);
        return -1;
    }

    size = return_length;

    // Query the token again with the correct buffer size
    let ntstatus = instance().ntdll.nt_query_information_token.run(
        token_handle,
        TOKEN_INTEGRITY_LEVEL as ULONG,
        &mut label as *mut _ as *mut c_void,
        size,
        &mut return_length as *mut ULONG,
    );

    if ntstatus == 0 {
        // Extract the RID (Relative Identifier) from the SID to determine the integrity level
        let sid = &*label.label.sid;
        let sub_authority_count = sid.sub_authority_count as usize;
        let rid = *sid.sub_authority.get_unchecked(sub_authority_count - 1);

        instance().ntdll.nt_close.run(token_handle);
        return rid as i32;
    } else {
        instance().ntdll.nt_close.run(token_handle);
        return -1;
    }
}

/// Creates a user process and its primary thread using the specified process, command line, and
/// current directory. This function returns the handles to the created process and thread.
///
/// # Parameters
/// - `sz_target_process`: A string slice that specifies the NT path of the process image to be created.
/// - `sz_target_process_parameters`: A string slice that specifies the command line for the process.
/// - `sz_target_process_path`: A string slice that specifies the current directory for the process.
/// - `debug`: A boolean that, if set to `true`, enables debug output for the function.
///
/// # Returns
/// A tuple containing the handles to the created process and thread. If the process creation fails,
/// both handles will be `null_mut()`.
///
/// # Safety
/// This function uses unsafe operations to interact with low-level Windows APIs. Ensure that the
/// inputs are valid and that the function is called in a safe context.
pub unsafe fn nt_create_user_process(
    sz_target_process: &str,
    sz_target_process_parameters: &str,
    sz_target_process_path: &str,
    debug: bool,
    h_parent_process: HANDLE,
) -> (*mut c_void, *mut c_void) {
    // Initialize UNICODE_STRING structures for the process image, command line, and current directory.
    let mut us_nt_image_path = UnicodeString::new();
    let mut us_command_line = UnicodeString::new();
    let mut us_current_directory = UnicodeString::new();

    // Pointer to the RTL_USER_PROCESS_PARAMETERS structure, initially set to null.
    let mut upp_process_parameters: *mut RtlUserProcessParameters = null_mut();

    // Convert the input strings to UTF-16 and initialize the corresponding UNICODE_STRING structures.
    let sz_target_process_utf16: Vec<u16> =
        sz_target_process.encode_utf16().chain(Some(0)).collect();
    us_nt_image_path.init(sz_target_process_utf16.as_ptr());

    let sz_target_process_parameters_utf16: Vec<u16> = sz_target_process_parameters
        .encode_utf16()
        .chain(Some(0))
        .collect();
    us_command_line.init(sz_target_process_parameters_utf16.as_ptr());

    let sz_target_process_path_utf16: Vec<u16> = sz_target_process_path
        .encode_utf16()
        .chain(Some(0))
        .collect();
    us_current_directory.init(sz_target_process_path_utf16.as_ptr());

    // Debug output for the initialized UNICODE_STRING structures.
    if debug {
        libc_println!(
            "us_nt_image_path: Length: {}, MaximumLength: {}, Buffer: {:?}",
            us_nt_image_path.length,
            us_nt_image_path.maximum_length,
            us_nt_image_path.buffer
        );
        libc_println!(
            "us_command_line: Length: {}, MaximumLength: {}, Buffer: {:?}",
            us_command_line.length,
            us_command_line.maximum_length,
            us_command_line.buffer
        );
        libc_println!(
            "us_current_directory: Length: {}, MaximumLength: {}, Buffer: {:?}",
            us_current_directory.length,
            us_current_directory.maximum_length,
            us_current_directory.buffer
        );
    }

    // Call RtlCreateProcessParametersEx to create the RTL_USER_PROCESS_PARAMETERS structure.
    let nt_status = (instance().ntdll.rtl_create_process_parameters_ex)(
        &mut upp_process_parameters,
        &us_nt_image_path,
        null(),
        &us_current_directory,
        &us_command_line,
        null(),
        null(),
        null(),
        null(),
        null(),
        RTL_USER_PROC_PARAMS_NORMALIZED,
    );

    // Check if the process parameters creation failed.
    if nt_status != 0 {
        libc_println!(
            "[!] RtlCreateProcessParametersEx Failed With Error: {:X}",
            nt_status
        );
        return (null_mut(), null_mut());
    }

    // Debug output for the created RTL_USER_PROCESS_PARAMETERS structure.
    if debug && !upp_process_parameters.is_null() {
        let upp = &*upp_process_parameters;
        libc_println!(
            "RtlCreateProcessParametersEx Success:\n\
             MaximumLength: {}\n\
             Length: {}\n\
             Flags: {}\n\
             DebugFlags: {}\n\
             ConsoleHandle: {:?}\n\
             ConsoleFlags: {}\n\
             StandardInput: {:?}\n\
             StandardOutput: {:?}\n\
             StandardError: {:?}\n\
             CurrentDirectoryPath: Length: {}, MaximumLength: {}, Buffer: {:?}\n\
             DllPath: Length: {}, MaximumLength: {}, Buffer: {:?}\n\
             ImagePathName: Length: {}, MaximumLength: {}, Buffer: {:?}\n\
             CommandLine: Length: {}, MaximumLength: {}, Buffer: {:?}",
            upp.maximum_length,
            upp.length,
            upp.flags,
            upp.debug_flags,
            upp.console_handle,
            upp.console_flags,
            upp.standard_input,
            upp.standard_output,
            upp.standard_error,
            upp.current_directory_path.length,
            upp.current_directory_path.maximum_length,
            upp.current_directory_path.buffer,
            upp.dll_path.length,
            upp.dll_path.maximum_length,
            upp.dll_path.buffer,
            upp.image_path_name.length,
            upp.image_path_name.maximum_length,
            upp.image_path_name.buffer,
            upp.command_line.length,
            upp.command_line.maximum_length,
            upp.command_line.buffer,
        );
    } else if debug {
        libc_println!("upp_process_parameters is null.");
    }

    // Initialize the PS_ATTRIBUTE_LIST structure.
    let mut attribute_list: PsAttributeList = mem::zeroed();
    attribute_list.total_length = mem::size_of::<PsAttributeList>();
    attribute_list.attributes[0].attribute = PS_ATTRIBUTE_IMAGE_NAME;
    attribute_list.attributes[0].size = us_nt_image_path.length as usize;
    attribute_list.attributes[0].value.value = us_nt_image_path.buffer as usize;

    if debug {
        libc_println!(
            "PS_ATTRIBUTE_LIST:\n\
             TotalLength: {}\n\
             Attribute: {}\n\
             Size: {}\n\
             Value: {:?}\n",
            attribute_list.total_length,
            attribute_list.attributes[0].attribute,
            attribute_list.attributes[0].size,
            attribute_list.attributes[0].value.value,
        );
    }

    // if !h_parent_process.is_null() {
    //     // intializing an attribute list of type 'PS_ATTRIBUTE_PARENT_PROCESS' that specifies the process's parent
    //     attribute_list.attributes[1].attribute = PS_ATTRIBUTE_PARENT_PROCESS;
    //     attribute_list.attributes[1].size = mem::size_of::<HANDLE>();
    //     attribute_list.attributes[1].value.value_ptr = h_parent_process;
    // }

    // if block_dll_policy {
    // intializing an attribute list of type 'PS_ATTRIBUTE_MITIGATION_OPTIONS' that specifies the use of process's mitigation policies
    //     attribute_list.attributes[1].attribute = PS_ATTRIBUTE_MITIGATION_OPTIONS;
    //     attribute_list.attributes[1].size = mem::size_of::<u64>();
    //     attribute_list.attributes[1].value.value =
    //         PROCESS_CREATION_MITIGATION_POLICY_BLOCK_NON_MICROSOFT_BINARIES_ALWAYS_ON as usize;
    // }

    // Initialize the PS_CREATE_INFO structure.
    let mut ps_create_info = PsCreateInfo {
        size: mem::size_of::<PsCreateInfo>(),
        state: PsCreateState::PsCreateInitialState,
        union_state: PsCreateInfoUnion {
            init_state: PsCreateInitialState {
                init_flags: PsCreateInitialFlags::default(),
                additional_file_access: 0,
            },
        },
    };

    if debug {
        libc_println!(
            "PS_CREATE_INFO:\n\
             Size: {}\n\
             State: {}",
            ps_create_info.size,
            ps_create_info.state as u32,
        );
    }

    // Initialize process and thread handles.
    let mut h_process: HANDLE = null_mut();
    let mut h_thread: HANDLE = null_mut();

    // Call NtCreateUserProcess to create the process and thread.
    let nt_status = instance().ntdll.nt_create_user_process.run(
        &mut h_process,
        &mut h_thread,
        PROCESS_ALL_ACCESS,
        THREAD_ALL_ACCESS,
        null_mut(),
        null_mut(),
        0,
        0,
        upp_process_parameters as *mut c_void,
        &mut ps_create_info,
        &mut attribute_list,
    );

    // Check if the process creation failed.
    if nt_status != 0 {
        libc_println!("[!] NtCreateUserProcess Failed With Error: {:X}", nt_status);
        return (null_mut(), null_mut());
    }

    // Return the handles to the created process and thread.
    return (h_process, h_thread);
}

#[cfg(test)]
mod tests {
    use super::*;
    use libc_print::libc_println;

    #[test]
    fn test_get_pid_and_ppid() {
        let (pid, ppid) = unsafe { get_pid_and_ppid() };
        libc_println!("PID: {:?}", pid);
        libc_println!("PPID: {:?}", ppid);
        assert!(pid != 0, "PID should not be zero");
        assert!(ppid != 0, "PPID should not be zero");
    }

    #[test]
    fn test_get_integrity_level() {
        unsafe {
            let process_handle = -1isize as _;
            // Alternative: get process handle for a specific process
            // let process_handle =
            //     get_proc_handle(17344, PROCESS_QUERY_INFORMATION | PROCESS_VM_READ);
            let rid = nt_get_integrity_level(process_handle);
            if rid != -1 {
                libc_println!("Process Integrity Level: {:?}", rid);
            } else {
                libc_println!("Failed to get process integrity level");
            }
        }
    }

    #[test]
    fn test_create_user_process() {
        unsafe {
            let sz_target_process = "\\??\\C:\\Windows\\System32\\cmd.exe";
            let sz_target_process_parameters = "C:\\Windows\\System32\\cmd.exe /c calc.exe";
            let sz_target_process_path = "C:\\Windows\\System32";

            let (h_process, h_thread) = nt_create_user_process(
                &sz_target_process,
                &sz_target_process_parameters,
                &sz_target_process_path,
                true,
                null_mut(),
            );

            if !h_process.is_null() {
                libc_println!(
                    "NtCreateUserProcess Success:\n\
                    hProcess: {:?}\n\
                    hThread: {:?}",
                    h_process,
                    h_thread
                );
            } else {
                libc_println!("Failed to create process.");
            }
        }
    }

    #[test]
    fn test_create_user_process_ppid_spoof() {
        unsafe {
            let sz_target_process = "\\??\\C:\\Windows\\System32\\RuntimeBroker.exe";
            let sz_target_process_parameters =
                "C:\\Windows\\System32\\RuntimeBroker.exe -Embedding";
            let sz_target_process_path = "C:\\Windows\\System32";

            let ppid_spoof: HANDLE = get_proc_handle(21544, PROCESS_ALL_ACCESS);

            if ppid_spoof.is_null() {
                libc_println!("[!] GetProcHandle Failed");
                return;
            } else {
                libc_println!("[!] Parent Process Handle: {:p}", ppid_spoof);
            }

            let (h_process, h_thread) = nt_create_user_process(
                &sz_target_process,
                &sz_target_process_parameters,
                &sz_target_process_path,
                true,
                ppid_spoof,
            );

            if !h_process.is_null() {
                libc_println!(
                    "NtCreateUserProcess Success:\n\
                    hProcess: {:?}\n\
                    hThread: {:?}",
                    h_process,
                    h_thread
                );
            } else {
                libc_println!("Failed to create process.");
            }
        }
    }
}
