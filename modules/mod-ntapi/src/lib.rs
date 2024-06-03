#![no_std]
extern crate alloc;

pub mod nt_get_computer_name_ex;

use core::ffi::c_void;

use lazy_static::lazy_static;
use rs2_indirect_syscall::{init_syscall, run_syscall, NtSyscall};
use rs2_winapi::{
    ntdef::{AccessMask, ObjectAttributes, UnicodeString, ULONG},
    ntdll_config::NtdllConfig,
};

const NT_OPEN_KEY_DBJ2: usize = 0x7682ed42;
const NT_QUERY_VALUE_KEY_DBJ2: usize = 0x85967123;
const NT_CLOSE_DBJ2: usize = 0x40d6e69d;

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
