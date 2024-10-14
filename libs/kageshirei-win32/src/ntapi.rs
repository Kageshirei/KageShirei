use core::{
    ffi::{c_uchar, c_void},
    ptr::null_mut,
};

use kageshirei_indirect_syscall::run_syscall;

use crate::ntdef::{
    AccessMask,
    ClientId,
    InitialTeb,
    IoStatusBlock,
    LargeInteger,
    ObjectAttributes,
    PEventType,
    PsAttributeList,
    PsCreateInfo,
    RtlPathType,
    RtlRelativeNameU,
    RtlUserProcessParameters,
    TokenPrivileges,
    UnicodeString,
    CONTEXT,
    HANDLE,
    NTSTATUS,
    PHANDLE,
    PWSTR,
    SIZE_T,
    ULONG,
};

pub struct NtSyscall {
    /// The number of the syscall
    pub number:  u16,
    /// The address of the syscall
    pub address: *mut u8,
    /// The hash of the syscall (used for lookup)
    pub hash:    usize,
}

/// We implement Sync for NtSyscall to ensure that it can be safely shared
/// across multiple threads. This is necessary because lazy_static requires
/// the types it manages to be Sync. Since NtSyscall only contains raw pointers
/// and does not perform any interior mutability, it is safe to implement Sync manually.
unsafe impl Sync for NtSyscall {}

impl NtSyscall {
    pub const fn new() -> Self {
        NtSyscall {
            number:  0,
            address: null_mut(),
            hash:    0,
        }
    }
}

/// Retrieves a handle to the current process.
///
/// # Safety
///
/// This function involves unsafe operations.
///
/// # Returns
///
/// A handle to the current process.
pub fn nt_current_process() -> HANDLE { -1isize as HANDLE }

pub struct NtClose {
    pub syscall: NtSyscall,
}

unsafe impl Sync for NtClose {}

impl NtClose {
    pub const fn new() -> Self {
        NtClose {
            syscall: NtSyscall::new(),
        }
    }

    /// Wrapper function for NtClose to avoid repetitive run_syscall calls.
    ///
    /// # Arguments
    ///
    /// * `[in]` - `handle` A handle to an object. This is a required parameter that must be valid. It represents the
    ///   handle that will be closed by the function.
    ///
    /// # Returns
    ///
    /// * `true` if the operation was successful, `false` otherwise. The function returns an NTSTATUS code; however, in
    ///   this wrapper, the result is simplified to a boolean.
    pub fn run(&self, handle: *mut c_void) -> i32 {
        run_syscall!(self.syscall.number, self.syscall.address as usize, handle)
    }
}

pub struct NtAllocateVirtualMemory {
    pub syscall: NtSyscall,
}

unsafe impl Sync for NtAllocateVirtualMemory {}

impl NtAllocateVirtualMemory {
    pub const fn new() -> Self {
        NtAllocateVirtualMemory {
            syscall: NtSyscall::new(),
        }
    }

    /// Wrapper function for NtAllocateVirtualMemory to allocate memory in the virtual address space of a specified
    /// process.
    ///
    /// # Arguments
    ///
    /// * `[in]` - `handle` A handle to the process in which the memory will be allocated.
    /// * `[in, out]` - `base_address` A pointer to a variable that will receive the base address of the allocated
    ///   region of pages. If the value of `*base_address` is non-null, the region is allocated starting at the
    ///   specified address. If `*base_address` is null, the system determines where to allocate the region.
    /// * `[in]` - `zero_bits` The number of high-order address bits that must be zero in the base address of the
    ///   section view. This parameter is optional and can often be set to 0.
    /// * `[in, out]` - `region_size` A pointer to a variable that specifies the size of the region of memory to
    ///   allocate, in bytes. This parameter is updated with the actual size of the allocated region.
    /// * `[in]` - `allocation_type` The type of memory allocation. This parameter is required and can be a combination
    ///   of various flags like `MEM_COMMIT`, `MEM_RESERVE`, etc.
    /// * `[in]` - `protect` The memory protection for the region of pages to be allocated. This is a required parameter
    ///   and can include values like `PAGE_READWRITE`, `PAGE_EXECUTE`, etc.
    ///
    /// # Returns
    ///
    /// * `true` if the operation was successful, `false` otherwise. The function simplifies the NTSTATUS result into a
    ///   boolean indicating success or failure.
    pub fn run(
        &self,
        handle: *mut c_void,
        base_address: &mut *mut c_void,
        zero_bits: ULONG,
        region_size: usize,
        allocation_type: ULONG,
        protect: ULONG,
    ) -> i32 {
        run_syscall!(
            self.syscall.number,
            self.syscall.address as usize,
            handle,
            base_address,
            zero_bits,
            &mut (region_size as usize) as *mut usize,
            allocation_type,
            protect
        )
    }
}

pub struct NtWriteVirtualMemory {
    pub syscall: NtSyscall,
}

unsafe impl Sync for NtWriteVirtualMemory {}

impl NtWriteVirtualMemory {
    pub const fn new() -> Self {
        NtWriteVirtualMemory {
            syscall: NtSyscall::new(),
        }
    }

    /// Wrapper for the NtWriteVirtualMemory syscall.
    ///
    /// This function writes data to the virtual memory of a process. It wraps the NtWriteVirtualMemory syscall.
    ///
    /// # Arguments
    ///
    /// * `[in]` - `process_handle` A handle to the process whose memory is to be written to.
    /// * `[in]` - `base_address` A pointer to the base address in the process's virtual memory where the data should be
    ///   written.
    /// * `[in]` - `buffer` A pointer to the buffer that contains the data to be written.
    /// * `[in]` - `buffer_size` The size, in bytes, of the buffer pointed to by the `buffer` parameter.
    /// * `[out]` - `number_of_bytes_written` A pointer to a variable that receives the number of bytes that were
    ///   actually written to the process's memory.
    ///
    /// # Returns
    ///
    /// * `i32` - The NTSTATUS code of the operation, indicating success or failure of the syscall.
    pub fn run(
        &self,
        process_handle: HANDLE,
        base_address: *mut c_void,
        buffer: *const c_void,
        buffer_size: usize,
        number_of_bytes_written: &mut usize,
    ) -> i32 {
        run_syscall!(
            self.syscall.number,
            self.syscall.address as usize,
            process_handle,
            base_address,
            buffer,
            buffer_size,
            number_of_bytes_written
        )
    }
}

pub struct NtFreeVirtualMemory {
    pub syscall: NtSyscall,
}

unsafe impl Sync for NtFreeVirtualMemory {}

impl NtFreeVirtualMemory {
    pub const fn new() -> Self {
        NtFreeVirtualMemory {
            syscall: NtSyscall::new(),
        }
    }

    /// Wrapper for the NtFreeVirtualMemory syscall.
    ///
    /// This function frees a region of pages within the virtual address space of a specified process. It wraps the
    /// NtFreeVirtualMemory syscall.
    ///
    /// # Arguments
    ///
    /// * `[in]` - `process_handle` A handle to the process whose memory is to be freed.
    /// * `[in, out]` - `base_address` A pointer to a variable that specifies the base address of the region of memory
    ///   to be freed. If `MEM_RELEASE` is specified, the pointer must be to the base address returned by
    ///   `NtAllocateVirtualMemory`. The value of this parameter is updated by the function.
    /// * `[in, out]` - `region_size` A pointer to a variable that specifies the size of the region of memory to be
    ///   freed, in bytes. If `MEM_RELEASE` is specified, `region_size` must be 0. The value of this parameter is
    ///   updated by the function.
    /// * `[in]` - `free_type` The type of free operation. This is a required parameter and can be `MEM_RELEASE`
    ///   (0x8000) or `MEM_DECOMMIT` (0x4000).
    ///
    /// # Returns
    ///
    /// * `i32` - The NTSTATUS code of the operation, indicating success or failure of the syscall.
    pub fn run(
        &self,
        process_handle: *mut c_void,
        base_address: *mut u8,
        mut region_size: usize,
        free_type: ULONG,
    ) -> i32 {
        run_syscall!(
            self.syscall.number,
            self.syscall.address as usize,
            process_handle,
            &mut (base_address as *mut c_void),
            &mut region_size,
            free_type
        )
    }
}

pub struct NtOpenKey {
    pub syscall: NtSyscall,
}

unsafe impl Sync for NtOpenKey {}

impl NtOpenKey {
    pub const fn new() -> Self {
        NtOpenKey {
            syscall: NtSyscall::new(),
        }
    }

    /// Wrapper for the NtOpenKey syscall.
    ///
    /// This function opens the specified registry key. It wraps the NtOpenKey syscall.
    ///
    /// # Arguments
    ///
    /// * `[out]` - `p_key_handle` A mutable pointer to a handle that will receive the key handle.
    /// * `[in]` - `desired_access` Specifies the desired access rights to the key. This is a required parameter and
    ///   determines the allowed operations on the key.
    /// * `[in]` - `object_attributes` A pointer to an `ObjectAttributes` structure that specifies the attributes of the
    ///   key object.
    ///
    /// # Returns
    ///
    /// * `i32` - The NTSTATUS code of the operation, indicating success or failure of the syscall.
    pub fn run(
        &self,
        p_key_handle: &mut *mut c_void,
        desired_access: AccessMask,
        object_attributes: &mut ObjectAttributes,
    ) -> i32 {
        run_syscall!(
            self.syscall.number,
            self.syscall.address as usize,
            p_key_handle,
            desired_access,
            object_attributes as *mut _ as *mut c_void
        )
    }
}

pub struct NtQueryValueKey {
    pub syscall: NtSyscall,
}

unsafe impl Sync for NtQueryValueKey {}

impl NtQueryValueKey {
    pub const fn new() -> Self {
        NtQueryValueKey {
            syscall: NtSyscall::new(),
        }
    }

    /// Wrapper for the NtQueryValueKey syscall.
    ///
    /// # Arguments
    ///
    /// * `[in]` - `key_handle` A handle to the key.
    /// * `[in]` - `value_name` A pointer to the UnicodeString structure containing the name of the value to be queried.
    /// * `[in]` - `key_value_information_class` Specifies the type of information to be returned.
    /// * `[out]` - `key_value_information` A pointer to a buffer that receives the requested information.
    /// * `[in]` - `length` The size, in bytes, of the buffer pointed to by the `key_value_information` parameter.
    /// * `[out]` - `result_length` A pointer to a variable that receives the size, in bytes, of the data returned.
    ///
    /// # Returns
    ///
    /// * `i32` - The NTSTATUS code of the operation.
    pub fn run(
        &self,
        key_handle: *mut c_void,
        value_name: &UnicodeString,
        key_value_information_class: u32,
        key_value_information: *mut c_void,
        length: u32,
        result_length: &mut u32,
    ) -> i32 {
        run_syscall!(
            self.syscall.number,
            self.syscall.address as usize,
            key_handle,
            value_name as *const _ as usize,
            key_value_information_class,
            key_value_information,
            length,
            result_length as *mut _ as usize
        )
    }
}

pub struct NtEnumerateKey {
    pub syscall: NtSyscall,
}

unsafe impl Sync for NtEnumerateKey {}

impl NtEnumerateKey {
    pub const fn new() -> Self {
        NtEnumerateKey {
            syscall: NtSyscall::new(),
        }
    }

    /// Wrapper for the NtEnumerateKey syscall.
    ///
    /// # Arguments
    ///
    /// * `[in]` - `key_handle` A handle to the key.
    /// * `[in]` - `index` The index of the subkey to be enumerated.
    /// * `[in]` - `key_information_class` Specifies the type of information to be returned.
    /// * `[out]` - `key_information` A pointer to a buffer that receives the requested information.
    /// * `[in]` - `length` The size, in bytes, of the buffer pointed to by the `key_information` parameter.
    /// * `[out]` - `result_length` A pointer to a variable that receives the size, in bytes, of the data returned.
    ///
    /// # Returns
    ///
    /// * `i32` - The NTSTATUS code of the operation.
    pub fn run(
        &self,
        key_handle: *mut c_void,
        index: ULONG,
        key_information_class: u32,
        key_information: *mut c_void,
        length: ULONG,
        result_length: &mut ULONG,
    ) -> i32 {
        run_syscall!(
            self.syscall.number,
            self.syscall.address as usize,
            key_handle,
            index,
            key_information_class,
            key_information,
            length,
            result_length
        )
    }
}

pub struct NtQuerySystemInformation {
    pub syscall: NtSyscall,
}

unsafe impl Sync for NtQuerySystemInformation {}

impl NtQuerySystemInformation {
    pub const fn new() -> Self {
        NtQuerySystemInformation {
            syscall: NtSyscall::new(),
        }
    }

    /// Wrapper for the NtQuerySystemInformation syscall.
    ///
    /// # Arguments
    ///
    /// * `[in]` - `system_information_class` The system information class to be queried.
    /// * `[out]` - `system_information` A pointer to a buffer that receives the requested information.
    /// * `[in]` - `system_information_length` The size, in bytes, of the buffer pointed to by the `system_information`
    ///   parameter.
    /// * `[out, opt]` - `return_length` A pointer to a variable that receives the size, in bytes, of the data returned.
    ///
    /// # Returns
    ///
    /// * `i32` - The NTSTATUS code of the operation.
    pub fn run(
        &self,
        system_information_class: u32,
        system_information: *mut c_void,
        system_information_length: u32,
        return_length: *mut u32,
    ) -> i32 {
        run_syscall!(
            self.syscall.number,
            self.syscall.address as usize,
            system_information_class,
            system_information,
            system_information_length,
            return_length
        )
    }
}

pub struct NtQueryInformationProcess {
    pub syscall: NtSyscall,
}

unsafe impl Sync for NtQueryInformationProcess {}

impl NtQueryInformationProcess {
    pub const fn new() -> Self {
        NtQueryInformationProcess {
            syscall: NtSyscall::new(),
        }
    }

    /// Wrapper for the NtQueryInformationProcess syscall.
    ///
    /// # Arguments
    ///
    /// * `[in]` - `process_handle` A handle to the process.
    /// * `[in]` - `process_information_class` The class of information to be queried.
    /// * `[out]` - `process_information` A pointer to a buffer that receives the requested information.
    /// * `[in]` - `process_information_length` The size, in bytes, of the buffer pointed to by the
    ///   `process_information` parameter.
    /// * `[out, opt]` - `return_length` A pointer to a variable that receives the size, in bytes, of the data returned.
    ///
    /// # Returns
    ///
    /// * `i32` - The NTSTATUS code of the operation.
    pub fn run(
        &self,
        process_handle: HANDLE,
        process_information_class: u32,
        process_information: *mut c_void,
        process_information_length: ULONG,
        return_length: *mut ULONG,
    ) -> i32 {
        run_syscall!(
            self.syscall.number,
            self.syscall.address as usize,
            process_handle,
            process_information_class,
            process_information,
            process_information_length,
            return_length
        )
    }
}

pub struct NtOpenProcess {
    pub syscall: NtSyscall,
}

unsafe impl Sync for NtOpenProcess {}

impl NtOpenProcess {
    pub const fn new() -> Self {
        NtOpenProcess {
            syscall: NtSyscall::new(),
        }
    }

    /// Wrapper for the NtOpenProcess syscall.
    ///
    /// # Arguments
    ///
    /// * `[out]` - `process_handle` A mutable pointer to a handle that will receive the process handle.
    /// * `[in]` - `desired_access` The desired access for the process.
    /// * `[in]` - `object_attributes` A pointer to the object attributes structure.
    /// * `[in, opt]` - `client_id` A pointer to the client ID structure.
    ///
    /// # Returns
    ///
    /// * `i32` - The NTSTATUS code of the operation.
    pub fn run(
        &self,
        process_handle: &mut HANDLE,
        desired_access: AccessMask,
        object_attributes: &mut ObjectAttributes,
        client_id: *mut c_void,
    ) -> i32 {
        run_syscall!(
            self.syscall.number,
            self.syscall.address as usize,
            process_handle,
            desired_access,
            object_attributes as *mut _ as *mut c_void,
            client_id
        )
    }
}

pub struct NtOpenProcessToken {
    pub syscall: NtSyscall,
}

unsafe impl Sync for NtOpenProcessToken {}

impl NtOpenProcessToken {
    pub const fn new() -> Self {
        NtOpenProcessToken {
            syscall: NtSyscall::new(),
        }
    }

    /// Wrapper for the NtOpenProcessToken syscall.
    ///
    /// # Arguments
    ///
    /// * `[in]` - `process_handle` The handle of the process whose token is to be opened.
    /// * `[in]` - `desired_access` The desired access for the token.
    /// * `[out]` - `token_handle` A mutable pointer to a handle that will receive the token handle.
    ///
    /// # Returns
    ///
    /// * `i32` - The NTSTATUS code of the operation.
    pub fn run(&self, process_handle: HANDLE, desired_access: AccessMask, token_handle: &mut HANDLE) -> i32 {
        run_syscall!(
            self.syscall.number,
            self.syscall.address as usize,
            process_handle,
            desired_access,
            token_handle
        )
    }
}

pub struct NtOpenProcessTokenEx {
    pub syscall: NtSyscall,
}

unsafe impl Sync for NtOpenProcessTokenEx {}

impl NtOpenProcessTokenEx {
    pub const fn new() -> Self {
        NtOpenProcessTokenEx {
            syscall: NtSyscall::new(),
        }
    }

    /// Wrapper for the NtOpenProcessTokenEx syscall.
    ///
    /// # Arguments
    ///
    /// * `[in]` - `process_handle` The handle of the process whose token is to be opened.
    /// * `[in]` - `desired_access` The desired access for the token.
    /// * `[in, opt]` - `handle_attributes` Attributes for the handle.
    /// * `[out]` - `token_handle` A mutable pointer to a handle that will receive the token handle.
    ///
    /// # Returns
    ///
    /// * `i32` - The NTSTATUS code of the operation.
    pub fn run(
        &self,
        process_handle: HANDLE,
        desired_access: AccessMask,
        handle_attributes: ULONG,
        token_handle: &mut HANDLE,
    ) -> i32 {
        run_syscall!(
            self.syscall.number,
            self.syscall.address as usize,
            process_handle,
            desired_access,
            handle_attributes,
            token_handle
        )
    }
}

pub struct NtQueryInformationToken {
    pub syscall: NtSyscall,
}

unsafe impl Sync for NtQueryInformationToken {}

impl NtQueryInformationToken {
    pub const fn new() -> Self {
        NtQueryInformationToken {
            syscall: NtSyscall::new(),
        }
    }

    /// Wrapper for the NtQueryInformationToken syscall.
    ///
    /// # Arguments
    ///
    /// * `[in]` - `token_handle` The handle of the token to be queried.
    /// * `[in]` - `token_information_class` The class of information to be queried.
    /// * `[out]` - `token_information` A pointer to a buffer that receives the requested information.
    /// * `[in]` - `token_information_length` The size, in bytes, of the buffer pointed to by the `token_information`
    ///   parameter.
    /// * `[out, opt]` - `return_length` A pointer to a variable that receives the size, in bytes, of the data returned.
    ///
    /// # Returns
    ///
    /// * `i32` - The NTSTATUS code of the operation.
    pub fn run(
        &self,
        token_handle: HANDLE,
        token_information_class: ULONG,
        token_information: *mut c_void,
        token_information_length: ULONG,
        return_length: *mut ULONG,
    ) -> i32 {
        run_syscall!(
            self.syscall.number,
            self.syscall.address as usize,
            token_handle,
            token_information_class,
            token_information,
            token_information_length,
            return_length
        )
    }
}

pub struct NtAdjustPrivilegesToken {
    pub syscall: NtSyscall,
}

unsafe impl Sync for NtAdjustPrivilegesToken {}

impl NtAdjustPrivilegesToken {
    pub const fn new() -> Self {
        NtAdjustPrivilegesToken {
            syscall: NtSyscall::new(),
        }
    }

    /// Wrapper for the NtAdjustPrivilegesToken syscall.
    ///
    /// # Arguments
    ///
    /// * `[in]` - `token_handle` The handle of the token to be adjusted.
    /// * `[in]` - `disable_all_privileges` Boolean to disable all privileges.
    /// * `[in, opt]` - `new_state` A pointer to a TOKEN_PRIVILEGES structure.
    /// * `[in]` - `buffer_length` The length of the buffer for previous privileges.
    /// * `[out, opt]` - `previous_state` A pointer to a buffer that receives the previous state.
    /// * `[out, opt]` - `return_length` A pointer to a variable that receives the length of the previous state.
    ///
    /// # Returns
    ///
    /// * `i32` - The NTSTATUS code of the operation.
    pub fn run(
        &self,
        token_handle: HANDLE,
        disable_all_privileges: bool,
        new_state: *mut TokenPrivileges,
        buffer_length: ULONG,
        previous_state: *mut TokenPrivileges,
        return_length: *mut ULONG,
    ) -> i32 {
        run_syscall!(
            self.syscall.number,
            self.syscall.address as usize,
            token_handle,
            disable_all_privileges as u32,
            new_state,
            buffer_length,
            previous_state,
            return_length
        )
    }
}

pub struct NtWaitForSingleObject {
    pub syscall: NtSyscall,
}

unsafe impl Sync for NtWaitForSingleObject {}

impl NtWaitForSingleObject {
    pub const fn new() -> Self {
        NtWaitForSingleObject {
            syscall: NtSyscall::new(),
        }
    }

    /// Wrapper for the NtWaitForSingleObject syscall.
    ///
    /// # Arguments
    ///
    /// * `[in]` - `handle` A handle to the object.
    /// * `[in]` - `alertable` A boolean value that specifies whether the wait is alertable.
    /// * `[in, opt]` - `timeout` An optional pointer to a time-out value.
    ///
    /// # Returns
    ///
    /// * `i32` - The NTSTATUS code of the operation.
    pub fn run(&self, handle: HANDLE, alertable: bool, timeout: *mut c_void) -> i32 {
        run_syscall!(
            self.syscall.number,
            self.syscall.address as usize,
            handle,
            alertable as u32,
            timeout
        )
    }
}

pub struct NtOpenFile {
    pub syscall: NtSyscall,
}

unsafe impl Sync for NtOpenFile {}

impl NtOpenFile {
    pub const fn new() -> Self {
        NtOpenFile {
            syscall: NtSyscall::new(),
        }
    }

    /// Wrapper for the NtOpenFile syscall.
    ///
    /// # Arguments
    ///
    /// * `[out]` - `file_handle` A pointer to a handle that receives the file handle.
    /// * `[in]` - `desired_access` The desired access for the file handle.
    /// * `[in]` - `object_attributes` A pointer to the OBJECT_ATTRIBUTES structure.
    /// * `[out]` - `io_status_block` A pointer to an IO_STATUS_BLOCK structure that receives the status block.
    /// * `[in]` - `share_access` The requested share access for the file.
    /// * `[in]` - `open_options` The options to be applied when opening the file.
    ///
    /// # Returns
    ///
    /// * `i32` - The NTSTATUS code of the operation.
    pub fn run(
        &self,
        file_handle: &mut HANDLE,
        desired_access: ULONG,
        object_attributes: &mut ObjectAttributes,
        io_status_block: &mut IoStatusBlock,
        share_access: ULONG,
        open_options: ULONG,
    ) -> i32 {
        run_syscall!(
            self.syscall.number,
            self.syscall.address as usize,
            file_handle,
            desired_access,
            object_attributes,
            io_status_block,
            share_access,
            open_options
        )
    }
}

pub struct NtCreateEvent {
    pub syscall: NtSyscall,
}

unsafe impl Sync for NtCreateEvent {}

impl NtCreateEvent {
    pub const fn new() -> Self {
        NtCreateEvent {
            syscall: NtSyscall::new(),
        }
    }

    /// Wrapper function for NtCreateEvent to avoid repetitive run_syscall calls.
    ///
    /// # Arguments
    ///
    /// * `[out]` - `event_handle` A mutable pointer to a handle that will receive the event handle.
    /// * `[in]` - `desired_access` The desired access for the event.
    /// * `[in, opt]` - `object_attributes` A pointer to the object attributes structure. This can be null.
    /// * `[in]` - `event_type` The type of event to be created.
    /// * `[in]` - `initial_state` The initial state of the event.
    ///
    /// # Returns
    ///
    /// * `i32` - The NTSTATUS code of the operation.
    pub fn run(
        &self,
        event_handle: &mut HANDLE,
        desired_access: AccessMask,
        object_attributes: Option<&mut ObjectAttributes>,
        event_type: PEventType,
        initial_state: *mut c_uchar,
    ) -> i32 {
        let obj_attr_ptr = match object_attributes {
            Some(attrs) => attrs as *mut _ as *mut c_void,
            None => null_mut(),
        };
        run_syscall!(
            self.syscall.number,
            self.syscall.address as usize,
            event_handle,
            desired_access,
            obj_attr_ptr,
            event_type,
            initial_state
        )
    }
}

pub struct NtWriteFile {
    pub syscall: NtSyscall,
}

unsafe impl Sync for NtWriteFile {}

impl NtWriteFile {
    pub const fn new() -> Self {
        NtWriteFile {
            syscall: NtSyscall::new(),
        }
    }

    /// Wrapper for the NtWriteFile syscall.
    ///
    /// This function writes data to a file or I/O device. It wraps the NtWriteFile syscall.
    ///
    /// # Arguments
    ///
    /// * `[in]` - `file_handle` A handle to the file or I/O device to be written to.
    /// * `[in, opt]` - `event` An optional handle to an event object that will be signaled when the operation
    ///   completes.
    /// * `[in, opt]` - `apc_routine` An optional pointer to an APC routine to be called when the operation completes.
    /// * `[in, opt]` - `apc_context` An optional pointer to a context for the APC routine.
    /// * `[out]` - `io_status_block` A pointer to an IO_STATUS_BLOCK structure that receives the final completion
    ///   status and information about the operation.
    /// * `[in]` - `buffer` A pointer to a buffer that contains the data to be written to the file or device.
    /// * `[in]` - `length` The length, in bytes, of the buffer pointed to by the `buffer` parameter.
    /// * `[in, opt]` - `byte_offset` A pointer to the byte offset in the file where the operation should begin. If this
    ///   parameter is `None`, the system writes data to the current file position.
    /// * `[in, opt]` - `key` A pointer to a caller-supplied variable to receive the I/O completion key. This parameter
    ///   is ignored if `event` is not `None`.
    ///
    /// # Returns
    ///
    /// * `i32` - The NTSTATUS code of the operation.
    pub fn run(
        &self,
        file_handle: HANDLE,
        event: HANDLE,
        apc_routine: *mut c_void,
        apc_context: *mut c_void,
        io_status_block: &mut IoStatusBlock,
        buffer: *mut c_void,
        length: ULONG,
        byte_offset: *mut u64,
        key: *mut ULONG,
    ) -> i32 {
        run_syscall!(
            self.syscall.number,
            self.syscall.address as usize,
            file_handle,
            event,
            apc_routine,
            apc_context,
            io_status_block,
            buffer,
            length,
            byte_offset,
            key
        )
    }
}

pub struct NtCreateFile {
    pub syscall: NtSyscall,
}

unsafe impl Sync for NtCreateFile {}

impl NtCreateFile {
    pub const fn new() -> Self {
        NtCreateFile {
            syscall: NtSyscall::new(),
        }
    }

    /// Wrapper for the NtCreateFile syscall.
    ///
    /// This function creates or opens a file or I/O device. It wraps the NtCreateFile syscall.
    ///
    /// # Arguments
    ///
    /// * `[out]` - `file_handle` A mutable pointer to a handle that will receive the file handle.
    /// * `[in]` - `desired_access` The access to the file or device, which can be read, write, or both.
    /// * `[in]` - `obj_attributes` A pointer to an OBJECT_ATTRIBUTES structure that specifies the object name and other
    ///   attributes.
    /// * `[out]` - `io_status_block` A pointer to an IO_STATUS_BLOCK structure that receives the final completion
    ///   status and information about the operation.
    /// * `[in, opt]` - `allocation_size` A pointer to a LARGE_INTEGER that specifies the initial allocation size in
    ///   bytes. If this parameter is `None`, the file is allocated with a default size.
    /// * `[in]` - `file_attributes` The file attributes for the file or device if it is created.
    /// * `[in]` - `share_access` The requested sharing mode of the file or device.
    /// * `[in]` - `create_disposition` The action to take depending on whether the file or device already exists.
    /// * `[in]` - `create_options` Options to be applied when creating or opening the file or device.
    /// * `[in, opt]` - `ea_buffer` A pointer to a buffer that contains the extended attributes (EAs) for the file or
    ///   device. This parameter is optional.
    /// * `[in]` - `ea_length` The length, in bytes, of the EaBuffer parameter.
    ///
    /// # Returns
    ///
    /// * `i32` - The NTSTATUS code of the operation.
    pub fn run(
        &self,
        file_handle: &mut HANDLE,
        desired_access: u32,
        obj_attributes: &mut ObjectAttributes,
        io_status_block: &mut IoStatusBlock,
        allocation_size: *mut u64,
        file_attributes: u32,
        share_access: u32,
        create_disposition: u32,
        create_options: u32,
        ea_buffer: *mut c_void,
        ea_length: u32,
    ) -> i32 {
        run_syscall!(
            self.syscall.number,
            self.syscall.address as usize,
            file_handle,
            desired_access,
            obj_attributes,
            io_status_block,
            allocation_size,
            file_attributes,
            share_access,
            create_disposition,
            create_options,
            ea_buffer,
            ea_length
        )
    }
}

pub struct NtReadFile {
    pub syscall: NtSyscall,
}

unsafe impl Sync for NtReadFile {}

impl NtReadFile {
    pub const fn new() -> Self {
        NtReadFile {
            syscall: NtSyscall::new(),
        }
    }

    /// Wrapper for the NtReadFile syscall.
    ///
    /// This function reads data from a file or I/O device. It wraps the NtReadFile syscall.
    ///
    /// # Arguments
    ///
    /// * `[in]` - `file_handle` A handle to the file or I/O device to be read from.
    /// * `[in, opt]` - `event` An optional handle to an event object that will be signaled when the operation
    ///   completes.
    /// * `[in, opt]` - `apc_routine` An optional pointer to an APC routine to be called when the operation completes.
    /// * `[in, opt]` - `apc_context` An optional pointer to a context for the APC routine.
    /// * `[out]` - `io_status_block` A pointer to an IO_STATUS_BLOCK structure that receives the final completion
    ///   status and information about the operation.
    /// * `[out]` - `buffer` A pointer to a buffer that receives the data read from the file or device.
    /// * `[in]` - `length` The length, in bytes, of the buffer pointed to by the `buffer` parameter.
    /// * `[in, opt]` - `byte_offset` A pointer to the byte offset in the file where the operation should begin. If this
    ///   parameter is `None`, the system reads data from the current file position.
    /// * `[in, opt]` - `key` A pointer to a caller-supplied variable to receive the I/O completion key. This parameter
    ///   is ignored if `event` is not `None`.
    ///
    /// # Returns
    ///
    /// * `i32` - The NTSTATUS code of the operation.
    pub fn run(
        &self,
        file_handle: HANDLE,
        event: HANDLE,
        apc_routine: *mut c_void,
        apc_context: *mut c_void,
        io_status_block: &mut IoStatusBlock,
        buffer: *mut c_void,
        length: ULONG,
        byte_offset: *mut u64,
        key: *mut ULONG,
    ) -> i32 {
        run_syscall!(
            self.syscall.number,
            self.syscall.address as usize,
            file_handle,
            event,
            apc_routine,
            apc_context,
            io_status_block,
            buffer,
            length,
            byte_offset,
            key
        )
    }
}

pub struct NtCreateProcessEx {
    pub syscall: NtSyscall,
}

unsafe impl Sync for NtCreateProcessEx {}

impl NtCreateProcessEx {
    pub const fn new() -> Self {
        NtCreateProcessEx {
            syscall: NtSyscall::new(),
        }
    }

    /// Wrapper for the NtCreateProcessEx syscall.
    ///
    /// # Arguments
    ///
    /// * `[out]` - `process_handle` A mutable pointer to a handle that will receive the process handle.
    /// * `[in]` - `desired_access` The desired access for the process.
    /// * `[in]` - `object_attributes` A pointer to the object attributes structure.
    /// * `[in]` - `parent_process` A handle to the parent process.
    /// * `[in]` - `flags` Flags for creating the process.
    /// * `[in, opt]` - `section_handle` A handle to a section object.
    /// * `[in, opt]` - `debug_port` A handle to the debug port.
    /// * `[in, opt]` - `exception_port` A handle to the exception port.
    /// * `[in, opt]` - `in_job` A flag indicating if the process is in a job.
    ///
    /// # Returns
    ///
    /// * `i32` - The NTSTATUS code of the operation.
    pub fn run(
        &self,
        process_handle: &mut HANDLE,
        desired_access: AccessMask,
        object_attributes: *mut ObjectAttributes,
        parent_process: HANDLE,
        flags: ULONG,
        section_handle: HANDLE,
        debug_port: HANDLE,
        exception_port: HANDLE,
        in_job: u32,
    ) -> i32 {
        run_syscall!(
            self.syscall.number,
            self.syscall.address as usize,
            process_handle,
            desired_access,
            object_attributes,
            parent_process,
            flags,
            section_handle,
            debug_port,
            exception_port,
            in_job
        )
    }
}

pub struct NtCreateThread {
    pub syscall: NtSyscall,
}

unsafe impl Sync for NtCreateThread {}

impl NtCreateThread {
    pub const fn new() -> Self {
        NtCreateThread {
            syscall: NtSyscall::new(),
        }
    }

    /// Wrapper for the NtCreateThread syscall.
    ///
    /// # Arguments
    ///
    /// * `[out]` - `ThreadHandle`: Un puntatore a un `HANDLE` che riceverà l'handle del thread creato.
    /// * `[in]` - `DesiredAccess`: Un `ACCESS_MASK` che specifica i diritti di accesso desiderati per il thread.
    /// * `[in]` - `ObjectAttributes`: Un puntatore a una struttura `OBJECT_ATTRIBUTES` che definisce gli attributi del
    ///   thread.
    /// * `[in]` - `ProcessHandle`: Un `HANDLE` al processo nel quale il thread sarà creato.
    /// * `[in]` - `ClientId`: Un puntatore a una struttura `CLIENT_ID` che identifica il thread e il processo.
    /// * `[in]` - `ThreadContext`: Un puntatore a una struttura `CONTEXT` che contiene il contesto iniziale del thread.
    /// * `[in]` - `InitialTeb`: Un puntatore a una struttura `INITIAL_TEB` che descrive l'initial TEB del thread.
    /// * `[in]` - `CreateSuspended`: Un `BOOLEAN` che specifica se il thread deve essere creato in stato sospeso.
    ///
    /// # Returns
    ///
    /// * `NTSTATUS` - Il codice NTSTATUS dell'operazione.
    pub fn run(
        &self,
        thread_handle: PHANDLE,
        desired_access: AccessMask,
        object_attributes: *mut ObjectAttributes,
        process_handle: HANDLE,
        client_id: *mut ClientId,
        thread_context: *mut CONTEXT,
        initial_teb: *mut InitialTeb,
        create_suspended: bool,
    ) -> NTSTATUS {
        run_syscall!(
            self.syscall.number,
            self.syscall.address as usize,
            thread_handle,
            desired_access,
            object_attributes,
            process_handle,
            client_id,
            thread_context,
            initial_teb,
            create_suspended as u32
        )
    }
}

pub struct NtCreateThreadEx {
    pub syscall: NtSyscall,
}

unsafe impl Sync for NtCreateThreadEx {}

impl NtCreateThreadEx {
    pub const fn new() -> Self {
        NtCreateThreadEx {
            syscall: NtSyscall::new(),
        }
    }

    /// Wrapper for the NtCreateThreadEx syscall.
    ///
    /// # Arguments
    ///
    /// * `[out]` - `thread_handle` A mutable pointer to a handle that will receive the thread handle.
    /// * `[in]` - `desired_access` The desired access for the thread.
    /// * `[in]` - `object_attributes` A pointer to the object attributes structure.
    /// * `[in]` - `process_handle` A handle to the process.
    /// * `[in]` - `start_routine` A pointer to the start routine.
    /// * `[in, opt]` - `argument` A pointer to the argument for the start routine.
    /// * `[in]` - `create_flags` Flags for creating the thread.
    /// * `[in, opt]` - `zero_bits` The zero bits.
    /// * `[in, opt]` - `stack_size` The stack size.
    /// * `[in, opt]` - `maximum_stack_size` The maximum stack size.
    /// * `[in, opt]` - `attribute_list` A pointer to an attribute list.
    ///
    /// # Returns
    ///
    /// * `i32` - The NTSTATUS code of the operation.
    pub fn run(
        &self,
        thread_handle: *mut HANDLE,
        desired_access: AccessMask,
        object_attributes: *mut ObjectAttributes,
        process_handle: HANDLE,
        start_routine: *mut c_void,
        argument: *mut c_void,
        create_flags: ULONG,
        zero_bits: SIZE_T,
        stack_size: SIZE_T,
        maximum_stack_size: SIZE_T,
        attribute_list: *mut c_void,
    ) -> i32 {
        run_syscall!(
            self.syscall.number,
            self.syscall.address as usize,
            thread_handle,
            desired_access,
            object_attributes,
            process_handle,
            start_routine,
            argument,
            create_flags,
            zero_bits,
            stack_size,
            maximum_stack_size,
            attribute_list
        )
    }
}

pub struct ZwCreateThreadEx {
    pub syscall: NtSyscall,
}

unsafe impl Sync for ZwCreateThreadEx {}

impl ZwCreateThreadEx {
    pub const fn new() -> Self {
        ZwCreateThreadEx {
            syscall: NtSyscall::new(),
        }
    }

    /// Wrapper for the ZwCreateThreadEx syscall.
    ///
    /// # Arguments
    ///
    /// * `[out]` - `ThreadHandle`: Un puntatore a un `HANDLE` che riceverà l'handle del thread creato.
    /// * `[in]` - `DesiredAccess`: Un `ACCESS_MASK` che specifica i diritti di accesso desiderati per il thread.
    /// * `[in]` - `ObjectAttributes`: Un puntatore a una struttura `OBJECT_ATTRIBUTES` che definisce gli attributi del
    ///   thread.
    /// * `[in]` - `ProcessHandle`: Un `HANDLE` al processo nel quale il thread sarà creato.
    /// * `[in]` - `StartRoutine`: Un puntatore alla funzione che rappresenta la routine iniziale del thread.
    /// * `[in, opt]` - `Argument`: Un puntatore agli argomenti da passare alla routine iniziale del thread.
    /// * `[in]` - `CreateFlags`: Flag che specificano come il thread deve essere creato (es. in stato sospeso).
    /// * `[in, opt]` - `ZeroBits`: Numero di bit zero per l'indirizzo dello stack.
    /// * `[in, opt]` - `StackSize`: Dimensione dello stack da allocare per il thread.
    /// * `[in, opt]` - `MaximumStackSize`: Dimensione massima dello stack del thread.
    /// * `[in, opt]` - `AttributeList`: Un puntatore a una lista di attributi opzionali per il thread.
    ///
    /// # Returns
    ///
    /// * `NTSTATUS` - Il codice NTSTATUS dell'operazione.
    pub fn run(
        &self,
        thread_handle: *mut HANDLE,
        desired_access: AccessMask,
        object_attributes: *mut ObjectAttributes,
        process_handle: HANDLE,
        start_routine: *mut c_void,
        argument: *mut c_void,
        create_flags: ULONG,
        zero_bits: SIZE_T,
        stack_size: SIZE_T,
        maximum_stack_size: SIZE_T,
        attribute_list: *mut c_void,
    ) -> NTSTATUS {
        run_syscall!(
            self.syscall.number,
            self.syscall.address as usize,
            thread_handle,
            desired_access,
            object_attributes,
            process_handle,
            start_routine,
            argument,
            create_flags,
            zero_bits,
            stack_size,
            maximum_stack_size,
            attribute_list
        )
    }
}

pub struct NtCreateUserProcess {
    pub syscall: NtSyscall,
}

unsafe impl Sync for NtCreateUserProcess {}

impl NtCreateUserProcess {
    pub const fn new() -> Self {
        NtCreateUserProcess {
            syscall: NtSyscall::new(),
        }
    }

    /// Wrapper for the NtCreateUserProcess syscall.
    ///
    /// # Arguments
    ///
    /// * `[out]` - `process_handle` A mutable pointer to a handle that will receive the process handle.
    /// * `[out]` - `thread_handle` A mutable pointer to a handle that will receive the thread handle.
    /// * `[in]` - `process_desired_access` The desired access for the process.
    /// * `[in]` - `thread_desired_access` The desired access for the thread.
    /// * `[in]` - `process_object_attributes` A pointer to the process object attributes structure.
    /// * `[in]` - `thread_object_attributes` A pointer to the thread object attributes structure.
    /// * `[in]` - `process_flags` Flags for creating the process.
    /// * `[in]` - `thread_flags` Flags for creating the thread.
    /// * `[in]` - `process_parameters` A pointer to the process parameters structure.
    /// * `[in]` - `create_info` A pointer to the create information structure.
    /// * `[in, opt]` - `attribute_list` A pointer to the attribute list structure.
    ///
    /// # Returns
    ///
    /// * `i32` - The NTSTATUS code of the operation.
    pub fn run(
        &self,
        process_handle: PHANDLE,
        thread_handle: PHANDLE,
        process_desired_access: AccessMask,
        thread_desired_access: AccessMask,
        process_object_attributes: *mut ObjectAttributes,
        thread_object_attributes: *mut ObjectAttributes,
        process_flags: ULONG,
        thread_flags: ULONG,
        process_parameters: *mut c_void,
        create_info: *mut PsCreateInfo,
        attribute_list: *mut PsAttributeList,
    ) -> i32 {
        run_syscall!(
            self.syscall.number,
            self.syscall.address as usize,
            process_handle,
            thread_handle,
            process_desired_access,
            thread_desired_access,
            process_object_attributes,
            thread_object_attributes,
            process_flags,
            thread_flags,
            process_parameters,
            create_info,
            attribute_list
        )
    }
}

pub struct NtResumeThread {
    pub syscall: NtSyscall,
}

unsafe impl Sync for NtResumeThread {}

impl NtResumeThread {
    pub const fn new() -> Self {
        NtResumeThread {
            syscall: NtSyscall::new(),
        }
    }

    /// Wrapper for the NtResumeThread syscall.
    ///
    /// This function resumes a suspended thread. It wraps the NtResumeThread syscall.
    ///
    /// # Arguments
    ///
    /// * `[in]` - `thread_handle` A handle to the thread to be resumed.
    /// * `[out, opt]` - `suspend_count` A pointer to a variable that receives the previous suspend count.
    ///
    /// # Returns
    ///
    /// * `i32` - The NTSTATUS code of the operation.
    pub fn run(&self, thread_handle: HANDLE, suspend_count: &mut u32) -> i32 {
        run_syscall!(
            self.syscall.number,
            self.syscall.address as usize,
            thread_handle,
            suspend_count
        )
    }
}

pub struct NtTerminateProcess {
    pub syscall: NtSyscall,
}

unsafe impl Sync for NtTerminateProcess {}

impl NtTerminateProcess {
    pub const fn new() -> Self {
        NtTerminateProcess {
            syscall: NtSyscall::new(),
        }
    }

    /// Wrapper for the NtTerminateProcess syscall.
    ///
    /// This function terminates a process. It wraps the NtTerminateProcess syscall.
    ///
    /// # Arguments
    ///
    /// * `process_handle` - A handle to the process to be terminated.
    /// * `exit_status` - The exit status to be returned by the process.
    ///
    /// # Returns
    ///
    /// * `i32` - The NTSTATUS code of the operation.
    pub fn run(&self, process_handle: HANDLE, exit_status: i32) -> i32 {
        run_syscall!(
            self.syscall.number,
            self.syscall.address as usize,
            process_handle,
            exit_status
        )
    }
}

pub struct NtTerminateThread {
    pub syscall: NtSyscall,
}

unsafe impl Sync for NtTerminateThread {}

impl NtTerminateThread {
    pub const fn new() -> Self {
        NtTerminateThread {
            syscall: NtSyscall::new(),
        }
    }

    /// Wrapper for the NtTerminateProcess syscall.
    ///
    /// This function terminates a process. It wraps the NtTerminateProcess syscall.
    ///
    /// # Arguments
    ///
    /// * `[in]` - `process_handle` A handle to the process to be terminated.
    /// * `[in]` - `exit_status` The exit status to be returned by the process.
    ///
    /// # Returns
    ///
    /// * `i32` - The NTSTATUS code of the operation.
    pub fn run(&self, thread_handle: HANDLE, exit_status: i32) -> i32 {
        run_syscall!(
            self.syscall.number,
            self.syscall.address as usize,
            thread_handle,
            exit_status
        )
    }
}

pub struct NtDelayExecution {
    pub syscall: NtSyscall,
}

unsafe impl Sync for NtDelayExecution {}

impl NtDelayExecution {
    pub const fn new() -> Self {
        NtDelayExecution {
            syscall: NtSyscall::new(),
        }
    }

    /// Wrapper for the NtDelayExecution syscall.
    ///
    /// This function delays the execution of the current thread for the specified interval.
    ///
    /// # Arguments
    ///
    /// * `[in]` - `alertable` A boolean indicating whether the delay can be interrupted by an alertable wait state.
    /// * `[in]` - `delay_interval` A pointer to the time interval for which execution is to be delayed.
    ///
    /// # Returns
    ///
    /// * `i32` - The NTSTATUS code of the operation.
    pub fn run(&self, alertable: bool, delay_interval: *const i64) -> i32 {
        run_syscall!(
            self.syscall.number,
            self.syscall.address as usize,
            alertable as u32,
            delay_interval
        )
    }
}

pub struct NtCreateNamedPipeFile {
    pub syscall: NtSyscall,
}

unsafe impl Sync for NtCreateNamedPipeFile {}

impl NtCreateNamedPipeFile {
    pub const fn new() -> Self {
        NtCreateNamedPipeFile {
            syscall: NtSyscall::new(),
        }
    }

    /// Wrapper for the NtCreateNamedPipeFile syscall.
    ///
    /// This function creates a named pipe file and returns a handle to it.
    ///
    /// # Arguments
    ///
    /// * `[out]` - `file_handle` A mutable pointer to a handle that will receive the file handle.
    /// * `[in]` - `desired_access` The desired access rights for the named pipe file.
    /// * `[in]` - `object_attributes` A pointer to an `OBJECT_ATTRIBUTES` structure that specifies the object
    ///   attributes.
    /// * `[out]` - `io_status_block` A pointer to an `IO_STATUS_BLOCK` structure that receives the status of the I/O
    ///   operation.
    /// * `[in]` - `share_access` The requested sharing mode of the file.
    /// * `[in]` - `create_disposition` Specifies the action to take on files that exist or do not exist.
    /// * `[in]` - `create_options` Specifies the options to apply when creating or opening the file.
    /// * `[in]` - `named_pipe_type` Specifies the type of named pipe (byte stream or message).
    /// * `[in]` - `read_mode` Specifies the read mode for the pipe.
    /// * `[in]` - `completion_mode` Specifies the completion mode for the pipe.
    /// * `[in]` - `maximum_instances` The maximum number of instances of the pipe.
    /// * `[in]` - `inbound_quota` The size of the input buffer, in bytes.
    /// * `[in]` - `outbound_quota` The size of the output buffer, in bytes.
    /// * `[in, opt]` - `default_timeout` A pointer to a `LARGE_INTEGER` structure that specifies the default time-out
    ///   value.
    ///
    /// # Returns
    ///
    /// * `i32` - The NTSTATUS code of the operation.
    pub fn run(
        &self,
        file_handle: *mut HANDLE,
        desired_access: ULONG,
        object_attributes: *mut ObjectAttributes,
        io_status_block: *mut IoStatusBlock,
        share_access: ULONG,
        create_disposition: ULONG,
        create_options: ULONG,
        named_pipe_type: ULONG,
        read_mode: ULONG,
        completion_mode: ULONG,
        maximum_instances: ULONG,
        inbound_quota: ULONG,
        outbound_quota: ULONG,
        default_timeout: *const LargeInteger,
    ) -> i32 {
        run_syscall!(
            self.syscall.number,
            self.syscall.address as usize,
            file_handle,
            desired_access,
            object_attributes,
            io_status_block,
            share_access,
            create_disposition,
            create_options,
            named_pipe_type,
            read_mode,
            completion_mode,
            maximum_instances,
            inbound_quota,
            outbound_quota,
            default_timeout
        )
    }
}

pub struct NtReadVirtualMemory {
    pub syscall: NtSyscall,
}

unsafe impl Sync for NtReadVirtualMemory {}

impl NtReadVirtualMemory {
    pub const fn new() -> Self {
        NtReadVirtualMemory {
            syscall: NtSyscall::new(),
        }
    }

    /// Wrapper for the NtReadVirtualMemory syscall.
    ///
    /// This function reads memory in the virtual address space of a specified process.
    ///
    /// # Arguments
    ///
    /// * `[in]` - `process_handle` A handle to the process whose memory is to be read.
    /// * `[in]` - `base_address` A pointer to the base address in the specified process from which to read.
    /// * `[out]` - `buffer` A pointer to a buffer that receives the contents from the address space of the specified
    ///   process.
    /// * `[in]` - `buffer_size` The number of bytes to be read into the buffer.
    /// * `[out, opt]` - `number_of_bytes_read` A pointer to a variable that receives the number of bytes transferred
    ///   into the buffer.
    ///
    /// # Returns
    ///
    /// * `i32` - The NTSTATUS code of the operation.
    pub fn run(
        &self,
        process_handle: HANDLE,
        base_address: *const c_void,
        buffer: *mut c_void,
        buffer_size: usize,
        number_of_bytes_read: *mut usize,
    ) -> i32 {
        run_syscall!(
            self.syscall.number,
            self.syscall.address as usize,
            process_handle,
            base_address,
            buffer,
            buffer_size,
            number_of_bytes_read
        )
    }
}

pub struct NtCreateProcess {
    pub syscall: NtSyscall,
}

unsafe impl Sync for NtCreateProcess {}

impl NtCreateProcess {
    pub const fn new() -> Self {
        NtCreateProcess {
            syscall: NtSyscall::new(),
        }
    }

    /// Wrapper for the NtCreateProcess syscall.
    ///
    /// This function creates a new process object. It wraps the NtCreateProcess syscall, which is used
    /// to create a new process in the Windows NT kernel. Unlike NtCreateUserProcess, this syscall
    /// does not create a new primary thread, and additional steps are needed to fully initialize the process.
    ///
    /// # Arguments
    ///
    /// * `[out]` - `process_handle` A mutable pointer to a handle that will receive the newly created process's handle.
    /// * `[in]` - `desired_access` The access rights desired for the process handle.
    /// * `[in]` - `object_attributes` A pointer to an `OBJECT_ATTRIBUTES` structure that specifies the object
    ///   attributes.
    /// * `[in]` - `parent_process` A handle to the parent process.
    /// * `[in]` - `inherit_object_table` A boolean indicating whether the new process should inherit the object table
    ///   of the parent process.
    /// * `[in, opt]` - `section_handle` A handle to a section object, which is mapped into the new process's virtual
    ///   address space.
    /// * `[in, opt]` - `debug_port` A handle to a debug port, which can be used for debugging the new process.
    /// * `[in, opt]` - `exception_port` A handle to an exception port, which can be used to handle exceptions in the
    ///   new process.
    ///
    /// # Returns
    ///
    /// * `i32` - The NTSTATUS code of the operation.
    pub fn run(
        &self,
        process_handle: *mut HANDLE,
        desired_access: AccessMask,
        object_attributes: *mut ObjectAttributes,
        parent_process: HANDLE,
        inherit_object_table: bool,
        section_handle: HANDLE,
        debug_port: HANDLE,
        exception_port: HANDLE,
    ) -> NTSTATUS {
        run_syscall!(
            self.syscall.number,
            self.syscall.address as usize,
            process_handle,
            desired_access,
            object_attributes,
            parent_process,
            inherit_object_table as u32,
            section_handle,
            debug_port,
            exception_port
        )
    }
}

pub struct NtQueryVirtualMemory {
    pub syscall: NtSyscall,
}

unsafe impl Sync for NtQueryVirtualMemory {}

impl NtQueryVirtualMemory {
    pub const fn new() -> Self {
        NtQueryVirtualMemory {
            syscall: NtSyscall::new(),
        }
    }

    /// Wrapper for the NtQueryVirtualMemory syscall.
    ///
    /// This function queries information about the virtual memory of a specified process.
    ///
    /// # Arguments
    ///
    /// * `[in]` - `process_handle` A handle to the process whose virtual memory is to be queried.
    /// * `[in]` - `base_address` A pointer to the base address in the process's virtual memory space.
    /// * `[in]` - `memory_information_class` Specifies the type of information to be queried (e.g.,
    ///   MemoryBasicInformation).
    /// * `[out]` - `memory_information` A pointer to a buffer that receives the information about the memory.
    /// * `[in]` - `memory_information_length` The size, in bytes, of the buffer pointed to by `memory_information`.
    /// * `[out, opt]` - `return_length` A pointer to a variable that receives the number of bytes returned.
    ///
    /// # Returns
    ///
    /// * `i32` - The NTSTATUS code of the operation.
    pub fn run(
        &self,
        process_handle: HANDLE,
        base_address: *const c_void,
        memory_information_class: u32,
        memory_information: *mut c_void,
        memory_information_length: usize,
        return_length: *mut usize,
    ) -> i32 {
        run_syscall!(
            self.syscall.number,
            self.syscall.address as usize,
            process_handle,
            base_address,
            memory_information_class as usize,
            memory_information,
            memory_information_length,
            return_length
        )
    }
}

/// Type definition for the LdrLoadDll function.
///
/// Loads a DLL into the address space of the calling process.
///
/// # Parameters
/// - `[in, opt]` - `DllPath`: A pointer to a `UNICODE_STRING` that specifies the fully qualified path of the DLL to
///   load. This can be `NULL`, in which case the system searches for the DLL.
/// - `[in, opt]` - `DllCharacteristics`: A pointer to a variable that specifies the DLL characteristics (optional, can
///   be `NULL`).
/// - `[in]` - `DllName`: A `UNICODE_STRING` that specifies the name of the DLL to load.
/// - `[out]` - `DllHandle`: A pointer to a variable that receives the handle to the loaded DLL.
///
/// # Returns
/// - `i32` - The NTSTATUS code of the operation.
type LdrLoadDll = unsafe extern "system" fn(
    DllPath: *mut u16,
    DllCharacteristics: *mut u32,
    DllName: UnicodeString,
    DllHandle: *mut c_void,
) -> i32;

/// Type definition for the RtlCreateProcessParametersEx function.
///
/// Creates process parameters for a new process.
///
/// # Parameters
/// - `[out]` - `pProcessParameters`: A pointer to a location that receives a pointer to the created
///   `RTL_USER_PROCESS_PARAMETERS` structure.
/// - `[in]` - `ImagePathName`: A pointer to a `UNICODE_STRING` that specifies the image path name for the process.
/// - `[in, opt]` - `DllPath`: A pointer to a `UNICODE_STRING` that specifies the DLL path (optional, can be `NULL`).
/// - `[in, opt]` - `CurrentDirectory`: A pointer to a `UNICODE_STRING` that specifies the current directory (optional).
/// - `[in, opt]` - `CommandLine`: A pointer to a `UNICODE_STRING` that specifies the command line for the process
///   (optional).
/// - `[in, opt]` - `Environment`: A pointer to an environment block (optional, can be `NULL`).
/// - `[in, opt]` - `WindowTitle`: A pointer to a `UNICODE_STRING` that specifies the window title (optional, can be
///   `NULL`).
/// - `[in, opt]` - `DesktopInfo`: A pointer to a `UNICODE_STRING` that specifies the desktop information (optional, can
///   be `NULL`).
/// - `[in, opt]` - `ShellInfo`: A pointer to a `UNICODE_STRING` that specifies the shell information (optional, can be
///   `NULL`).
/// - `[in, opt]` - `RuntimeData`: A pointer to a `UNICODE_STRING` that specifies runtime data (optional, can be
///   `NULL`).
/// - `[in]` - `Flags`: An unsigned integer that specifies various flags that control the creation of process
///   parameters.
///
/// # Returns
/// - `STATUS_SUCCESS` if successful, or an NTSTATUS error code if the function fails.
type RtlCreateProcessParametersEx = unsafe extern "system" fn(
    pProcessParameters: *mut *mut RtlUserProcessParameters,
    ImagePathName: *const UnicodeString,
    DllPath: *const UnicodeString,
    CurrentDirectory: *const UnicodeString,
    CommandLine: *const UnicodeString,
    Environment: *const c_void,
    WindowTitle: *const UnicodeString,
    DesktopInfo: *const UnicodeString,
    ShellInfo: *const UnicodeString,
    RuntimeData: *const UnicodeString,
    Flags: u32,
) -> i32;

/// Type definition for the RtlCreateHeap function.
///
/// Creates a heap with the specified attributes. The heap can be used to allocate and manage memory dynamically.
///
/// # Parameters
/// - `[in]` - `Flags`: Specifies the attributes of the heap. This can include options such as enabling heap
///   serialization.
/// - `[in, opt]` - `HeapBase`: A pointer to a memory block that will serve as the base of the heap. This parameter can
///   be `NULL`, in which case the system determines the base address.
/// - `[in]` - `ReserveSize`: The initial size, in bytes, to reserve for the heap. This is the amount of virtual memory
///   reserved for the heap.
/// - `[in]` - `CommitSize`: The initial size, in bytes, of committed memory in the heap. This is the amount of physical
///   memory initially allocated for the heap.
/// - `[in, opt]` - `Lock`: A pointer to a lock for heap synchronization. This can be `NULL` if no lock is required.
/// - `[in, opt]` - `Parameters`: A pointer to an optional structure that specifies advanced parameters for heap
///   creation. This can be `NULL`.
///
/// # Returns
/// - `HANDLE`: A handle to the newly created heap. If the heap creation fails, the handle will be `NULL`.
type RtlCreateHeap = unsafe extern "system" fn(
    Flags: u32,
    HeapBase: *mut u8,
    ReserveSize: usize,
    CommitSize: usize,
    Lock: *mut u8,
    Parameters: *mut u8,
) -> HANDLE;

/// Type definition for the RtlAllocateHeap function.
///
/// Allocates a block of memory from the specified heap. The allocated memory is uninitialized.
///
/// # Parameters
/// - `[in]` - `hHeap`: A handle to the heap from which the memory will be allocated.
/// - `[in]` - `dwFlags`: Flags that control aspects of the allocation, such as whether to generate exceptions on
///   failure.
/// - `[in]` - `dwBytes`: The number of bytes to allocate from the heap.
///
/// # Returns
/// - `*mut u8`: A pointer to the allocated memory block. If the allocation fails, the pointer will be `NULL`.
type RtlAllocateHeap = unsafe extern "system" fn(hHeap: HANDLE, dwFlags: u32, dwBytes: usize) -> *mut u8;

/// Type definition for the RtlFreeHeap function.
///
/// Frees a memory block allocated from the specified heap. The freed memory is returned to the heap and can be reused.
///
/// # Parameters
/// - `[in]` - `hHeap`: A handle to the heap from which the memory was allocated.
/// - `[in]` - `dwFlags`: Flags that control aspects of the free operation, such as whether to perform validation
///   checks.
/// - `[in]` - `lpMem`: A pointer to the memory block to be freed.
///
/// # Returns
/// - `BOOL`: A boolean value indicating whether the operation was successful (`TRUE`) or not (`FALSE`).
type RtlFreeHeap = unsafe extern "system" fn(hHeap: HANDLE, dwFlags: u32, lpMem: *mut u8) -> i32;

/// Type definition for the RtlReAllocateHeap function.
///
/// Reallocates a memory block from the specified heap, changing its size. The contents of the memory block are
/// preserved up to the smaller of the new or old sizes.
///
/// # Parameters
/// - `[in]` - `hHeap`: A handle to the heap from which the memory will be reallocated.
/// - `[in]` - `dwFlags`: Flags that control aspects of the reallocation, such as whether to generate exceptions on
///   failure.
/// - `[in]` - `lpMem`: A pointer to the memory block to be reallocated.
/// - `[in]` - `dwBytes`: The new size, in bytes, for the memory block.
///
/// # Returns
/// - `*mut u8`: A pointer to the reallocated memory block. If the reallocation fails, the pointer will be `NULL`.
type RtlReAllocateHeap =
    unsafe extern "system" fn(hHeap: HANDLE, dwFlags: u32, lpMem: *mut u8, dwBytes: usize) -> *mut u8;

/// Type definition for the RtlDestroyHeap function.
///
/// Destroys the specified heap and releases all of its memory. Once a heap is destroyed, it cannot be used.
///
/// # Parameters
/// - `[in]` - `hHeap`: A handle to the heap to be destroyed.
///
/// # Returns
/// - `HANDLE`: The function returns `NULL` if the heap was successfully destroyed. If the function fails, it returns
///   the handle to the heap.
type RtlDestroyHeap = unsafe extern "system" fn(hHeap: HANDLE) -> HANDLE;
/// Type definition for the `RtlGetFullPathName_U` function.
///
/// Retrieves the full path and file name for the specified file, resolving any relative path
/// components.
///
/// # Parameters
/// - `[in]` - `FileName`: A pointer to a wide string (`PWSTR`) that specifies the relative or absolute file name. This
///   string is expected to be null-terminated.
/// - `[in]` - `BufferLength`: The size, in characters, of the buffer that will receive the full path and file name.
///   This size includes space for the null terminator.
/// - `[out]` - `Buffer`: A pointer to a buffer (`PWSTR`) that receives the full path and file name as a wide string.
///   This string is null-terminated if the buffer is large enough.
/// - `[out, optional]` - `FilePart`: A pointer to a `PWSTR` that receives the address of the final file name component
///   within the full path. This parameter can be `NULL` if the caller does not need this information.
///
/// # Returns
/// - `ULONG`: The function returns the length, in characters, of the string copied to the buffer, excluding the null
///   terminator. If the buffer is too small, the function returns the size, in characters, required to hold the full
///   path and file name.
///
/// # Remarks
/// - If the `Buffer` is too small to hold the full path, the function does not null-terminate the string.
/// - This function operates on wide character strings (`wchar_t`), meaning it is designed for use with the Windows
///   Unicode string types.
type RtlGetFullPathNameU =
    unsafe extern "system" fn(FileName: PWSTR, BufferLength: ULONG, Buffer: PWSTR, FilePart: *mut PWSTR) -> ULONG;

/// Type definition for the RtlGetFullPathName_UstrEx function.
///
/// Retrieves the full path and file name for the specified relative path, with additional options
/// for dynamic memory management and validation.
///
/// # Parameters
/// - `[in]` - `FileName`: A pointer to a `UNICODE_STRING` that specifies the relative file name.
/// - `[in, out]` - `StaticString`: A pointer to a `UNICODE_STRING` that receives the full path and file name if it fits
///   within the static buffer.
/// - `[in, out, opt]` - `DynamicString`: A pointer to a `UNICODE_STRING` that receives the full path and file name if
///   the static buffer is insufficient. This is optional and can be `NULL`.
/// - `[out]` - `StringUsed`: A pointer to a `UNICODE_STRING` that receives a pointer to the used string (either static
///   or dynamic).
/// - `[out]` - `FilePartPrefixCch`: A pointer to a `SIZE_T` that receives the number of characters in the file part
///   prefix.
/// - `[out]` - `NameInvalid`: A pointer to a `BOOLEAN` that indicates whether the file name is invalid.
/// - `[out]` - `InputPathType`: A pointer to a `RTL_PATH_TYPE` that receives the type of the input path.
/// - `[out]` - `BytesRequired`: A pointer to a `SIZE_T` that receives the number of bytes required if the provided
///   buffers are insufficient.
///
/// # Returns
/// - `NTSTATUS`: The function returns `STATUS_SUCCESS` if successful, or an NTSTATUS error code if the function fails.
type RtlGetFullPathNameUstrEx = unsafe extern "system" fn(
    FileName: *const UnicodeString,
    StaticString: *mut UnicodeString,
    DynamicString: *mut UnicodeString,
    StringUsed: *mut *mut UnicodeString,
    FilePartPrefixCch: *mut usize,
    NameInvalid: *mut bool,
    InputPathType: *mut RtlPathType,
    BytesRequired: *mut usize,
) -> NTSTATUS;

/// Type definition for the `RtlDosPathNameToNtPathName_U` function.
///
/// Converts a DOS path (e.g., "C:\Windows\System32") to an NT path (e.g., "\??\C:\Windows\System32").
///
/// This function converts a DOS path name to an NT path name, which is the format used internally
/// by the Windows NT kernel. It also optionally returns the file part and relative path information.
///
/// # Parameters
/// - `[in]` - `DosFileName`: A pointer to a null-terminated Unicode string that specifies the DOS file path name to be
///   converted.
/// - `[out]` - `NtFileName`: A pointer to a `UNICODE_STRING` structure that receives the converted NT file path name.
/// - `[out, opt]` - `FilePart`: A pointer to a `PWSTR` that receives the address of the file part of the path, which is
///   the final component of the path (optional, can be `NULL`).
/// - `[out, opt]` - `RelativeName`: A pointer to a `RTL_RELATIVE_NAME_U` structure that receives relative path
///   information if the path is relative (optional, can be `NULL`).
///
/// # Returns
/// - `BOOLEAN`: The function returns `TRUE` if the conversion was successful, or `FALSE` if the conversion failed.
type RtlDosPathNameToNtPathNameU = unsafe extern "system" fn(
    DosFileName: PWSTR,                  // Pointer to the DOS path to convert
    NtFileName: *mut UnicodeString,      // Receives the converted NT path
    FilePart: *mut PWSTR,                // Receives the file part of the path (optional)
    RelativeName: *mut RtlRelativeNameU, // Receives relative path information (optional)
) -> bool;

pub struct NtDll {
    pub module_base: *mut u8,

    // Direct Syscall
    pub ldr_load_dll: LdrLoadDll,
    pub rtl_create_process_parameters_ex: RtlCreateProcessParametersEx,
    pub rtl_get_full_path_name_u: RtlGetFullPathNameU,
    pub rtl_get_full_path_name_ustrex: RtlGetFullPathNameUstrEx,
    pub rtl_dos_path_name_to_nt_path_name_u: RtlDosPathNameToNtPathNameU,

    // Heap management functions
    pub rtl_create_heap:     RtlCreateHeap,
    pub rtl_allocate_heap:   RtlAllocateHeap,
    pub rtl_free_heap:       RtlFreeHeap,
    pub rtl_reallocate_heap: RtlReAllocateHeap,
    pub rtl_destroy_heap:    RtlDestroyHeap,

    // Process Management functions
    pub nt_create_process:            NtCreateProcess,
    pub nt_create_process_ex:         NtCreateProcessEx,
    pub nt_create_user_process:       NtCreateUserProcess,
    pub nt_terminate_process:         NtTerminateProcess,
    pub nt_open_process:              NtOpenProcess,
    pub nt_query_information_process: NtQueryInformationProcess,

    // File Management functions
    pub nt_open_file:   NtOpenFile,
    pub nt_write_file:  NtWriteFile,
    pub nt_create_file: NtCreateFile,
    pub nt_read_file:   NtReadFile,

    // Registry Management functions
    pub nt_open_key:        NtOpenKey,
    pub nt_query_value_key: NtQueryValueKey,
    pub nt_enumerate_key:   NtEnumerateKey,

    // Memory Management functions
    pub nt_allocate_virtual_memory: NtAllocateVirtualMemory,
    pub nt_free_virtual_memory:     NtFreeVirtualMemory,
    pub nt_read_virtual_memory:     NtReadVirtualMemory,
    pub nt_write_virtual_memory:    NtWriteVirtualMemory,

    // Thread Management functions
    pub nt_create_thread:          NtCreateThread,
    pub nt_create_thread_ex:       NtCreateThreadEx,
    pub nt_zw_create_thread_ex:    ZwCreateThreadEx,
    pub nt_resume_thread:          NtResumeThread,
    pub nt_wait_for_single_object: NtWaitForSingleObject,
    pub nt_terminate_thread:       NtTerminateThread,

    // Token Management functions
    pub nt_query_information_token: NtQueryInformationToken,
    pub nt_adjust_privileges_token: NtAdjustPrivilegesToken,
    pub nt_open_process_token:      NtOpenProcessToken,
    pub nt_open_process_token_ex:   NtOpenProcessTokenEx,

    pub nt_close:                    NtClose,
    pub nt_query_system_information: NtQuerySystemInformation,
    pub nt_delay_execution:          NtDelayExecution,
    pub nt_create_named_pipe_file:   NtCreateNamedPipeFile,
}

impl NtDll {
    pub fn new() -> Self {
        NtDll {
            module_base: null_mut(),

            // Direct Syscall
            ldr_load_dll: unsafe { core::mem::transmute(null_mut::<c_void>()) },
            rtl_create_process_parameters_ex: unsafe { core::mem::transmute(null_mut::<c_void>()) },
            rtl_get_full_path_name_u: unsafe { core::mem::transmute(null_mut::<c_void>()) },
            rtl_get_full_path_name_ustrex: unsafe { core::mem::transmute(null_mut::<c_void>()) },
            rtl_dos_path_name_to_nt_path_name_u: unsafe { core::mem::transmute(null_mut::<c_void>()) },

            // Heap management functions
            rtl_create_heap:     unsafe { core::mem::transmute(null_mut::<c_void>()) },
            rtl_allocate_heap:   unsafe { core::mem::transmute(null_mut::<c_void>()) },
            rtl_free_heap:       unsafe { core::mem::transmute(null_mut::<c_void>()) },
            rtl_reallocate_heap: unsafe { core::mem::transmute(null_mut::<c_void>()) },
            rtl_destroy_heap:    unsafe { core::mem::transmute(null_mut::<c_void>()) },

            // Process Management functions
            nt_create_process:            NtCreateProcess::new(),
            nt_create_process_ex:         NtCreateProcessEx::new(), // unused
            nt_create_user_process:       NtCreateUserProcess::new(),
            nt_terminate_process:         NtTerminateProcess::new(),
            nt_open_process:              NtOpenProcess::new(),
            nt_query_information_process: NtQueryInformationProcess::new(),

            // File Management functions
            nt_open_file:   NtOpenFile::new(),
            nt_write_file:  NtWriteFile::new(),
            nt_create_file: NtCreateFile::new(),
            nt_read_file:   NtReadFile::new(),

            // Registry Management functions
            nt_open_key:        NtOpenKey::new(),
            nt_query_value_key: NtQueryValueKey::new(),
            nt_enumerate_key:   NtEnumerateKey::new(),

            // Memory Management functions
            nt_allocate_virtual_memory: NtAllocateVirtualMemory::new(),
            nt_free_virtual_memory:     NtFreeVirtualMemory::new(),
            nt_read_virtual_memory:     NtReadVirtualMemory::new(),
            nt_write_virtual_memory:    NtWriteVirtualMemory::new(), // unused

            // Thread Management functions
            nt_create_thread:          NtCreateThread::new(),
            nt_create_thread_ex:       NtCreateThreadEx::new(),
            nt_zw_create_thread_ex:    ZwCreateThreadEx::new(),
            nt_resume_thread:          NtResumeThread::new(), // unused
            nt_wait_for_single_object: NtWaitForSingleObject::new(),
            nt_terminate_thread:       NtTerminateThread::new(),

            // Token Management functions
            nt_query_information_token: NtQueryInformationToken::new(),
            nt_adjust_privileges_token: NtAdjustPrivilegesToken::new(), // unused, untested
            nt_open_process_token:      NtOpenProcessToken::new(),
            nt_open_process_token_ex:   NtOpenProcessTokenEx::new(),

            nt_close:                    NtClose::new(),
            nt_query_system_information: NtQuerySystemInformation::new(),
            nt_delay_execution:          NtDelayExecution::new(),
            nt_create_named_pipe_file:   NtCreateNamedPipeFile::new(), // untested
        }
    }
}

unsafe impl Sync for NtDll {}
unsafe impl Send for NtDll {}
