pub const NT_SUCCESS: fn(i32) -> bool = |status| status >= 0;

pub const STATUS_SUCCESS: i32 = 0;
pub const STATUS_BUFFER_OVERFLOW: i32 = 0x80000005u32 as i32;
pub const STATUS_BUFFER_TOO_SMALL: i32 = 0xC0000023u32 as i32;
pub const STATUS_OBJECT_NAME_NOT_FOUND: i32 = 0xC0000034u32 as i32;
