use core::arch::asm;
use core::ffi::{c_ulong, c_void};

use crate::ntdef::{ListEntry, UnicodeString, HANDLE, PVOID};

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

/// Find the Process Environment Block (PEB) of the current process on x86_64
#[cfg(target_arch = "x86_64")]
pub fn find_peb() -> *mut PEB {
    let peb_ptr: *mut PEB;
    unsafe {
        asm!(
        "mov {}, gs:[0x60]",
        out(reg) peb_ptr
        );
    }
    peb_ptr
}

/// Find the Process Environment Block (PEB) of the current process on x86
#[cfg(target_arch = "x86")]
pub fn find_peb() -> *const PEB {
    let peb_ptr: *const PEB;
    unsafe {
        asm!(
        "mov {}, gs:[0x30]",
        out(reg) peb_ptr
        );
    }
    peb_ptr
}

//TODO: TEB

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
