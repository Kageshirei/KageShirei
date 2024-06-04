use core::{ffi::c_void, ptr::null_mut};

use rs2_winapi::{
    ntdef::{HANDLE, ULONG},
    ntstatus::{STATUS_BUFFER_OVERFLOW, STATUS_BUFFER_TOO_SMALL},
    winnt::{TokenMandatoryLabel, TOKEN_INTEGRITY_LEVEL, TOKEN_QUERY},
};

use crate::{nt_close, nt_open_process_token, nt_query_information_token};

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
    nt_open_process_token(process_handle, TOKEN_QUERY, &mut token_handle);
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
    let ntstatus = nt_query_information_token(
        token_handle,
        TOKEN_INTEGRITY_LEVEL as ULONG,
        &mut label as *mut _ as *mut c_void,
        size,
        &mut return_length as *mut ULONG,
    );

    if ntstatus != STATUS_BUFFER_OVERFLOW && ntstatus != STATUS_BUFFER_TOO_SMALL {
        nt_close(token_handle);
        return -1;
    }

    if return_length == 0 {
        nt_close(token_handle);
        return -1;
    }

    size = return_length;

    // Query the token again with the correct buffer size
    let ntstatus = nt_query_information_token(
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

        nt_close(token_handle);
        return rid as i32;
    } else {
        nt_close(token_handle);
        return -1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libc_print::libc_println;

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
