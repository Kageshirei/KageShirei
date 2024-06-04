use crate::ntdef::{AccessMask, PVOID};

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
