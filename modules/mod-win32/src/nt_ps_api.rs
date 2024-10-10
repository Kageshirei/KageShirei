use core::{ffi::c_void, mem::size_of, ptr::null_mut};

use mod_agentcore::instance;
use rs2_win32::{
    ntdef::{
        AccessMask, ClientId, ObjectAttributes, ProcessBasicInformation, TokenMandatoryLabel,
        HANDLE, OBJ_CASE_INSENSITIVE, TOKEN_INTEGRITY_LEVEL, TOKEN_QUERY, ULONG,
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
}
