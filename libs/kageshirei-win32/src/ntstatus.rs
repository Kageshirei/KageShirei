pub const NT_SUCCESS: fn(i32) -> bool = |status| status >= 0;

pub const STATUS_SUCCESS: i32 = 0;
pub const STATUS_BUFFER_OVERFLOW: i32 = 0x80000005u32 as i32;
pub const STATUS_BUFFER_TOO_SMALL: i32 = 0xc0000023u32 as i32;
pub const STATUS_OBJECT_NAME_NOT_FOUND: i32 = 0xc0000034u32 as i32;
pub const STATUS_INFO_LENGTH_MISMATCH: i32 = 0xc0000004u32 as i32;
pub const STATUS_ACCESS_VIOLATION: i32 = 0xc0000005u32 as i32;
pub const STATUS_ACCESS_DENIED: i32 = 0xc0000022u32 as i32;
pub const STATUS_INVALID_HANDLE: i32 = 0xc0000008u32 as i32;
pub const STATUS_INSUFFICIENT_RESOURCES: i32 = 0xc000009au32 as i32;
pub const STATUS_NOT_IMPLEMENTED: i32 = 0xc0000002u32 as i32;
pub const STATUS_INVALID_PARAMETER: i32 = 0xc000000du32 as i32;
pub const STATUS_CONFLICTING_ADDRESSES: i32 = 0xc0000018u32 as i32;
pub const STATUS_PRIVILEGE_NOT_HELD: i32 = 0xc0000061u32 as i32;
pub const STATUS_MEMORY_NOT_ALLOCATED: i32 = 0xc00000a0u32 as i32;
pub const STATUS_INVALID_PAGE_PROTECTION: i32 = 0xc0000045u32 as i32;
pub const STATUS_ILLEGAL_INSTRUCTION: i32 = 0xc000001du32 as i32;
pub const STATUS_INTEGER_DIVIDE_BY_ZERO: i32 = 0xc0000094u32 as i32;
pub const STATUS_DLL_NOT_FOUND: i32 = 0xc0000135u32 as i32;
pub const STATUS_DLL_INIT_FAILED: i32 = 0xc0000142u32 as i32;
pub const STATUS_NO_SUCH_FILE: i32 = 0xc000000fu32 as i32;
pub const STATUS_INVALID_DEVICE_REQUEST: i32 = 0xc0000010u32 as i32;
pub const STATUS_NOT_FOUND: i32 = 0xc0000225u32 as i32;
pub const STATUS_DATATYPE_MISALIGNMENT: i32 = 0x80000002u32 as i32;
pub const STATUS_OBJECT_NAME_INVALID: i32 = 0xc0000033u32 as i32;
pub const STATUS_NAME_TOO_LONG: i32 = 0xc0000106u32 as i32;
pub const STATUS_OBJECT_PATH_SYNTAX_BAD: i32 = 0xc000003bu32 as i32;
pub const STATUS_NO_MEMORY: i32 = 0xc0000017u32 as i32;
pub const STATUS_END_OF_FILE: i32 = 0xc0000011u32 as i32;
pub const STATUS_PENDING: i32 = 0x00000103u32 as i32;
