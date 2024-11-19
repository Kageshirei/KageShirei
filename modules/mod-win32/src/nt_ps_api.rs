use alloc::{boxed::Box, vec, vec::Vec};
use core::{
    ffi::{c_ulong, c_void},
    mem::{self, size_of},
    ptr::null_mut,
};

use kageshirei_win32::{
    ntapi::nt_current_process,
    ntdef::{
        AccessMask,
        ClientId,
        IoStatusBlock,
        LargeInteger,
        ObjectAttributes,
        ProcessBasicInformation,
        ProcessInformation,
        PsAttributeList,
        PsCreateInfo,
        PsCreateInfoUnion,
        PsCreateInitialFlags,
        PsCreateInitialState,
        PsCreateState,
        RtlUserProcessParameters,
        SecurityAttributes,
        StartupInfoW,
        SystemInformationClass,
        SystemProcessInformation,
        TokenMandatoryLabel,
        UnicodeString,
        CREATE_NO_WINDOW,
        FILE_CREATE,
        FILE_GENERIC_WRITE,
        FILE_NON_DIRECTORY_FILE,
        FILE_PIPE_BYTE_STREAM_MODE,
        FILE_PIPE_BYTE_STREAM_TYPE,
        FILE_PIPE_QUEUE_OPERATION,
        FILE_SHARE_READ,
        FILE_SHARE_WRITE,
        FILE_SYNCHRONOUS_IO_NONALERT,
        FILE_WRITE_ATTRIBUTES,
        GENERIC_READ,
        HANDLE,
        NTSTATUS,
        OBJ_CASE_INSENSITIVE,
        OBJ_INHERIT,
        PROCESS_ALL_ACCESS,
        PROCESS_CREATE_FLAGS_INHERIT_HANDLES,
        PS_ATTRIBUTE_IMAGE_NAME,
        PS_ATTRIBUTE_PARENT_PROCESS,
        RTL_USER_PROC_PARAMS_NORMALIZED,
        STARTF_USESHOWWINDOW,
        STARTF_USESTDHANDLES,
        SYNCHRONIZE,
        THREAD_ALL_ACCESS,
        TOKEN_INTEGRITY_LEVEL,
        TOKEN_QUERY,
        ULONG,
    },
    ntstatus::{
        NT_SUCCESS,
        STATUS_BUFFER_OVERFLOW,
        STATUS_BUFFER_TOO_SMALL,
        STATUS_END_OF_FILE,
        STATUS_INFO_LENGTH_MISMATCH,
        STATUS_PENDING,
    },
};
use libc_print::libc_println;
use mod_agentcore::{
    instance,
    ldr::{nt_current_teb, nt_get_last_error},
};

use crate::{
    nt_time::wait_until,
    utils::{format_named_pipe_string, unicodestring_to_string, NT_STATUS},
};

/// Retrieves the PID (Process ID) and PPID (Parent Process ID) of the current process using the NT
/// API.
///
/// This function queries the `ProcessBasicInformation` of the current process to obtain its
/// unique process ID and the process ID of its parent. The syscall `NtQueryInformationProcess`
/// is used to retrieve this information.
///
/// # Safety
/// This function performs unsafe operations, such as making direct system calls to retrieve process
/// information. Care should be taken when handling the raw pointers and interpreting the returned
/// data.
///
/// # Returns
/// A tuple `(u32, u32)` containing the PID and PPID of the current process.
/// If the operation fails, both values in the tuple will be `0`.
pub unsafe fn get_pid_and_ppid() -> (u32, u32) {
    let mut pbi: ProcessBasicInformation = core::mem::zeroed();
    let mut return_length: u32 = 0;

    // Perform a system call to NtQueryInformationProcess to retrieve the basic process information.
    let status = instance().ntdll.nt_query_information_process.run(
        -1isize as HANDLE,            // Use the special handle `-1` to refer to the current process.
        0,                            // Query the `ProcessBasicInformation`.
        &mut pbi as *mut _ as *mut _, // Provide a pointer to the ProcessBasicInformation structure.
        size_of::<ProcessBasicInformation>() as u32, // Specify the size of the structure.
        &mut return_length as *mut u32, // Receive the actual size of the returned data.
    );

    if status == 0 {
        return (
            pbi.unique_process_id as u32,                // Return the PID of the current process.
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
/// This function performs unsafe operations, such as making direct system calls to retrieve process
/// information. Care should be taken when handling the raw pointers and interpreting the returned
/// data.
///
/// # Returns
/// A `u32` containing the PID of the current process. If the operation fails, it returns `0`.
pub unsafe fn get_current_process_id() -> u32 {
    let mut pbi: ProcessBasicInformation = core::mem::zeroed();
    let mut return_length: u32 = 0;

    // Perform a system call to NtQueryInformationProcess to retrieve the basic process information.
    let status = instance().ntdll.nt_query_information_process.run(
        -1isize as HANDLE,            // Use the special handle `-1` to refer to the current process.
        0,                            // Query the `ProcessBasicInformation`.
        &mut pbi as *mut _ as *mut _, // Provide a pointer to the ProcessBasicInformation structure.
        size_of::<ProcessBasicInformation>() as u32, // Specify the size of the structure.
        &mut return_length as *mut u32, // Receive the actual size of the returned data.
    );

    if status == 0 {
        return pbi.unique_process_id as u32; // Return the PID of the current process.
    }

    0 // Return 0 if the operation fails.
}

/// Retrieves a handle to a process with the specified PID and desired access rights using the NT
/// API.
///
/// This function opens a handle to a target process by specifying its process ID (PID) and the
/// desired access rights. The syscall `NtOpenProcess` is used to obtain the handle, and the
/// function initializes the required structures (`OBJECT_ATTRIBUTES` and `CLIENT_ID`) needed to
/// make the system call.
///
/// # Safety
/// This function involves unsafe operations, including raw pointer dereferencing and direct system
/// calls. Ensure that the parameters passed to the function are valid and the function is called in
/// a safe context.
///
/// # Parameters
/// - `pid`: The process ID of the target process.
/// - `desired_access`: The desired access rights for the process handle, specified as an
///   `AccessMask`.
///
/// # Returns
/// A handle to the process if successful, otherwise `null_mut()` if the operation fails.
pub unsafe fn get_process_handle(pid: i32, desired_access: AccessMask) -> HANDLE {
    let mut process_handle: HANDLE = null_mut();

    // Initialize object attributes for the process, setting up the basic structure with default
    // options.
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
    client_id.unique_process = pid as *mut core::ffi::c_void;

    // Perform a system call to NtOpenProcess to obtain a handle to the specified process.
    instance().ntdll.nt_open_process.run(
        &mut process_handle,    // Pointer to the handle that will receive the process handle.
        desired_access,         // Specify the access rights desired for the process handle.
        &mut object_attributes, // Provide the object attributes for the process.
        &mut client_id as *mut _ as *mut c_void, // Pass the client ID (target process ID).
    );

    process_handle // Return the obtained process handle, or `null_mut()` if the operation fails.
}

/// Retrieves a handle to a process with the specified name and desired access rights using the NT
/// API.
///
/// This function takes a process name, searches for its process ID (PID) using the
/// `process_snapshot` function, and then opens a handle to the process using `NtOpenProcess`.
///
/// # Safety
/// This function involves unsafe operations, including raw pointer dereferencing, memory
/// allocations, and direct system calls. Ensure that the parameters passed to the function are
/// valid and the function is called in a safe context.
///
/// # Parameters
/// - `process_name`: The name of the target process as a string slice.
/// - `desired_access`: The desired access rights for the process handle, specified as an
///   `AccessMask`.
///
/// # Returns
/// A handle to the process if successful, otherwise `null_mut()` if the operation fails or if the
/// process is not found.
pub unsafe fn get_process_handle_by_name(process_name: &str, desired_access: AccessMask) -> HANDLE {
    let mut snapshot: *mut SystemProcessInformation = null_mut();
    let mut size: usize = 0;

    // Take a snapshot of all currently running processes.
    let status = nt_process_snapshot(&mut snapshot, &mut size);

    if !NT_SUCCESS(status) {
        libc_println!(
            "Failed to retrieve process snapshot: Status[{}]",
            NT_STATUS(status)
        );
        return null_mut();
    }

    let mut current = snapshot;
    while !current.is_null() {
        // Convert the process name from Unicode to a Rust string.
        if let Some(name) = unicodestring_to_string(&(*current).image_name) {
            if name.eq_ignore_ascii_case(process_name) {
                // Process name matches, now open a handle to this process.
                return get_process_handle((*current).unique_process_id as i32, desired_access);
            }
        }

        // Move to the next process in the list.
        if (*current).next_entry_offset == 0 {
            break;
        }
        current = (current as *const u8).add((*current).next_entry_offset as usize) as *mut SystemProcessInformation;
    }

    null_mut() // Return null_mut() if the process is not found.
}

/// Opens the process token for a given process handle using the NT API.
///
/// This function opens a handle to the access token associated with a specified process handle.
/// The syscall `NtOpenProcessToken` is used to obtain the token handle, which is required for
/// operations that need access to the security context of the process.
///
/// # Safety
/// This function involves unsafe operations, including raw pointer dereferencing and direct system
/// calls. Ensure that the `process_handle` provided is valid and that this function is called in a
/// safe context.
///
/// # Parameters
/// - `process_handle`: A handle to the process for which the token is to be opened.
///
/// # Returns
/// A handle to the process token if successful, otherwise `null_mut()` if the operation fails.
pub unsafe fn nt_open_process_token(process_handle: HANDLE) -> HANDLE {
    let mut token_handle: HANDLE = null_mut();
    instance()
        .ntdll
        .nt_open_process_token
        .run(process_handle, TOKEN_QUERY, &mut token_handle);
    token_handle
}

/// Retrieves the integrity level of a specified process using the NT API.
///
/// This function queries the access token of a process to determine its integrity level.
/// The integrity level is an important security feature in Windows, and it is represented as a
/// Relative Identifier (RID) in the Security Identifier (SID) of the token's integrity level.
/// The syscall `NtQueryInformationToken` is used to retrieve this information.
///
/// # Safety
/// This function involves unsafe operations, including raw pointer dereferencing and direct system
/// calls. Ensure that the `process_handle` provided is valid and that this function is called in a
/// safe context.
///
/// # Parameters
/// - `process_handle`: A handle to the process for which the integrity level is to be retrieved.
///
/// # Returns
/// The integrity level as an `i32` representing the RID if successful, otherwise `-1` if the
/// operation fails.
pub unsafe fn get_process_integrity(process_handle: HANDLE) -> i16 {
    // Get the process token
    let token_handle = nt_open_process_token(process_handle);

    if token_handle.is_null() {
        libc_println!("[!] NtOpenProcessToken failed");
        return -1;
    }

    let mut label: TokenMandatoryLabel = core::mem::zeroed();
    let mut return_length: ULONG = 0;
    let mut size = 0;

    // Query the token for integrity level information
    let status = instance().ntdll.nt_query_information_token.run(
        token_handle,
        TOKEN_INTEGRITY_LEVEL as ULONG,
        &mut label as *mut _ as *mut c_void,
        size,
        &mut return_length as *mut ULONG,
    );

    // Handle potential buffer size issues
    if status != STATUS_BUFFER_OVERFLOW && status != STATUS_BUFFER_TOO_SMALL {
        libc_println!("[!] NtQueryInformationToken failed: {}", NT_STATUS(status));
        instance().ntdll.nt_close.run(token_handle);
        return -1;
    }

    if return_length == 0 {
        libc_println!("[!] NtQueryInformationToken failed return length is 0");
        instance().ntdll.nt_close.run(token_handle);
        return -1;
    }

    size = return_length;

    // Query the token again with the correct buffer size
    let status = instance().ntdll.nt_query_information_token.run(
        token_handle,
        TOKEN_INTEGRITY_LEVEL as ULONG,
        &mut label as *mut _ as *mut c_void,
        size,
        &mut return_length as *mut ULONG,
    );

    if NT_SUCCESS(status) {
        // Extract the RID (Relative Identifier) from the SID to determine the integrity level
        let sid = &*label.label.sid;
        let sub_authority_count = sid.sub_authority_count as usize;
        let rid = *sid
            .sub_authority
            .get_unchecked(sub_authority_count.overflowing_sub(1).0);

        instance().ntdll.nt_close.run(token_handle);
        rid as i16
    }
    else {
        libc_println!("[!] NtQueryInformationToken failed: {}", NT_STATUS(status));
        instance().ntdll.nt_close.run(token_handle);
        -1
    }
}

/// Creates a user process and its primary thread using the specified process image, command line,
/// and current directory. This function returns the handles to the created process and thread.
///
/// If a parent process handle is not provided (i.e., `null_mut()` is passed), the current process
/// will be used as the parent, ensuring that the new process remains within the correct process
/// tree.
///
/// The function uses the `NtCreateUserProcess` syscall to create the process and thread,
/// initializing the necessary structures, such as `RTL_USER_PROCESS_PARAMETERS`,
/// `PS_ATTRIBUTE_LIST`, and `PS_CREATE_INFO`.
///
/// # Parameters
/// - `sz_target_process`: A string slice that specifies the NT path of the process image to be
///   created.
/// - `sz_target_process_parameters`: A string slice that specifies the command line for the
///   process.
/// - `sz_target_process_path`: A string slice that specifies the current directory for the process.
/// - `h_parent_process`: A handle to the parent process. If `null_mut()`, the current process will
///   be used as the parent.
/// - `h_process`: A mutable reference to a `HANDLE` that will receive the process handle upon
///   creation.
/// - `h_thread`: A mutable reference to a `HANDLE` that will receive the thread handle upon
///   creation.
///
/// # Returns
/// - `NTSTATUS`: The status code indicating the result of the process creation. A value of `0`
///   indicates success, while any other value represents an error.
///
/// # Safety
/// This function uses unsafe operations to interact with low-level Windows APIs. Ensure that the
/// inputs are valid and that the function is called in a safe context.
pub unsafe fn nt_create_user_process(
    sz_target_process: &str,
    sz_target_process_parameters: &str,
    sz_target_process_path: &str,
    h_parent_process: HANDLE, // PPID Spoofing
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
    let sz_target_process_utf16: Vec<u16> = sz_target_process.encode_utf16().chain(Some(0)).collect();
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

    if let Some(rtl_create_process_parameters_ex) = instance().ntdll.rtl_create_process_parameters_ex {
        // Call RtlCreateProcessParametersEx to create the RTL_USER_PROCESS_PARAMETERS structure.
        let status = rtl_create_process_parameters_ex(
            &mut upp_process_parameters,
            &us_nt_image_path,
            null_mut(),
            &us_current_directory,
            &us_command_line,
            null_mut(),
            null_mut(),
            null_mut(),
            null_mut(),
            null_mut(),
            RTL_USER_PROC_PARAMS_NORMALIZED, // Normalized process parameters
        );

        // Check if the process parameters creation failed.
        if status != 0 {
            return status; // Return error status if creation failed
        }
    }
    // // Call RtlCreateProcessParametersEx to create the RTL_USER_PROCESS_PARAMETERS structure.
    // let status = (instance().ntdll.rtl_create_process_parameters_ex)(
    //     &mut upp_process_parameters,
    //     &us_nt_image_path,
    //     null(),
    //     &us_current_directory,
    //     &us_command_line,
    //     null(),
    //     null(),
    //     null(),
    //     null(),
    //     null(),
    //     RTL_USER_PROC_PARAMS_NORMALIZED, // Normalized process parameters
    // );

    // // Check if the process parameters creation failed.
    // if status != 0 {
    //     return status; // Return error status if creation failed
    // }

    // // Initialize the PS_ATTRIBUTE_LIST structure, which holds attributes for the new process
    let mut attribute_list: PsAttributeList = mem::zeroed();
    attribute_list.total_length = mem::size_of::<PsAttributeList>();
    attribute_list.attributes[0].attribute = PS_ATTRIBUTE_IMAGE_NAME; // Attribute for image name
    attribute_list.attributes[0].size = us_nt_image_path.length as usize;
    attribute_list.attributes[0].value.value = us_nt_image_path.buffer as usize;

    // If a parent process handle is provided, add it to the attribute list
    attribute_list.attributes[1].attribute = PS_ATTRIBUTE_PARENT_PROCESS; // Attribute for parent process
    attribute_list.attributes[1].size = mem::size_of::<HANDLE>();

    if !h_parent_process.is_null() {
        attribute_list.attributes[1].value.value = h_parent_process as usize;
    }
    else {
        attribute_list.attributes[1].value.value = nt_current_process() as usize;
    }

    // Optional: Add additional attributes such as DLL mitigation policies, if necessary
    // attribute_list.attributes[2].attribute = PS_ATTRIBUTE_MITIGATION_OPTIONS;
    // attribute_list.attributes[2].size = mem::size_of::<u64>();
    // attribute_list.attributes[2].value.value =
    //     PROCESS_CREATION_MITIGATION_POLICY_BLOCK_NON_MICROSOFT_BINARIES_ALWAYS_ON as usize;

    // Initialize the PS_CREATE_INFO structure, which contains process creation information
    let mut ps_create_info = PsCreateInfo {
        size:        mem::size_of::<PsCreateInfo>(),
        state:       PsCreateState::PsCreateInitialState,
        union_state: PsCreateInfoUnion {
            init_state: PsCreateInitialState {
                init_flags:             PsCreateInitialFlags::default(),
                additional_file_access: 0,
            },
        },
    };

    // Initialize process and thread handles to null
    *h_process = null_mut();
    *h_thread = null_mut();

    // Call NtCreateUserProcess to create the process and thread.
    let status = instance().ntdll.nt_create_user_process.run(
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
    if status != 0 {
        return status; // Return error status if creation failed
    }

    0 // Return success status
}

/// Creates a named pipe and returns handles for reading and writing.
///
/// This function sets up a named pipe with specified security attributes, buffer size,
/// and other options. It creates the pipe with both read and write handles, making it
/// ready for inter-process communication using the `NtCreateNamedPipeFile` NT API function.
///
/// # Parameters
///
/// - `h_read_pipe`: A mutable reference to a handle that will receive the read handle of the
///   created pipe.
/// - `h_write_pipe`: A mutable reference to a handle that will receive the write handle of the
///   created pipe.
/// - `lp_pipe_attributes`: A pointer to a `SecurityAttributes` structure that specifies the
///   security attributes for the pipe.
/// - `n_size`: The buffer size for the pipe. If set to 0, a default size of 4096 bytes is used.
///
/// # Returns
///
/// Returns an `NTSTATUS` code indicating the success or failure of the operation. A value of 0
/// indicates success, while any other value represents an error.
///
/// # Pipe Naming Convention
///
/// The function generates a unique named pipe path using the format:
/// `\\Device\\NamedPipe\\Win32Pipes.<process_id>.<pipe_id>`.
///
/// - `<process_id>`: The ID of the process creating the pipe, ensuring that the pipe is uniquely
///   associated with the current process.
/// - `<pipe_id>`: A unique identifier for the pipe within the process, allowing multiple pipes to
///   be created by the same process without name collisions.
///
/// This name ensures that the pipe is uniquely identifiable by the creating process and the
/// specific instance of the pipe.
///
/// # Safety
///
/// This function performs operations that involve raw pointers and direct system calls,
/// which require careful handling. The function should be called within a safe context.
pub unsafe fn nt_create_named_pipe_file(
    h_read_pipe: &mut HANDLE,
    h_write_pipe: &mut HANDLE,
    lp_pipe_attributes: *mut SecurityAttributes,
    n_size: u32,
) -> NTSTATUS {
    // Initialize the necessary structures
    let mut pipe_name = UnicodeString::new();
    let mut object_attributes = ObjectAttributes::new();
    let mut status_block = IoStatusBlock::new();
    let mut default_timeout = LargeInteger::new();
    let mut read_pipe_handle: HANDLE = null_mut();
    let mut write_pipe_handle: HANDLE = null_mut();
    let mut pipe_id: u32 = 0;
    let mut security_descriptor: *mut c_void = null_mut();

    // Set the default timeout to 120 seconds
    default_timeout.high_part = -1200000000;

    // Use the default buffer size if not provided
    let n_size = if n_size == 0 { 0x1000 } else { n_size };

    // Increment the pipe ID (normally done with InterlockedIncrement)
    pipe_id = pipe_id.overflowing_add(1).0;

    // Format the pipe name using the process ID and pipe ID
    let pipe_name_utf16 = format_named_pipe_string(
        nt_current_teb().as_ref().unwrap().client_id.unique_process as usize,
        pipe_id,
    );

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
        instance().ntdll.nt_close.run(read_pipe_handle);
        return status;
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
        instance().ntdll.nt_close.run(read_pipe_handle);
        return status;
    }

    // Assign the read and write handles to the output parameters
    *h_read_pipe = read_pipe_handle;
    *h_write_pipe = write_pipe_handle;
    0
}

/// Reads data from a pipe using the `NtReadFile` syscall. This function continuously reads from
/// the pipe until all available data is consumed.
///
/// # Parameters
/// - `handle`: The handle to the pipe from which to read data.
/// - `buffer`: A mutable reference to a vector where the read data will be stored.
///
/// # Returns
/// - `true` if the data was successfully read from the pipe.
/// - `false` if the read operation failed.
///
/// # Safety
/// This function uses unsafe operations to interact with low-level Windows APIs. Ensure that the
/// inputs are valid and that the function is called in a safe context.
pub unsafe fn nt_read_pipe(handle: HANDLE, buffer: &mut Vec<u8>) -> bool {
    let mut io_status_block = IoStatusBlock::new(); // IO status block for the read operation
    let mut local_buffer = [0u8; 1024]; // Local buffer to store each chunk of data read
    let mut has_data = false; // Track if we have read any data

    loop {
        // Call the NtReadFile syscall to read from the pipe
        let status = instance().ntdll.nt_read_file.run(
            handle,                                   // Handle to the pipe
            null_mut(),                               // Event, usually null
            null_mut(),                               // APC routine, usually null
            null_mut(),                               // APC context, usually null
            &mut io_status_block,                     // IO status block to receive status and information
            local_buffer.as_mut_ptr() as *mut c_void, // Buffer to receive the data
            local_buffer.len() as u32,                // Length of the buffer
            null_mut(),                               // Offset, usually null for pipes
            null_mut(),                               // Key, usually null
        );

        // If the call was not successful, handle the error
        if !NT_SUCCESS(status) {
            if status == STATUS_END_OF_FILE || io_status_block.information == 0 {
                // End of file or no more data is available, break the loop
                break;
            }
            else if status == STATUS_PENDING {
                // If operation is pending, continue waiting for data
                continue;
            }
            else {
                // Some other error occurred, so break the loop
                return false;
            }
        }

        // Number of bytes read in this iteration
        let bytes_read = io_status_block.information;

        // If no bytes were read, it means there's no more data
        if bytes_read == 0 {
            break;
        }
        // Append the data from the local buffer to the provided buffer
        if let Some(slice) = local_buffer.get(.. bytes_read as usize) {
            buffer.extend_from_slice(slice);
        }
        else {
            // This should never happen because `bytes_read` is guaranteed to be within `local_buffer.len()`.
            return false;
        }

        // Mark that we have successfully read some data
        has_data = true;

        if bytes_read < local_buffer.len() as u32 {
            break;
        }
    }

    // Return true if we have read any data, false otherwise
    has_data
}

/// Creates a process using the specified executable and command line, and captures its output
/// by utilizing named pipes for inter-process communication. The process is created without
/// a visible window.
///
/// # Parameters
/// - `target_process`: A string slice containing the NT path of the process executable to be
///   created.
/// - `cmdline`: A string slice containing the command line to execute within the created process.
///
/// # Returns
/// A `Vec<u8>` containing the output of the executed command. If the execution fails, an empty
/// vector is returned.
///
/// # Details
/// This function uses the following API functions:
/// - `NtCreateNamedPipeFile`: To create the named pipes for communication.
/// - `CreateProcessW`: A kernel32 function to create the process.
/// - `NtReadFile`: To read the output from the created process.
/// - `NtClose`: To close handles to the pipes after use.
///
/// # Safety
/// This function performs unsafe operations to interact with low-level Windows APIs and to manage
/// process handles. Ensure that the function is called in a safe context. Proper error handling
/// is crucial to avoid resource leaks.
///
/// # Note
/// The function assumes that `target_process` is a valid NT path and that the command line is
/// valid. If the paths or command lines are invalid, the function will fail and return an empty
/// output vector.
pub unsafe fn nt_create_process_w_piped(target_process: &str, cmdline: &str) -> Vec<u8> {
    // Initialize the vector to store the output of the command.
    let mut output = Vec::new();

    // Validate the input parameters to ensure they are not empty.
    if target_process.is_empty() || cmdline.is_empty() {
        libc_println!("[!] Invalid parameters: target_process or cmdline is empty.");
        return output;
    }

    // Initialize handles for the read and write ends of the pipe.
    let mut h_read_pipe: HANDLE = null_mut();
    let mut h_write_pipe: HANDLE = null_mut();

    // Define security attributes to allow handle inheritance.
    let mut lp_pipe_attributes = SecurityAttributes {
        n_length:               mem::size_of::<SecurityAttributes>() as u32, // Size of the structure.
        lp_security_descriptor: null_mut(),                                  // No specific security descriptor.
        b_inherit_handle:       true,                                        // Allow handle inheritance.
    };

    unsafe {
        // Create a named pipe for communication between processes.
        let status = nt_create_named_pipe_file(
            &mut h_read_pipe,
            &mut h_write_pipe,
            &mut lp_pipe_attributes,
            0, // Use the default buffer size of 4096 bytes.
        );

        // If pipe creation fails, log the error and return an empty vector.
        if !NT_SUCCESS(status) {
            libc_println!(
                "[!] Failed to create named pipe: NTSTATUS [{}]",
                NT_STATUS(status)
            );
            return Vec::new();
        }

        // Initialize the startup info structure for process creation.
        let mut startup_info: StartupInfoW = mem::zeroed();

        // Configure the startup information to use the pipe handles for standard output and error.
        startup_info.cb = mem::size_of::<StartupInfoW>() as u32;
        startup_info.dw_flags = STARTF_USESTDHANDLES | STARTF_USESHOWWINDOW; // Use standard handles and show window.
        startup_info.h_std_error = h_write_pipe; // Redirect standard error to the write pipe.
        startup_info.h_std_input = null_mut(); // No standard input.
        startup_info.h_std_output = h_write_pipe; // Redirect standard output to the write pipe.
        startup_info.w_show_window = 0; // Hide the window.

        // Initialize the process information structure.
        let mut process_info: ProcessInformation = mem::zeroed();

        // Convert the target process path and command line to UTF-16.
        let target_process_utf16: Vec<u16> = target_process.encode_utf16().chain(Some(0)).collect();
        let mut cmdline_utf16: Vec<u16> = cmdline.encode_utf16().chain(Some(0)).collect();

        if let Some(create_process_w) = instance().kernel32.create_process_w {
            // Create the process using CreateProcessW from kernel32.dll.
            let success = create_process_w(
                target_process_utf16.as_ptr(), // Path to the target executable.
                cmdline_utf16.as_mut_ptr(),    // Command line to execute.
                null_mut(),                    // No process security attributes.
                null_mut(),                    // No thread security attributes.
                true,                          // Inherit handles.
                CREATE_NO_WINDOW,              // Create the process without a window.
                null_mut(),                    // No environment block.
                null_mut(),                    // Use the current directory.
                &mut startup_info,             // Startup info structure.
                &mut process_info,             // Process information structure.
            );

            // Delay slightly to allow the process to start.
            wait_until(3);

            // If process creation fails, log the error and return the collected output (likely empty).
            if !success {
                libc_println!(
                    "[!] Failed to create process: GetLastError [{}]",
                    nt_get_last_error()
                );
                return output;
            }

            // Read the output from the read pipe using NtReadFile.
            if !nt_read_pipe(h_read_pipe, &mut output) {
                libc_println!(
                    "[!] Failed to read from pipe: NTSTATUS [{}]",
                    NT_STATUS(status)
                );
                return output;
            }

            // Clean up handles using NtClose.
            instance().ntdll.nt_close.run(h_write_pipe);
            instance().ntdll.nt_close.run(h_read_pipe);

            return output; // Return the collected output.
        }

        output // Return the collected output.
    }
}

/// Takes a snapshot of the currently running processes.
///
/// This function utilizes the `NtQuerySystemInformation` function from the NT API to retrieve
/// information about all processes currently running on the system. It first determines the
/// necessary buffer size, then allocates memory, and finally retrieves the process information.
///
/// # Parameters
/// - `snapshot`: A mutable reference to a pointer that will receive the snapshot of the process
///   information.
/// - `size`: A mutable reference to a variable that will receive the size of the snapshot.
///
/// # Returns
/// An `NTSTATUS` code indicating the success or failure of the operation.
///
/// # Safety
/// This function performs unsafe operations, such as memory allocation and system calls. The caller
/// must ensure that the inputs are valid and that the function is called in a safe context.
pub unsafe fn nt_process_snapshot(snapshot: &mut *mut SystemProcessInformation, size: &mut usize) -> NTSTATUS {
    let mut length: u32 = 0;

    // First call to determine the required length of the buffer for process information.
    let mut status = instance().ntdll.nt_query_system_information.run(
        SystemInformationClass::SystemProcessInformation as u32,
        null_mut(),  // No buffer is provided, so the length will be returned.
        0,           // Buffer length is 0 to request the required size.
        &mut length, // The required buffer size will be stored here.
    );

    // Check if the call returned STATUS_INFO_LENGTH_MISMATCH (expected) or another error.
    if status != STATUS_INFO_LENGTH_MISMATCH && !NT_SUCCESS(status) {
        return status;
    }

    // Allocate memory for the SystemProcessInformation structure.
    let mut buffer = vec![0; length as usize].into_boxed_slice();

    // let mut buffer = Vec::new();
    // buffer.resize(length as usize, 0); // Initialize buffer with the required length.

    // Second call to actually retrieve the process information into the allocated buffer.
    status = instance().ntdll.nt_query_system_information.run(
        SystemInformationClass::SystemProcessInformation as u32,
        buffer.as_mut_ptr() as *mut c_void, // Provide the allocated buffer.
        length,                             // Buffer length.
        &mut length,                        // Update the actual size used.
    );

    // Check if the process information retrieval was successful.
    if !NT_SUCCESS(status) {
        return status;
    }

    // Cast the buffer to the SystemProcessInformation structure.
    *snapshot = buffer.as_mut_ptr() as *mut SystemProcessInformation;
    *size = length as usize;

    // // Keep the buffer alive by preventing its deallocation.
    // core::mem::forget(buffer);

    // Transfer ownership of the buffer so it isn't deallocated.
    // By turning it into a Box, Rust will ensure it stays alive until explicitly dropped elsewhere.
    Box::leak(buffer);

    status
}

/// Creates a new thread in the current process using the NT API `NtCreateThreadEx`.
///
/// This function initializes the necessary structures and makes a system call to `NtCreateThreadEx`
/// to create a new thread in the specified process with the given start routine and arguments.
///
/// # Parameters
/// - `proc_handle`: The handle to the process in which to create the thread. This is usually
///   obtained via `get_process_handle`.
/// - `start_routine`: A pointer to the function that the new thread will execute.
/// - `arg_ptr`: A pointer to the arguments that will be passed to the thread's start routine.
///
/// # Returns
/// - `HANDLE`: A handle to the newly created thread if successful. The handle will be `null_mut()`
///   if the thread creation fails.
///
/// # Safety
/// This function is unsafe because it directly interacts with low-level Windows APIs and performs
/// raw pointer dereferencing.
#[expect(
    clippy::fn_to_numeric_cast_any,
    reason = "Required to pass function pointer to NtCreateThreadEx API"
)]
pub unsafe fn nt_create_thread_ex(
    proc_handle: HANDLE,
    start_routine: extern "system" fn(*mut c_void) -> u32,
    arg_ptr: *mut c_void,
) -> HANDLE {
    let mut thread_handle: HANDLE = null_mut();
    let mut client_id = ClientId::new();
    client_id.unique_process = get_current_process_id() as *mut core::ffi::c_void;

    // Initialize object attributes for the thread
    let mut obj_attr = ObjectAttributes::new();

    ObjectAttributes::initialize(
        &mut obj_attr,
        null_mut(),
        OBJ_CASE_INSENSITIVE, // 0x40
        null_mut(),
        null_mut(),
    );

    // Call NtCreateThreadEx to create the thread
    let status = instance().ntdll.nt_create_thread_ex.run(
        &mut thread_handle,
        THREAD_ALL_ACCESS,                       // Full access to the thread
        &mut obj_attr,                           // ObjectAttributes can be null
        proc_handle,                             // Handle to the current process
        start_routine as *mut core::ffi::c_void, // Start routine for the new thread
        arg_ptr,                                 // Argument to pass to the start routine
        0,                                       // Non create the thread in suspended state
        0,                                       // StackZeroBits
        0,                                       // SizeOfStackCommit
        0,                                       // SizeOfStackReserve
        null_mut(),                              // BytesBuffer can be null
    );

    if !NT_SUCCESS(status) {
        libc_println!("Failed to create thread: {}", NT_STATUS(status));
        return null_mut();
    }

    let status = instance()
        .ntdll
        .nt_wait_for_single_object
        .run(thread_handle, false, null_mut());

    if status != 0 {
        libc_println!(
            "NTWaitForSingleObject failed with status: 0x{}",
            NT_STATUS(status)
        );
        return null_mut();
    }

    libc_println!("Thread created and completed successfully.");

    thread_handle
}

pub extern "system" fn my_thread_start_routine(param: *mut c_void) -> c_ulong {
    let arg: *mut u32 = param as *mut u32;
    if !arg.is_null() {
        unsafe {
            libc_println!("Thread running with param: {}", *arg);
        }
    }
    else {
        libc_println!("Thread running with no param");
    }
    0
}

#[cfg(test)]
mod tests {

    use alloc::string::String;

    use libc_print::libc_println;

    use super::*;
    use crate::utils::NT_STATUS;

    #[test]
    fn test_nt_create_thread_ex() {
        let proc_handle = -1isize as HANDLE;
        // unsafe { get_process_handle(get_current_process_id() as i32, PROCESS_ALL_ACCESS) };

        let arg: i32 = 42; // Esempio di argomento da passare al thread
        let arg_ptr = &arg as *const _ as *mut c_void;

        let thread_handle = unsafe { nt_create_thread_ex(proc_handle, my_thread_start_routine, arg_ptr) };

        // Assicurati di chiudere il thread handle quando non è più necessario.
        if !thread_handle.is_null() {
            unsafe {
                instance().ntdll.nt_close.run(thread_handle);
            }
        }
    }

    #[test]
    fn test_get_pid_and_ppid() {
        let (pid, ppid) = unsafe { get_pid_and_ppid() };
        libc_println!("PID: {:?}", pid);
        libc_println!("PPID: {:?}", ppid);
        assert!(pid != 0, "PID should not be zero");
        assert!(ppid != 0, "PPID should not be zero");
    }

    #[test]
    fn test_get_process_integrity() {
        unsafe {
            let rid = get_process_integrity(nt_current_process());
            libc_println!("Process Integrity Level: {:?}", rid);
            assert!(rid != -1, "RID should not be -1");
        }
    }

    #[test]
    fn test_get_process_handle_by_name() {
        unsafe {
            let handle = get_process_handle_by_name("explorer.exe", PROCESS_ALL_ACCESS);
            if !handle.is_null() {
                libc_println!("Handle obtained: {:?}", handle);
                instance().ntdll.nt_close.run(handle);
            }
            else {
                libc_println!("Process not found.");
            }
        }
    }

    #[test]
    fn test_nt_create_user_process() {
        unsafe {
            let sz_target_process = "\\??\\C:\\Windows\\System32\\cmd.exe";
            let sz_target_process_parameters = "C:\\Windows\\System32\\cmd.exe";
            let sz_target_process_path = "C:\\Windows\\System32";

            let mut h_process: HANDLE = null_mut();
            let mut h_thread: HANDLE = null_mut();

            let status = nt_create_user_process(
                &sz_target_process,
                &sz_target_process_parameters,
                &sz_target_process_path,
                null_mut(),
                &mut h_process,
                &mut h_thread,
            );

            if !h_process.is_null() {
                libc_println!(
                    "NtCreateUserProcess Success:\nhProcess: {:?}\nhThread: {:?}",
                    h_process,
                    h_thread
                );

                instance().ntdll.nt_terminate_process.run(h_process, 0);
            }
            else {
                libc_println!("Failed to create process: {}", NT_STATUS(status));
            }
        }
    }

    #[test]
    fn test_nt_create_user_process_ppid_spoof() {
        unsafe {
            let sz_target_process = "\\??\\C:\\Windows\\System32\\cmd.exe";
            let sz_target_process_parameters = "C:\\Windows\\System32\\cmd.exe";
            let sz_target_process_path = "C:\\Windows\\System32";

            let mut h_process: HANDLE = null_mut();
            let mut h_thread: HANDLE = null_mut();

            let h_parent_process_handle: HANDLE = get_process_handle_by_name("explorer.exe", PROCESS_ALL_ACCESS);

            if h_parent_process_handle.is_null() {
                libc_println!("[!] GetProcHandle Failed");
                return;
            }
            else {
                libc_println!("[!] Parent Process Handle: {:p}", h_parent_process_handle);
            }

            let status = nt_create_user_process(
                &sz_target_process,
                &sz_target_process_parameters,
                &sz_target_process_path,
                h_parent_process_handle,
                &mut h_process,
                &mut h_thread,
            );

            if !h_process.is_null() {
                libc_println!(
                    "NtCreateUserProcess Success:\nhProcess: {:?}\nhThread: {:?}",
                    h_process,
                    h_thread
                );

                // instance().ntdll.nt_terminate_process.run(h_process, 0);
            }
            else {
                libc_println!("Failed to create process: {:?}", NT_STATUS(status));
            }
        }
    }

    #[test]
    fn test_nt_shell_create_process_w() {
        unsafe {
            let target_process = "C:\\Windows\\System32\\cmd.exe";
            let sz_target_process_parameters = "cmd.exe /c powershell -v 2 -c $PSVersionTable";

            let output = nt_create_process_w_piped(&target_process, sz_target_process_parameters);

            if !output.is_empty() {
                let output_str = String::from_utf8_lossy(&output);
                libc_println!("Output: {}", output_str);
            }
        }
    }

    #[test]
    fn test_nt_process_snapshot() {
        unsafe {
            let mut snapshot: *mut SystemProcessInformation = null_mut();
            let mut size: usize = 0;

            let status = nt_process_snapshot(&mut snapshot, &mut size);

            if NT_SUCCESS(status) {
                // Process snapshot successfully retrieved, now you can work with it
                // For example, iterate over processes in the snapshot

                let mut current = snapshot;
                while !current.is_null() {
                    // If ImageName is not null, print the process name
                    if (*current).image_name.buffer != null_mut() {
                        libc_println!(
                            "PID: {} - Name: {}",
                            (*current).unique_process_id as u32,
                            unicodestring_to_string(&(*current).image_name).unwrap()
                        );
                    }
                    else {
                        libc_println!(
                            "PID: {} - Name: <unknown>",
                            (*current).unique_process_id as u32
                        );
                    }

                    // Move to the next process in the list
                    if (*current).next_entry_offset == 0 {
                        break;
                    }
                    current = (current as *const u8).add((*current).next_entry_offset as usize)
                        as *mut SystemProcessInformation;
                }
            }
            else {
                libc_println!(
                    "Failed to retrieve process snapshot: Status[{}]",
                    NT_STATUS(status),
                );
            }
        }
    }
}
