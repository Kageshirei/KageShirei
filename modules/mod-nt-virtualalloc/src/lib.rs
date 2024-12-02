#![no_std]
//! # nt_virtual_alloc
//!
//! This crate provides a custom memory allocator for `no_std` environments using Windows NT system
//! calls. It implements the `GlobalAlloc` trait, leveraging low-level Windows APIs such as
//! `NtAllocateVirtualMemory` and `NtFreeVirtualMemory` for memory allocation and deallocation. This
//! enables efficient and direct control of memory in constrained or specialized environments.
//!
//! ## Features
//! - **Custom Memory Allocator:** Provides a `GlobalAlloc` implementation using the Windows NT
//!   Virtual Memory API.
//! - **Low-Level Control:** Uses `kageshirei_indirect_syscall` to call system functions indirectly
//!   for enhanced stealth and flexibility.
//! - **Thread Safety:** Ensures safe usage of system resources with atomic initialization and mutex
//!   protection.
//!
//! ## Examples
//!
//! ### Allocating and Deallocating Memory
//! ```rust ignore
//! use core::alloc::Layout;
//!
//! use nt_virtual_alloc::NtVirtualAlloc;
//!
//! // Example of a global allocator
//! #[global_allocator]
//! static GLOBAL_ALLOCATOR: NtVirtualAlloc = NtVirtualAlloc;
//!
//! fn main() {
//!     let layout = Layout::from_size_align(1024, 8).unwrap();
//!
//!     unsafe {
//!         let ptr = GLOBAL_ALLOCATOR.alloc(layout);
//!         assert!(!ptr.is_null(), "Allocation failed");
//!
//!         // Use the allocated memory...
//!
//!         GLOBAL_ALLOCATOR.dealloc(ptr, layout);
//!     }
//! }
//! ```
//!
//! ## Safety
//! The crate interacts directly with low-level Windows system calls and includes unsafe operations,
//! such as:
//! - Raw pointer manipulations
//! - System resource management
//! - Indirect syscall handling
use core::{
    alloc::{GlobalAlloc, Layout},
    cell::UnsafeCell,
    ffi::c_void,
    ptr::null_mut,
    sync::atomic::{AtomicBool, AtomicIsize, Ordering},
};

use kageshirei_indirect_syscall::run;
use mod_agentcore::ldr::{peb_get_function_addr, peb_get_module};
use mod_hhtgates::get_syscall_number;
use spin::RwLock;

/// Structure to hold information about an NT syscall.
pub struct NtAllocSyscall {
    /// Address of the syscall function.
    pub address: *mut u8,
    /// Number of the syscall.
    pub number:  u16,
    /// Hash of the syscall function name.
    pub hash:    usize,
}

/// Atomic flag contains the last status of an NT syscall.
pub static NT_ALLOCATOR_STATUS: AtomicIsize = AtomicIsize::new(0);

/// Atomic flag to ensure initialization happens only once.
static INIT: AtomicBool = AtomicBool::new(false);

/// Static variables to hold the configuration and syscall information, wrapped in UnsafeCell for
/// interior mutability.
static mut NT_ALLOCATE_VIRTUAL_MEMORY_SYSCALL: RwLock<UnsafeCell<Option<NtAllocSyscall>>> =
    RwLock::new(UnsafeCell::new(None));

/// Static variables to hold the configuration and syscall information, wrapped in UnsafeCell for
/// interior mutability.
static mut NT_FREE_VIRTUAL_MEMORY_SYSCALL: RwLock<UnsafeCell<Option<NtAllocSyscall>>> =
    RwLock::new(UnsafeCell::new(None));

/// Unsafe function to perform the initialization of the static variables.
/// This includes locating and storing the addresses and syscall numbers for
/// `NtAllocateVirtualMemory` and `NtFreeVirtualMemory`.
///
/// # Safety
///
/// This function is unsafe because it performs memory operations that can lead to undefined
/// behavior if not handled correctly.
pub unsafe fn initialize() {
    // Check if initialization has already occurred.
    if !INIT.load(Ordering::Acquire) {
        // Get the address of ntdll module in memory.
        let ntdll_address = peb_get_module(0x1edab0ed);

        // Initialize the syscall for NtAllocateVirtualMemory.
        let alloc_syscall_address = peb_get_function_addr(ntdll_address, 0xf783b8ec);
        let alloc_syscall = NtAllocSyscall {
            address: alloc_syscall_address,
            number:  get_syscall_number(alloc_syscall_address),
            hash:    0xf783b8ec,
        };

        #[expect(
            static_mut_refs,
            reason = "This is a controlled access to a mutable static using a RwLock, ensuring that only one thread \
                      can write at a time and preventing data races."
        )]
        let nt_allocate_virtual_memory_lock = NT_ALLOCATE_VIRTUAL_MEMORY_SYSCALL.write();
        *nt_allocate_virtual_memory_lock.get() = Some(alloc_syscall);
        // *NT_ALLOCATE_VIRTUAL_MEMORY_SYSCALL.lock().get() = Some(alloc_syscall);

        // Initialize the syscall for NtFreeVirtualMemory.
        let free_syscall_address = peb_get_function_addr(ntdll_address, 0x2802c609);
        let free_syscall = NtAllocSyscall {
            address: free_syscall_address,
            number:  get_syscall_number(free_syscall_address),
            hash:    0x2802c609,
        };

        #[expect(
            static_mut_refs,
            reason = "This is a controlled access to a mutable static using a RwLock, ensuring that only one thread \
                      can write at a time and preventing data races."
        )]
        let nt_free_virtual_memory_lock = NT_FREE_VIRTUAL_MEMORY_SYSCALL.write();
        *nt_free_virtual_memory_lock.get() = Some(free_syscall);
        // *NT_FREE_VIRTUAL_MEMORY_SYSCALL.lock().get() = Some(free_syscall);

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

/// Function to get a reference to the NtAllocateVirtualMemory syscall, ensuring initialization
/// first.
///
/// # Safety
///
/// This function is unsafe because it involves mutable static data.
/// The caller must ensure no data races occur when accessing the global instance.
#[expect(
    static_mut_refs,
    reason = "Access to mutable static data is protected by a RwLock, ensuring shared references are safe and \
              preventing data races."
)]
unsafe fn get_nt_allocate_virtual_memory_syscall() -> &'static NtAllocSyscall {
    ensure_initialized();
    let lock = NT_ALLOCATE_VIRTUAL_MEMORY_SYSCALL.read();
    (*lock.get()).as_ref().unwrap()
}

/// Function to get a reference to the NtFreeVirtualMemory syscall, ensuring initialization first.
///
/// # Safety
///
/// This function is unsafe because it involves mutable static data.
/// The caller must ensure no data races occur when accessing the global instance.
#[expect(
    static_mut_refs,
    reason = "Access to mutable static data is protected by a RwLock, ensuring shared references are safe and \
              preventing data races."
)]
unsafe fn get_nt_free_virtual_memory_syscall() -> &'static NtAllocSyscall {
    ensure_initialized();
    let lock = NT_FREE_VIRTUAL_MEMORY_SYSCALL.read();
    (*lock.get()).as_ref().unwrap()
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
        let h_process: *mut u8 = -1isize as *mut u8;

        // Retrieve the syscall information.
        let alloc_syscall = get_nt_allocate_virtual_memory_syscall();

        // Perform the system call to allocate virtual memory.
        let ntstatus = run!(
            alloc_syscall.number,
            alloc_syscall.address as usize,
            h_process,
            &mut p_address,
            0,
            &mut { region_size } as *mut usize,
            0x3000, // MEM_COMMIT | MEM_RESERVE
            0x04    // PAGE_READWRITE
        );

        NT_ALLOCATOR_STATUS.store(ntstatus as isize, Ordering::SeqCst);

        // If the allocation fails, return null; otherwise, return the allocated address.
        p_address as *mut u8
    }

    /// Deallocates the block of memory at the given `ptr` pointer with the given `layout` using NT
    /// system calls.
    ///
    /// This function uses the `NtFreeVirtualMemory` system call to deallocate memory.
    ///
    /// # Safety
    ///
    /// This function is unsafe because undefined behavior can result
    /// if the caller does not ensure all the following:
    ///
    /// * `ptr` must denote a block of memory currently allocated via this allocator,
    ///
    /// * `layout` must be the same layout that was used to allocate that block of memory.
    ///
    /// Note: `NtFreeVirtualMemory` will deallocate memory in multiples of the page size (usually
    /// 4096 bytes).
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // Size of the memory to deallocate.
        let mut region_size = layout.size();
        // Handle to the current process (-1).
        let h_process: *mut u8 = -1isize as *mut u8;

        // Retrieve the syscall information.
        let free_syscall = get_nt_free_virtual_memory_syscall();

        // Perform the system call to free virtual memory.
        let ntstatus = run!(
            free_syscall.number,
            free_syscall.address as usize,
            h_process,
            &mut (ptr as *mut c_void),
            &mut region_size,
            0x8000 // MEM_RELEASE
        );

        NT_ALLOCATOR_STATUS.store(ntstatus as isize, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;
    use alloc::{boxed::Box, string::String, vec::Vec};
    use core::{ptr::null_mut, slice};

    use super::*;

    static GLOBAL: NtVirtualAlloc = NtVirtualAlloc;

    /// Test to check memory allocation and deallocation using `alloc` and `dealloc`.
    #[test]
    fn test_alloc_dealloc() {
        let layout = Layout::from_size_align(1024, 8).unwrap();

        unsafe {
            // Allocate 1024 bytes of memory
            let ptr = GLOBAL.alloc(layout);
            assert_ne!(ptr, null_mut(), "Allocation failed");

            // Deallocate the memory
            GLOBAL.dealloc(ptr, layout);
        }
    }

    /// Test to check zeroed memory allocation using `alloc_zeroed`.
    #[test]
    fn test_alloc_zeroed() {
        let layout = Layout::from_size_align(512, 8).unwrap();

        unsafe {
            // Allocate 512 bytes of zeroed memory
            let ptr = GLOBAL.alloc_zeroed(layout);
            assert_ne!(ptr, null_mut(), "Zeroed allocation failed");

            // Verify that the memory is actually zeroed
            let data = slice::from_raw_parts(ptr, 512);
            for &byte in data {
                assert_eq!(byte, 0, "Memory not zeroed");
            }

            // Deallocate the memory
            GLOBAL.dealloc(ptr, layout);
        }
    }

    /// Test to check memory reallocation using `realloc`.
    #[test]
    fn test_realloc() {
        let initial_layout = Layout::from_size_align(256, 8).unwrap();
        let new_size = 512;

        unsafe {
            // Initial allocation of 256 bytes
            let ptr = GLOBAL.alloc(initial_layout);
            assert_ne!(ptr, null_mut(), "Initial allocation failed");

            // Reallocate the memory to 512 bytes
            let new_ptr = GLOBAL.realloc(ptr, initial_layout, new_size);
            assert_ne!(new_ptr, null_mut(), "Reallocation failed");

            // Deallocate the memory
            let new_layout = Layout::from_size_align(new_size, 8).unwrap();
            GLOBAL.dealloc(new_ptr, new_layout);
        }
    }

    /// Test to check memory allocation and deallocation using a `Vec`.
    #[test]
    fn test_vec_allocation() {
        // Test Vec allocation and deallocation
        let mut vec: Vec<i32> = Vec::new();
        for i in 0 .. 10 {
            vec.push(i);
        }

        // Verify the contents of the vector
        for (i, &value) in vec.iter().enumerate() {
            assert_eq!(value, i as i32, "Vec contains incorrect value");
        }

        // Deallocation is automatic when the vector goes out of scope
    }

    /// Test to check memory allocation and deallocation using a `String`.
    #[test]
    fn test_string_allocation() {
        // Test String allocation and deallocation
        let mut string = String::from("Hello, ");
        string.push_str("world!");

        // Verify the contents of the string
        assert_eq!(string, "Hello, world!", "String contains incorrect value");

        // Deallocation is automatic when the string goes out of scope
    }

    /// Test to check memory allocation and deallocation using a `Box`.
    #[test]
    fn test_box_allocation() {
        // Test Box allocation and deallocation
        let boxed_value = Box::new(42);

        // Verify the value
        assert_eq!(*boxed_value, 42, "Box contains incorrect value");

        // Deallocation is automatic when the Box goes out of scope
    }
}
