#![no_std]

use kageshirei_win32::ntdef::HANDLE;

pub mod nt_get_adapters_info;
pub mod nt_get_computer_name_ex;
pub mod nt_path;
pub mod nt_peb;
pub mod nt_ps_api;
pub mod nt_reg_api;
pub mod nt_time;
pub mod nt_winhttp;
pub mod nt_ws2;
pub mod utils;

extern crate alloc;

/// Returns a handle to the current process.
///
/// In Windows, `-1` is used as a special value to represent the current process handle.
/// This function mimics the behavior of the `NtCurrentProcess` macro in C.
pub fn nt_current_process() -> HANDLE { (-1isize) as HANDLE }

/// Returns a handle to the current thread.
///
/// Similar to the process handle, `-2` is used as a special value to represent the current thread
/// handle. This function mimics the behavior of the `NtCurrentThread` macro in C.
pub fn nt_current_thread() -> HANDLE { (-2isize) as HANDLE }
