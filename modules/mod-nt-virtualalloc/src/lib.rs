#![no_std]

use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::ffi::c_void;
use core::ptr::null_mut;
use core::sync::atomic::Ordering;
use core::sync::atomic::{AtomicBool, AtomicIsize};
use spin::Mutex;

use mod_agentcore::ldr::{ldr_function_addr, ldr_module_peb};
use mod_hhtgates::get_syscall_number;

use rs2_indirect_syscall::run_syscall;
use rs2_win32::ntapi::NtSyscall;

// Atomic flag contains the last status of an NT syscall.
pub static NT_ALLOCATOR_STATUS: AtomicIsize = AtomicIsize::new(0);

// Atomic flag to ensure initialization happens only once.
static INIT: AtomicBool = AtomicBool::new(false);

// Static variables to hold the configuration and syscall information, wrapped in UnsafeCell for interior mutability.
static mut NT_ALLOCATE_VIRTUAL_MEMORY_SYSCALL: Mutex<UnsafeCell<Option<NtSyscall>>> =
    Mutex::new(UnsafeCell::new(None));

static mut NT_FREE_VIRTUAL_MEMORY_SYSCALL: Mutex<UnsafeCell<Option<NtSyscall>>> =
    Mutex::new(UnsafeCell::new(None));

pub const NTDLL_HASH: u32 = 0x1edab0ed;
pub const NT_ALLOCATE_VIRTUAL_MEMORY_DBJ2: usize = 0xf783b8ec;
pub const NT_FREE_VIRTUAL_MEMORY_DBJ2: usize = 0x2802c609;

/// Unsafe function to perform the initialization of the static variables.
/// This includes locating and storing the addresses and syscall numbers for `NtAllocateVirtualMemory` and `NtFreeVirtualMemory`.
pub unsafe fn initialize() {
    // Check if initialization has already occurred.
    if !INIT.load(Ordering::Acquire) {
        // Get the address of ntdll module in memory.
        let ntdll_address = ldr_module_peb(NTDLL_HASH);

        // Initialize the syscall for NtAllocateVirtualMemory.
        let alloc_syscall_address =
            ldr_function_addr(ntdll_address, NT_ALLOCATE_VIRTUAL_MEMORY_DBJ2);
        let alloc_syscall = NtSyscall {
            address: alloc_syscall_address,
            number: get_syscall_number(alloc_syscall_address),
            hash: NT_ALLOCATE_VIRTUAL_MEMORY_DBJ2,
        };

        *NT_ALLOCATE_VIRTUAL_MEMORY_SYSCALL.lock().get() = Some(alloc_syscall);

        // Initialize the syscall for NtFreeVirtualMemory.
        let free_syscall_address = ldr_function_addr(ntdll_address, NT_FREE_VIRTUAL_MEMORY_DBJ2);
        let free_syscall = NtSyscall {
            address: free_syscall_address,
            number: get_syscall_number(free_syscall_address),
            hash: NT_FREE_VIRTUAL_MEMORY_DBJ2,
        };

        *NT_FREE_VIRTUAL_MEMORY_SYSCALL.lock().get() = Some(free_syscall);

        // Set the initialization flag to true.
        INIT.store(true, Ordering::Release);
    }
}

/// Function to ensure that initialization is performed if it hasn't been already.
fn ensure_initialized() {
    unsafe {
        // Check and call initialize if not already done.
        if !INIT.load(Ordering::Acquire) {
            initialize();
        }
    }
}

/// Function to get a reference to the NtAllocateVirtualMemory syscall, ensuring initialization first.
fn get_nt_allocate_virtual_memory_syscall() -> &'static NtSyscall {
    ensure_initialized();
    unsafe {
        NT_ALLOCATE_VIRTUAL_MEMORY_SYSCALL
            .lock()
            .get()
            .as_ref()
            .unwrap()
            .as_ref()
            .unwrap()
    }
}

/// Function to get a reference to the NtFreeVirtualMemory syscall, ensuring initialization first.
fn get_nt_free_virtual_memory_syscall() -> &'static NtSyscall {
    ensure_initialized();
    unsafe {
        NT_FREE_VIRTUAL_MEMORY_SYSCALL
            .lock()
            .get()
            .as_ref()
            .unwrap()
            .as_ref()
            .unwrap()
    }
}

/// Custom allocator using NT system calls.
pub struct NtVirtualAlloc;

unsafe impl GlobalAlloc for NtVirtualAlloc {
    /// Allocates memory as described by the given `layout` using NT system calls.
    ///
    /// This function uses the `NtAllocateVirtualMemory` system call to allocate memory.
    /// The memory is allocated with `PAGE_READWRITE` protection, which allows both
    /// reading and writing. This is appropriate for most use cases like vectors and strings.
    ///
    /// # Safety
    ///
    /// This function is unsafe because undefined behavior can result
    /// if the caller does not ensure that `layout` has non-zero size.
    ///
    /// The allocated block of memory may or may not be initialized.
    ///
    /// # Returns
    ///
    /// A pointer to newly-allocated memory, or null to indicate allocation failure.
    ///
    /// # Errors
    ///
    /// Returning a null pointer indicates that either memory is exhausted
    /// or `layout` does not meet this allocator's size or alignment constraints.
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // Pointer to the allocated memory.
        let mut p_address: *mut c_void = null_mut();
        // Size of the memory to allocate.
        let region_size = layout.size();
        // Handle to the current process (-1).
        let h_process: *mut u8 = -1isize as _;

        // Retrieve the syscall information.
        let alloc_syscall = get_nt_allocate_virtual_memory_syscall();

        // Perform the system call to allocate virtual memory.
        let ntstatus = run_syscall!(
            (*alloc_syscall).number,
            (*alloc_syscall).address as usize,
            h_process,
            &mut p_address,
            0,
            &mut (region_size as usize) as *mut usize,
            0x3000, // MEM_COMMIT | MEM_RESERVE
            0x04    // PAGE_READWRITE
        );

        NT_ALLOCATOR_STATUS.store(ntstatus as isize, Ordering::SeqCst);

        // If the allocation fails, return null; otherwise, return the allocated address.
        p_address as *mut u8
    }

    /// Deallocates the block of memory at the given `ptr` pointer with the given `layout` using NT system calls.
    ///
    /// This function uses the `NtFreeVirtualMemory` system call to deallocate memory.
    ///
    /// # Safety
    ///
    /// This function is unsafe because undefined behavior can result
    /// if the caller does not ensure all the following:
    ///
    /// * `ptr` must denote a block of memory currently allocated via
    ///   this allocator,
    ///
    /// * `layout` must be the same layout that was used
    ///   to allocate that block of memory.
    ///
    /// Note: `NtFreeVirtualMemory` will deallocate memory in multiples of the page size (usually 4096 bytes).
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // Size of the memory to deallocate.
        let mut region_size = layout.size();
        // Handle to the current process (-1).
        let h_process: *mut u8 = -1isize as _;

        // Retrieve the syscall information.
        let free_syscall = get_nt_free_virtual_memory_syscall();

        // Perform the system call to free virtual memory.
        let ntstatus = run_syscall!(
            (*free_syscall).number,
            (*free_syscall).address as usize,
            h_process,
            &mut (ptr as *mut c_void),
            &mut region_size,
            0x8000 // MEM_RELEASE
        );

        NT_ALLOCATOR_STATUS.store(ntstatus as isize, Ordering::SeqCst);
    }
}
