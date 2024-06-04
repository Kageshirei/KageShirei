use core::ffi::c_void;

use crate::ntdef::AccessMask;

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
pub struct TokenMandatoryPolicy {
    policy: u32,
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
pub const SE_DEBUG_NAME: &str = "SeDebugPrivilege";
