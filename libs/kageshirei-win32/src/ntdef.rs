use core::{
    ffi::{c_long, c_ushort, c_void},
    ptr::{self, null_mut},
};

use crate::utils::string_length_w;

pub type NTSTATUS = i32;

// Definition of Windows types
pub type HANDLE = *mut c_void;
pub type PHANDLE = *mut HANDLE;
pub type LONG = c_long;
pub type ULONG = u32;
pub type PVOID = *mut c_void;
pub type AccessMask = u32;
pub type USHORT = c_ushort;
#[expect(
    non_camel_case_types,
    reason = "Windows API types use screaming snake case for types, this aliases it"
)]
pub type SIZE_T = usize;
pub type ULONGLONG = u64;
pub type LONGLONG = i64;
pub type DWORD = u32;

pub type HRESULT = i32;
pub type HSTRING = *mut c_void;
pub type IUnknown = *mut c_void;
pub type IInspectable = *mut c_void;
pub type PSTR = *mut u8;
pub type PWSTR = *mut u16;
pub type PCSTR = *const u8;
pub type PCWSTR = *const u16;
pub type BSTR = *const u16;

pub type LPCWSTR = *const u16;
pub type LPWSTR = *mut u16;
#[expect(
    non_camel_case_types,
    reason = "Windows API types use screaming snake case for types, this aliases it"
)]
pub type LPSECURITY_ATTRIBUTES = *mut SecurityAttributes;

#[expect(
    non_camel_case_types,
    reason = "Windows API types use screaming snake case for types, this aliases it"
)]
pub type ULONG_PTR = usize;

pub type DWORD64 = u64;
pub type WORD = c_ushort;

// Windows NT Headers
pub const IMAGE_DOS_SIGNATURE: u16 = 0x5a4d; // "MZ"
pub const IMAGE_NT_SIGNATURE: u32 = 0x00004550; // "PE\0\0"

#[repr(C)]
pub struct ImageDosHeader {
    pub e_magic:    u16,
    pub e_cblp:     u16,
    pub e_cp:       u16,
    pub e_crlc:     u16,
    pub e_cparhdr:  u16,
    pub e_minalloc: u16,
    pub e_maxalloc: u16,
    pub e_ss:       u16,
    pub e_sp:       u16,
    pub e_csum:     u16,
    pub e_ip:       u16,
    pub e_cs:       u16,
    pub e_lfarlc:   u16,
    pub e_ovno:     u16,
    pub e_res:      [u16; 4],
    pub e_oemid:    u16,
    pub e_oeminfo:  u16,
    pub e_res2:     [u16; 10],
    pub e_lfanew:   i32,
}

#[repr(C)]
pub struct ImageFileHeader {
    pub machine:                 u16,
    pub number_of_sections:      u16,
    pub time_date_stamp:         u32,
    pub pointer_to_symbol_table: u32,
    pub number_of_symbols:       u32,
    pub size_of_optional_header: u16,
    pub characteristics:         u16,
}

#[repr(C)]
pub struct ImageDataDirectory {
    pub virtual_address: u32,
    pub size:            u32,
}

#[repr(C)]
pub struct ImageExportDirectory {
    pub characteristics:          u32,
    pub time_date_stamp:          u32,
    pub major_version:            u16,
    pub minor_version:            u16,
    pub name:                     u32,
    pub base:                     u32,
    pub number_of_functions:      u32,
    pub number_of_names:          u32,
    pub address_of_functions:     u32,
    pub address_of_names:         u32,
    pub address_of_name_ordinals: u32,
}

#[cfg(target_arch = "x86_64")]
#[repr(C)]
pub struct ImageNtHeaders {
    pub signature:       u32,
    pub file_header:     ImageFileHeader,
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
    pub signature:       u32,
    pub file_header:     ImageFileHeader,
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
    pub length:         u16,
    pub maximum_length: u16,
    pub buffer:         *mut u16,
}

impl Default for UnicodeString {
    fn default() -> Self { Self::new() }
}

impl UnicodeString {
    pub const fn new() -> Self {
        Self {
            length:         0,
            maximum_length: 0,
            buffer:         null_mut(),
        }
    }

    // RtlInitUnicodeString
    #[expect(
        clippy::not_unsafe_ptr_arg_deref,
        reason = "The function implementation internally handles the case in which source_string is null making it a \
                  good candidate to become safe"
    )]
    pub fn init(&mut self, source_string: *const u16) {
        if !source_string.is_null() {
            // Safety: source_string is a valid pointer to a null-terminated string
            let dest_size = unsafe { string_length_w(source_string).saturating_mul(2) }; // 2 bytes per u16
            self.length = dest_size as u16;
            self.maximum_length = dest_size.saturating_add(2) as u16; // 2 bytes for the null terminator
            self.buffer = source_string as *mut u16;
        }
        else {
            self.length = 0;
            self.maximum_length = 0;
            self.buffer = null_mut();
        }
    }
}

impl From<*const u16> for UnicodeString {
    fn from(source_string: *const u16) -> Self {
        let mut unicode_string = Self::new();
        unicode_string.init(source_string);
        unicode_string
    }
}

#[repr(C)]
pub struct ClientId {
    pub unique_process: HANDLE,
    pub unique_thread:  HANDLE,
}

// Safety: ClientId is a pointer to itself, so it's safe to share across threads
unsafe impl Sync for ClientId {}
// Safety: ClientId is a pointer to itself, so it's safe to send across threads
unsafe impl Send for ClientId {}

impl Default for ClientId {
    fn default() -> Self { Self::new() }
}

impl ClientId {
    pub const fn new() -> Self {
        Self {
            unique_process: ptr::null_mut(),
            unique_thread:  ptr::null_mut(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct SectionPointer {
    pub section_pointer: PVOID,
    pub check_sum:       u32,
}

#[repr(C)]
pub union HashLinksOrSectionPointer {
    pub hash_links:      ListEntry,
    pub section_pointer: SectionPointer,
}

#[repr(C)]
pub union TimeDateStampOrLoadedImports {
    pub time_date_stamp: u32,
    pub loaded_imports:  PVOID,
}

#[repr(C)]
pub struct LoaderDataTableEntry {
    pub in_load_order_links: ListEntry,
    pub in_memory_order_links: ListEntry,
    pub in_initialization_order_links: ListEntry,
    pub dll_base: PVOID,
    pub entry_point: PVOID,
    pub size_of_image: u32,
    pub full_dll_name: UnicodeString,
    pub base_dll_name: UnicodeString,
    pub flags: u32,
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
    pub length: u32,
    pub initialized: u32,
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
    pub environment_update_count: u32,
    pub kernel_callback_table: *const PVOID,
    pub event_log_section: PVOID,
    pub event_log: PVOID,
    pub free_list: PVOID,
    pub tls_expansion_counter: u32,
    pub tls_bitmap: PVOID,
    pub tls_bitmap_bits: [u32; 2],
    pub read_only_shared_memory_base: PVOID,
    pub read_only_shared_memory_heap: PVOID,
    pub read_only_static_server_data: *const PVOID,
    pub ansi_code_page_data: PVOID,
    pub oem_code_page_data: PVOID,
    pub unicode_case_table_data: PVOID,
    pub number_of_processors: u32,
    pub nt_global_flag: u32,
    pub spare_2: [u8; 4],
    pub critical_section_timeout: i64,
    pub heap_segment_reserve: u32,
    pub heap_segment_commit: u32,
    pub heap_de_commit_total_free_threshold: u32,
    pub heap_de_commit_free_block_threshold: u32,
    pub number_of_heaps: u32,
    pub maximum_number_of_heaps: u32,
    pub process_heaps: *const *const PVOID,
    pub gdi_shared_handle_table: PVOID,
    pub process_starter_helper: PVOID,
    pub gdi_dc_attribute_list: PVOID,
    pub loader_lock: PVOID,
    pub os_major_version: u32,
    pub os_minor_version: u32,
    pub os_build_number: u32,
    pub os_platform_id: u32,
    pub image_sub_system: u32,
    pub image_sub_system_major_version: u32,
    pub image_sub_system_minor_version: u32,
    pub gdi_handle_buffer: [u32; 22],
    pub post_process_init_routine: u32,
    pub tls_expansion_bitmap: u32,
    pub tls_expansion_bitmap_bits: [u8; 80],
    pub session_id: u32,
}

pub struct CURDIR {
    pub dos_path: UnicodeString,
    pub handle:   HANDLE,
}

impl Default for CURDIR {
    fn default() -> Self { Self::new() }
}

impl CURDIR {
    pub const fn new() -> Self {
        Self {
            dos_path: UnicodeString::new(),
            handle:   null_mut(),
        }
    }
}

#[repr(C)]
pub struct RtlUserProcessParameters {
    pub maximum_length:          u32,
    pub length:                  u32,
    pub flags:                   u32,
    pub debug_flags:             u32,
    pub console_handle:          HANDLE,
    pub console_flags:           u32,
    pub standard_input:          HANDLE,
    pub standard_output:         HANDLE,
    pub standard_error:          HANDLE,
    pub current_directory:       CURDIR,
    // pub current_directory_path: UnicodeString,
    // pub current_directory_handle: HANDLE,
    pub dll_path:                UnicodeString,
    pub image_path_name:         UnicodeString,
    pub command_line:            UnicodeString,
    pub environment:             *mut c_void,
    pub starting_x:              u32,
    pub starting_y:              u32,
    pub count_x:                 u32,
    pub count_y:                 u32,
    pub count_chars_x:           u32,
    pub count_chars_y:           u32,
    pub fill_attribute:          u32,
    pub window_flags:            u32,
    pub show_window_flags:       u32,
    pub window_title:            UnicodeString,
    pub desktop_info:            UnicodeString,
    pub shell_info:              UnicodeString,
    pub runtime_data:            UnicodeString,
    pub current_directories:     [UnicodeString; 32],
    pub environment_size:        u32,
    pub environment_version:     u32,
    pub package_dependency_data: *mut c_void,
    pub process_group_id:        u32,
    pub loader_threads:          u32,
}

impl Default for RtlUserProcessParameters {
    fn default() -> Self { Self::new() }
}

impl RtlUserProcessParameters {
    pub const fn new() -> Self {
        Self {
            maximum_length:          0,
            length:                  0,
            flags:                   0,
            debug_flags:             0,
            console_handle:          null_mut(),
            console_flags:           0,
            standard_input:          null_mut(),
            standard_output:         null_mut(),
            standard_error:          null_mut(),
            current_directory:       CURDIR::new(),
            // current_directory_path: UnicodeString::new(),
            // current_directory_handle: null_mut(),
            dll_path:                UnicodeString::new(),
            image_path_name:         UnicodeString::new(),
            command_line:            UnicodeString::new(),
            environment:             null_mut(),
            starting_x:              0,
            starting_y:              0,
            count_x:                 0,
            count_y:                 0,
            count_chars_x:           0,
            count_chars_y:           0,
            fill_attribute:          0,
            window_flags:            0,
            show_window_flags:       0,
            window_title:            UnicodeString::new(),
            desktop_info:            UnicodeString::new(),
            shell_info:              UnicodeString::new(),
            runtime_data:            UnicodeString::new(),
            current_directories:     [UnicodeString::new(); 32],
            environment_size:        0,
            environment_version:     0,
            package_dependency_data: null_mut(),
            process_group_id:        0,
            loader_threads:          0,
        }
    }
}

#[repr(C)]
pub struct NtTib {
    pub exception_list:         *mut c_void,
    pub stack_base:             *mut c_void,
    pub stack_limit:            *mut c_void,
    pub sub_system_tib:         *mut c_void,
    pub fiber_data:             *mut c_void,
    pub arbitrary_user_pointer: *mut c_void,
    pub self_:                  *mut NtTib,
}

// Safety: NtTib is a pointer to itself, so it's safe to share across threads
unsafe impl Sync for NtTib {}
// Safety: NtTib is a pointer to itself, so it's safe to send across threads
unsafe impl Send for NtTib {}

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

// Safety: TEB is a pointer to itself, so it's safe to share across threads
unsafe impl Sync for TEB {}
// Safety: TEB is a pointer to itself, so it's safe to send across threads
unsafe impl Send for TEB {}

#[repr(C)]
pub struct InitialTebOldInitialTeb {
    pub old_stack_base:  PVOID,
    pub old_stack_limit: PVOID,
}

pub struct InitialTeb {
    pub old_initial_teb:       InitialTebOldInitialTeb,
    pub stack_base:            PVOID,
    pub stack_limit:           PVOID,
    pub stack_allocation_base: PVOID,
}

pub const OBJ_CASE_INSENSITIVE: ULONG = 0x40;
pub const OBJ_INHERIT: ULONG = 0x00000002;

#[repr(C)]
pub struct ObjectAttributes {
    pub length: ULONG,
    pub root_directory: HANDLE,
    pub object_name: *mut UnicodeString,
    pub attributes: ULONG,
    pub security_descriptor: PVOID,
    pub security_quality_of_service: PVOID,
}

impl Default for ObjectAttributes {
    fn default() -> Self { Self::new() }
}

impl ObjectAttributes {
    pub const fn new() -> Self {
        Self {
            length: 0,
            root_directory: ptr::null_mut(),
            object_name: ptr::null_mut(),
            attributes: 0,
            security_descriptor: ptr::null_mut(),
            security_quality_of_service: ptr::null_mut(),
        }
    }

    // InitializeObjectAttributes
    pub fn initialize(p: &mut Self, n: *mut UnicodeString, a: ULONG, r: HANDLE, s: PVOID) {
        p.length = core::mem::size_of::<Self>() as ULONG;
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
    pub dw_os_version_info_size:   u32,
    pub dw_major_version:          u32,
    pub dw_minor_version:          u32,
    pub dw_build_number:           u32,
    pub dw_platform_id:            u32,
    pub sz_csd_version:            [u16; 128], // WCHAR is usually represented as u16 in Rust
    pub dw_os_version_info_size_2: u32,
    pub dw_major_version_2:        u32,
    pub dw_minor_version_2:        u32,
    pub dw_build_number_2:         u32,
    pub dw_platform_id_2:          u32,
}

impl Default for OSVersionInfo {
    fn default() -> Self { Self::new() }
}

impl OSVersionInfo {
    pub const fn new() -> Self {
        Self {
            dw_os_version_info_size:   core::mem::size_of::<Self>() as u32,
            dw_major_version:          0,
            dw_minor_version:          0,
            dw_build_number:           0,
            dw_platform_id:            0,
            sz_csd_version:            [0; 128],
            dw_os_version_info_size_2: core::mem::size_of::<Self>() as u32,
            dw_major_version_2:        0,
            dw_minor_version_2:        0,
            dw_build_number_2:         0,
            dw_platform_id_2:          0,
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
    pub title_index:     u32,
    pub name_length:     u32,
    pub name:            [u16; 1],
}

#[repr(C)]
pub struct KeyValuePartialInformation {
    pub title_index: ULONG,
    pub data_type:   ULONG,
    pub data_length: ULONG,
    pub data:        [u8; 1],
}

#[repr(C)]
pub struct KeyValueFullInformation {
    pub title_index: u32,
    pub data_type:   u32,
    pub data_offset: u32,
    pub data_length: u32,
    pub name_length: u32,
    pub name:        [u16; 1],
}

// NT PROCESS DEFINES
pub const PROCESS_ALL_ACCESS: u32 = 0x1f0fff;
pub const PROCESS_QUERY_INFORMATION: AccessMask = 0x0400;
pub const PROCESS_VM_READ: AccessMask = 0x0010;
pub const PROCESS_CREATE_THREAD: AccessMask = 0x0002;
pub const PROCESS_VM_OPERATION: AccessMask = 0x0008;
pub const PROCESS_VM_WRITE: AccessMask = 0x0020;
pub const PROCESS_TERMINATE: AccessMask = 0x0001;
pub const PROCESS_SUSPEND_RESUME: AccessMask = 0x0800;
pub const PROCESS_SET_INFORMATION: AccessMask = 0x0200;
pub const PROCESS_SET_QUOTA: AccessMask = 0x0100;

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
    pub cb:                u32,
    pub lp_reserved:       *mut u8,
    pub lp_desktop:        *mut u8,
    pub lp_title:          *mut u8,
    pub dw_x:              u32,
    pub dw_y:              u32,
    pub dw_x_size:         u32,
    pub dw_y_size:         u32,
    pub dw_x_count_chars:  u32,
    pub dw_y_count_chars:  u32,
    pub dw_fill_attribute: u32,
    pub dw_flags:          u32,
    pub w_show_window:     u16,
    pub cb_reserved2:      u16,
    pub lp_reserved2:      *mut u8,
    pub h_std_input:       *mut c_void,
    pub h_std_output:      *mut c_void,
    pub h_std_error:       *mut c_void,
}

impl Default for StartupInfoA {
    fn default() -> Self { Self::new() }
}

impl StartupInfoA {
    pub const fn new() -> Self {
        Self {
            cb:                core::mem::size_of::<Self>() as u32,
            lp_reserved:       ptr::null_mut(),
            lp_desktop:        ptr::null_mut(),
            lp_title:          ptr::null_mut(),
            dw_x:              0,
            dw_y:              0,
            dw_x_size:         0,
            dw_y_size:         0,
            dw_x_count_chars:  0,
            dw_y_count_chars:  0,
            dw_fill_attribute: 0,
            dw_flags:          0,
            w_show_window:     0,
            cb_reserved2:      0,
            lp_reserved2:      ptr::null_mut(),
            h_std_input:       ptr::null_mut(),
            h_std_output:      ptr::null_mut(),
            h_std_error:       ptr::null_mut(),
        }
    }
}

#[repr(C)]
pub struct StartupInfoW {
    pub cb:                u32,
    pub lp_reserved:       *mut u16,
    pub lp_desktop:        *mut u16,
    pub lp_title:          *mut u16,
    pub dw_x:              u32,
    pub dw_y:              u32,
    pub dw_x_size:         u32,
    pub dw_y_size:         u32,
    pub dw_x_count_chars:  u32,
    pub dw_y_count_chars:  u32,
    pub dw_fill_attribute: u32,
    pub dw_flags:          u32,
    pub w_show_window:     u16,
    pub cb_reserved2:      u16,
    pub lp_reserved2:      *mut u8,
    pub h_std_input:       *mut c_void,
    pub h_std_output:      *mut c_void,
    pub h_std_error:       *mut c_void,
}

impl Default for StartupInfoW {
    fn default() -> Self { Self::new() }
}

impl StartupInfoW {
    pub const fn new() -> Self {
        Self {
            cb:                core::mem::size_of::<Self>() as u32,
            lp_reserved:       ptr::null_mut(),
            lp_desktop:        ptr::null_mut(),
            lp_title:          ptr::null_mut(),
            dw_x:              0,
            dw_y:              0,
            dw_x_size:         0,
            dw_y_size:         0,
            dw_x_count_chars:  0,
            dw_y_count_chars:  0,
            dw_fill_attribute: 0,
            dw_flags:          0,
            w_show_window:     0,
            cb_reserved2:      0,
            lp_reserved2:      ptr::null_mut(),
            h_std_input:       ptr::null_mut(),
            h_std_output:      ptr::null_mut(),
            h_std_error:       ptr::null_mut(),
        }
    }
}

#[repr(C)]
pub struct ProcessInformation {
    pub h_process:     *mut c_void,
    pub h_thread:      *mut c_void,
    pub dw_process_id: u32,
    pub dw_thread_id:  u32,
}

impl Default for ProcessInformation {
    fn default() -> Self { Self::new() }
}

impl ProcessInformation {
    pub const fn new() -> Self {
        Self {
            h_process:     ptr::null_mut(),
            h_thread:      ptr::null_mut(),
            dw_process_id: 0,
            dw_thread_id:  0,
        }
    }
}

// Define the valid flags for process creation based on the provided mask
pub const PROCESS_CREATE_FLAGS_ALL_LARGE_PAGE_FLAGS: u32 = 0x00000010;

// https://captmeelo.com/redteam/maldev/2022/05/10/ntcreateuserprocess.html
pub const PROCESS_CREATE_FLAGS_BREAKAWAY: u32 = 0x00000001; // NtCreateProcessEx & NtCreateUserProcess
pub const PROCESS_CREATE_FLAGS_NO_DEBUG_INHERIT: u32 = 0x00000002; // NtCreateProcessEx & NtCreateUserProcess
pub const PROCESS_CREATE_FLAGS_INHERIT_HANDLES: u32 = 0x00000004; // NtCreateProcessEx & NtCreateUserProcess
pub const PROCESS_CREATE_FLAGS_OVERRIDE_ADDRESS_SPACE: u32 = 0x00000008; // NtCreateProcessEx only
pub const PROCESS_CREATE_FLAGS_LARGE_PAGES: u32 = 0x00000010; // NtCreateProcessEx only, requires SeLockMemory
pub const PROCESS_CREATE_FLAGS_LARGE_PAGE_SYSTEM_DLL: u32 = 0x00000020; // NtCreateProcessEx only, requires SeLockMemory
pub const PROCESS_CREATE_FLAGS_PROTECTED_PROCESS: u32 = 0x00000040; // NtCreateUserProcess only
pub const PROCESS_CREATE_FLAGS_CREATE_SESSION: u32 = 0x00000080; // NtCreateProcessEx & NtCreateUserProcess, requires SeLoadDriver
pub const PROCESS_CREATE_FLAGS_INHERIT_FROM_PARENT: u32 = 0x00000100; // NtCreateProcessEx & NtCreateUserProcess
pub const PROCESS_CREATE_FLAGS_SUSPENDED: u32 = 0x00000200; // NtCreateProcessEx & NtCreateUserProcess
pub const PROCESS_CREATE_FLAGS_FORCE_BREAKAWAY: u32 = 0x00000400; // NtCreateProcessEx & NtCreateUserProcess, requires SeTcb
pub const PROCESS_CREATE_FLAGS_MINIMAL_PROCESS: u32 = 0x00000800; // NtCreateProcessEx only
pub const PROCESS_CREATE_FLAGS_RELEASE_SECTION: u32 = 0x00001000; // NtCreateProcessEx & NtCreateUserProcess
pub const PROCESS_CREATE_FLAGS_CLONE_MINIMAL: u32 = 0x00002000; // NtCreateProcessEx only
pub const PROCESS_CREATE_FLAGS_CLONE_MINIMAL_REDUCED_COMMIT: u32 = 0x00004000; //
pub const PROCESS_CREATE_FLAGS_AUXILIARY_PROCESS: u32 = 0x00008000; // NtCreateProcessEx & NtCreateUserProcess, requires SeTcb
pub const PROCESS_CREATE_FLAGS_CREATE_STORE: u32 = 0x00020000; // NtCreateProcessEx only
pub const PROCESS_CREATE_FLAGS_USE_PROTECTED_ENVIRONMENT: u32 = 0x00040000; // NtCreateProcessEx & NtCreateUserProcess

pub const THREAD_CREATE_FLAGS_CREATE_SUSPENDED: u32 = 0x00000001; // NtCreateUserProcess & NtCreateThreadEx
pub const THREAD_CREATE_FLAGS_SKIP_THREAD_ATTACH: u32 = 0x00000002; // NtCreateThreadEx only
pub const THREAD_CREATE_FLAGS_HIDE_FROM_DEBUGGER: u32 = 0x00000004; // NtCreateThreadEx only
pub const THREAD_CREATE_FLAGS_LOADER_WORKER: u32 = 0x00000010; // NtCreateThreadEx only
pub const THREAD_CREATE_FLAGS_SKIP_LOADER_INIT: u32 = 0x00000020; // NtCreateThreadEx only
pub const THREAD_CREATE_FLAGS_BYPASS_PROCESS_FREEZE: u32 = 0x00000040; // NtCreateThreadEx only
pub const THREAD_CREATE_FLAGS_INITIAL_THREAD: u32 = 0x00000080; // ?

/// Enables the use of standard input, output, and error handles.
pub const STARTF_USESTDHANDLES: u32 = 0x00000100;
/// Allows control over the window display using the wShowWindow member.
pub const STARTF_USESHOWWINDOW: u32 = 0x00000001;
/// Creates the process without displaying a window.
pub const CREATE_NO_WINDOW: u32 = 0x08000000;

pub struct TokenInformationClass(pub i32);
pub struct TokenAccessMask(pub u32);
// pub const TOKEN_QUERY: TokenAccessMask = TokenAccessMask(8u32);
pub const TOKEN_READ: TokenAccessMask = TokenAccessMask(0x0002_0008u32);
// pub const TOKEN_QUERY: TokenAccessMask = TokenAccessMask(0x0008);
// pub const TOKEN_ADJUST_PRIVILEGES: TokenAccessMask = TokenAccessMask(0x0020);

pub const TOKEN_QUERY: AccessMask = 0x0008;
pub const TOKEN_ADJUST_PRIVILEGES: AccessMask = 0x0020;
pub const TOKEN_INTEGRITY_LEVEL: u32 = 25;

pub const SECURITY_MANDATORY_UNTRUSTED_RID: u32 = 0x0000_0000;
pub const SECURITY_MANDATORY_LOW_RID: u32 = 0x0000_1000;
pub const SECURITY_MANDATORY_MEDIUM_RID: u32 = 0x0000_2000;
pub const SECURITY_MANDATORY_HIGH_RID: u32 = 0x0000_3000;
pub const SECURITY_MANDATORY_SYSTEM_RID: u32 = 0x0000_4000;

#[repr(C)]
pub struct Sid {
    pub revision:             u8,
    pub sub_authority_count:  u8,
    pub identifier_authority: [u8; 6],
    pub sub_authority:        [u32; 1], // Note: This is a flexible array member in C
}

#[repr(C)]
pub struct SidAndAttributes {
    pub sid:        *mut Sid,
    pub attributes: u32,
}

#[repr(C)]
pub struct TokenMandatoryLabel {
    pub label: SidAndAttributes,
}

#[repr(C)]
pub struct LUID {
    pub low_part:  u32,
    pub high_part: i32,
}

#[repr(C)]
pub struct LuidAndAttributes {
    pub luid:       LUID,
    pub attributes: u32,
}

#[repr(C)]
pub struct TokenPrivileges {
    pub privilege_count: u32,
    pub privileges:      [LuidAndAttributes; 1],
}

pub const SE_PRIVILEGE_ENABLED: u32 = 0x00000002;

#[repr(C)]
pub union IO_STATUS_BLOCK_u {
    pub status:  i32,
    pub pointer: *mut c_void,
}

#[repr(C)]
pub struct IoStatusBlock {
    pub u:           IO_STATUS_BLOCK_u,
    pub information: ULONG,
}

impl Default for IoStatusBlock {
    fn default() -> Self { Self::new() }
}

impl IoStatusBlock {
    /// Creates a new `IoStatusBlock` with default values.
    ///
    /// # Returns
    ///
    /// A new instance of `IoStatusBlock` with default initialization.
    pub const fn new() -> Self {
        Self {
            u:           IO_STATUS_BLOCK_u {
                status: 0,
            },
            information: 0,
        }
    }
}

#[repr(C)]
pub enum EventType {
    NotificationEvent    = 0,
    SynchronizationEvent = 1,
}

pub type PEventType = *mut EventType;

/// Standard rights required to read a file.
pub const STANDARD_RIGHTS_READ: AccessMask = 0x00020000;
/// Standard rights required for most file operations.
pub const STANDARD_RIGHTS_REQUIRED: AccessMask = 0x000f0000;
/// Standard rights required to execute a file.
pub const STANDARD_RIGHTS_EXECUTE: AccessMask = 0x00020000;
/// Standard rights required to write to a file.
pub const STANDARD_RIGHTS_WRITE: AccessMask = 0x00020000;

/// Allows shared read access to a file.
pub const FILE_SHARE_READ: AccessMask = 0x00000001;
/// Allows shared write access to a file.
pub const FILE_SHARE_WRITE: AccessMask = 0x00000002;
/// Represents no specific access requirements.
pub const FILE_ANY_ACCESS: u32 = 0;
/// Represents a network file device.
pub const FILE_DEVICE_NETWORK: u32 = 0x12;

/// Allows synchronization access to a file.
pub const SYNCHRONIZE: AccessMask = 0x00100000;

/// Allows delete access to a file.
pub const DELETE: AccessMask = 0x00010000;
/// Allows read access to a file's data.
pub const FILE_READ_DATA: AccessMask = 0x00000001;
/// Allows read access to a file's attributes.
pub const FILE_READ_ATTRIBUTES: AccessMask = 0x00000080;
/// Allows read access to a file's extended attributes.
pub const FILE_READ_EA: AccessMask = 0x00000008;
/// Allows read access to the file's security descriptor.
pub const READ_CONTROL: AccessMask = 0x00020000;
/// Allows write access to a file's data.
pub const FILE_WRITE_DATA: AccessMask = 0x00000002;
/// Allows write access to a file's attributes.
pub const FILE_WRITE_ATTRIBUTES: AccessMask = 0x00000100;
/// Allows write access to a file's extended attributes.
pub const FILE_WRITE_EA: AccessMask = 0x00000010;
/// Allows append access to a file's data.
pub const FILE_APPEND_DATA: AccessMask = 0x00000004;
/// Allows write access to a file's discretionary access control list (DACL).
pub const WRITE_DAC: AccessMask = 0x00040000;
/// Allows write access to change the owner of a file.
pub const WRITE_OWNER: AccessMask = 0x00080000;
/// Allows execute access to a file.
pub const FILE_EXECUTE: AccessMask = 0x00000020;
/// Allows traversal access to a directory.
pub const FILE_TRAVERSE: AccessMask = 0x00000020;

/// Generic read access mask for a file.
pub const FILE_GENERIC_READ: u32 =
    STANDARD_RIGHTS_READ | FILE_READ_DATA | FILE_READ_ATTRIBUTES | FILE_READ_EA | SYNCHRONIZE;

/// Generic write access mask for a file.
pub const FILE_GENERIC_WRITE: u32 =
    STANDARD_RIGHTS_WRITE | FILE_WRITE_DATA | FILE_WRITE_ATTRIBUTES | FILE_WRITE_EA | FILE_APPEND_DATA | SYNCHRONIZE;

/// Generic execute access mask for a file.
pub const FILE_GENERIC_EXECUTE: u32 = STANDARD_RIGHTS_EXECUTE | FILE_READ_ATTRIBUTES | FILE_EXECUTE | SYNCHRONIZE;

/// IoStatusBlock return value indicating a file was created.
pub const FILE_CREATED: u32 = 0x00000001;
/// IoStatusBlock return value indicating a file was opened.
pub const FILE_OPENED: u32 = 0x00000002;
/// IoStatusBlock return value indicating a file was overwritten.
pub const FILE_OVERWRITTEN: u32 = 0x00000003;
/// IoStatusBlock return value indicating a file was superseded.
pub const FILE_SUPERSEDED: u32 = 0x00000004;
/// IoStatusBlock return value indicating a file already exists.
pub const FILE_EXISTS: u32 = 0x00000005;
/// IoStatusBlock return value indicating a file does not exist.
pub const FILE_DOES_NOT_EXIST: u32 = 0x00000006;

/// Normal file attribute constant.
pub const FILE_ATTRIBUTE_NORMAL: u32 = 0x00000080;

/// Disposition value specifying to supersede an existing file or create a new one.
pub const FILE_SUPERSEDE: u32 = 0x00000000;
/// Disposition value specifying to open an existing file or fail if it does not exist.
pub const FILE_OPEN: u32 = 0x00000001;
/// Disposition value specifying to create a new file or fail if it already exists.
pub const FILE_CREATE: u32 = 0x00000002;
/// Disposition value specifying to open an existing file or create a new one if it does not exist.
pub const FILE_OPEN_IF: u32 = 0x00000003;
/// Disposition value specifying to overwrite an existing file or fail if it does not exist.
pub const FILE_OVERWRITE: u32 = 0x00000004;
/// Disposition value specifying to overwrite an existing file or create a new one if it does not
/// exist.
pub const FILE_OVERWRITE_IF: u32 = 0x00000005;

/// Option to indicate that the file to be created or opened is a directory.
pub const FILE_DIRECTORY_FILE: u32 = 0x00000001;
/// Option to ensure the file being opened is not a directory.
pub const FILE_NON_DIRECTORY_FILE: u32 = 0x00000040;
/// Option to ensure that all writes to the file are transferred to the file before the write
/// operation completes.
pub const FILE_WRITE_THROUGH: u32 = 0x00000002;
/// Option indicating that all file accesses must be sequential.
pub const FILE_SEQUENTIAL_ONLY: u32 = 0x00000004;
/// Option allowing random access to the file.
pub const FILE_RANDOM_ACCESS: u32 = 0x00000008;
/// Option indicating that the file cannot be cached or buffered.
pub const FILE_NO_INTERMEDIATE_BUFFERING: u32 = 0x00000010;
/// Option indicating that all file operations are performed synchronously and are subject to alert
/// termination.
pub const FILE_SYNCHRONOUS_IO_ALERT: u32 = 0x00000010;
/// Option indicating that all file operations are performed synchronously without alert
/// termination.
pub const FILE_SYNCHRONOUS_IO_NONALERT: u32 = 0x00000020;
/// Option to create a tree connection for the file through the network.
pub const FILE_CREATE_TREE_CONNECTION: u32 = 0x00000080;
/// Option to fail the operation if the file has extended attributes that the caller does not
/// understand.
pub const FILE_NO_EA_KNOWLEDGE: u32 = 0x00000200;
/// Option to open a file with a reparse point and bypass the normal reparse point processing.
pub const FILE_OPEN_REPARSE_POINT: u32 = 0x00200000;
/// Option to delete the file when the last handle to it is closed.
pub const FILE_DELETE_ON_CLOSE: u32 = 0x00001000;
/// Option indicating that the file name includes the 8-byte file reference number.
pub const FILE_OPEN_BY_FILE_ID: u32 = 0x00002000;
/// Option indicating that the file is opened for backup intent.
pub const FILE_OPEN_FOR_BACKUP_INTENT: u32 = 0x00004000;
/// Option to allow the application to request a filter opportunistic lock (oplock) to prevent share
/// violations.
pub const FILE_RESERVE_OPFILTER: u32 = 0x00100000;
/// Option to open the file and request an opportunistic lock (oplock) as a single atomic operation.
pub const FILE_OPEN_REQUIRING_OPLOCK: u32 = 0x00010000;
/// Option to complete the operation immediately with a successful alternative status if the target
/// file is oplocked.
pub const FILE_COMPLETE_IF_OPLOCKED: u32 = 0x00020000;

/// FILE_PIPE_BYTE_STREAM_TYPE specifies that the named pipe will be of a byte stream type.
/// This type of pipe transmits data as a stream of bytes.
pub const FILE_PIPE_BYTE_STREAM_TYPE: u32 = 0x00000000;

/// FILE_PIPE_BYTE_STREAM_MODE specifies that the pipe will operate in byte stream mode.
/// Data is written and read in a continuous stream of bytes.
pub const FILE_PIPE_BYTE_STREAM_MODE: u32 = 0x00000000;

/// FILE_PIPE_QUEUE_OPERATION specifies that the pipe will operate in queue operation mode.
/// Multiple instances of the pipe can be created, and the system manages a queue of connections.
pub const FILE_PIPE_QUEUE_OPERATION: u32 = 0x00000000;

pub const FILE_PIPE_COMPLETE_OPERATION: u32 = 0x00000001; // Completion mode: operations complete immediately

/// GENERIC_READ grants read access to the object. Data can be read from the file or pipe.
pub const GENERIC_READ: u32 = 0x80000000;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct KSystemTime {
    pub low_part:   u32,
    pub high1_time: i32,
    pub high2_time: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct LargeInteger {
    pub low_part:  u32,
    pub high_part: i32,
}

impl Default for LargeInteger {
    fn default() -> Self { Self::new() }
}

impl LargeInteger {
    pub const fn new() -> Self {
        Self {
            high_part: 0,
            low_part:  0,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union TickCountUnion {
    pub tick_count_quad:   u64,
    pub tick_count_struct: TickCountStruct,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TickCountStruct {
    pub reserved_tick_count_overlay: [u32; 3],
    pub tick_count_pad:              [u32; 1],
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

// START NtCreateUserProcess STRUCT
#[repr(C)]
pub struct PsCreateInfo {
    pub size:        SIZE_T,
    pub state:       PsCreateState,
    pub union_state: PsCreateInfoUnion,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub enum PsCreateState {
    PsCreateInitialState        = 0,
    PsCreateFailOnFileOpen      = 1,
    PsCreateFailOnSectionCreate = 2,
    PsCreateFailExeFormat       = 3,
    PsCreateFailMachineMismatch = 4,
    PsCreateFailExeName         = 5,
    PsCreateSuccess             = 6,
    PsCreateMaximumStates       = 7,
}

#[repr(C)]
pub union PsCreateInfoUnion {
    pub init_state:          PsCreateInitialState,
    pub file_handle:         HANDLE,
    pub dll_characteristics: USHORT,
    pub ifeo_key:            HANDLE,
    pub success_state:       PsCreateSuccess,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PsCreateInitialState {
    pub init_flags:             PsCreateInitialFlags,
    pub additional_file_access: AccessMask,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union PsCreateInitialFlags {
    pub flags:     ULONG, // 4 byte
    pub flag_bits: PsCreateInitialFlagBits,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PsCreateInitialFlagBits {
    pub bits: ULONG, // Rappresenta tutti i bit in un singolo campo da 32 bit
}

impl Default for PsCreateInitialFlagBits {
    fn default() -> Self { Self::new() }
}

impl PsCreateInitialFlagBits {
    pub const fn new() -> Self {
        let mut bits: ULONG = 0;
        bits |= 1 << 0; // WriteOutputOnExit : 1;
        bits |= 1 << 1; // DetectManifest : 1;
        bits |= 1 << 2; // IFEOSkipDebugger : 1;
        bits |= 1 << 3; // IFEODoNotPropagateKeyState : 1;
        bits |= 4 << 4; // SpareBits1 : 4;
        bits |= 8 << 8; // SpareBits2 : 8;
        bits |= 16 << 16; // ProhibitedImageCharacteristics : 16;

        Self {
            bits,
        }
    }
}

impl Default for PsCreateInitialFlags {
    fn default() -> Self {
        PsCreateInitialFlags {
            flag_bits: PsCreateInitialFlagBits::new(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union PsCreateSuccessFlags {
    pub flags:     ULONG, // 4 byte
    pub flag_bits: PsCreateSuccessFlagBits,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PsCreateSuccessFlagBits {
    pub bits: ULONG, // Rappresenta tutti i bit in un singolo campo da 32 bit
}

impl Default for PsCreateSuccessFlagBits {
    fn default() -> Self { Self::new() }
}

impl PsCreateSuccessFlagBits {
    pub const fn new() -> Self {
        let mut bits: ULONG = 0;
        bits |= 1 << 0; // ProtectedProcess : 1;
        bits |= 1 << 1; // AddressSpaceOverride : 1;
        bits |= 1 << 2; // DevOverrideEnabled : 1;
        bits |= 1 << 3; // ManifestDetected : 1;
        bits |= 1 << 4; // ProtectedProcessLight : 1;
        bits |= 3 << 5; // SpareBits1 : 3;
        bits |= 8 << 8; // SpareBits2 : 8;
        bits |= 16 << 16; // SpareBits3 : 16;

        Self {
            bits,
        }
    }
}

impl Default for PsCreateSuccessFlags {
    fn default() -> Self {
        PsCreateSuccessFlags {
            flag_bits: PsCreateSuccessFlagBits::new(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PsCreateSuccess {
    pub output_flags:                   PsCreateSuccessFlags, // 4 byte
    pub file_handle:                    *mut c_void,          // HANDLE
    pub section_handle:                 *mut c_void,          // HANDLE
    pub user_process_parameters_native: u64,
    pub user_process_parameters_wow64:  ULONG,
    pub current_parameter_flags:        ULONG,
    pub peb_address_native:             u64,
    pub peb_address_wow64:              ULONG,
    pub manifest_address:               u64,
    pub manifest_size:                  ULONG,
}

#[repr(C)]
pub struct PsAttribute {
    pub attribute:     usize,
    pub size:          usize,
    pub value:         PsAttributeValueUnion,
    pub return_length: *mut usize,
}

#[repr(C)]
pub union PsAttributeValueUnion {
    pub value:     usize,
    pub value_ptr: PVOID,
}

#[repr(C)]
pub struct PsAttributeList {
    pub total_length: usize,
    pub attributes:   [PsAttribute; 2],
}

impl PsAttribute {
    pub const fn new(attribute: usize, size: usize, value: usize, return_length: *mut usize) -> Self {
        Self {
            attribute,
            size,
            value: PsAttributeValueUnion {
                value,
            },
            return_length,
        }
    }

    pub const fn new_ptr(attribute: usize, size: usize, value_ptr: PVOID, return_length: *mut usize) -> Self {
        Self {
            attribute,
            size,
            value: PsAttributeValueUnion {
                value_ptr,
            },
            return_length,
        }
    }
}

/// Specifies the parent process attribute.
pub const PS_ATTRIBUTE_PARENT_PROCESS: ULONG_PTR = 0x00060000;
/// Specifies the debug port attribute.
pub const PS_ATTRIBUTE_DEBUG_PORT: ULONG_PTR = 0x00060001;
/// Specifies the token to assign to the process.
pub const PS_ATTRIBUTE_TOKEN: ULONG_PTR = 0x00060002;
/// Specifies the client ID attribute.
pub const PS_ATTRIBUTE_CLIENT_ID: ULONG_PTR = 0x00010003;
/// Specifies the TEB (Thread Environment Block) address attribute.
pub const PS_ATTRIBUTE_TEB_ADDRESS: ULONG_PTR = 0x00010004;
/// Specifies the image name attribute.
pub const PS_ATTRIBUTE_IMAGE_NAME: ULONG_PTR = 0x00020005;
/// Specifies the image information attribute.
pub const PS_ATTRIBUTE_IMAGE_INFO: ULONG_PTR = 0x00000006;
/// Specifies the memory reserve attribute.
pub const PS_ATTRIBUTE_MEMORY_RESERVE: ULONG_PTR = 0x00020007;
/// Specifies the priority class attribute.
pub const PS_ATTRIBUTE_PRIORITY_CLASS: ULONG_PTR = 0x00020008;
/// Specifies the error mode attribute.
pub const PS_ATTRIBUTE_ERROR_MODE: ULONG_PTR = 0x00020009;
/// Specifies the standard handle information attribute.
pub const PS_ATTRIBUTE_STD_HANDLE_INFO: ULONG_PTR = 0x0002000a;
/// Specifies the handle list attribute.
pub const PS_ATTRIBUTE_HANDLE_LIST: ULONG_PTR = 0x0002000b;
/// Specifies the group affinity attribute.
pub const PS_ATTRIBUTE_GROUP_AFFINITY: ULONG_PTR = 0x0003000c;
/// Specifies the preferred NUMA (Non-Uniform Memory Access) node attribute.
pub const PS_ATTRIBUTE_PREFERRED_NODE: ULONG_PTR = 0x0002000d;
/// Specifies the ideal processor attribute.
pub const PS_ATTRIBUTE_IDEAL_PROCESSOR: ULONG_PTR = 0x0003000e;
/// Specifies the UMS (User-Mode Scheduling) thread attribute.
pub const PS_ATTRIBUTE_UMS_THREAD: ULONG_PTR = 0x0003000f;
/// Specifies the mitigation options attribute.
pub const PS_ATTRIBUTE_MITIGATION_OPTIONS: ULONG_PTR = 0x00060010;
/// Specifies the protection level attribute.
pub const PS_ATTRIBUTE_PROTECTION_LEVEL: ULONG_PTR = 0x00060011;
/// Specifies whether the process is secure.
pub const PS_ATTRIBUTE_SECURE_PROCESS: ULONG_PTR = 0x00020012;
/// Specifies the job list attribute.
pub const PS_ATTRIBUTE_JOB_LIST: ULONG_PTR = 0x00020013;
/// Specifies the child process policy attribute.
pub const PS_ATTRIBUTE_CHILD_PROCESS_POLICY: ULONG_PTR = 0x00020014;
/// Specifies the all application packages policy attribute.
pub const PS_ATTRIBUTE_ALL_APPLICATION_PACKAGES_POLICY: ULONG_PTR = 0x00020015;
/// Specifies the Win32k filter attribute.
pub const PS_ATTRIBUTE_WIN32K_FILTER: ULONG_PTR = 0x00020016;
/// Specifies the safe open prompt origin claim attribute.
pub const PS_ATTRIBUTE_SAFE_OPEN_PROMPT_ORIGIN_CLAIM: ULONG_PTR = 0x00020017;
/// Specifies the BNO (Broad Network Objects) isolation attribute.
pub const PS_ATTRIBUTE_BNO_ISOLATION: ULONG_PTR = 0x00020018;
/// Specifies the desktop app policy attribute.
pub const PS_ATTRIBUTE_DESKTOP_APP_POLICY: ULONG_PTR = 0x00020019;

/// Indicates that the parameters passed to the process are already in a normalized form.
pub const RTL_USER_PROC_PARAMS_NORMALIZED: u32 = 0x00000001;
/// Enables user-mode profiling for the process.
pub const RTL_USER_PROC_PROFILE_USER: u32 = 0x00000002;
/// Enables kernel-mode profiling for the process.
pub const RTL_USER_PROC_PROFILE_KERNEL: u32 = 0x00000004;
/// Enables server-mode profiling for the process.
pub const RTL_USER_PROC_PROFILE_SERVER: u32 = 0x00000008;
/// Reserves 1 megabyte (MB) of virtual address space for the process.
pub const RTL_USER_PROC_RESERVE_1MB: u32 = 0x00000020;
/// Reserves 16 megabytes (MB) of virtual address space for the process.
pub const RTL_USER_PROC_RESERVE_16MB: u32 = 0x00000040;
/// Sets the process to be case-sensitive.
pub const RTL_USER_PROC_CASE_SENSITIVE: u32 = 0x00000080;
/// Disables heap decommitting for the process.
pub const RTL_USER_PROC_DISABLE_HEAP_DECOMMIT: u32 = 0x00000100;
/// Enables local DLL redirection for the process.
pub const RTL_USER_PROC_DLL_REDIRECTION_LOCAL: u32 = 0x00001000;
/// Indicates that an application manifest is present for the process.
pub const RTL_USER_PROC_APP_MANIFEST_PRESENT: u32 = 0x00002000;
/// Indicates that the image key is missing for the process.
pub const RTL_USER_PROC_IMAGE_KEY_MISSING: u32 = 0x00004000;
/// Indicates that the process has opted in to some specific behavior or feature.
pub const RTL_USER_PROC_OPTIN_PROCESS: u32 = 0x00020000;

/// Provides all possible access rights to a thread.
pub const THREAD_ALL_ACCESS: u32 = STANDARD_RIGHTS_REQUIRED | SYNCHRONIZE | 0xffff_u32;

/// Mask to extract the attribute number from a PS_ATTRIBUTE value.
pub const PS_ATTRIBUTE_NUMBER_MASK: usize = 0x0000ffff;
/// Indicates that the attribute is specific to a thread rather than a process.
pub const PS_ATTRIBUTE_THREAD: usize = 0x10000000;
/// Indicates that the attribute is an input to the process or thread creation function.
pub const PS_ATTRIBUTE_INPUT: usize = 0x20000000;
/// Indicates that the attribute is additive, meaning it adds to or modifies an existing attribute.
pub const PS_ATTRIBUTE_ADDITIVE: usize = 0x40000000;

/// This constant enables a mitigation policy that blocks non-Microsoft binaries
/// from loading into the process. The policy is always enforced.
///
/// This constant is equivalent to
/// `PROCESS_CREATION_MITIGATION_POLICY_BLOCK_NON_MICROSOFT_BINARIES_ALWAYS_ON` in the Windows API,
/// defined as `0x00000001ui64 << 44`.
pub const PROCESS_CREATION_MITIGATION_POLICY_BLOCK_NON_MICROSOFT_BINARIES_ALWAYS_ON: u64 = 0x00000001u64 << 44;

#[repr(C)]
pub struct SecurityAttributes {
    pub n_length:               u32,
    pub lp_security_descriptor: *mut c_void,
    pub b_inherit_handle:       bool,
}

// END NtCreateUserProcess STRUCT

#[cfg(test)]
mod tests {
    use super::*;

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
            length:         0,
            maximum_length: 0,
            buffer:         ptr::null_mut(),
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

pub enum SystemInformationClass {
    SystemBasicInformation,
    SystemProcessorInformation,
    SystemPerformanceInformation,
    SystemTimeOfDayInformation,
    SystemPathInformation,
    SystemProcessInformation,
    SystemCallCountInformation,
    SystemDeviceInformation,
    SystemProcessorPerformanceInformation,
    SystemFlagsInformation,
    SystemCallTimeInformation,
    SystemModuleInformation,
    SystemLocksInformation,
    SystemStackTraceInformation,
    SystemPagedPoolInformation,
    SystemNonPagedPoolInformation,
    SystemHandleInformation,
    SystemObjectInformation,
    SystemPageFileInformation,
    SystemVdmInstemulInformation,
    SystemVdmBopInformation,
    SystemFileCacheInformation,
    SystemPoolTagInformation,
    SystemInterruptInformation,
    SystemDpcBehaviorInformation,
    SystemFullMemoryInformation,
    SystemLoadGdiDriverInformation,
    SystemUnloadGdiDriverInformation,
    SystemTimeAdjustmentInformation,
    SystemSummaryMemoryInformation,
    SystemMirrorMemoryInformation,
    SystemPerformanceTraceInformation,
    SystemObsolete0,
    SystemExceptionInformation,
    SystemCrashDumpStateInformation,
    SystemKernelDebuggerInformation,
    SystemContextSwitchInformation,
    SystemRegistryQuotaInformation,
    SystemExtendServiceTableInformation,
    SystemPrioritySeperation,
    SystemVerifierAddDriverInformation,
    SystemVerifierRemoveDriverInformation,
    SystemProcessorIdleInformation,
    SystemLegacyDriverInformation,
    SystemCurrentTimeZoneInformation,
    SystemLookasideInformation,
    SystemTimeSlipNotification,
    SystemSessionCreate,
    SystemSessionDetach,
    SystemSessionInformation,
    SystemRangeStartInformation,
    SystemVerifierInformation,
    SystemVerifierThunkExtend,
    SystemSessionProcessInformation,
    SystemLoadGdiDriverInSystemSpace,
    SystemNumaProcessorMap,
    SystemPrefetcherInformation,
    SystemExtendedProcessInformation,
    SystemRecommendedSharedDataAlignment,
    SystemComPlusPackage,
    SystemNumaAvailableMemory,
    SystemProcessorPowerInformation,
    SystemEmulationBasicInformation,     // WOW64
    SystemEmulationProcessorInformation, // WOW64
    SystemExtendedHandleInformation,
    SystemLostDelayedWriteInformation,
    SystemBigPoolInformation,
    SystemSessionPoolTagInformation,
    SystemSessionMappedViewInformation,
    SystemHotpatchInformation,
    SystemObjectSecurityMode,
    SystemWatchdogTimerHandler,
    SystemWatchdogTimerInformation,
    SystemLogicalProcessorInformation,
    SystemWow64SharedInformation,
    SystemRegisterFirmwareTableInformationHandler,
    SystemFirmwareTableInformation,
    SystemModuleInformationEx,
    SystemVerifierTriageInformation,
    SystemSuperfetchInformation,
    SystemMemoryListInformation,
    SystemFileCacheInformationEx,
    SystemThreadPriorityClientIdInformation,
    SystemProcessorIdleCycleTimeInformation,
    SystemVerifierCancellationInformation,
    SystemProcessorPowerInformationEx,
    SystemRefTraceInformation,
    SystemSpecialPoolInformation,
    SystemProcessIdInformation,
    SystemErrorPortInformation,
    SystemBootEnvironmentInformation,
    SystemHypervisorInformation,
    SystemVerifierInformationEx,
    SystemTimeZoneInformation,
    SystemImageFileExecutionOptionsInformation,
    SystemCoverageInformation,
    SystemPrefetchPatchInformation,
    SystemVerifierFaultsInformation,
    SystemSystemPartitionInformation,
    SystemSystemDiskInformation,
    SystemProcessorPerformanceDistribution,
    SystemNumaProximityNodeInformation,
    SystemDynamicTimeZoneInformation,
    SystemCodeIntegrityInformation,
    SystemProcessorMicrocodeUpdateInformation,
    SystemProcessorBrandString,
    SystemVirtualAddressInformation,
    MaxSystemInfoClass,
}

#[repr(C)]
pub struct SystemProcessInformation {
    pub next_entry_offset: ULONG,
    pub number_of_threads: ULONG,
    pub working_set_private_size: LargeInteger,
    pub hard_fault_count: ULONG,
    pub number_of_threads_high_watermark: ULONG,
    pub cycle_time: ULONGLONG,
    pub create_time: LargeInteger,
    pub user_time: LargeInteger,
    pub kernel_time: LargeInteger,
    pub image_name: UnicodeString,
    pub base_priority: i32,
    pub unique_process_id: HANDLE,
    pub inherited_from_unique_process_id: HANDLE,
    pub handle_count: ULONG,
    pub session_id: ULONG,
    pub unique_process_key: ULONG_PTR,
    pub peak_virtual_size: SIZE_T,
    pub virtual_size: SIZE_T,
    pub page_fault_count: ULONG,
    pub peak_working_set_size: SIZE_T,
    pub working_set_size: SIZE_T,
    pub quota_peak_paged_pool_usage: SIZE_T,
    pub quota_paged_pool_usage: SIZE_T,
    pub quota_peak_non_paged_pool_usage: SIZE_T,
    pub quota_non_paged_pool_usage: SIZE_T,
    pub pagefile_usage: SIZE_T,
    pub peak_pagefile_usage: SIZE_T,
    pub private_page_count: SIZE_T,
    pub read_operation_count: LargeInteger,
    pub write_operation_count: LargeInteger,
    pub other_operation_count: LargeInteger,
    pub read_transfer_count: LargeInteger,
    pub write_transfer_count: LargeInteger,
    pub other_transfer_count: LargeInteger,
    pub threads: [SystemThreadInformation; 1],
}

#[repr(C)]
pub struct SystemThreadInformation {
    pub kernel_time:      LargeInteger,
    pub user_time:        LargeInteger,
    pub create_time:      LargeInteger,
    pub wait_time:        ULONG,
    pub start_address:    PVOID,
    pub client_id:        ClientId,
    pub priority:         c_long,
    pub base_priority:    c_long,
    pub context_switches: ULONG,
    pub thread_state:     u32,
    pub wait_reason:      u32,
}

#[repr(C)]
pub struct SystemProcessInformation2 {
    pub next_entry_offset: u32,
    pub number_of_threads: u32,
    pub spare_li1: LargeInteger,
    pub spare_li2: LargeInteger,
    pub spare_li3: LargeInteger,
    pub create_time: LargeInteger,
    pub user_time: LargeInteger,
    pub kernel_time: LargeInteger,
    pub image_name: UnicodeString,
    pub base_priority: i32,
    pub unique_process_id: HANDLE,
    pub inherited_from_unique_process_id: HANDLE,
    pub handle_count: u32,
    pub session_id: u32,
    pub page_directory_base: usize,
    pub peak_virtual_size: usize,
    pub virtual_size: usize,
    pub page_fault_count: u32,
    pub peak_working_set_size: usize,
    pub working_set_size: usize,
    pub quota_peak_paged_pool_usage: usize,
    pub quota_paged_pool_usage: usize,
    pub quota_peak_non_paged_pool_usage: usize,
    pub quota_non_paged_pool_usage: usize,
    pub pagefile_usage: usize,
    pub peak_pagefile_usage: usize,
    pub private_page_count: usize,
    pub read_operation_count: LargeInteger,
    pub write_operation_count: LargeInteger,
    pub other_operation_count: LargeInteger,
    pub read_transfer_count: LargeInteger,
    pub write_transfer_count: LargeInteger,
    pub other_transfer_count: LargeInteger,
}

#[repr(C)]
#[expect(
    non_snake_case,
    reason = "The CONTEXT structure is a Windows API structure"
)]
pub struct M128A {
    pub Low:  ULONGLONG,
    pub High: LONGLONG,
}

#[repr(C)]
#[expect(
    non_snake_case,
    reason = "The CONTEXT structure is a Windows API structure"
)]
pub struct CONTEXT {
    pub P1Home:               DWORD64,
    pub P2Home:               DWORD64,
    pub P3Home:               DWORD64,
    pub P4Home:               DWORD64,
    pub P5Home:               DWORD64,
    pub P6Home:               DWORD64,
    pub ContextFlags:         DWORD,
    pub MxCsr:                DWORD,
    pub SegCs:                WORD,
    pub SegDs:                WORD,
    pub SegEs:                WORD,
    pub SegFs:                WORD,
    pub SegGs:                WORD,
    pub SegSs:                WORD,
    pub EFlags:               DWORD,
    pub Dr0:                  DWORD64,
    pub Dr1:                  DWORD64,
    pub Dr2:                  DWORD64,
    pub Dr3:                  DWORD64,
    pub Dr6:                  DWORD64,
    pub Dr7:                  DWORD64,
    pub Rax:                  DWORD64,
    pub Rcx:                  DWORD64,
    pub Rdx:                  DWORD64,
    pub Rbx:                  DWORD64,
    pub Rsp:                  DWORD64,
    pub Rbp:                  DWORD64,
    pub Rsi:                  DWORD64,
    pub Rdi:                  DWORD64,
    pub R8:                   DWORD64,
    pub R9:                   DWORD64,
    pub R10:                  DWORD64,
    pub R11:                  DWORD64,
    pub R12:                  DWORD64,
    pub R13:                  DWORD64,
    pub R14:                  DWORD64,
    pub R15:                  DWORD64,
    pub Rip:                  DWORD64,
    pub u:                    [u64; 64],
    pub VectorRegister:       [M128A; 26],
    pub VectorControl:        DWORD64,
    pub DebugControl:         DWORD64,
    pub LastBranchToRip:      DWORD64,
    pub LastBranchFromRip:    DWORD64,
    pub LastExceptionToRip:   DWORD64,
    pub LastExceptionFromRip: DWORD64,
}

pub const HEAP_NO_SERIALIZE: DWORD = 0x0000_0001;
pub const HEAP_GROWABLE: DWORD = 0x0000_0002;
pub const HEAP_GENERATE_EXCEPTIONS: DWORD = 0x0000_0004;
pub const HEAP_ZERO_MEMORY: DWORD = 0x0000_0008;
pub const HEAP_REALLOC_IN_PLACE_ONLY: DWORD = 0x0000_0010;
pub const HEAP_TAIL_CHECKING_ENABLED: DWORD = 0x0000_0020;
pub const HEAP_FREE_CHECKING_ENABLED: DWORD = 0x0000_0040;
pub const HEAP_DISABLE_COALESCE_ON_FREE: DWORD = 0x0000_0080;
pub const HEAP_CREATE_ALIGN_16: DWORD = 0x0001_0000;
pub const HEAP_CREATE_ENABLE_TRACING: DWORD = 0x0002_0000;
pub const HEAP_CREATE_ENABLE_EXECUTE: DWORD = 0x0004_0000;
pub const HEAP_MAXIMUM_TAG: DWORD = 0x0fff;
pub const HEAP_PSEUDO_TAG_FLAG: DWORD = 0x8000;
pub const HEAP_TAG_SHIFT: usize = 18;
pub const HEAP_CREATE_SEGMENT_HEAP: DWORD = 0x00000100;
pub const HEAP_CREATE_HARDENED: DWORD = 0x00000200;

/// Represents different types of paths that can be recognized by the system.
#[derive(Clone)]
#[repr(C)]
pub enum RtlPathType {
    /// Unknown path type, typically when the input cannot be classified.
    ///
    /// Example: An empty string or an invalid path.
    RtlPathTypeUnknown,

    /// UNC (Universal Naming Convention) absolute path, used for network resources.
    ///
    /// Example: `\\Server\Share\Folder\File.txt`
    RtlPathTypeUncAbsolute,

    /// Drive absolute path, specifying a specific drive.
    ///
    /// Example: `C:\Folder\File.txt`
    RtlPathTypeDriveAbsolute,

    /// Drive relative path, where the path is relative to the current directory on a specific
    /// drive.
    ///
    /// Example: `C:Folder\File.txt`
    RtlPathTypeDriveRelative,

    /// Rooted path, which starts from the root directory but does not specify the drive.
    ///
    /// Example: `\Folder\File.txt`
    RtlPathTypeRooted,

    /// Relative path, which is relative to the current working directory.
    ///
    /// Example: `Folder\File.txt`
    RtlPathTypeRelative,

    /// Local device path, typically used to access device namespaces.
    ///
    /// Example: `\\.\PhysicalDrive0`
    RtlPathTypeLocalDevice,

    /// Root local device path, similar to local device paths but rooted.
    ///
    /// Example: `\\?\C:\Folder\File.txt`
    RtlPathTypeRootLocalDevice,
}

#[repr(C)]
pub struct RtlRelativeNameU {
    pub relative_name:        UnicodeString,
    pub containing_directory: HANDLE,
    pub cur_dir_ref:          *mut RtlpCurdirRef,
}

impl Default for RtlRelativeNameU {
    fn default() -> Self { Self::new() }
}

impl RtlRelativeNameU {
    /// Creates a new `RtlRelativeNameU` with default values.
    ///
    /// # Returns
    /// A new instance of `RtlRelativeNameU` with an empty `UnicodeString`, a null `HANDLE`,
    /// and a null pointer for `cur_dir_ref`.
    pub const fn new() -> Self {
        Self {
            relative_name:        UnicodeString::new(), // Initialize with an empty UnicodeString
            containing_directory: null_mut(),           // Set HANDLE to null
            cur_dir_ref:          null_mut(),           // Set pointer to null
        }
    }
}

pub struct RtlpCurdirRef {
    pub reference_count:  LONG,
    pub directory_handle: HANDLE,
}

pub const UNICODE_STRING_MAX_BYTES: u32 = 0xfffe;

pub enum MemoryInformationClass {
    MemoryBasicInformation,
    MemoryWorkingSetInformation,
    MemoryMappedFilenameInformation,
    MemoryRegionInformation,
    MemoryWorkingSetExInformation,
    MemorySharedCommitInformation,
    MemoryImageInformation,
    MemoryRegionInformationEx,
    MemoryPrivilegedBasicInformation,
    MemoryEnclaveImageInformation,
    MemoryBasicInformationCapped,
}
