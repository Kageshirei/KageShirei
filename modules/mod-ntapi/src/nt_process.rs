use crate::{nt_open_process, nt_query_information_process};
use core::{ffi::c_void, mem::size_of, ptr::null_mut};
use rs2_winapi::{
    ntdef::{AccessMask, ClientId, ObjectAttributes, HANDLE, OBJ_CASE_INSENSITIVE},
    ntpsapi::ProcessBasicInformation,
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
    let status = nt_query_information_process(
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
    let status = nt_query_information_process(
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
    nt_open_process(
        &mut process_handle,
        desired_access,
        &mut object_attributes,
        &mut client_id as *mut _ as *mut c_void,
    );

    process_handle
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
}
