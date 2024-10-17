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
    pub create_pipe:        Option<CreatePipe>,
    pub write_file:         Option<WriteFile>,
    pub read_file:          Option<ReadFile>,
    pub create_process_w:   Option<CreateProcessW>,
    pub get_console_window: Option<GetConsoleWindow>,
}

impl Default for Kernel32 {
    fn default() -> Self { Self::new() }
}

impl Kernel32 {
    pub fn new() -> Self {
        Self {
            module_base:        null_mut(),
            create_pipe:        None,
            write_file:         None,
            read_file:          None,
            create_process_w:   None,
            get_console_window: None,
        }
    }
}

// Safety: Kernel32 is a safe wrapper around the Windows API.
unsafe impl Sync for Kernel32 {}
// Safety: Kernel32 is a safe wrapper around the Windows API.
unsafe impl Send for Kernel32 {}
