use crate::ntdef::{AccessMask, STANDARD_RIGHTS_READ, SYNCHRONIZE, ULONG};

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
    pub name: [u16; 1], // Flexible array member in C, single element array in Rust
}

#[repr(C)]
pub struct KeyValuePartialInformation {
    pub title_index: ULONG,
    pub data_type: ULONG,
    pub data_length: ULONG,
    pub data: [u8; 1], // Flexible array member in C, single element array in Rust
}

#[repr(C)]
pub struct KeyValueFullInformation {
    pub title_index: u32,
    pub data_type: u32,
    pub data_offset: u32,
    pub data_length: u32,
    pub name_length: u32,
    pub name: [u16; 1], // Flexible array member in C, single element array in Rust
}
