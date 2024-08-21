use core::{
    ffi::{c_uchar, c_void},
    ptr::null_mut,
};

use crate::ntdef::{
    AccessMask, IoStatusBlock, ObjectAttributes, PEventType, TokenPrivileges, UnicodeString,
    HANDLE, ULONG,
};
use rs2_indirect_syscall::run_syscall;

pub struct NtSyscall {
    /// The number of the syscall
    pub number: u16,
    /// The address of the syscall
    pub address: *mut u8,
    /// The hash of the syscall (used for lookup)
    pub hash: usize,
}

/// We implement Sync for NtSyscall to ensure that it can be safely shared
/// across multiple threads. This is necessary because lazy_static requires
/// the types it manages to be Sync. Since NtSyscall only contains raw pointers
/// and does not perform any interior mutability, it is safe to implement Sync manually.
unsafe impl Sync for NtSyscall {}

impl NtSyscall {
    pub const fn new() -> Self {
        NtSyscall {
            number: 0,
            address: null_mut(),
            hash: 0,
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
pub unsafe fn nt_current_process() -> HANDLE {
    -1isize as HANDLE
}

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
    /// * `handle` - The handle to be closed.
    ///
    /// # Returns
    ///
    /// * `true` if the operation was successful, `false` otherwise.
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
    /// * `process_handle` - A handle to the process whose memory is to be written to.
    /// * `base_address` - A pointer to the base address in the process's virtual memory.
    /// * `buffer` - A pointer to the buffer that contains the data to be written.
    /// * `buffer_size` - The size, in bytes, of the buffer pointed to by the `buffer` parameter.
    /// * `number_of_bytes_written` - A pointer to a variable that receives the number of bytes written.
    ///
    /// # Returns
    ///
    /// * `i32` - The NTSTATUS code of the operation.
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
            free_type // 0x8000 // MEM_RELEASE
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
    /// # Arguments
    ///
    /// * `p_key_handle` - A mutable pointer to a handle that will receive the key handle.
    /// * `desired_access` - The desired access for the key.
    /// * `object_attributes` - A pointer to the object attributes structure.
    ///
    /// # Returns
    ///
    /// * `i32` - The NTSTATUS code of the operation.
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
    /// * `key_handle` - A handle to the key.
    /// * `value_name` - A pointer to the UnicodeString structure containing the name of the value to be queried.
    /// * `key_value_information_class` - Specifies the type of information to be returned.
    /// * `key_value_information` - A pointer to a buffer that receives the requested information.
    /// * `length` - The size, in bytes, of the buffer pointed to by the `key_value_information` parameter.
    /// * `result_length` - A pointer to a variable that receives the size, in bytes, of the data returned.
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
    /// * `key_handle` - A handle to the key.
    /// * `index` - The index of the subkey to be enumerated.
    /// * `key_information_class` - Specifies the type of information to be returned.
    /// * `key_information` - A pointer to a buffer that receives the requested information.
    /// * `length` - The size, in bytes, of the buffer pointed to by the `key_information` parameter.
    /// * `result_length` - A pointer to a variable that receives the size, in bytes, of the data returned.
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
    /// * `system_information_class` - The system information class to be queried.
    /// * `system_information` - A pointer to a buffer that receives the requested information.
    /// * `system_information_length` - The size, in bytes, of the buffer pointed to by the `system_information` parameter.
    /// * `return_length` - A pointer to a variable that receives the size, in bytes, of the data returned.
    ///
    /// # Returns
    ///
    /// * `NTSTATUS` - The NTSTATUS code of the operation.
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
    /// * `process_handle` - A handle to the process.
    /// * `process_information_class` - The class of information to be queried.
    /// * `process_information` - A pointer to a buffer that receives the requested information.
    /// * `process_information_length` - The size, in bytes, of the buffer pointed to by the `process_information` parameter.
    /// * `return_length` - A pointer to a variable that receives the size, in bytes, of the data returned.
    ///
    /// # Returns
    ///
    /// * `NTSTATUS` - The NTSTATUS code of the operation.
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
    /// * `process_handle` - A mutable pointer to a handle that will receive the process handle.
    /// * `desired_access` - The desired access for the process.
    /// * `object_attributes` - A pointer to the object attributes structure.
    /// * `client_id` - A pointer to the client ID structure.
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
    /// * `process_handle` - The handle of the process whose token is to be opened.
    /// * `desired_access` - The desired access for the token.
    /// * `token_handle` - A mutable pointer to a handle that will receive the token handle.
    ///
    /// # Returns
    ///
    /// * `i32` - The NTSTATUS code of the operation.
    pub fn run(
        &self,
        process_handle: HANDLE,
        desired_access: AccessMask,
        token_handle: &mut HANDLE,
    ) -> i32 {
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
    /// * `process_handle` - The handle of the process whose token is to be opened.
    /// * `desired_access` - The desired access for the token.
    /// * `handle_attributes` - Attributes for the handle.
    /// * `token_handle` - A mutable pointer to a handle that will receive the token handle.
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
    /// * `token_handle` - The handle of the token to be queried.
    /// * `token_information_class` - The class of information to be queried.
    /// * `token_information` - A pointer to a buffer that receives the requested information.
    /// * `token_information_length` - The size, in bytes, of the buffer pointed to by the `token_information` parameter.
    /// * `return_length` - A pointer to a variable that receives the size, in bytes, of the data returned.
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
    /// * `token_handle` - The handle of the token to be adjusted.
    /// * `disable_all_privileges` - Boolean to disable all privileges.
    /// * `new_state` - A pointer to a TOKEN_PRIVILEGES structure.
    /// * `buffer_length` - The length of the buffer for previous privileges.
    /// * `previous_state` - A pointer to a buffer that receives the previous state.
    /// * `return_length` - A pointer to a variable that receives the length of the previous state.
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
    /// * `handle` - A handle to the object.
    /// * `alertable` - A boolean value that specifies whether the wait is alertable.
    /// * `timeout` - An optional pointer to a time-out value.
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
    /// * `file_handle` - A pointer to a handle that receives the file handle.
    /// * `desired_access` - The desired access for the file handle.
    /// * `object_attributes` - A pointer to the OBJECT_ATTRIBUTES structure.
    /// * `io_status_block` - A pointer to an IO_STATUS_BLOCK structure that receives the status block.
    /// * `share_access` - The requested share access for the file.
    /// * `open_options` - The options to be applied when opening the file.
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
    /// * `event_handle` - A mutable pointer to a handle that will receive the event handle.
    /// * `desired_access` - The desired access for the event.
    /// * `object_attributes` - A pointer to the object attributes structure. This can be null.
    /// * `event_type` - The type of event to be created.
    /// * `initial_state` - The initial state of the event.
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
    /// * `file_handle` - A handle to the file or I/O device to be written to.
    /// * `event` - An optional handle to an event object that will be signaled when the operation completes.
    /// * `apc_routine` - An optional pointer to an APC routine to be called when the operation completes.
    /// * `apc_context` - An optional pointer to a context for the APC routine.
    /// * `io_status_block` - A pointer to an IO_STATUS_BLOCK structure that receives the final completion status and information about the operation.
    /// * `buffer` - A pointer to a buffer that contains the data to be written to the file or device.
    /// * `length` - The length, in bytes, of the buffer pointed to by the `buffer` parameter.
    /// * `byte_offset` - A pointer to the byte offset in the file where the operation should begin. If this parameter is `None`, the system writes data to the current file position.
    /// * `key` - A pointer to a caller-supplied variable to receive the I/O completion key. This parameter is ignored if `event` is not `None`.
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
    /// * `file_handle` - A mutable pointer to a handle that will receive the file handle.
    /// * `desired_access` - The access to the file or device, which can be read, write, or both.
    /// * `obj_attributes` - A pointer to an OBJECT_ATTRIBUTES structure that specifies the object name and other attributes.
    /// * `io_status_block` - A pointer to an IO_STATUS_BLOCK structure that receives the final completion status and information about the operation.
    /// * `allocation_size` - A pointer to a LARGE_INTEGER that specifies the initial allocation size in bytes. If this parameter is `None`, the file is allocated with a default size.
    /// * `file_attributes` - The file attributes for the file or device if it is created.
    /// * `share_access` - The requested sharing mode of the file or device.
    /// * `create_disposition` - The action to take depending on whether the file or device already exists.
    /// * `create_options` - Options to be applied when creating or opening the file or device.
    /// * `ea_buffer` - A pointer to a buffer that contains the extended attributes (EAs) for the file or device. This parameter is optional.
    /// * `ea_length` - The length, in bytes, of the EaBuffer parameter.
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
    /// * `file_handle` - A handle to the file or I/O device to be read from.
    /// * `event` - An optional handle to an event object that will be signaled when the operation completes.
    /// * `apc_routine` - An optional pointer to an APC routine to be called when the operation completes.
    /// * `apc_context` - An optional pointer to a context for the APC routine.
    /// * `io_status_block` - A pointer to an IO_STATUS_BLOCK structure that receives the final completion status and information about the operation.
    /// * `buffer` - A pointer to a buffer that receives the data read from the file or device.
    /// * `length` - The length, in bytes, of the buffer pointed to by the `buffer` parameter.
    /// * `byte_offset` - A pointer to the byte offset in the file where the operation should begin. If this parameter is `None`, the system reads data from the current file position.
    /// * `key` - A pointer to a caller-supplied variable to receive the I/O completion key. This parameter is ignored if `event` is not `None`.
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
    /// * `process_handle` - A mutable pointer to a handle that will receive the process handle.
    /// * `desired_access` - The desired access for the process.
    /// * `object_attributes` - A pointer to the object attributes structure.
    /// * `parent_process` - A handle to the parent process.
    /// * `flags` - Flags for creating the process.
    /// * `section_handle` - A handle to a section object.
    /// * `debug_port` - A handle to the debug port.
    /// * `token_handle` - A handle to the token.
    /// * `reserved` - Reserved for future use.
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
    /// * `thread_handle` - A mutable pointer to a handle that will receive the thread handle.
    /// * `desired_access` - The desired access for the thread.
    /// * `object_attributes` - A pointer to the object attributes structure.
    /// * `process_handle` - A handle to the process.
    /// * `start_routine` - A pointer to the start routine.
    /// * `argument` - A pointer to the argument for the start routine.
    /// * `create_flags` - Flags for creating the thread.
    /// * `zero_bits` - The zero bits.
    /// * `stack_size` - The stack size.
    /// * `maximum_stack_size` - The maximum stack size.
    /// * `attribute_list` - A pointer to an attribute list.
    ///
    /// # Returns
    ///
    /// * `i32` - The NTSTATUS code of the operation.
    pub fn run(
        &self,
        thread_handle: &mut HANDLE,
        desired_access: AccessMask,
        object_attributes: *mut ObjectAttributes,
        process_handle: HANDLE,
        start_routine: *mut c_void,
        argument: *mut c_void,
        create_flags: ULONG,
        zero_bits: ULONG,
        stack_size: ULONG,
        maximum_stack_size: ULONG,
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
    /// * `process_handle` - A mutable pointer to a handle that will receive the process handle.
    /// * `thread_handle` - A mutable pointer to a handle that will receive the thread handle.
    /// * `process_desired_access` - The desired access for the process.
    /// * `thread_desired_access` - The desired access for the thread.
    /// * `process_object_attributes` - A pointer to the process object attributes structure.
    /// * `thread_object_attributes` - A pointer to the thread object attributes structure.
    /// * `process_flags` - Flags for creating the process.
    /// * `thread_flags` - Flags for creating the thread.
    /// * `process_parameters` - A pointer to the process parameters structure.
    /// * `create_info` - A pointer to the create information structure.
    /// * `attribute_list` - A pointer to the attribute list structure.
    ///
    /// # Returns
    ///
    /// * `i32` - The NTSTATUS code of the operation.
    pub fn run(
        &self,
        process_handle: &mut HANDLE,
        thread_handle: &mut HANDLE,
        process_desired_access: AccessMask,
        thread_desired_access: AccessMask,
        process_object_attributes: *mut ObjectAttributes,
        thread_object_attributes: *mut ObjectAttributes,
        process_flags: ULONG,
        thread_flags: ULONG,
        process_parameters: *mut c_void,
        create_info: *mut c_void,
        attribute_list: *mut c_void,
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
    /// * `thread_handle` - A handle to the thread to be resumed.
    /// * `suspend_count` - A pointer to a variable that receives the previous suspend count.
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

    /// Wrapper for the NtTerminateThread syscall.
    ///
    /// This function terminates a thread. It wraps the NtTerminateThread syscall.
    ///
    /// # Arguments
    ///
    /// * `thread_handle` - A handle to the thread to be terminated.
    /// * `exit_status` - The exit status to be returned by the thread.
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
    /// * `alertable` - A boolean indicating whether the delay can be interrupted by an alertable wait state.
    /// * `delay_interval` - A pointer to the time interval for which execution is to be delayed.
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

// Type definition for loading DLL function
type LdrLoadDll = unsafe extern "system" fn(
    DllPath: *mut u16,
    DllCharacteristics: *mut u32,
    DllName: UnicodeString,
    DllHandle: *mut c_void,
) -> i32;

pub struct NtDll {
    pub module_base: *mut u8,
    pub ldr_load_dll: LdrLoadDll,
    pub nt_close: NtClose,
    pub nt_allocate_virtual_memory: NtAllocateVirtualMemory,
    pub nt_free_virtual_memory: NtFreeVirtualMemory,
    pub nt_open_key: NtOpenKey,
    pub nt_query_value_key: NtQueryValueKey,
    pub nt_enumerate_key: NtEnumerateKey,
    pub nt_query_system_information: NtQuerySystemInformation,
    pub nt_query_information_process: NtQueryInformationProcess,
    pub nt_open_process: NtOpenProcess,
    pub nt_open_process_token: NtOpenProcessToken,
    pub nt_open_process_token_ex: NtOpenProcessTokenEx,
    pub nt_query_information_token: NtQueryInformationToken,
    pub nt_adjust_privileges_token: NtAdjustPrivilegesToken,
    pub nt_wait_for_single_object: NtWaitForSingleObject,
    pub nt_open_file: NtOpenFile,
    pub nt_write_file: NtWriteFile,
    pub nt_create_file: NtCreateFile,
    pub nt_read_file: NtReadFile,
    pub nt_create_process_ex: NtCreateProcessEx,
    pub nt_create_thread_ex: NtCreateThreadEx,
    pub nt_create_user_process: NtCreateUserProcess,
    pub nt_write_virtual_memory: NtWriteVirtualMemory,
    pub nt_resume_thread: NtResumeThread,
    pub nt_terminate_thread: NtTerminateThread,
    pub nt_terminate_process: NtTerminateProcess,
    pub nt_delay_execution: NtDelayExecution,
}

impl NtDll {
    pub fn new() -> Self {
        NtDll {
            module_base: null_mut(),
            ldr_load_dll: unsafe { core::mem::transmute(null_mut::<c_void>()) },
            nt_close: NtClose::new(),
            nt_allocate_virtual_memory: NtAllocateVirtualMemory::new(),
            nt_free_virtual_memory: NtFreeVirtualMemory::new(),
            nt_open_key: NtOpenKey::new(),
            nt_query_value_key: NtQueryValueKey::new(),
            nt_enumerate_key: NtEnumerateKey::new(),
            nt_query_system_information: NtQuerySystemInformation::new(),
            nt_query_information_process: NtQueryInformationProcess::new(),
            nt_open_process: NtOpenProcess::new(),
            nt_open_process_token: NtOpenProcessToken::new(),
            nt_open_process_token_ex: NtOpenProcessTokenEx::new(),
            nt_query_information_token: NtQueryInformationToken::new(),
            nt_adjust_privileges_token: NtAdjustPrivilegesToken::new(), //unused, untested
            nt_wait_for_single_object: NtWaitForSingleObject::new(),    //unused, untested
            nt_open_file: NtOpenFile::new(),                            //unused, untested
            nt_write_file: NtWriteFile::new(),                          //unused
            nt_create_file: NtCreateFile::new(),                        //unused,
            nt_read_file: NtReadFile::new(),                            //unused, untested
            nt_create_process_ex: NtCreateProcessEx::new(),             //unused
            nt_create_thread_ex: NtCreateThreadEx::new(),               //unused, untested
            nt_create_user_process: NtCreateUserProcess::new(),         //unused, untested
            nt_write_virtual_memory: NtWriteVirtualMemory::new(),       //unused
            nt_resume_thread: NtResumeThread::new(),                    //unused
            nt_terminate_thread: NtTerminateThread::new(),
            nt_terminate_process: NtTerminateProcess::new(),
            nt_delay_execution: NtDelayExecution::new(),
        }
    }
}

unsafe impl Sync for NtDll {}
unsafe impl Send for NtDll {}
