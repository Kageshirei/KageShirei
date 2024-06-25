#![no_std]

extern crate alloc;

use alloc::{string::String, vec::Vec};
use core::ffi::c_void;
use core::{cell::UnsafeCell, ptr::null_mut};
use spin::Mutex;

use rs2_win32::ntapi::NtDll;
use rs2_win32::ntdef::TEB;

/// Represents a session containing connection information.
pub struct Session {
    /// Indicates if the session is connected.
    pub connected: bool,
    /// Unique identifier for the session.
    pub id: String,
    /// Process ID.
    pub pid: i64,
    /// Parent Process ID.
    pub ppid: i64,
    /// Thread ID.
    pub tid: i64,
    /// Encryptor
    pub encryptor_ptr: *mut c_void,
    /// Protocol
    pub protocol_ptr: *mut c_void,
}

impl Session {
    /// Creates a new, disconnected session with default values.
    pub fn new() -> Self {
        Session {
            connected: false,
            id: String::new(),
            pid: 0,
            ppid: 0,
            tid: 0,
            encryptor_ptr: null_mut(),
            protocol_ptr: null_mut(),
        }
    }

    /// Set the generic pointer to data.
    pub fn set_encryptor_ptr(&mut self, encryptor: *mut c_void) {
        self.encryptor_ptr = encryptor;
    }

    /// Set the generic pointer to data.
    pub fn set_protocol_ptr(&mut self, protocol: *mut c_void) {
        self.protocol_ptr = protocol;
    }
}

/// Configuration settings for the instance.
pub struct Config {
    /// The unique identifier of the agent, required to poll for tasks
    pub id: String,
    /// The unix timestamp of the kill date, if any.
    pub kill_date: Option<i64>,
    /// The working hours for the current day (unix timestamp), if any.
    pub working_hours: Option<Vec<Option<i64>>>,
    /// The agent polling interval in milliseconds.
    pub polling_interval: i64,
    /// The agent polling jitter in milliseconds.
    pub polling_jitter: i64,
}

impl Config {
    /// Creates a new configuration with default values.
    pub fn new() -> Self {
        Config {
            id: String::new(),
            kill_date: None,
            polling_interval: 0,
            polling_jitter: 0,
            working_hours: None,
        }
    }
}

/// Represents the global instance containing configuration and session information.
pub struct Instance {
    /// Pointer to the Thread Environment Block.
    pub teb: *mut TEB,
    /// NtDll instance for native API calls.
    pub ntdll: NtDll,
    /// Session information.
    pub session: Session,
    /// Configuration settings.
    pub config: Config,
    /// Pointer to Checkin Data
    pub pcheckindata: *mut c_void,
}

impl Instance {
    /// Creates a new instance with default values.
    pub fn new() -> Self {
        Instance {
            teb: null_mut(),
            ntdll: NtDll::new(),
            session: Session::new(),
            config: Config::new(),
            pcheckindata: null_mut(),
        }
    }

    /// Set the generic pointer to data.
    pub fn set_checkin_data(&mut self, data: *mut c_void) {
        self.pcheckindata = data;
    }
}

/// Global mutable instance of the agent.
pub static mut INSTANCE: Mutex<UnsafeCell<Option<Instance>>> = Mutex::new(UnsafeCell::new(None));

/// Retrieves a reference to the global instance of the agent.
///
/// # Safety
///
/// This function is unsafe because it involves mutable static data.
/// The caller must ensure no data races occur when accessing the global instance.
pub unsafe fn instance() -> &'static Instance {
    return INSTANCE.lock().get().as_ref().unwrap().as_ref().unwrap();
}

/// Retrieves a mutable reference to the global instance of the agent.
///
/// # Safety
///
/// This function is unsafe because it involves mutable static data.
/// The caller must ensure no data races occur when accessing the global instance.
pub unsafe fn instance_mut() -> &'static mut Instance {
    INSTANCE.lock().get().as_mut().unwrap().as_mut().unwrap()
}
