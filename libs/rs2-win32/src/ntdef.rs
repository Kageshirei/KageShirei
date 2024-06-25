use core::{
    ffi::{c_ulong, c_void},
    ptr::{self, null_mut},
};

use crate::utils::string_length_w;

// Definition of Windows types
pub type HANDLE = *mut c_void;
pub type ULONG = u32;
pub type PVOID = *mut c_void;
pub type AccessMask = ULONG;

pub type HRESULT = i32;
pub type HSTRING = *mut ::core::ffi::c_void;
pub type IUnknown = *mut ::core::ffi::c_void;
pub type IInspectable = *mut ::core::ffi::c_void;
pub type PSTR = *mut u8;
pub type PWSTR = *mut u16;
pub type PCSTR = *const u8;
pub type PCWSTR = *const u16;
pub type BSTR = *const u16;

// Windows NT Headers
pub const IMAGE_DOS_SIGNATURE: u16 = 0x5A4D; // "MZ"
pub const IMAGE_NT_SIGNATURE: u32 = 0x00004550; // "PE\0\0"

#[repr(C)]
pub struct ImageDosHeader {
    pub e_magic: u16,
    pub e_cblp: u16,
    pub e_cp: u16,
    pub e_crlc: u16,
    pub e_cparhdr: u16,
    pub e_minalloc: u16,
    pub e_maxalloc: u16,
    pub e_ss: u16,
    pub e_sp: u16,
    pub e_csum: u16,
    pub e_ip: u16,
    pub e_cs: u16,
    pub e_lfarlc: u16,
    pub e_ovno: u16,
    pub e_res: [u16; 4],
    pub e_oemid: u16,
    pub e_oeminfo: u16,
    pub e_res2: [u16; 10],
    pub e_lfanew: i32,
}

#[repr(C)]
pub struct ImageFileHeader {
    pub machine: u16,
    pub number_of_sections: u16,
    pub time_date_stamp: u32,
    pub pointer_to_symbol_table: u32,
    pub number_of_symbols: u32,
    pub size_of_optional_header: u16,
    pub characteristics: u16,
}

#[repr(C)]
pub struct ImageDataDirectory {
    pub virtual_address: u32,
    pub size: u32,
}

#[repr(C)]
pub struct ImageExportDirectory {
    pub characteristics: u32,
    pub time_date_stamp: u32,
    pub major_version: u16,
    pub minor_version: u16,
    pub name: u32,
    pub base: u32,
    pub number_of_functions: u32,
    pub number_of_names: u32,
    pub address_of_functions: u32,
    pub address_of_names: u32,
    pub address_of_name_ordinals: u32,
}

#[cfg(target_arch = "x86_64")]
#[repr(C)]
pub struct ImageNtHeaders {
    pub signature: u32,
    pub file_header: ImageFileHeader,
    pub optional_header: ImageOptionalHeader64,
}

#[cfg(target_arch = "x86_64")]
#[repr(C)]
pub struct ImageOptionalHeader64 {
    pub magic: u16,
    pub major_linker_version: u8,
    pub minor_linker_version: u8,
    pub size_of_code: u32,
    pub size_of_initialized_data: u32,
    pub size_of_uninitialized_data: u32,
    pub address_of_entry_point: u32,
    pub base_of_code: u32,
    pub image_base: u64,
    pub section_alignment: u32,
    pub file_alignment: u32,
    pub major_operating_system_version: u16,
    pub minor_operating_system_version: u16,
    pub major_image_version: u16,
    pub minor_image_version: u16,
    pub major_subsystem_version: u16,
    pub minor_subsystem_version: u16,
    pub win32_version_value: u32,
    pub size_of_image: u32,
    pub size_of_headers: u32,
    pub check_sum: u32,
    pub subsystem: u16,
    pub dll_characteristics: u16,
    pub size_of_stack_reserve: u64,
    pub size_of_stack_commit: u64,
    pub size_of_heap_reserve: u64,
    pub size_of_heap_commit: u64,
    pub loader_flags: u32,
    pub number_of_rva_and_sizes: u32,
    pub data_directory: [ImageDataDirectory; 16],
}

#[cfg(target_arch = "x86")]
#[repr(C)]
pub struct ImageOptionalHeader32 {
    pub magic: u16,
    pub major_linker_version: u8,
    pub minor_linker_version: u8,
    pub size_of_code: u32,
    pub size_of_initialized_data: u32,
    pub size_of_uninitialized_data: u32,
    pub address_of_entry_point: u32,
    pub base_of_code: u32,
    pub base_of_data: u32,
    pub image_base: u32,
    pub section_alignment: u32,
    pub file_alignment: u32,
    pub major_operating_system_version: u16,
    pub minor_operating_system_version: u16,
    pub major_image_version: u16,
    pub minor_image_version: u16,
    pub major_subsystem_version: u16,
    pub minor_subsystem_version: u16,
    pub win32_version_value: u32,
    pub size_of_image: u32,
    pub size_of_headers: u32,
    pub check_sum: u32,
    pub subsystem: u16,
    pub dll_characteristics: u16,
    pub size_of_stack_reserve: u32,
    pub size_of_stack_commit: u32,
    pub size_of_heap_reserve: u32,
    pub size_of_heap_commit: u32,
    pub loader_flags: u32,
    pub number_of_rva_and_sizes: u32,
    pub data_directory: [ImageDataDirectory; 16],
}

#[cfg(target_arch = "x86")]
#[repr(C)]
pub struct ImageNtHeaders {
    pub signature: u32,
    pub file_header: ImageFileHeader,
    pub optional_header: ImageOptionalHeader32,
}

// Definition of LIST_ENTRY
#[repr(C)]
#[derive(Copy, Clone)]
pub struct ListEntry {
    pub flink: *mut ListEntry,
    pub blink: *mut ListEntry,
}

// Definition of UNICODE_STRING
#[repr(C)]
#[derive(Copy, Clone)]
pub struct UnicodeString {
    pub length: u16,
    pub maximum_length: u16,
    pub buffer: *mut u16,
}

impl UnicodeString {
    pub fn new() -> Self {
        UnicodeString {
            length: 0,
            maximum_length: 0,
            buffer: ptr::null_mut(),
        }
    }

    pub fn from_str(source_string: *const u16) -> Self {
        let mut unicode_string = UnicodeString::new();
        unicode_string.init(source_string);
        unicode_string
    }

    //RtlInitUnicodeString
    pub fn init(&mut self, source_string: *const u16) {
        if !source_string.is_null() {
            let dest_size = string_length_w(source_string) * 2; // 2 bytes per u16
            self.length = dest_size as u16;
            self.maximum_length = (dest_size + 2) as u16; // 2 bytes for the null terminator
            self.buffer = source_string as *mut u16;
        } else {
            self.length = 0;
            self.maximum_length = 0;
            self.buffer = ptr::null_mut();
        }
    }
}

#[repr(C)]
pub struct ClientId {
    pub unique_process: HANDLE,
    pub unique_thread: HANDLE,
}

impl ClientId {
    pub fn new() -> Self {
        ClientId {
            unique_process: ptr::null_mut(),
            unique_thread: ptr::null_mut(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct SectionPointer {
    pub section_pointer: PVOID,
    pub check_sum: c_ulong,
}

#[repr(C)]
pub union HashLinksOrSectionPointer {
    pub hash_links: ListEntry,
    pub section_pointer: SectionPointer,
}

#[repr(C)]
pub union TimeDateStampOrLoadedImports {
    pub time_date_stamp: c_ulong,
    pub loaded_imports: PVOID,
}

#[repr(C)]
pub struct LoaderDataTableEntry {
    pub in_load_order_links: ListEntry,
    pub in_memory_order_links: ListEntry,
    pub in_initialization_order_links: ListEntry,
    pub dll_base: PVOID,
    pub entry_point: PVOID,
    pub size_of_image: c_ulong,
    pub full_dll_name: UnicodeString,
    pub base_dll_name: UnicodeString,
    pub flags: c_ulong,
    pub load_count: i16,
    pub tls_index: i16,
    pub hash_links_or_section_pointer: HashLinksOrSectionPointer,
    pub time_date_stamp_or_loaded_imports: TimeDateStampOrLoadedImports,
    pub entry_point_activation_context: PVOID,
    pub patch_information: PVOID,
    pub forwarder_links: ListEntry,
    pub service_tag_links: ListEntry,
    pub static_links: ListEntry,
}

#[repr(C)]
pub struct PebLoaderData {
    pub length: c_ulong,
    pub initialized: c_ulong,
    pub ss_handle: PVOID,
    pub in_load_order_module_list: ListEntry,
    pub in_memory_order_module_list: ListEntry,
    pub in_initialization_order_module_list: ListEntry,
}

#[repr(C)]
pub struct PEB {
    pub inherited_address_space: bool,
    pub read_image_file_exec_options: bool,
    pub being_debugged: bool,
    pub spare: bool,
    pub mutant: HANDLE,
    pub image_base: PVOID,
    pub loader_data: *const PebLoaderData,
    pub process_parameters: *const RtlUserProcessParameters,
    pub sub_system_data: PVOID,
    pub process_heap: PVOID,
    pub fast_peb_lock: PVOID,
    pub fast_peb_lock_routine: PVOID,
    pub fast_peb_unlock_routine: PVOID,
    pub environment_update_count: c_ulong,
    pub kernel_callback_table: *const PVOID,
    pub event_log_section: PVOID,
    pub event_log: PVOID,
    pub free_list: PVOID,
    pub tls_expansion_counter: c_ulong,
    pub tls_bitmap: PVOID,
    pub tls_bitmap_bits: [c_ulong; 2],
    pub read_only_shared_memory_base: PVOID,
    pub read_only_shared_memory_heap: PVOID,
    pub read_only_static_server_data: *const PVOID,
    pub ansi_code_page_data: PVOID,
    pub oem_code_page_data: PVOID,
    pub unicode_case_table_data: PVOID,
    pub number_of_processors: c_ulong,
    pub nt_global_flag: c_ulong,
    pub spare_2: [u8; 4],
    pub critical_section_timeout: i64,
    pub heap_segment_reserve: c_ulong,
    pub heap_segment_commit: c_ulong,
    pub heap_de_commit_total_free_threshold: c_ulong,
    pub heap_de_commit_free_block_threshold: c_ulong,
    pub number_of_heaps: c_ulong,
    pub maximum_number_of_heaps: c_ulong,
    pub process_heaps: *const *const PVOID,
    pub gdi_shared_handle_table: PVOID,
    pub process_starter_helper: PVOID,
    pub gdi_dc_attribute_list: PVOID,
    pub loader_lock: PVOID,
    pub os_major_version: c_ulong,
    pub os_minor_version: c_ulong,
    pub os_build_number: c_ulong,
    pub os_platform_id: c_ulong,
    pub image_sub_system: c_ulong,
    pub image_sub_system_major_version: c_ulong,
    pub image_sub_system_minor_version: c_ulong,
    pub gdi_handle_buffer: [c_ulong; 22],
    pub post_process_init_routine: c_ulong,
    pub tls_expansion_bitmap: c_ulong,
    pub tls_expansion_bitmap_bits: [u8; 80],
    pub session_id: c_ulong,
}

#[repr(C)]
pub struct RtlUserProcessParameters {
    pub maximum_length: u32,
    pub length: u32,
    pub flags: u32,
    pub debug_flags: u32,
    pub console_handle: HANDLE,
    pub console_flags: u32,
    pub standard_input: HANDLE,
    pub standard_output: HANDLE,
    pub standard_error: HANDLE,
    pub current_directory_path: UnicodeString,
    pub current_directory_handle: HANDLE,
    pub dll_path: UnicodeString,
    pub image_path_name: UnicodeString,
    pub command_line: UnicodeString,
    pub environment: *mut c_void,
    pub starting_x: u32,
    pub starting_y: u32,
    pub count_x: u32,
    pub count_y: u32,
    pub count_chars_x: u32,
    pub count_chars_y: u32,
    pub fill_attribute: u32,
    pub window_flags: u32,
    pub show_window_flags: u32,
    pub window_title: UnicodeString,
    pub desktop_info: UnicodeString,
    pub shell_info: UnicodeString,
    pub runtime_data: UnicodeString,
    pub current_directories: [UnicodeString; 32],
    pub environment_size: u32,
    pub environment_version: u32,
    pub package_dependency_data: *mut c_void,
    pub process_group_id: u32,
    pub loader_threads: u32,
}

impl RtlUserProcessParameters {
    pub fn new() -> Self {
        RtlUserProcessParameters {
            maximum_length: 0,
            length: 0,
            flags: 0,
            debug_flags: 0,
            console_handle: null_mut(),
            console_flags: 0,
            standard_input: null_mut(),
            standard_output: null_mut(),
            standard_error: null_mut(),
            current_directory_path: UnicodeString::new(),
            current_directory_handle: null_mut(),
            dll_path: UnicodeString::new(),
            image_path_name: UnicodeString::new(),
            command_line: UnicodeString::new(),
            environment: null_mut(),
            starting_x: 0,
            starting_y: 0,
            count_x: 0,
            count_y: 0,
            count_chars_x: 0,
            count_chars_y: 0,
            fill_attribute: 0,
            window_flags: 0,
            show_window_flags: 0,
            window_title: UnicodeString::new(),
            desktop_info: UnicodeString::new(),
            shell_info: UnicodeString::new(),
            runtime_data: UnicodeString::new(),
            current_directories: [UnicodeString::new(); 32],
            environment_size: 0,
            environment_version: 0,
            package_dependency_data: null_mut(),
            process_group_id: 0,
            loader_threads: 0,
        }
    }
}

#[repr(C)]
pub struct NtTib {
    pub exception_list: *mut c_void,
    pub stack_base: *mut c_void,
    pub stack_limit: *mut c_void,
    pub sub_system_tib: *mut c_void,
    pub fiber_data: *mut c_void,
    pub arbitrary_user_pointer: *mut c_void,
    pub self_: *mut NtTib,
}

#[cfg(target_arch = "x86")]
#[repr(C)]
pub struct TEB {
    pub nt_tib: NtTib,
    pub environment_pointer: *mut c_void,
    pub client_id: ClientId,
    pub active_rpc_handle: *mut c_void,
    pub thread_local_storage_pointer: *mut c_void,
    pub process_environment_block: *mut PEB,
    pub last_error_value: u32,
    pub count_of_owned_critical_sections: u32,
    pub csr_client_thread: *mut c_void,
    pub win32_thread_info: *mut c_void,
    pub user32_reserved: [u32; 26],
    pub user_reserved: [u32; 5],
    pub wow64_reserved: *mut c_void,
    pub current_locale: u32,
    pub fp_software_status_register: u32,
    pub system_reserved1: [*mut c_void; 54],
    pub exception_code: u32,
    pub activation_context_stack_pointer: *mut c_void,
    pub spare_bytes: [u8; 36],
    pub tx_fs_context: u32,
    pub gdi_tcell_buffer: *mut c_void,
    pub gdi_prev_spare_tcell: u32,
    pub gdi_prev_spare_tx: u32,
    pub gdi_batch_count: u32,
    pub spare_stack_array: [u32; 0x200],
    pub spare1: [u8; 40],
    pub x86_spare2: [u32; 0x3d],
    pub x86_spare3: [u32; 0x3d],
    pub tx_fb_context: u32,
    pub gdi_last_spare_tcell: u32,
    pub gdi_last_spare_tx: u32,
    pub gdi_last_spare_stack_array: [u32; 0x200],
}

#[cfg(target_arch = "x86_64")]
#[repr(C)]
pub struct TEB {
    pub nt_tib: NtTib,
    pub environment_pointer: *mut c_void,
    pub client_id: ClientId,
    pub active_rpc_handle: *mut c_void,
    pub thread_local_storage_pointer: *mut c_void,
    pub process_environment_block: *mut PEB,
    pub last_error_value: u32,
    pub count_of_owned_critical_sections: u32,
    pub csr_client_thread: *mut c_void,
    pub win32_thread_info: *mut c_void,
    pub user32_reserved: [u32; 26],
    pub user_reserved: [u32; 5],
    pub wow64_reserved: *mut c_void,
    pub current_locale: u32,
    pub fp_software_status_register: u32,
    pub system_reserved1: [*mut c_void; 54],
    pub exception_code: u32,
    pub activation_context_stack_pointer: *mut c_void,
    pub spare_bytes: [u8; 24],
    pub tx_fs_context: u32,
    pub gdi_tcell_buffer: *mut c_void,
    pub gdi_prev_spare_tcell: u32,
    pub gdi_prev_spare_tx: u32,
    pub gdi_batch_count: u32,
    pub spare_stack_array: [u32; 0x200],
    pub spare1: [u8; 40],
    pub x64_spare2: [u32; 0x3d],
    pub x64_spare3: [u32; 0x3d],
    pub tx_fb_context: u32,
    pub gdi_last_spare_tcell: u32,
    pub gdi_last_spare_tx: u32,
    pub gdi_last_spare_stack_array: [u32; 0x200],
}

unsafe impl Sync for TEB {}
unsafe impl Send for TEB {}

pub const OBJ_CASE_INSENSITIVE: ULONG = 0x40;

#[repr(C)]
pub struct ObjectAttributes {
    pub length: ULONG,
    pub root_directory: HANDLE,
    pub object_name: *mut UnicodeString,
    pub attributes: ULONG,
    pub security_descriptor: PVOID,
    pub security_quality_of_service: PVOID,
}

impl ObjectAttributes {
    pub fn new() -> Self {
        ObjectAttributes {
            length: 0,
            root_directory: ptr::null_mut(),
            object_name: ptr::null_mut(),
            attributes: 0,
            security_descriptor: ptr::null_mut(),
            security_quality_of_service: ptr::null_mut(),
        }
    }

    //InitializeObjectAttributes
    pub fn initialize(
        p: &mut ObjectAttributes,
        n: *mut UnicodeString,
        a: ULONG,
        r: HANDLE,
        s: PVOID,
    ) {
        p.length = core::mem::size_of::<ObjectAttributes>() as ULONG;
        p.root_directory = r;
        p.attributes = a;
        p.object_name = n;
        p.security_descriptor = s;
        p.security_quality_of_service = ptr::null_mut();
    }
}

pub const SYSTEM_BASIC_INFORMATION: u32 = 0;

#[repr(C)]
pub struct SystemBasicInformation {
    pub reserved: [u8; 4],
    pub maximum_increment: u32,
    pub physical_page_size: u32,
    pub number_of_physical_pages: u32,
    pub lowest_physical_page_number: u32,
    pub highest_physical_page_number: u32,
    pub allocation_granularity: u32,
    pub minimum_user_mode_address: *mut c_void,
    pub maximum_user_mode_address: *mut c_void,
    pub active_processors_affinity_mask: usize,
    pub number_of_processors: u8,
}

#[repr(C)]
pub struct OSVersionInfo {
    pub dw_os_version_info_size: u32,
    pub dw_major_version: u32,
    pub dw_minor_version: u32,
    pub dw_build_number: u32,
    pub dw_platform_id: u32,
    pub sz_csd_version: [u16; 128], // WCHAR is usually represented as u16 in Rust
    pub dw_os_version_info_size_2: u32,
    pub dw_major_version_2: u32,
    pub dw_minor_version_2: u32,
    pub dw_build_number_2: u32,
    pub dw_platform_id_2: u32,
}

impl OSVersionInfo {
    pub fn new() -> Self {
        OSVersionInfo {
            dw_os_version_info_size: core::mem::size_of::<OSVersionInfo>() as u32,
            dw_major_version: 0,
            dw_minor_version: 0,
            dw_build_number: 0,
            dw_platform_id: 0,
            sz_csd_version: [0; 128],
            dw_os_version_info_size_2: core::mem::size_of::<OSVersionInfo>() as u32,
            dw_major_version_2: 0,
            dw_minor_version_2: 0,
            dw_build_number_2: 0,
            dw_platform_id_2: 0,
        }
    }
}

// NT REGISTRY DEF
pub const REG_SZ: u32 = 0x1;

pub const KEY_QUERY_VALUE: AccessMask = 0x0001;
pub const KEY_ENUMERATE_SUB_KEYS: AccessMask = 0x0008;
pub const KEY_NOTIFY: AccessMask = 0x0010;
pub const KEY_READ: AccessMask =
    (STANDARD_RIGHTS_READ | KEY_QUERY_VALUE | KEY_ENUMERATE_SUB_KEYS | KEY_NOTIFY) & (!SYNCHRONIZE);

#[repr(C)]
pub struct KeyBasicInformation {
    pub last_write_time: i64,
    pub title_index: u32,
    pub name_length: u32,
    pub name: [u16; 1],
}

#[repr(C)]
pub struct KeyValuePartialInformation {
    pub title_index: ULONG,
    pub data_type: ULONG,
    pub data_length: ULONG,
    pub data: [u8; 1],
}

#[repr(C)]
pub struct KeyValueFullInformation {
    pub title_index: u32,
    pub data_type: u32,
    pub data_offset: u32,
    pub data_length: u32,
    pub name_length: u32,
    pub name: [u16; 1],
}

// NT PROCESS DEFINES
pub const PROCESS_ALL_ACCESS: u32 = 0x1F0FFF;
pub const PROCESS_QUERY_INFORMATION: AccessMask = 0x0400;
pub const PROCESS_VM_READ: AccessMask = 0x0010;

#[repr(C)]
pub struct ProcessBasicInformation {
    pub exit_status: i32,
    pub peb_base_address: PVOID,
    pub affinity_mask: usize,
    pub base_priority: i32,
    pub unique_process_id: PVOID,
    pub inherited_from_unique_process_id: PVOID,
}

#[repr(C)]
pub struct StartupInfoA {
    pub cb: u32,
    pub lp_reserved: *mut u8,
    pub lp_desktop: *mut u8,
    pub lp_title: *mut u8,
    pub dw_x: u32,
    pub dw_y: u32,
    pub dw_x_size: u32,
    pub dw_y_size: u32,
    pub dw_x_count_chars: u32,
    pub dw_y_count_chars: u32,
    pub dw_fill_attribute: u32,
    pub dw_flags: u32,
    pub w_show_window: u16,
    pub cb_reserved2: u16,
    pub lp_reserved2: *mut u8,
    pub h_std_input: *mut c_void,
    pub h_std_output: *mut c_void,
    pub h_std_error: *mut c_void,
}

impl StartupInfoA {
    pub fn new() -> Self {
        StartupInfoA {
            cb: core::mem::size_of::<StartupInfoA>() as u32,
            lp_reserved: ptr::null_mut(),
            lp_desktop: ptr::null_mut(),
            lp_title: ptr::null_mut(),
            dw_x: 0,
            dw_y: 0,
            dw_x_size: 0,
            dw_y_size: 0,
            dw_x_count_chars: 0,
            dw_y_count_chars: 0,
            dw_fill_attribute: 0,
            dw_flags: 0,
            w_show_window: 0,
            cb_reserved2: 0,
            lp_reserved2: ptr::null_mut(),
            h_std_input: ptr::null_mut(),
            h_std_output: ptr::null_mut(),
            h_std_error: ptr::null_mut(),
        }
    }
}

#[repr(C)]
pub struct ProcessInformation {
    pub h_process: *mut c_void,
    pub h_thread: *mut c_void,
    pub dw_process_id: u32,
    pub dw_thread_id: u32,
}

impl ProcessInformation {
    pub fn new() -> Self {
        ProcessInformation {
            h_process: ptr::null_mut(),
            h_thread: ptr::null_mut(),
            dw_process_id: 0,
            dw_thread_id: 0,
        }
    }
}

// Define the valid flags for process creation based on the provided mask
pub const PROCESS_CREATE_FLAGS_BREAKAWAY: u32 = 0x00000001;
pub const PROCESS_CREATE_FLAGS_NO_DEBUG_INHERIT: u32 = 0x00000002;
pub const PROCESS_CREATE_FLAGS_INHERIT_HANDLES: u32 = 0x00000004;
pub const PROCESS_CREATE_FLAGS_OVERRIDE_ADDRESS_SPACE: u32 = 0x00000008;
pub const PROCESS_CREATE_FLAGS_ALL_LARGE_PAGE_FLAGS: u32 = 0x00000010;

// Desired Access
pub const THREAD_CREATE_FLAGS_SKIP_THREAD_ATTACH: u32 = 0x00000002;
pub const THREAD_CREATE_FLAGS_HIDE_FROM_DEBUGGER: u32 = 0x00000004;
pub const THREAD_CREATE_FLAGS_LOADER_WORKER: u32 = 0x00000010;
pub const THREAD_CREATE_FLAGS_SKIP_LOADER_INIT: u32 = 0x00000020;
pub const THREAD_CREATE_FLAGS_BYPASS_PROCESS_FREEZE: u32 = 0x00000040;
pub const THREAD_CREATE_FLAGS_CREATE_SUSPENDED: u32 = 0x00000001;

pub struct TokenInformationClass(pub i32);
pub struct TokenAccessMask(pub u32);
// pub const TOKEN_QUERY: TokenAccessMask = TokenAccessMask(8u32);
pub const TOKEN_READ: TokenAccessMask = TokenAccessMask(131080u32);
// pub const TOKEN_QUERY: TokenAccessMask = TokenAccessMask(0x0008);
// pub const TOKEN_ADJUST_PRIVILEGES: TokenAccessMask = TokenAccessMask(0x0020);

pub const TOKEN_QUERY: AccessMask = 0x0008;
pub const TOKEN_ADJUST_PRIVILEGES: AccessMask = 0x0020;
pub const TOKEN_INTEGRITY_LEVEL: u32 = 25;

pub const SECURITY_MANDATORY_UNTRUSTED_RID: u32 = 0x00000000;
pub const SECURITY_MANDATORY_LOW_RID: u32 = 0x00001000;
pub const SECURITY_MANDATORY_MEDIUM_RID: u32 = 0x00002000;
pub const SECURITY_MANDATORY_HIGH_RID: u32 = 0x00003000;
pub const SECURITY_MANDATORY_SYSTEM_RID: u32 = 0x00004000;

#[repr(C)]
pub struct Sid {
    pub revision: u8,
    pub sub_authority_count: u8,
    pub identifier_authority: [u8; 6],
    pub sub_authority: [u32; 1], // Note: This is a flexible array member in C
}

#[repr(C)]
pub struct SidAndAttributes {
    pub sid: *mut Sid,
    pub attributes: u32,
}

#[repr(C)]
pub struct TokenMandatoryLabel {
    pub label: SidAndAttributes,
}

#[repr(C)]
pub struct LUID {
    pub low_part: u32,
    pub high_part: i32,
}

#[repr(C)]
pub struct LuidAndAttributes {
    pub luid: LUID,
    pub attributes: u32,
}

#[repr(C)]
pub struct TokenPrivileges {
    pub privilege_count: u32,
    pub privileges: [LuidAndAttributes; 1],
}

pub const SE_PRIVILEGE_ENABLED: u32 = 0x00000002;

#[repr(C)]
pub union IO_STATUS_BLOCK_u {
    pub status: i32,
    pub pointer: *mut c_void,
}

#[repr(C)]
pub struct IoStatusBlock {
    pub u: IO_STATUS_BLOCK_u,
    pub information: ULONG,
}

#[repr(C)]
pub enum EventType {
    NotificationEvent = 0,
    SynchronizationEvent = 1,
}

pub type PEventType = *mut EventType;

// Constant definitions
pub const STANDARD_RIGHTS_READ: AccessMask = 0x00020000;
pub const STANDARD_RIGHTS_REQUIRED: AccessMask = 0x000F0000;
pub const STANDARD_RIGHTS_EXECUTE: AccessMask = 0x00020000;
pub const STANDARD_RIGHTS_WRITE: AccessMask = 0x00020000;

pub const FILE_SHARE_READ: AccessMask = 0x00000001;
pub const FILE_SHARE_WRITE: AccessMask = 0x00000002;
pub const FILE_ANY_ACCESS: u32 = 0;
pub const FILE_DEVICE_NETWORK: u32 = 0x12;

pub const SYNCHRONIZE: AccessMask = 0x00100000;
pub const DELETE: AccessMask = 0x00010000;
pub const FILE_READ_DATA: AccessMask = 0x00000001;
pub const FILE_READ_ATTRIBUTES: AccessMask = 0x00000080;
pub const FILE_READ_EA: AccessMask = 0x00000008;
pub const READ_CONTROL: AccessMask = 0x00020000;
pub const FILE_WRITE_DATA: AccessMask = 0x00000002;
pub const FILE_WRITE_ATTRIBUTES: AccessMask = 0x00000100;
pub const FILE_WRITE_EA: AccessMask = 0x00000010;
pub const FILE_APPEND_DATA: AccessMask = 0x00000004;
pub const WRITE_DAC: AccessMask = 0x00040000;
pub const WRITE_OWNER: AccessMask = 0x00080000;
pub const FILE_EXECUTE: AccessMask = 0x00000020;

pub const FILE_GENERIC_READ: u32 =
    STANDARD_RIGHTS_READ | FILE_READ_DATA | FILE_READ_ATTRIBUTES | FILE_READ_EA | SYNCHRONIZE;

pub const FILE_GENERIC_WRITE: u32 = STANDARD_RIGHTS_WRITE
    | FILE_WRITE_DATA
    | FILE_WRITE_ATTRIBUTES
    | FILE_WRITE_EA
    | FILE_APPEND_DATA
    | SYNCHRONIZE;

pub const FILE_GENERIC_EXECUTE: u32 =
    STANDARD_RIGHTS_EXECUTE | FILE_READ_ATTRIBUTES | FILE_EXECUTE | SYNCHRONIZE;

// IoStatusBlock Information return value
pub const FILE_CREATED: u32 = 0x00000001;
pub const FILE_OPENED: u32 = 0x00000002;
pub const FILE_OVERWRITTEN: u32 = 0x00000003;
pub const FILE_SUPERSEDED: u32 = 0x00000004;
pub const FILE_EXISTS: u32 = 0x00000005;
pub const FILE_DOES_NOT_EXIST: u32 = 0x00000006;

// File attribute constant
pub const FILE_ATTRIBUTE_NORMAL: u32 = 0x00000080;

// Content disposition Constatnt
// Specifies to supersede the file if it exists, or create the file if it does not.
pub const FILE_SUPERSEDE: u32 = 0x00000000;
// Specifies to open the file if it exists. If the file does not exist, the operation fails.
pub const FILE_OPEN: u32 = 0x00000001;
// Specifies to create the file. If the file already exists, the operation fails.
pub const FILE_CREATE: u32 = 0x00000002;
// Specifies to open the file if it exists. If the file does not exist, it is created.
pub const FILE_OPEN_IF: u32 = 0x00000003;
// Specifies to open the file and overwrite it if it exists. If the file does not exist, the operation fails.
pub const FILE_OVERWRITE: u32 = 0x00000004;
// Specifies to open the file and overwrite it if it exists. If the file does not exist, it is created.
pub const FILE_OVERWRITE_IF: u32 = 0x00000005;

/// Create options constant
// The file to be created or opened is a directory file.
pub const FILE_DIRECTORY_FILE: u32 = 0x00000001;
// The file to be opened must not be a directory file, or the operation fails.
pub const FILE_NON_DIRECTORY_FILE: u32 = 0x00000040;
// Writes to the file must be transferred to the file before the write operation completes.
pub const FILE_WRITE_THROUGH: u32 = 0x00000002;
// All accesses to the file are sequential.
pub const FILE_SEQUENTIAL_ONLY: u32 = 0x00000004;
// Access to the file can be random.
pub const FILE_RANDOM_ACCESS: u32 = 0x00000008;
// The file cannot be cached or buffered.
pub const FILE_NO_INTERMEDIATE_BUFFERING: u32 = 0x00000010;
// All operations on the file are performed synchronously and are subject to alert termination.
pub const FILE_SYNCHRONOUS_IO_ALERT: u32 = 0x00000010;
// All operations on the file are performed synchronously without alert termination.
pub const FILE_SYNCHRONOUS_IO_NONALERT: u32 = 0x00000020;
// A tree connection for this file is created to open it through the network.
pub const FILE_CREATE_TREE_CONNECTION: u32 = 0x00000080;
// If the existing file has extended attributes, the caller does not understand, the request fails.
pub const FILE_NO_EA_KNOWLEDGE: u32 = 0x00000200;
// Opens a file with a reparse point and bypasses the normal reparse point processing.
pub const FILE_OPEN_REPARSE_POINT: u32 = 0x00200000;
// Deletes the file when the last handle to it is closed.
pub const FILE_DELETE_ON_CLOSE: u32 = 0x00001000;
// The file name includes the 8-byte file reference number for the file.
pub const FILE_OPEN_BY_FILE_ID: u32 = 0x00002000;
// Opens the file for backup intent.
pub const FILE_OPEN_FOR_BACKUP_INTENT: u32 = 0x00004000;
// Allows the application to request a filter opportunistic lock (oplock) to prevent share violations.
pub const FILE_RESERVE_OPFILTER: u32 = 0x00100000;
// Opens the file and requests an opportunistic lock (oplock) as a single atomic operation.
pub const FILE_OPEN_REQUIRING_OPLOCK: u32 = 0x00010000;
// Completes the operation immediately with a successful alternative status if the target file is oplocked.
pub const FILE_COMPLETE_IF_OPLOCKED: u32 = 0x00020000;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct KSystemTime {
    pub low_part: u32,
    pub high1_time: i32,
    pub high2_time: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct LargeInteger {
    pub low_part: u32,
    pub high_part: i32,
}

impl LargeInteger {
    pub fn new() -> Self {
        LargeInteger {
            high_part: 0,
            low_part: 0,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union TickCountUnion {
    pub tick_count_quad: u64,
    pub tick_count_struct: TickCountStruct,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TickCountStruct {
    pub reserved_tick_count_overlay: [u32; 3],
    pub tick_count_pad: [u32; 1],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct KUserSharedData {
    pub tick_count_low_deprecated: u32,
    pub tick_count_multiplier: u32,
    pub interrupt_time: KSystemTime,
    pub system_time: KSystemTime,
    pub time_zone_bias: KSystemTime,
    pub image_number_low: u16,
    pub image_number_high: u16,
    pub nt_system_root: [u16; 260],
    pub max_stack_trace_depth: u32,
    pub crypto_exponent: u32,
    pub time_zone_id: u32,
    pub large_page_minimum: u32,
    pub ait_sampling_value: u32,
    pub app_compat_flag: u32,
    pub rng_seed_version: u64,
    pub global_validation_runlevel: u32,
    pub time_zone_bias_stamp: i32,
    pub nt_build_number: u32,
    pub nt_product_type: u32,
    pub product_type_is_valid: u8,
    pub reserved0: [u8; 1],
    pub native_processor_architecture: u16,
    pub nt_major_version: u32,
    pub nt_minor_version: u32,
    pub processor_features: [u8; 64],
    pub reserved1: u32,
    pub reserved3: u32,
    pub time_slip: u32,
    pub alternative_architecture: u32,
    pub boot_id: u32,
    pub system_expiration_date: LargeInteger,
    pub suite_mask: u32,
    pub kd_debugger_enabled: u8,
    pub mitigation_policies: u8,
    pub cycles_per_yield: u16,
    pub active_console_id: u32,
    pub dismount_count: u32,
    pub com_plus_package: u32,
    pub last_system_rit_event_tick_count: u32,
    pub number_of_physical_pages: u32,
    pub safe_boot_mode: u8,
    pub virtualization_flags: u8,
    pub reserved12: [u8; 2],
    pub shared_data_flags: u32,
    pub data_flags_pad: [u32; 1],
    pub test_ret_instruction: u64,
    pub qpc_frequency: i64,
    pub system_call: u32,
    pub reserved2: u32,
    pub full_number_of_physical_pages: u64,
    pub system_call_pad: [u64; 1],
    pub tick_count: TickCountUnion,
    pub cookie: u32,
    pub cookie_pad: [u32; 1],
    pub console_session_foreground_process_id: i64,
    pub time_update_lock: u64,
    pub baseline_system_time_qpc: u64,
    pub baseline_interrupt_time_qpc: u64,
    pub qpc_system_time_increment: u64,
    pub qpc_interrupt_time_increment: u64,
    pub qpc_system_time_increment_shift: u8,
    pub qpc_interrupt_time_increment_shift: u8,
    pub unparked_processor_count: u16,
    pub enclave_feature_mask: [u32; 4],
    pub telemetry_coverage_round: u32,
    pub user_mode_global_logger: [u16; 16],
    pub image_file_execution_options: u32,
    pub lang_generation_count: u32,
    pub reserved4: u64,
    pub interrupt_time_bias: u64,
    pub qpc_bias: u64,
    pub active_processor_count: u32,
    pub active_group_count: u8,
    pub reserved9: u8,
    pub qpc_data: u16,
    pub time_zone_bias_effective_start: LargeInteger,
    pub time_zone_bias_effective_end: LargeInteger,
    pub xstate: [u8; 384], // Placeholder for XSTATE_CONFIGURATION
    pub feature_configuration_change_stamp: KSystemTime,
    pub spare: u32,
    pub user_pointer_auth_mask: u64,
    pub xstate_arm64: [u8; 384], // Placeholder for XSTATE_CONFIGURATION
    pub reserved10: [u32; 210],
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn test_string_length_w() {
    //     let string: [u16; 6] = [
    //         b'h' as u16,
    //         b'e' as u16,
    //         b'l' as u16,
    //         b'l' as u16,
    //         b'o' as u16,
    //         0,
    //     ];
    //     let length = string_length_w(string.as_ptr());
    //     assert_eq!(length, 5);
    // }

    #[test]
    fn test_rtl_init_unicode_string() {
        let source_string: [u16; 6] = [
            b'h' as u16,
            b'e' as u16,
            b'l' as u16,
            b'l' as u16,
            b'o' as u16,
            0,
        ];
        let mut destination_string = UnicodeString::new();

        destination_string.init(source_string.as_ptr());

        assert_eq!(destination_string.length, 5 * 2); // 5 characters, 2 bytes each
        assert_eq!(destination_string.maximum_length, (5 * 2 + 2) as u16); // 5 characters, 2 bytes each, plus 2 bytes for null terminator
        assert_eq!(
            destination_string.buffer,
            source_string.as_ptr() as *mut u16
        );
    }

    #[test]
    fn test_rtl_init_unicode_string_null_source() {
        let mut destination_string = UnicodeString::new();

        destination_string.init(ptr::null());

        assert_eq!(destination_string.length, 0);
        assert_eq!(destination_string.maximum_length, 0);
        assert_eq!(destination_string.buffer, ptr::null_mut());
    }

    #[test]
    fn test_initialize_object_attributes() {
        let mut object_name = UnicodeString {
            length: 0,
            maximum_length: 0,
            buffer: ptr::null_mut(),
        };

        let mut obj_attrs = ObjectAttributes {
            length: 0,
            root_directory: ptr::null_mut(),
            object_name: ptr::null_mut(),
            attributes: 0,
            security_descriptor: ptr::null_mut(),
            security_quality_of_service: ptr::null_mut(),
        };

        ObjectAttributes::initialize(
            &mut obj_attrs,
            &mut object_name,
            0x1234, // Some attributes
            ptr::null_mut(),
            ptr::null_mut(),
        );

        assert_eq!(
            obj_attrs.length,
            core::mem::size_of::<ObjectAttributes>() as ULONG
        );
        assert_eq!(obj_attrs.root_directory, ptr::null_mut());
        assert_eq!(obj_attrs.attributes, 0x1234);
        assert_eq!(obj_attrs.object_name, &mut object_name as *mut _);
        assert_eq!(obj_attrs.security_descriptor, ptr::null_mut());
        assert_eq!(obj_attrs.security_quality_of_service, ptr::null_mut());
    }
}
