use core::{
    ffi::c_void,
    mem::{self, size_of},
    ptr::{null, null_mut},
};

use alloc::{format, string::String, vec::Vec};
use libc_print::libc_println;
use mod_agentcore::{instance, ldr::nt_current_teb};
use rs2_win32::{
    ntdef::{
        AccessMask, ClientId, IoStatusBlock, LargeInteger, ObjectAttributes,
        ProcessBasicInformation, PsAttributeList, PsCreateInfo, PsCreateInfoUnion,
        PsCreateInitialFlags, PsCreateInitialState, PsCreateState, RtlUserProcessParameters,
        SecurityAttributes, TokenMandatoryLabel, UnicodeString, FILE_CREATE, FILE_GENERIC_WRITE,
        FILE_NON_DIRECTORY_FILE, FILE_PIPE_BYTE_STREAM_MODE, FILE_PIPE_BYTE_STREAM_TYPE,
        FILE_PIPE_QUEUE_OPERATION, FILE_SHARE_READ, FILE_SHARE_WRITE, FILE_SYNCHRONOUS_IO_NONALERT,
        FILE_WRITE_ATTRIBUTES, GENERIC_READ, HANDLE, NTSTATUS, OBJ_CASE_INSENSITIVE, OBJ_INHERIT,
        PROCESS_ALL_ACCESS, PROCESS_CREATE_FLAGS_INHERIT_HANDLES, PS_ATTRIBUTE_IMAGE_NAME,
        PS_ATTRIBUTE_PARENT_PROCESS, RTL_USER_PROC_PARAMS_NORMALIZED, SYNCHRONIZE,
        THREAD_ALL_ACCESS, TOKEN_INTEGRITY_LEVEL, TOKEN_QUERY, ULONG,
    },
    ntstatus::{NT_SUCCESS, STATUS_BUFFER_OVERFLOW, STATUS_BUFFER_TOO_SMALL},
};

use crate::nt_time::delay;

/// Retrieves the PID (Process ID) and PPID (Parent Process ID) of the current process using the NT API.
///
/// This function queries the `ProcessBasicInformation` of the current process to obtain its
/// unique process ID and the process ID of its parent. The syscall `NtQueryInformationProcess`
/// is used to retrieve this information.
///
/// # Safety
/// This function performs unsafe operations, such as making direct system calls to retrieve process information.
/// Care should be taken when handling the raw pointers and interpreting the returned data.
///
/// # Returns
/// A tuple `(u32, u32)` containing the PID and PPID of the current process.
/// If the operation fails, both values in the tuple will be `0`.
pub unsafe fn get_pid_and_ppid() -> (u32, u32) {
    let mut pbi: ProcessBasicInformation = core::mem::zeroed();
    let mut return_length: u32 = 0;

    // Perform a system call to NtQueryInformationProcess to retrieve the basic process information.
    let status = instance().ntdll.nt_query_information_process.run(
        -1isize as HANDLE, // Use the special handle `-1` to refer to the current process.
        0,                 // Query the `ProcessBasicInformation`.
        &mut pbi as *mut _ as *mut _, // Provide a pointer to the ProcessBasicInformation structure.
        size_of::<ProcessBasicInformation>() as u32, // Specify the size of the structure.
        &mut return_length as *mut u32, // Receive the actual size of the returned data.
    );

    if status == 0 {
        return (
            pbi.unique_process_id as u32, // Return the PID of the current process.
            pbi.inherited_from_unique_process_id as u32, // Return the PPID of the parent process.
        );
    }

    (0, 0) // Return (0, 0) if the operation fails.
}

/// Retrieves the PID (Process ID) of the current process using the NT API.
///
/// This function queries the `ProcessBasicInformation` of the current process to obtain its
/// unique process ID. The syscall `NtQueryInformationProcess` is used to retrieve this information.
///
/// # Safety
/// This function performs unsafe operations, such as making direct system calls to retrieve process information.
/// Care should be taken when handling the raw pointers and interpreting the returned data.
///
/// # Returns
/// A `u32` containing the PID of the current process. If the operation fails, it returns `0`.
pub unsafe fn get_pid() -> u32 {
    let mut pbi: ProcessBasicInformation = core::mem::zeroed();
    let mut return_length: u32 = 0;

    // Perform a system call to NtQueryInformationProcess to retrieve the basic process information.
    let status = instance().ntdll.nt_query_information_process.run(
        -1isize as HANDLE, // Use the special handle `-1` to refer to the current process.
        0,                 // Query the `ProcessBasicInformation`.
        &mut pbi as *mut _ as *mut _, // Provide a pointer to the ProcessBasicInformation structure.
        size_of::<ProcessBasicInformation>() as u32, // Specify the size of the structure.
        &mut return_length as *mut u32, // Receive the actual size of the returned data.
    );

    if status == 0 {
        return pbi.unique_process_id as u32; // Return the PID of the current process.
    }

    0 // Return 0 if the operation fails.
}

/// Retrieves a handle to a process with the specified PID and desired access rights using the NT API.
///
/// This function opens a handle to a target process by specifying its process ID (PID) and the desired access rights.
/// The syscall `NtOpenProcess` is used to obtain the handle, and the function initializes the required structures
/// (`OBJECT_ATTRIBUTES` and `CLIENT_ID`) needed to make the system call.
///
/// # Safety
/// This function involves unsafe operations, including raw pointer dereferencing and direct system calls.
/// Ensure that the parameters passed to the function are valid and the function is called in a safe context.
///
/// # Parameters
/// - `pid`: The process ID of the target process.
/// - `desired_access`: The desired access rights for the process handle, specified as an `AccessMask`.
///
/// # Returns
/// A handle to the process if successful, otherwise `null_mut()` if the operation fails.
pub unsafe fn get_proc_handle(pid: i32, desired_access: AccessMask) -> HANDLE {
    let mut process_handle: HANDLE = null_mut();

    // Initialize object attributes for the process, setting up the basic structure with default options.
    let mut object_attributes = ObjectAttributes::new();

    ObjectAttributes::initialize(
        &mut object_attributes,
        null_mut(),           // No name for the object.
        OBJ_CASE_INSENSITIVE, // Case-insensitive name comparison.
        null_mut(),           // No root directory.
        null_mut(),           // No security descriptor.
    );

    // Initialize client ID structure with the target process ID.
    let mut client_id = ClientId::new();
    client_id.unique_process = pid as _;

    // Perform a system call to NtOpenProcess to obtain a handle to the specified process.
    instance().ntdll.nt_open_process.run(
        &mut process_handle, // Pointer to the handle that will receive the process handle.
        desired_access,      // Specify the access rights desired for the process handle.
        &mut object_attributes, // Provide the object attributes for the process.
        &mut client_id as *mut _ as *mut c_void, // Pass the client ID (target process ID).
    );

    process_handle // Return the obtained process handle, or `null_mut()` if the operation fails.
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

/// Creates a user process and its primary thread using the specified process image, command line,
/// and current directory. This function returns the handles to the created process and thread.
///
/// # Parameters
/// - `sz_target_process`: A string slice that specifies the NT path of the process image to be created.
/// - `sz_target_process_parameters`: A string slice that specifies the command line for the process.
/// - `sz_target_process_path`: A string slice that specifies the current directory for the process.
/// - `h_parent_process`: A handle to the parent process. If `null_mut()`, no parent process is specified.
/// - `h_process`: A mutable reference to a `HANDLE` that will receive the process handle upon creation.
/// - `h_thread`: A mutable reference to a `HANDLE` that will receive the thread handle upon creation.
///
/// # Returns
/// - `NTSTATUS`: The status code indicating the result of the process creation. A value of 0 indicates success,
///   while any other value represents an error.
///
/// # Safety
/// This function uses unsafe operations to interact with low-level Windows APIs. Ensure that the inputs
/// are valid and that the function is called in a safe context.
pub unsafe fn nt_create_user_process(
    sz_target_process: &str,
    sz_target_process_parameters: &str,
    sz_target_process_path: &str,
    h_parent_process: HANDLE,
    h_process: &mut HANDLE,
    h_thread: &mut HANDLE,
) -> NTSTATUS {
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
        RTL_USER_PROC_PARAMS_NORMALIZED, // Normalized process parameters
    );

    // Check if the process parameters creation failed.
    if nt_status != 0 {
        return nt_status; // Return error status if creation failed
    }

    // Initialize the PS_ATTRIBUTE_LIST structure, which holds attributes for the new process
    let mut attribute_list: PsAttributeList = mem::zeroed();
    attribute_list.total_length = mem::size_of::<PsAttributeList>();
    attribute_list.attributes[0].attribute = PS_ATTRIBUTE_IMAGE_NAME; // Attribute for image name
    attribute_list.attributes[0].size = us_nt_image_path.length as usize;
    attribute_list.attributes[0].value.value = us_nt_image_path.buffer as usize;

    // If a parent process handle is provided, add it to the attribute list
    if !h_parent_process.is_null() {
        attribute_list.attributes[1].attribute = PS_ATTRIBUTE_PARENT_PROCESS; // Attribute for parent process
        attribute_list.attributes[1].size = mem::size_of::<HANDLE>();
        attribute_list.attributes[1].value.value = h_parent_process as usize;
    }

    // Optional: Add additional attributes such as DLL mitigation policies, if necessary
    // Example:
    // attribute_list.attributes[2].attribute = PS_ATTRIBUTE_MITIGATION_OPTIONS;
    // attribute_list.attributes[2].size = mem::size_of::<u64>();
    // attribute_list.attributes[2].value.value =
    //     PROCESS_CREATION_MITIGATION_POLICY_BLOCK_NON_MICROSOFT_BINARIES_ALWAYS_ON as usize;

    // Initialize the PS_CREATE_INFO structure, which contains process creation information
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

    // Initialize process and thread handles to null
    *h_process = null_mut();
    *h_thread = null_mut();

    // Call NtCreateUserProcess to create the process and thread.
    let nt_status = instance().ntdll.nt_create_user_process.run(
        h_process,
        h_thread,
        PROCESS_ALL_ACCESS,                    // Full access to the process
        THREAD_ALL_ACCESS,                     // Full access to the thread
        null_mut(),                            // No process security descriptor
        null_mut(),                            // No thread security descriptor
        PROCESS_CREATE_FLAGS_INHERIT_HANDLES,  // Inherit handles flag
        0,                                     // No additional flags
        upp_process_parameters as *mut c_void, // Process parameters
        &mut ps_create_info,                   // Process creation information
        &mut attribute_list,                   // Attribute list
    );

    // Check if the process creation failed.
    if nt_status != 0 {
        return nt_status; // Return error status if creation failed
    }

    return 0; // Return success status
}

/// Formats a named pipe string and stores it in a `Vec<u16>`.
///
/// This function generates a named pipe path in the format:
/// `\\Device\\NamedPipe\\Win32Pipes.<process_id>.<pipe_id>`
/// and stores the UTF-16 encoded string in a `Vec<u16>`.
///
/// # Parameters
/// - `process_id`: The process ID to be included in the pipe name.
/// - `pipe_id`: The pipe ID to be included in the pipe name.
///
/// # Returns
/// A `Vec<u16>` containing the UTF-16 encoded string, null-terminated.
fn format_named_pipe_string(process_id: usize, pipe_id: u32) -> Vec<u16> {
    // Use `format!` to create the pipe name as a String
    let pipe_name = format!(
        "\\Device\\NamedPipe\\Win32Pipes.{:016x}.{:08x}",
        process_id, pipe_id
    );

    // Convert the formatted string into a UTF-16 encoded vector
    let mut pipe_name_utf16: Vec<u16> = pipe_name.encode_utf16().collect();

    // Null-terminate the buffer by pushing a 0 at the end
    pipe_name_utf16.push(0);

    // Return the UTF-16 encoded vector with a null terminator
    pipe_name_utf16
}

/// Creates a named pipe and returns handles for reading and writing.
///
/// This function sets up a named pipe with specified security attributes, buffer size,
/// and other options. It creates the pipe with both read and write handles, making it
/// ready for inter-process communication.
///
/// # Parameters
///
/// - `h_read_pipe`: A mutable reference to a handle that will receive the read handle of the created pipe.
/// - `h_write_pipe`: A mutable reference to a handle that will receive the write handle of the created pipe.
/// - `lp_pipe_attributes`: A pointer to a `SecurityAttributes` structure that specifies the security attributes for the pipe.
/// - `n_size`: The buffer size for the pipe. If set to 0, a default size of 4096 bytes is used.
///
/// # Returns
///
/// Returns `true` if the pipe was successfully created; otherwise, returns `false`.
///
/// # Safety
///
/// This function performs operations that involve raw pointers and direct system calls,
/// which require careful handling. The function should be called within a safe context.
pub unsafe fn create_pipe(
    h_read_pipe: &mut HANDLE,
    h_write_pipe: &mut HANDLE,
    lp_pipe_attributes: *mut SecurityAttributes,
    n_size: u32,
) -> bool {
    // Initialize the necessary structures
    let mut pipe_name: UnicodeString = UnicodeString::new();
    let mut object_attributes: ObjectAttributes = ObjectAttributes::new();
    let mut status_block: IoStatusBlock = IoStatusBlock::new();
    let mut default_timeout: LargeInteger = LargeInteger::new();
    let mut read_pipe_handle: HANDLE = null_mut();
    let mut write_pipe_handle: HANDLE = null_mut();
    let mut pipe_id: u32 = 0;
    let mut security_descriptor: *mut c_void = null_mut();

    // Set the default timeout to 120 seconds
    default_timeout.high_part = -1200000000;

    // Use the default buffer size if not provided
    let n_size = if n_size == 0 { 0x1000 } else { n_size };

    // Increment the pipe ID (normally done with InterlockedIncrement)
    pipe_id += 1;

    // Format the pipe name using the process ID and pipe ID
    let pipe_name_utf16 = format_named_pipe_string(
        nt_current_teb().as_ref().unwrap().client_id.unique_process as usize,
        pipe_id,
    );

    // Print the formatted pipe name for debugging purposes
    let pipe_name_str = String::from_utf16_lossy(&pipe_name_utf16);
    libc_println!("Formatted pipe name: {}", pipe_name_str);

    // Initialize the `UnicodeString` with the formatted pipe name
    pipe_name.init(pipe_name_utf16.as_ptr());

    // Use case-insensitive object attributes by default
    let mut attributes: ULONG = OBJ_CASE_INSENSITIVE;

    // Check if custom security attributes were provided
    if !lp_pipe_attributes.is_null() {
        // Use the provided security descriptor
        security_descriptor = (*lp_pipe_attributes).lp_security_descriptor;

        // Set the OBJ_INHERIT flag if handle inheritance is requested
        if (*lp_pipe_attributes).b_inherit_handle {
            attributes |= OBJ_INHERIT;
        }
    }

    // Initialize the object attributes for the named pipe
    ObjectAttributes::initialize(
        &mut object_attributes,
        &mut pipe_name,
        attributes, // Case-insensitive and possibly inheritable
        null_mut(),
        security_descriptor,
    );

    // Create the named pipe for reading
    let status = instance().ntdll.nt_create_named_pipe_file.run(
        &mut read_pipe_handle,
        GENERIC_READ | FILE_WRITE_ATTRIBUTES | SYNCHRONIZE, // Desired access: read, write attributes, sync
        &mut object_attributes,
        &mut status_block,
        FILE_SHARE_READ | FILE_SHARE_WRITE, // Share mode: allows read/write by other processes
        FILE_CREATE,                        // Creation disposition: create new, fail if exists
        FILE_SYNCHRONOUS_IO_NONALERT,       // Create options: synchronous I/O, no alerts
        FILE_PIPE_BYTE_STREAM_TYPE,         // Pipe type: byte stream (no message boundaries)
        FILE_PIPE_BYTE_STREAM_MODE,         // Read mode: byte stream mode for reading
        FILE_PIPE_QUEUE_OPERATION,          // Completion mode: operations are queued
        1,                                  // Max instances: only one instance of the pipe
        n_size,                             // Inbound quota: input buffer size
        n_size,                             // Outbound quota: output buffer size
        &default_timeout,                   // Default timeout for pipe operations
    );

    // Check if the pipe creation failed
    if !NT_SUCCESS(status) {
        libc_println!("[!] NtCreateNamedPipeFile Failed With Error: {:X}", status);
        instance().ntdll.nt_close.run(read_pipe_handle);
        return false;
    }

    let mut status_block_2 = IoStatusBlock::new();

    // Open the pipe for writing
    let status = instance().ntdll.nt_open_file.run(
        &mut write_pipe_handle,
        FILE_GENERIC_WRITE,
        &mut object_attributes,
        &mut status_block_2,
        FILE_SHARE_READ,
        FILE_SYNCHRONOUS_IO_NONALERT | FILE_NON_DIRECTORY_FILE,
    );

    // Check if the pipe opening failed
    if !NT_SUCCESS(status) {
        libc_println!("[!] NtOpenFile Failed With Error: {:X}", status);
        instance().ntdll.nt_close.run(read_pipe_handle);
        return false;
    }

    // Assign the read and write handles to the output parameters
    *h_read_pipe = read_pipe_handle;
    *h_write_pipe = write_pipe_handle;
    true
}

pub unsafe fn nt_create_cmd(
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

    let mut h_read_pipe: HANDLE = null_mut();
    let mut h_write_pipe: HANDLE = null_mut();

    // Check if the inherit handle is true
    let mut lp_pipe_attributes = SecurityAttributes {
        n_length: mem::size_of::<SecurityAttributes>() as u32,
        lp_security_descriptor: null_mut(),
        b_inherit_handle: true,
    };

    let result = (instance().kernel32.create_pipe)(
        &mut h_read_pipe,
        &mut h_write_pipe,
        &mut lp_pipe_attributes,
        0,
    );

    if result {
        libc_println!(
            "NtCreateNamedPipeFile Success:\n\
                h_read_pipe: {:?}\n\
                h_write_pipe: {:?}",
            h_read_pipe,
            h_write_pipe
        );
    }

    (*upp_process_parameters).standard_input = h_read_pipe;
    (*upp_process_parameters).standard_output = h_write_pipe;
    (*upp_process_parameters).standard_error = null_mut();

    // Check if the process parameters creation failed.
    if nt_status != 0 {
        libc_println!(
            "[!] RtlCreateProcessParametersEx Failed With Error: {:X}",
            nt_status
        );
        return (null_mut(), null_mut());
    }

    // Get the current console window handle
    // let console_handle = (instance().kernel32.get_console_window)();
    // let current_peb = instance().teb.as_ref().unwrap().process_environment_block;
    // let process_parameters = (*current_peb).process_parameters as *mut RtlUserProcessParameters;

    // Set the console handle and flags in the process parameters
    // (*upp_process_parameters).console_handle = (*process_parameters).console_handle;
    // (*upp_process_parameters).console_flags = 1; // Flags indicating an active console

    libc_println!(
        "GetConsoleWindow handle Success:\n\
            console_handle: {:?}",
        (*upp_process_parameters).console_handle
    );

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
        PROCESS_CREATE_FLAGS_INHERIT_HANDLES,
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

    // Write command to process's stdin
    let command = "whoami\r\n";
    let mut written_bytes: u32 = 0;

    let status = (instance().kernel32.write_file)(
        h_write_pipe,
        command.as_ptr() as *const c_void,
        command.len() as u32,
        &mut written_bytes,
        null_mut(),
    );

    if status == 0 {
        libc_println!("[!] WriteFile failed with errore: {:X}", status);
    }

    delay(10);

    // Read output from process's stdout
    let mut buffer = [0u8; 1024];
    let mut bytes_read: u32 = 0;
    let mut output = Vec::new();

    // Loop to read until there's no more data
    loop {
        let status = (instance().kernel32.read_file)(
            h_read_pipe,
            buffer.as_mut_ptr() as *mut c_void,
            buffer.len() as u32,
            &mut bytes_read,
            null_mut(),
        );

        if status == 0 || bytes_read == 0 {
            break;
        } else {
            output.extend_from_slice(&buffer[..bytes_read as usize]);
            if bytes_read < 1024 {
                break;
            }
        }
    }

    // Convert and print the output if available
    if !output.is_empty() {
        let output_str = String::from_utf8_lossy(&output);
        libc_println!("Output: {}", output_str);
    }

    // let status = (instance().kernel32.read_file)(
    //     h_read_pipe,
    //     buffer.as_mut_ptr() as *mut c_void,
    //     buffer.len() as u32,
    //     &mut bytes_read,
    //     null_mut(),
    // );

    // if status == 0 {
    //     libc_println!("[!] ReadFile failed with error.");
    // } else {
    //     // Output result
    //     let output = String::from_utf8_lossy(&buffer[..bytes_read as usize]);
    //     libc_println!("Output: {}", output);
    // }

    // Clean up handles
    instance().ntdll.nt_close.run(h_write_pipe);
    instance().ntdll.nt_close.run(h_read_pipe);

    // Return the handles to the created process and thread.
    return (h_process, h_thread);
}

#[cfg(test)]
mod tests {

    use super::*;
    use libc_print::libc_println;
    use rs2_win32::ntdef::{NtStatusError, ProcessInformation, StartupInfoW};

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
            let sz_target_process_parameters = "C:\\Windows\\System32\\cmd.exe";
            let sz_target_process_path = "C:\\Windows\\System32";

            let mut h_process: HANDLE = null_mut();
            let mut h_thread: HANDLE = null_mut();

            let nt_status = nt_create_user_process(
                &sz_target_process,
                &sz_target_process_parameters,
                &sz_target_process_path,
                null_mut(),
                &mut h_process,
                &mut h_thread,
            );

            if !h_process.is_null() {
                libc_println!(
                    "NtCreateUserProcess Success:\n\
                    hProcess: {:?}\n\
                    hThread: {:?}",
                    h_process,
                    h_thread
                );

                instance().ntdll.nt_terminate_process.run(h_process, 0);
            } else {
                libc_println!(
                    "Failed to create process: {:?}",
                    NtStatusError::from_code(nt_status)
                );
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

            let h_parent_process_handle: HANDLE = get_proc_handle(14172, PROCESS_ALL_ACCESS);

            if h_parent_process_handle.is_null() {
                libc_println!("[!] GetProcHandle Failed");
                return;
            } else {
                libc_println!("[!] Parent Process Handle: {:p}", h_parent_process_handle);
            }

            let mut h_process: HANDLE = null_mut();
            let mut h_thread: HANDLE = null_mut();

            let nt_status = nt_create_user_process(
                &sz_target_process,
                &sz_target_process_parameters,
                &sz_target_process_path,
                h_parent_process_handle,
                &mut h_process,
                &mut h_thread,
            );

            if !h_process.is_null() {
                libc_println!(
                    "NtCreateUserProcess Success:\n\
                    hProcess: {:?}\n\
                    hThread: {:?}",
                    h_process,
                    h_thread
                );
                // instance().ntdll.nt_close.run(h_parent_process_handle);

                // instance().ntdll.nt_terminate_process.run(h_process, 0);
            } else {
                libc_println!(
                    "Failed to create process: {:?}",
                    NtStatusError::from_code(nt_status)
                );

                instance().ntdll.nt_close.run(h_parent_process_handle);
            }
        }
    }

    // Helper function to create a wide string (UTF-16 encoded)
    pub fn wide_string(s: &str) -> Vec<u16> {
        let mut vec: Vec<u16> = s.encode_utf16().collect();
        vec.push(0); // Null-terminate the string
        vec
    }

    #[test]
    fn test_nt_create_named_pipe_file() {
        const STARTF_USESTDHANDLES: u32 = 0x00000100;
        const STARTF_USESHOWWINDOW: u32 = 0x00000001;
        const CREATE_NO_WINDOW: u32 = 0x08000000;

        let mut h_read_pipe: HANDLE = null_mut();
        let mut h_write_pipe: HANDLE = null_mut();

        let mut lp_pipe_attributes = SecurityAttributes {
            n_length: mem::size_of::<SecurityAttributes>() as u32,
            lp_security_descriptor: null_mut(),
            b_inherit_handle: true, // Questo deve essere `true` per ereditare le handle delle pipe
        };

        unsafe {
            // let result = (instance().kernel32.create_pipe)(
            //     &mut h_read_pipe,
            //     &mut h_write_pipe,
            //     &mut lp_pipe_attributes,
            //     0,
            // );

            let result = create_pipe(
                &mut h_read_pipe,
                &mut h_write_pipe,
                &mut lp_pipe_attributes,
                0,
            );

            if result {
                libc_println!(
                    "NtCreateNamedPipeFile Success:\n\
                    h_read_pipe: {:?}\n\
                    h_write_pipe: {:?}",
                    h_read_pipe,
                    h_write_pipe
                );

                let mut startup_info: StartupInfoW = mem::zeroed();

                startup_info.cb = mem::size_of::<StartupInfoW>() as u32;
                startup_info.dw_flags = STARTF_USESTDHANDLES | STARTF_USESHOWWINDOW;
                startup_info.h_std_error = null_mut();
                startup_info.h_std_input = h_read_pipe;
                startup_info.h_std_output = h_write_pipe;
                startup_info.w_show_window = 0 as u16;

                let mut process_info: ProcessInformation = mem::zeroed();

                let app_name = wide_string("C:\\Windows\\System32\\cmd.exe");
                let mut cmdline = wide_string("cmd.exe");

                let success = (instance().kernel32.create_process_w)(
                    app_name.as_ptr(),
                    cmdline.as_mut_ptr(),
                    null_mut(),
                    null_mut(),
                    true,
                    CREATE_NO_WINDOW,
                    null_mut(),
                    null_mut(),
                    &mut startup_info,
                    &mut process_info,
                );

                if success {
                    libc_println!(
                        "Process created successfully: hProcess: {:?}, hThread: {:?}",
                        process_info.h_process,
                        process_info.h_thread
                    );

                    libc_println!(
                        "StartUpInfo: hstdOutput: {:?}, hstdInput: {:?}",
                        startup_info.h_std_output,
                        startup_info.h_std_input
                    );

                    // Write command to process's stdin

                    // Scarta l'output iniziale

                    // Leggi e scarta l'output iniziale di cmd.exe
                    // let mut initial_buffer = [0u8; 1024];
                    // let mut initial_bytes_read: u32 = 0;
                    // let mut status: i32;
                    // loop {
                    //     status = (instance().kernel32.read_file)(
                    //         h_read_pipe,
                    //         initial_buffer.as_mut_ptr() as *mut c_void,
                    //         initial_buffer.len() as u32,
                    //         &mut initial_bytes_read,
                    //         null_mut(),
                    //     );

                    //     // Interrompi il ciclo se non ci sono più dati da leggere
                    //     if initial_bytes_read == 0 || status == 0 {
                    //         break;
                    //     }

                    //     // (Opzionale) Puoi stampare o loggare l'output scartato per il debug
                    //     let initial_output =
                    //         String::from_utf8_lossy(&initial_buffer[..initial_bytes_read as usize]);
                    //     libc_println!("Scartato output iniziale: {}", initial_output);
                    // }

                    // let mut pbi: ProcessBasicInformation = core::mem::zeroed();
                    // let mut return_length: u32 = 0;
                    // let status = instance().ntdll.nt_query_information_process.run(
                    //     process_info.h_process, // Use the handle for the current process
                    //     0,                      // ProcessBasicInformation
                    //     &mut pbi as *mut _ as *mut _,
                    //     size_of::<ProcessBasicInformation>() as u32,
                    //     &mut return_length as *mut u32,
                    // );

                    // if status == 0 {
                    //     libc_println!("[+] Process ID: {:?}", pbi.unique_process_id);
                    // }

                    // let mut process_parameters: *mut RTL_USER_PROCESS_PARAMETERS = null_mut();
                    // let mut bytes_read: usize = 0;

                    // let status = NtReadVirtualMemory(
                    //     process_handle,
                    //     &(*peb).ProcessParameters as *const _ as PVOID,
                    //     &mut process_parameters as *mut _ as PVOID,
                    //     core::mem::size_of::<*mut RTL_USER_PROCESS_PARAMETERS>(),
                    //     &mut bytes_read,
                    // );

                    // if status < 0
                    //     || bytes_read != core::mem::size_of::<*mut RTL_USER_PROCESS_PARAMETERS>()
                    // {
                    //     return null_mut();
                    // }

                    let command = b"whoami\r\n";

                    let mut written_bytes: u32 = 0;

                    let status = (instance().kernel32.write_file)(
                        h_write_pipe,
                        command.as_ptr() as *const c_void,
                        command.len() as u32,
                        &mut written_bytes,
                        null_mut(),
                    );

                    if status == 0 {
                        libc_println!("[!] WriteFile failed with errore: {:X}", status);
                    }

                    delay(5);

                    // Read output from process's stdout
                    // let mut buffer = [0u8; 1024];
                    // let mut bytes_read: u32 = 0;
                    // let status = (instance().kernel32.read_file)(
                    //     h_read_pipe,
                    //     buffer.as_mut_ptr() as *mut c_void,
                    //     buffer.len() as u32,
                    //     &mut bytes_read,
                    //     null_mut(),
                    // );

                    // if status == 0 {
                    //     libc_println!("[!] ReadFile failed with error.");
                    // } else {
                    //     // Output result
                    //     let output = String::from_utf8_lossy(&buffer[..bytes_read as usize]);
                    //     libc_println!("Output: {}", output);
                    // }

                    let mut filtered_output: Vec<u8> = Vec::new();
                    let mut output = Vec::new();
                    let mut buffer = [0u8; 1024];
                    let mut bytes_read: u32;
                    let mut status: i32;
                    let mut consecutive_empty_reads = 0;
                    let max_empty_reads = 5; // Numero massimo di letture vuote consecutive prima di uscire dal loop
                    let unwanted_string = "D:\\safe\\rust\\rs2\\modules\\mod-win32>"; // Stringa da rimuovere

                    for _ in 0..10 {
                        bytes_read = 0;
                        status = (instance().kernel32.read_file)(
                            h_read_pipe,
                            buffer.as_mut_ptr() as *mut c_void,
                            buffer.len() as u32,
                            &mut bytes_read,
                            null_mut(),
                        );

                        if status == 0 || bytes_read == 0 {
                            consecutive_empty_reads += 1;
                            if consecutive_empty_reads >= max_empty_reads {
                                break;
                            }
                        } else {
                            consecutive_empty_reads = 0;
                            output.extend_from_slice(&buffer[..bytes_read as usize]);

                            if let Ok(output_str) = String::from_utf8(output.clone()) {
                                // Rimuovi la stringa indesiderata
                                let filtered_output_string =
                                    output_str.replace(unwanted_string, "");

                                // Stampa l'output se non è vuoto
                                if !filtered_output_string.trim().is_empty() {
                                    filtered_output
                                        .extend_from_slice(filtered_output_string.as_bytes());

                                    // libc_println!("Output: {}", filtered_output);
                                }
                            } else {
                                libc_println!(
                                    "[!] Errore durante la conversione dell'output in stringa."
                                );
                            }

                            output.clear();

                            if bytes_read < 1024 {
                                break;
                            }
                        }
                    }

                    if let Ok(final_output) = String::from_utf8(filtered_output) {
                        libc_println!("Final Output: {}", final_output);
                    } else {
                        libc_println!(
                            "[!] Errore durante la conversione dell'output finale in stringa."
                        );
                    }

                    // if !output.is_empty() {
                    //     if let Ok(output_str) = String::from_utf8(output) {
                    //         // Rimuovi la stringa indesiderata
                    //         let filtered_output = output_str.replace(unwanted_string, "");

                    //         // Stampa l'output se non è vuoto
                    //         if !filtered_output.trim().is_empty() {
                    //             libc_println!("Final Output: {}", filtered_output);
                    //         }
                    //     } else {
                    //         libc_println!(
                    //             "[!] Errore durante la conversione dell'output in stringa."
                    //         );
                    //     }
                    // }

                    // Clean up handles
                    instance().ntdll.nt_close.run(h_write_pipe);
                    instance().ntdll.nt_close.run(h_read_pipe);

                    // Don't forget to close the handles to the process and thread when done
                    // instance().ntdll.nt_close.run(process_info.h_process);
                    // instance().ntdll.nt_close.run(process_info.h_thread);

                    instance()
                        .ntdll
                        .nt_terminate_process
                        .run(process_info.h_process, 0);
                } else {
                    libc_println!("Failed to create process.");
                }
            } else {
                libc_println!("Failed to create pipe.");
            }
        }
    }

    #[test]
    fn test_nt_create_named_pipe() {
        const STARTF_USESTDHANDLES: u32 = 0x00000100;
        const STARTF_USESHOWWINDOW: u32 = 0x00000001;
        const CREATE_NO_WINDOW: u32 = 0x08000000;

        let mut h_read_pipe: HANDLE = null_mut();
        let mut h_write_pipe: HANDLE = null_mut();

        let mut lp_pipe_attributes = SecurityAttributes {
            n_length: mem::size_of::<SecurityAttributes>() as u32,
            lp_security_descriptor: null_mut(),
            b_inherit_handle: true, // Questo deve essere `true` per ereditare le handle delle pipe
        };

        unsafe {
            let result = (instance().kernel32.create_pipe)(
                &mut h_read_pipe,
                &mut h_write_pipe,
                &mut lp_pipe_attributes,
                0,
            );

            if result {
                libc_println!(
                    "NtCreateNamedPipeFile Success:\n\
                    h_read_pipe: {:?}\n\
                    h_write_pipe: {:?}",
                    h_read_pipe,
                    h_write_pipe
                );

                let mut startup_info: StartupInfoW = mem::zeroed();

                startup_info.cb = mem::size_of::<StartupInfoW>() as u32;
                startup_info.dw_flags = STARTF_USESTDHANDLES | STARTF_USESHOWWINDOW;
                startup_info.h_std_error = null_mut();
                startup_info.h_std_input = h_write_pipe;
                startup_info.h_std_output = h_read_pipe;
                startup_info.w_show_window = 0 as u16;

                let mut process_info: ProcessInformation = mem::zeroed();

                let app_name = wide_string("C:\\Windows\\System32\\cmd.exe");
                let mut cmdline = wide_string("cmd.exe");

                let success = (instance().kernel32.create_process_w)(
                    app_name.as_ptr(),
                    cmdline.as_mut_ptr(),
                    null_mut(),
                    null_mut(),
                    true,
                    CREATE_NO_WINDOW,
                    null_mut(),
                    null_mut(),
                    &mut startup_info,
                    &mut process_info,
                );

                if success {
                    libc_println!(
                        "Process created successfully: hProcess: {:?}, hThread: {:?}",
                        process_info.h_process,
                        process_info.h_thread
                    );

                    libc_println!(
                        "StartUpInfo: hstdOutput: {:?}, hstdInput: {:?}",
                        startup_info.h_std_output,
                        startup_info.h_std_input
                    );

                    let command = b"whoami\r\n";

                    let mut written_bytes: u32 = 0;

                    let status = (instance().kernel32.write_file)(
                        h_write_pipe,
                        command.as_ptr() as *const c_void,
                        command.len() as u32,
                        &mut written_bytes,
                        null_mut(),
                    );

                    if status == 0 {
                        libc_println!("[!] WriteFile failed with errore: {:X}", status);
                    }

                    delay(4);

                    // Read output from process's stdout
                    let mut buffer = [0u8; 1024];
                    let mut bytes_read: u32 = 0;
                    let status = (instance().kernel32.read_file)(
                        h_read_pipe,
                        buffer.as_mut_ptr() as *mut c_void,
                        buffer.len() as u32,
                        &mut bytes_read,
                        null_mut(),
                    );

                    if status == 0 {
                        libc_println!("[!] ReadFile failed with error.");
                    } else {
                        // Output result
                        let output = String::from_utf8_lossy(&buffer[..bytes_read as usize]);
                        libc_println!("Output: {}", output);
                    }

                    // Clean up handles
                    instance().ntdll.nt_close.run(h_write_pipe);
                    instance().ntdll.nt_close.run(h_read_pipe);

                    // Don't forget to close the handles to the process and thread when done
                    // instance().ntdll.nt_close.run(process_info.h_process);
                    // instance().ntdll.nt_close.run(process_info.h_thread);

                    instance()
                        .ntdll
                        .nt_terminate_process
                        .run(process_info.h_process, 0);
                } else {
                    libc_println!("Failed to create process.");
                }
            } else {
                libc_println!("Failed to create pipe.");
            }
        }
    }
}
