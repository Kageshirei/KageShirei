use core::{ffi::c_void, ptr::null_mut};

use crate::ntdef::{
    ProcessInformation,
    SecurityAttributes,
    StartupInfoW,
    DWORD,
    HANDLE,
    LPCWSTR,
    LPSECURITY_ATTRIBUTES,
    LPWSTR,
};

pub type CreatePipe = unsafe extern "system" fn(
    hReadPipe: *mut HANDLE,
    hWritePipe: *mut HANDLE,
    lpPipeAttributes: *mut SecurityAttributes,
    nSize: u32,
) -> bool;

// Define the WriteFile function type
pub type WriteFile = unsafe extern "system" fn(
    hFile: *mut c_void,
    lpBuffer: *const c_void,
    nNumberOfBytesToWrite: u32,
    lpNumberOfBytesWritten: *mut u32,
    lpOverlapped: *mut c_void,
) -> i32;

// Define the ReadFile function type
pub type ReadFile = unsafe extern "system" fn(
    hFile: *mut c_void,
    lpBuffer: *mut c_void,
    nNumberOfBytesToRead: u32,
    lpNumberOfBytesRead: *mut u32,
    lpOverlapped: *mut c_void,
) -> i32;

pub type CreateProcessW = unsafe extern "system" fn(
    lpApplicationName: LPCWSTR,
    lpCommandLine: LPWSTR,
    lpProcessAttributes: LPSECURITY_ATTRIBUTES,
    lpThreadAttributes: LPSECURITY_ATTRIBUTES,
    bInheritHandles: bool,
    dwCreationFlags: DWORD,
    lpEnvironment: *mut c_void,
    lpCurrentDirectory: LPCWSTR,
    lpStartupInfo: *mut StartupInfoW,
    lpProcessInformation: *mut ProcessInformation,
) -> bool;

pub type GetConsoleWindow = unsafe extern "system" fn() -> *mut c_void;

pub struct Kernel32 {
    pub module_base:        *mut u8,
    pub create_pipe:        CreatePipe,
    pub write_file:         WriteFile,
    pub read_file:          ReadFile,
    pub create_process_w:   CreateProcessW,
    pub get_console_window: GetConsoleWindow,
}

impl Default for Kernel32 {
    fn default() -> Self { Self::new() }
}

impl Kernel32 {
    pub fn new() -> Self {
        Self {
            module_base:        null_mut(),
            create_pipe:        unsafe { core::mem::transmute(null_mut::<c_void>()) },
            write_file:         unsafe { core::mem::transmute(null_mut::<c_void>()) },
            read_file:          unsafe { core::mem::transmute(null_mut::<c_void>()) },
            create_process_w:   unsafe { core::mem::transmute(null_mut::<c_void>()) },
            get_console_window: unsafe { core::mem::transmute(null_mut::<c_void>()) },
        }
    }
}

unsafe impl Sync for Kernel32 {}
unsafe impl Send for Kernel32 {}
