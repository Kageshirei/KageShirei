#![no_std]
extern crate alloc;

pub mod nt_get_computer_name_ex;
pub mod nt_get_integrity_level;
pub mod nt_peb;
pub mod nt_process;
pub mod nt_query_system;
pub mod nt_sid;

use core::ffi::c_void;

use lazy_static::lazy_static;
use rs2_indirect_syscall::{init_syscall, run_syscall, NtSyscall};
use rs2_winapi::{
    ntdef::{AccessMask, ObjectAttributes, UnicodeString, HANDLE, ULONG},
    ntdll_config::NtdllConfig,
    winnt::TokenPrivileges,
};

const NT_OPEN_KEY_DBJ2: usize = 0x7682ed42;
const NT_QUERY_VALUE_KEY_DBJ2: usize = 0x85967123;
const NT_CLOSE_DBJ2: usize = 0x40d6e69d;
const NT_QUERY_SYSTEM_INFORMATION_DBJ2: usize = 0x7bc23928;
const NT_QUERY_INFORMATION_PROCESS: usize = 0x8cdc5dc2;
const NT_OPEN_PROCESS_DBJ2: usize = 0x4b82f718;
const NT_OPEN_PROCESS_TOKEN_DBJ2: usize = 0x350dca99;
const NT_OPEN_PROCESS_TOKEN_EX_DBJ2: usize = 0xafaade16;
const NT_QUERY_INFORMATION_TOKEN_DBJ2: usize = 0xf371fe4;
const NT_ADJUST_PRIVILEGES_TOKEN_DBJ2: usize = 0x2dbc736d;

const NT_OPEN_PARTITION_DBJ2: usize = 0x969d3173;

lazy_static! {
    static ref NTDLL_CONFIG: NtdllConfig = unsafe { NtdllConfig::instance().unwrap() };
    static ref NT_CLOSE_SYSCALL: NtSyscall = {
        let ntdll_config = &*NTDLL_CONFIG;
        init_syscall(ntdll_config, NT_CLOSE_DBJ2)
    };
    static ref NT_OPEN_KEY_SYSCALL: NtSyscall = {
        let ntdll_config = &*NTDLL_CONFIG;
        init_syscall(ntdll_config, NT_OPEN_KEY_DBJ2)
    };
    static ref NT_QUERY_VALUE_KEY_SYSCALL: NtSyscall = {
        let ntdll_config = &*NTDLL_CONFIG;
        init_syscall(ntdll_config, NT_QUERY_VALUE_KEY_DBJ2)
    };
    static ref NT_QUERY_SYSTEM_INFORMATION_SYSCALL: NtSyscall = {
        let ntdll_config = &*NTDLL_CONFIG;
        init_syscall(ntdll_config, NT_QUERY_SYSTEM_INFORMATION_DBJ2)
    };
    static ref NT_QUERY_INFORMATION_PROCESS_SYSCALL: NtSyscall = {
        let ntdll_config = &*NTDLL_CONFIG;
        init_syscall(ntdll_config, NT_QUERY_INFORMATION_PROCESS)
    };
    static ref NT_OPEN_PROCESS_SYSCALL: NtSyscall = {
        let ntdll_config = &*NTDLL_CONFIG;
        init_syscall(ntdll_config, NT_OPEN_PROCESS_DBJ2)
    };
    static ref NT_OPEN_PROCESS_TOKEN_SYSCALL: NtSyscall = {
        let ntdll_config = &*NTDLL_CONFIG;
        init_syscall(ntdll_config, NT_OPEN_PROCESS_TOKEN_DBJ2)
    };
    static ref NT_OPEN_PROCESS_TOKEN_EX_SYSCALL: NtSyscall = {
        let ntdll_config = &*NTDLL_CONFIG;
        init_syscall(ntdll_config, NT_OPEN_PROCESS_TOKEN_EX_DBJ2)
    };
    static ref NT_QUERY_INFORMATION_TOKEN_SYSCALL: NtSyscall = {
        let ntdll_config = &*NTDLL_CONFIG;
        init_syscall(ntdll_config, NT_QUERY_INFORMATION_TOKEN_DBJ2)
    };
    static ref NT_ADJUST_PRIVILEGES_TOKEN_SYSCALL: NtSyscall = {
        let ntdll_config = &*NTDLL_CONFIG;
        init_syscall(ntdll_config, NT_ADJUST_PRIVILEGES_TOKEN_DBJ2)
    };
    static ref NT_OPEN_PARTITION_SYSCALL: NtSyscall = {
        let ntdll_config = &*NTDLL_CONFIG;
        init_syscall(ntdll_config, NT_OPEN_PARTITION_DBJ2)
    };
}

/// Retrieves a handle to the current process.
///
/// # Safety
///
/// This function involves unsafe operations.
///
/// # Returns
///
/// A handle to the current process.
pub unsafe fn nt_current_process() -> HANDLE {
    -1isize as HANDLE
}

/// Wrapper function for NtClose to avoid repetitive run_syscall calls.
///
/// # Arguments
///
/// * `handle` - The handle to be closed.
///
/// # Returns
///
/// * `true` if the operation was successful, `false` otherwise.
pub fn nt_close(handle: *mut c_void) -> i32 {
    let nt_close_table = &*NT_CLOSE_SYSCALL;
    run_syscall!(
        nt_close_table.number,
        nt_close_table.address as usize,
        handle
    )
}

/// Wrapper for the NtOpenKey syscall.
///
/// # Arguments
///
/// * `p_key_handle` - A mutable pointer to a handle that will receive the key handle.
/// * `desired_access` - The desired access for the key.
/// * `object_attributes` - A pointer to the object attributes structure.
///
/// # Returns
///
/// * `i32` - The NTSTATUS code of the operation.
pub fn nt_open_key(
    p_key_handle: &mut *mut c_void,
    desired_access: AccessMask,
    object_attributes: &mut ObjectAttributes,
) -> i32 {
    let nt_open_key_table = &*NT_OPEN_KEY_SYSCALL;
    run_syscall!(
        nt_open_key_table.number,
        nt_open_key_table.address as usize,
        p_key_handle,
        desired_access,
        object_attributes as *mut _ as *mut c_void
    )
}

/// Wrapper for the NtQueryValueKey syscall.
///
/// # Arguments
///
/// * `key_handle` - A handle to the key.
/// * `value_name` - A pointer to the UnicodeString structure containing the name of the value to be queried.
/// * `key_value_information_class` - Specifies the type of information to be returned.
/// * `key_value_information` - A pointer to a buffer that receives the requested information.
/// * `length` - The size, in bytes, of the buffer pointed to by the `key_value_information` parameter.
/// * `result_length` - A pointer to a variable that receives the size, in bytes, of the data returned.
///
/// # Returns
///
/// * `i32` - The NTSTATUS code of the operation.
pub fn nt_query_value_key(
    key_handle: *mut c_void,
    value_name: &UnicodeString,
    key_value_information_class: u32,
    key_value_information: *mut c_void,
    length: ULONG,
    result_length: &mut ULONG,
) -> i32 {
    let nt_query_value_key_table = &*NT_QUERY_VALUE_KEY_SYSCALL;
    run_syscall!(
        nt_query_value_key_table.number,
        nt_query_value_key_table.address as usize,
        key_handle,
        value_name,
        key_value_information_class,
        key_value_information,
        length,
        result_length
    )
}

/// Wrapper for the NtQuerySystemInformation syscall.
///
/// # Arguments
///
/// * `system_information_class` - The system information class to be queried.
/// * `system_information` - A pointer to a buffer that receives the requested information.
/// * `system_information_length` - The size, in bytes, of the buffer pointed to by the `system_information` parameter.
/// * `return_length` - A pointer to a variable that receives the size, in bytes, of the data returned.
///
/// # Returns
///
/// * `NTSTATUS` - The NTSTATUS code of the operation.
pub fn nt_query_system_information(
    system_information_class: u32,
    system_information: *mut c_void,
    system_information_length: u32,
    return_length: *mut u32,
) -> i32 {
    let syscall_info = &*NT_QUERY_SYSTEM_INFORMATION_SYSCALL;
    run_syscall!(
        syscall_info.number,
        syscall_info.address as usize,
        system_information_class,
        system_information,
        system_information_length,
        return_length
    )
}

/// Wrapper for the NtQueryInformationProcess syscall.
///
/// # Arguments
///
/// * `process_handle` - A handle to the process.
/// * `process_information_class` - The class of information to be queried.
/// * `process_information` - A pointer to a buffer that receives the requested information.
/// * `process_information_length` - The size, in bytes, of the buffer pointed to by the `process_information` parameter.
/// * `return_length` - A pointer to a variable that receives the size, in bytes, of the data returned.
///
/// # Returns
///
/// * `NTSTATUS` - The NTSTATUS code of the operation.
pub fn nt_query_information_process(
    process_handle: HANDLE,
    process_information_class: u32,
    process_information: *mut c_void,
    process_information_length: ULONG,
    return_length: *mut ULONG,
) -> i32 {
    let nt_query_information_process_table = &*NT_QUERY_INFORMATION_PROCESS_SYSCALL;
    run_syscall!(
        nt_query_information_process_table.number,
        nt_query_information_process_table.address as usize,
        process_handle,
        process_information_class,
        process_information,
        process_information_length,
        return_length
    )
}

/// Wrapper for the NtOpenProcess syscall.
///
/// # Arguments
///
/// * `process_handle` - A mutable pointer to a handle that will receive the process handle.
/// * `desired_access` - The desired access for the process.
/// * `object_attributes` - A pointer to the object attributes structure.
/// * `client_id` - A pointer to the client ID structure.
///
/// # Returns
///
/// * `i32` - The NTSTATUS code of the operation.
pub fn nt_open_process(
    process_handle: &mut HANDLE,
    desired_access: AccessMask,
    object_attributes: &mut ObjectAttributes,
    client_id: *mut c_void,
) -> i32 {
    let nt_open_process_table = &*NT_OPEN_PROCESS_SYSCALL;
    run_syscall!(
        nt_open_process_table.number,
        nt_open_process_table.address as usize,
        process_handle,
        desired_access,
        object_attributes as *mut _ as *mut c_void,
        client_id
    )
}

/// Wrapper for the NtOpenProcessToken syscall.
///
/// # Arguments
///
/// * `process_handle` - The handle of the process whose token is to be opened.
/// * `desired_access` - The desired access for the token.
/// * `token_handle` - A mutable pointer to a handle that will receive the token handle.
///
/// # Returns
///
/// * `i32` - The NTSTATUS code of the operation.
pub fn nt_open_process_token(
    process_handle: HANDLE,
    desired_access: AccessMask,
    token_handle: &mut HANDLE,
) -> i32 {
    let nt_open_process_token_table = &*NT_OPEN_PROCESS_TOKEN_SYSCALL;
    // nt_open_process_token_table.number = 0x131;
    run_syscall!(
        nt_open_process_token_table.number,
        nt_open_process_token_table.address as usize,
        process_handle,
        desired_access,
        token_handle
    )
}

/// Wrapper for the NtOpenProcessTokenEx syscall.
///
/// # Arguments
///
/// * `process_handle` - The handle of the process whose token is to be opened.
/// * `desired_access` - The desired access for the token.
/// * `handle_attributes` - Attributes for the handle.
/// * `token_handle` - A mutable pointer to a handle that will receive the token handle.
///
/// # Returns
///
/// * `i32` - The NTSTATUS code of the operation.
pub fn nt_open_process_token_ex(
    process_handle: HANDLE,
    desired_access: AccessMask,
    handle_attributes: ULONG,
    token_handle: &mut HANDLE,
) -> i32 {
    let nt_open_process_token_ex_table = &*NT_OPEN_PROCESS_TOKEN_EX_SYSCALL;
    run_syscall!(
        nt_open_process_token_ex_table.number,
        nt_open_process_token_ex_table.address as usize,
        process_handle,
        desired_access,
        handle_attributes,
        token_handle
    )
}

/// Wrapper for the NtQueryInformationToken syscall.
///
/// # Arguments
///
/// * `token_handle` - The handle of the token to be queried.
/// * `token_information_class` - The class of information to be queried.
/// * `token_information` - A pointer to a buffer that receives the requested information.
/// * `token_information_length` - The size, in bytes, of the buffer pointed to by the `token_information` parameter.
/// * `return_length` - A pointer to a variable that receives the size, in bytes, of the data returned.
///
/// # Returns
///
/// * `i32` - The NTSTATUS code of the operation.
pub fn nt_query_information_token(
    token_handle: HANDLE,
    token_information_class: ULONG,
    token_information: *mut c_void,
    token_information_length: ULONG,
    return_length: *mut ULONG,
) -> i32 {
    let nt_query_information_token_table = &*NT_QUERY_INFORMATION_TOKEN_SYSCALL;
    run_syscall!(
        nt_query_information_token_table.number,
        nt_query_information_token_table.address as usize,
        token_handle,
        token_information_class,
        token_information,
        token_information_length,
        return_length
    )
}

/// Wrapper for the NtAdjustPrivilegesToken syscall.
///
/// # Arguments
///
/// * `token_handle` - The handle of the token to be adjusted.
/// * `disable_all_privileges` - Boolean to disable all privileges.
/// * `new_state` - A pointer to a TOKEN_PRIVILEGES structure.
/// * `buffer_length` - The length of the buffer for previous privileges.
/// * `previous_state` - A pointer to a buffer that receives the previous state.
/// * `return_length` - A pointer to a variable that receives the length of the previous state.
///
/// # Returns
///
/// * `i32` - The NTSTATUS code of the operation.
pub fn nt_adjust_privileges_token(
    token_handle: HANDLE,
    disable_all_privileges: bool,
    new_state: *mut TokenPrivileges,
    buffer_length: ULONG,
    previous_state: *mut TokenPrivileges,
    return_length: *mut ULONG,
) -> i32 {
    let nt_adjust_privileges_token_table = &*NT_ADJUST_PRIVILEGES_TOKEN_SYSCALL;
    run_syscall!(
        nt_adjust_privileges_token_table.number,
        nt_adjust_privileges_token_table.address as usize,
        token_handle,
        disable_all_privileges as u32,
        new_state,
        buffer_length,
        previous_state,
        return_length
    )
}
