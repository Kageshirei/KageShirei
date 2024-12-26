#![no_std]
//! # nt_heap_alloc
//!
//! This crate provides a custom memory allocator for `no_std` environments using the NT Heap API.
//! It implements the `GlobalAlloc` trait and relies on low-level Windows APIs such as
//! `RtlCreateHeap`, `RtlAllocateHeap`, and `RtlFreeHeap`. The allocator enables dynamic memory
//! management while maintaining a lightweight footprint suitable for specialized applications.
//!
//! ## Features
//! - **Custom Heap Management:** Provides a `GlobalAlloc` implementation using Windows NT Heap APIs.
//! - **Dynamic Allocation:** Supports memory allocation, reallocation, and deallocation with options for zeroed memory.
//! - **Thread Safety:** Ensures safe access and initialization of heap functions with atomic flags and RwLockes.
//!
//! ## Examples
//!
//! ### Allocating and Deallocating Memory
//! ```rust ignore
//! use core::alloc::Layout;
//!
//! use nt_heap_alloc::NT_HEAPALLOCATOR;
//!
//! fn main() {
//!     let layout = Layout::from_size_align(1024, 8).unwrap();
//!
//!     unsafe {
//!         let ptr = NT_HEAPALLOCATOR.alloc(layout);
//!         assert!(!ptr.is_null(), "Allocation failed");
//!
//!         // Use the allocated memory...
//!
//!         NT_HEAPALLOCATOR.dealloc(ptr, layout);
//!     }
//! }
//! ```
//!
//! ### Using a Global Allocator
//! ```rust ignore
//! use nt_heap_alloc::NT_HEAPALLOCATOR;
//!
//! fn main() {
//!     NT_HEAPALLOCATOR.initialize();
//!
//!     let boxed = Box::new(42);
//!     assert_eq!(*boxed, 42, "Box contains incorrect value");
//! }
//! ```
//!
//! ## Safety
//! The crate performs direct interactions with low-level Windows APIs and includes unsafe
//! operations, such as:
//! - Raw pointer manipulations
//! - Heap creation and destruction
//! - Memory management at the system level
//!
//! ## Testing
//! Includes comprehensive tests for allocation, deallocation, and reallocation to ensure the
//! allocator works as intended. The tests validate compatibility with various data structures like
//! `Vec`, `Box`, and `String`.
extern crate alloc;

pub mod nt_heapalloc_def;

use core::{
    alloc::{GlobalAlloc, Layout},
    arch::asm,
    cell::UnsafeCell,
    ptr::null_mut,
    sync::atomic::{AtomicBool, AtomicIsize, Ordering},
};

use kageshirei_win32::ntdef::{HANDLE, HEAP_GROWABLE, HEAP_ZERO_MEMORY};
use mod_agentcore::ldr::{ldr_function_addr, ldr_module_peb};
use nt_heapalloc_def::{
    RtlAllocateHeap,
    RtlCreateHeap,
    RtlDestroyHeap,
    RtlFreeHeap,
    RtlReAllocateHeap,
    NTDLL_HASH,
    RTL_ALLOCATE_HEAP_H,
    RTL_CREATE_HEAP_H,
    RTL_DESTROY_HEAP_H,
    RTL_FREE_HEAP_H,
    RTL_REALLOCATE_HEAP_H,
};
use spin::RwLock;

/// Atomic flag to ensure that the initialization of function pointers happens only once.
static INIT_NT_HEAPALLOC: AtomicBool = AtomicBool::new(false);

/// Holds a reference to the RtlCreateHeap function pointer.
static mut RTL_CREATE_HEAP: RwLock<UnsafeCell<Option<RtlCreateHeap>>> = RwLock::new(UnsafeCell::new(None));

/// Holds a reference to the RtlAllocateHeap function pointer.
static mut RTL_ALLOCATE_HEAP: RwLock<UnsafeCell<Option<RtlAllocateHeap>>> = RwLock::new(UnsafeCell::new(None));

/// Holds a reference to the RtlFreeHeap function pointer.
static mut RTL_FREE_HEAP: RwLock<UnsafeCell<Option<RtlFreeHeap>>> = RwLock::new(UnsafeCell::new(None));

/// Holds a reference to the RtlReAllocateHeap function pointer.
static mut RTL_REALLOCATE_HEAP: RwLock<UnsafeCell<Option<RtlReAllocateHeap>>> = RwLock::new(UnsafeCell::new(None));

/// Holds a reference to the RtlDestroyHeap function pointer.
static mut RTL_DESTROY_HEAP: RwLock<UnsafeCell<Option<RtlDestroyHeap>>> = RwLock::new(UnsafeCell::new(None));

/// Ensures that the heap-related function pointers are initialized.
/// If they have not been initialized, this function will call `initialize_nt_heapalloc_funcs` to resolve them.
fn ensure_nt_heapalloc_funcs_initialize() {
    // Check and call initialize if not already done.
    if !INIT_NT_HEAPALLOC.load(Ordering::Acquire) {
        initialize_nt_heapalloc_funcs();
    }
}

/// Initializes the function pointers by resolving the addresses of the heap-related functions
/// from ntdll.dll using the `ldr_function_addr` function. The functions are:
/// - `RtlCreateHeap`
/// - `RtlAllocateHeap`
/// - `RtlFreeHeap`
/// - `RtlReAllocateHeap`
/// - `RtlDestroyHeap`
#[expect(
    static_mut_refs,
    reason = "This is a controlled access to a mutable static using a RwLock, ensuring that only one thread can write \
              at a time and preventing data races."
)]
fn initialize_nt_heapalloc_funcs() {
    let ntdll_address = unsafe { ldr_module_peb(NTDLL_HASH) };

    unsafe {
        // Resolve RtlCreateHeap
        let rtl_create_heap_addr = peb_get_function_addr(ntdll_address, RTL_CREATE_HEAP_H);
        let rtl_create_heap_lock = RTL_CREATE_HEAP.write();
        *rtl_create_heap_lock.get() = Some(core::mem::transmute::<*mut u8, RtlCreateHeap>(
            rtl_create_heap_addr,
        ));

        // Resolve RtlAllocateHeap
        let rtl_allocate_heap_addr = peb_get_function_addr(ntdll_address, RTL_ALLOCATE_HEAP_H);
        let rtl_allocate_heap_lock = RTL_ALLOCATE_HEAP.write();
        *rtl_allocate_heap_lock.get() = Some(core::mem::transmute::<*mut u8, RtlAllocateHeap>(
            rtl_allocate_heap_addr,
        ));

        // Resolve RtlFreeHeap
        let rtl_free_heap_addr = peb_get_function_addr(ntdll_address, RTL_FREE_HEAP_H);
        let rtl_free_heap_lock = RTL_FREE_HEAP.write();
        *rtl_free_heap_lock.get() = Some(core::mem::transmute::<*mut u8, RtlFreeHeap>(
            rtl_free_heap_addr,
        ));

        // Resolve RtlReAllocateHeap
        let rtl_reallocate_heap_addr = peb_get_function_addr(ntdll_address, RTL_REALLOCATE_HEAP_H);
        let rtl_reallocate_heap_lock = RTL_REALLOCATE_HEAP.write();
        *rtl_reallocate_heap_lock.get() = Some(core::mem::transmute::<*mut u8, RtlReAllocateHeap>(
            rtl_reallocate_heap_addr,
        ));

        // Resolve RtlDestroyHeap
        let rtl_destroy_heap_addr = peb_get_function_addr(ntdll_address, RTL_DESTROY_HEAP_H);
        let rtl_destroy_heap_lock = RTL_DESTROY_HEAP.write();
        *rtl_destroy_heap_lock.get() = Some(core::mem::transmute::<*mut u8, RtlDestroyHeap>(
            rtl_destroy_heap_addr,
        ));
    }

    // Set the initialization flag to true.
    INIT_NT_HEAPALLOC.store(true, Ordering::Release);

    // Initialize the NT_HEAPALLOCATOR by creating a new heap.
    NT_HEAPALLOCATOR.initialize();
}

/// Function to get a reference to the `RtlCreateHeap` syscall, ensuring initialization first.
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
unsafe fn get_rtl_create_heap() -> &'static RtlCreateHeap {
    ensure_nt_heapalloc_funcs_initialize();
    let lock = RTL_CREATE_HEAP.read();
    (*lock.get()).as_ref().unwrap()
}

/// Function to get a reference to the `RtlAllocateHeap` syscall, ensuring initialization first.
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
unsafe fn get_rtl_allocate_heap() -> &'static RtlAllocateHeap {
    ensure_nt_heapalloc_funcs_initialize();
    let lock = RTL_ALLOCATE_HEAP.read();
    (*lock.get()).as_ref().unwrap()
}

/// Function to get a reference to the `RtlFreeHeap` syscall, ensuring initialization first.
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
unsafe fn get_rtl_free_heap() -> &'static RtlFreeHeap {
    ensure_nt_heapalloc_funcs_initialize();
    let lock = RTL_FREE_HEAP.read();
    (*lock.get()).as_ref().unwrap()
}

/// Function to get a reference to the `RtlReAllocateHeap` syscall, ensuring initialization first.
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
unsafe fn get_rtl_reallocate_heap() -> &'static RtlReAllocateHeap {
    ensure_nt_heapalloc_funcs_initialize();
    let lock = RTL_REALLOCATE_HEAP.read();
    (*lock.get()).as_ref().unwrap()
}

/// Function to get a reference to the `RtlDestroyHeap` syscall, ensuring initialization first.
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
unsafe fn get_rtl_destroy_heap() -> &'static RtlDestroyHeap {
    ensure_nt_heapalloc_funcs_initialize();
    let lock = RTL_DESTROY_HEAP.read();
    (*lock.get()).as_ref().unwrap()
}

/// Global allocator implementation using NT Heap API.
#[global_allocator]
#[link_section = ".data"]
pub static NT_HEAPALLOCATOR: NtHeapAlloc = NtHeapAlloc::new_uninitialized();

/// Struct representing a custom heap allocator using the NT Heap API.
pub struct NtHeapAlloc(AtomicIsize);

unsafe impl Send for NtHeapAlloc {}
unsafe impl Sync for NtHeapAlloc {}

/// Handles out-of-memory situations by triggering a crash.
#[no_mangle]
unsafe fn rust_oom() -> ! {
    asm!("ud2", options(noreturn));
}

impl NtHeapAlloc {
    /// Creates a new, uninitialized `NtHeapAlloc`.
    pub const fn new_uninitialized() -> NtHeapAlloc { NtHeapAlloc(AtomicIsize::new(0)) }

    /// Retrieves the raw handle to the heap managed by this allocator.
    #[inline]
    fn raw_handle(&self) -> HANDLE { self.0.load(Ordering::Relaxed) as _ }

    /// Initializes the heap by calling `RtlCreateHeap` and storing the resulting handle.
    #[inline]
    pub fn initialize(&self) {
        let hh = unsafe { get_rtl_create_heap()(HEAP_GROWABLE, null_mut(), 0, 0, null_mut(), null_mut()) };
        self.0.store(hh as _, Ordering::SeqCst);
    }

    /// Checks if the allocator has been initialized.
    #[inline]
    pub fn is_initialized(&self) -> bool { self.0.load(Ordering::Relaxed) != 0 }

    /// Initializes the allocator if it has not been initialized yet.
    pub unsafe fn init_if_required(&self) {
        if !self.is_initialized() {
            self.initialize();
        }
    }

    /// Destroys the allocator via `RtlDestroyHeap`.
    /// # Safety
    /// This will render all underlying allocations invalid.
    #[inline]
    pub unsafe fn destroy(&self) {
        if self.is_initialized() {
            get_rtl_destroy_heap()(self.0.swap(0, Ordering::SeqCst) as _);
        }
    }
}

/// Implementation of the `GlobalAlloc` trait for `NtHeapAlloc`,
/// using the NT Heap API for memory management.
unsafe impl GlobalAlloc for NtHeapAlloc {
    /// Allocates a block of memory with the specified layout.
    ///
    /// # Arguments
    /// * `layout` - A `Layout` object that specifies the size and alignment of the desired memory block.
    ///
    /// # Returns
    /// * A pointer to the allocated memory block. Returns `null_mut()` if the allocation fails.
    ///
    /// # Safety
    /// This function is marked as `unsafe` because it directly interacts with low-level memory management,
    /// which can lead to undefined behavior if not handled correctly.
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // Use the `RtlAllocateHeap` function to allocate memory from the heap.
        // The function takes the heap handle, allocation flags (set to 0 here), and the size of the memory block.
        get_rtl_allocate_heap()(self.raw_handle(), 0, layout.size())
    }

    /// Allocates a block of zeroed memory with the specified layout.
    ///
    /// # Arguments
    /// * `layout` - A `Layout` object that specifies the size and alignment of the desired memory block.
    ///
    /// # Returns
    /// * A pointer to the allocated memory block, which is initialized to zero. Returns `null_mut()` if the allocation
    ///   fails.
    ///
    /// # Safety
    /// This function is marked as `unsafe` because it directly interacts with low-level memory management,
    /// which can lead to undefined behavior if not handled correctly.
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        // Use the `RtlAllocateHeap` function to allocate zeroed memory from the heap.
        // The `HEAP_ZERO_MEMORY` flag ensures that the allocated memory is set to zero.
        get_rtl_allocate_heap()(self.raw_handle(), HEAP_ZERO_MEMORY, layout.size())
    }

    /// Deallocates a previously allocated block of memory.
    ///
    /// # Arguments
    /// * `ptr` - A pointer to the memory block to be deallocated.
    /// * `_layout` - The layout of the memory block. Although it's passed in, it's not used directly in this function.
    ///
    /// # Safety
    /// This function is marked as `unsafe` because it directly interacts with low-level memory management,
    /// which can lead to undefined behavior if not handled correctly. The caller must ensure that the pointer
    /// was previously allocated by this allocator and that it is not used after deallocation.
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        // Use the `RtlFreeHeap` function to free the memory block.
        // The function takes the heap handle, flags (set to 0 here), and the pointer to the memory block.
        get_rtl_free_heap()(self.raw_handle(), 0, ptr);
    }

    /// Reallocates a previously allocated block of memory, changing its size.
    ///
    /// # Arguments
    /// * `ptr` - A pointer to the memory block to be reallocated.
    /// * `_layout` - The current layout of the memory block. Although it's passed in, it's not used directly in this
    ///   function.
    /// * `new_size` - The new size for the memory block.
    ///
    /// # Returns
    /// * A pointer to the reallocated memory block. Returns `null_mut()` if the reallocation fails.
    ///
    /// # Safety
    /// This function is marked as `unsafe` because it directly interacts with low-level memory management,
    /// which can lead to undefined behavior if not handled correctly. The caller must ensure that the pointer
    /// was previously allocated by this allocator and that it is not used after reallocation.
    unsafe fn realloc(&self, ptr: *mut u8, _layout: Layout, new_size: usize) -> *mut u8 {
        // Use the `RtlReAllocateHeap` function to reallocate the memory block.
        // The function takes the heap handle, flags (set to 0 here), the pointer to the memory block, and the new size.
        get_rtl_reallocate_heap()(self.raw_handle(), 0, ptr, new_size)
    }
}

#[cfg(test)]
mod tests {
    use alloc::{boxed::Box, string::String, vec::Vec};
    use core::{ptr::null_mut, slice};

    use super::*;

    /// Test to check memory allocation and deallocation using `alloc` and `dealloc`.
    #[test]
    fn test_alloc_dealloc() {
        let layout = Layout::from_size_align(1024, 8).unwrap();

        unsafe {
            // Allocate 1024 bytes of memory
            let ptr = NT_HEAPALLOCATOR.alloc(layout);
            assert_ne!(ptr, null_mut(), "Allocation failed");

            // Deallocate the memory
            NT_HEAPALLOCATOR.dealloc(ptr, layout);
        }
    }

    /// Test to check zeroed memory allocation using `alloc_zeroed`.
    #[test]
    fn test_alloc_zeroed() {
        let layout = Layout::from_size_align(512, 8).unwrap();

        unsafe {
            // Allocate 512 bytes of zeroed memory
            let ptr = NT_HEAPALLOCATOR.alloc_zeroed(layout);
            assert_ne!(ptr, null_mut(), "Zeroed allocation failed");

            // Verify that the memory is actually zeroed
            let data = slice::from_raw_parts(ptr, 512);
            for &byte in data {
                assert_eq!(byte, 0, "Memory not zeroed");
            }

            // Deallocate the memory
            NT_HEAPALLOCATOR.dealloc(ptr, layout);
        }
    }

    /// Test to check memory reallocation using `realloc`.
    #[test]
    fn test_realloc() {
        let initial_layout = Layout::from_size_align(256, 8).unwrap();
        let new_size = 512;

        unsafe {
            // Initial allocation of 256 bytes
            let ptr = NT_HEAPALLOCATOR.alloc(initial_layout);
            assert_ne!(ptr, null_mut(), "Initial allocation failed");

            // Reallocate the memory to 512 bytes
            let new_ptr = NT_HEAPALLOCATOR.realloc(ptr, initial_layout, new_size);
            assert_ne!(new_ptr, null_mut(), "Reallocation failed");

            // Deallocate the memory
            let new_layout = Layout::from_size_align(new_size, 8).unwrap();
            NT_HEAPALLOCATOR.dealloc(new_ptr, new_layout);
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
