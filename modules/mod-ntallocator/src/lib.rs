#![no_std]

extern crate alloc;

use core::alloc::{GlobalAlloc, Layout};
use core::cmp;
use core::ffi::c_void;
use core::ptr::{copy_nonoverlapping, null_mut};

use rs2_indirect_syscall::{init_syscall, run_syscall, NtSyscall, NtdllConfig};

use lazy_static::lazy_static;

// Dbj2 hash of NT functions
const NT_ALLOCATE_VIRTUAL_MEMORY_DBJ2: usize = 0xf783b8ec;
const NT_FREE_VIRTUAL_MEMORY_DBJ2: usize = 0x2802c609;

/// Custom allocator using NT system calls.
pub struct NtAllocator;

lazy_static! {
    // Lazy initialization of the NtdllConfig.
    static ref NTDLL_CONFIG: NtdllConfig = unsafe { NtdllConfig::instance().unwrap() };
    // Lazy initialization of the syscall for NtAllocateVirtualMemory.
    static ref NT_ALLOCATE_VIRTUAL_MEMORY_SYSCALL: NtSyscall = {
        let ntdll_config = &*NTDLL_CONFIG;
        init_syscall(ntdll_config, NT_ALLOCATE_VIRTUAL_MEMORY_DBJ2)
    };
    // Lazy initialization of the syscall for NtFreeVirtualMemory.
    static ref NT_FREE_VIRTUAL_MEMORY_SYSCALL: NtSyscall = {
        let ntdll_config = &*NTDLL_CONFIG;
        init_syscall(ntdll_config, NT_FREE_VIRTUAL_MEMORY_DBJ2)
    };
}

unsafe impl GlobalAlloc for NtAllocator {
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
        let alloc_syscall = &*NT_ALLOCATE_VIRTUAL_MEMORY_SYSCALL;

        // Perform the system call to allocate virtual memory.
        let ntstatus = run_syscall!(
            alloc_syscall.number,
            alloc_syscall.address as usize,
            h_process,
            &mut p_address,
            0,
            &mut (region_size as usize) as *mut usize,
            0x3000, // MEM_COMMIT | MEM_RESERVE
            0x04    // PAGE_READWRITE
        );

        // If the allocation fails, return null; otherwise, return the allocated address.
        if ntstatus < 0 {
            null_mut()
        } else {
            p_address as *mut u8
        }
    }

    /// Allocates memory as described by the given `layout` and initializes it to zero using NT system calls.
    ///
    /// This function uses the `NtAllocateVirtualMemory` system call to allocate memory and then
    /// sets the allocated memory to zero manually.
    ///
    /// # Safety
    ///
    /// This function is unsafe for the same reasons that `alloc` is.
    /// However, the allocated block of memory is guaranteed to be initialized to zero.
    ///
    /// # Returns
    ///
    /// A pointer to newly-allocated and zero-initialized memory, or null to indicate allocation failure.
    ///
    /// # Errors
    ///
    /// Returning a null pointer indicates that either memory is exhausted
    /// or `layout` does not meet this allocator's size or alignment constraints.
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        let ptr = self.alloc(layout); // Allocate the memory.
        if !ptr.is_null() {
            // Zero out the allocated memory manually.
            for i in 0..size {
                ptr.add(i).write_volatile(0);
            }
        }
        ptr
    }

    /// Deallocates the block of memory at the given `ptr` pointer with the given `layout` using NT system calls.
    ///
    /// This function uses the `NtFreeVirtualMemory` system call to deallocate memory.
    ///
    /// # Safety
    ///
    /// This function is unsafe because undefined behavior can result
    /// if the caller does not ensure all of the following:
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

        let free_syscall = &*NT_FREE_VIRTUAL_MEMORY_SYSCALL; // Retrieve the syscall information.

        // Perform the system call to free virtual memory.
        let ntstatus = run_syscall!(
            free_syscall.number,
            free_syscall.address as usize,
            h_process,
            &mut (ptr as *mut c_void),
            &mut region_size,
            0x8000 // MEM_RELEASE
        );

        // Handle error if necessary (currently commented out).
        if ntstatus < 0 {
            // STATUS_SUCCESS is zero
            // Handle error if necessary
            // libc_println!("Deallocation failed: ntstatus={}", ntstatus);
        }
    }

    /// Reallocates memory as described by the given `layout` using NT system calls.
    ///
    /// This function allocates a new block of memory, copies the contents from the old block to the new block,
    /// and then deallocates the old block of memory.
    ///
    /// # Safety
    ///
    /// This function is unsafe because undefined behavior can result
    /// if the caller does not ensure that `new_size` does not overflow.
    /// `layout.align()` comes from a `Layout` and is thus guaranteed to be valid.
    ///
    /// The reallocated block of memory may or may not be initialized.
    ///
    /// # Returns
    ///
    /// A pointer to the newly-reallocated memory, or null to indicate allocation failure.
    ///
    /// # Errors
    ///
    /// Returning a null pointer indicates that either memory is exhausted
    /// or `layout` does not meet this allocator's size or alignment constraints.
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // SAFETY: the caller must ensure that the `new_size` does not overflow.
        // `layout.align()` comes from a `Layout` and is thus guaranteed to be valid.
        let new_layout = Layout::from_size_align_unchecked(new_size, layout.align());
        let new_ptr = self.alloc(new_layout); // Allocate new memory.
        if !new_ptr.is_null() {
            // SAFETY: the previously allocated block cannot overlap the newly allocated block.
            // The safety contract for `dealloc` must be upheld by the caller.
            let copy_size = cmp::min(layout.size(), new_size);
            copy_nonoverlapping(ptr, new_ptr, copy_size);
            self.dealloc(ptr, layout);
        }
        new_ptr
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::{String, ToString};
    use alloc::vec::Vec;
    use libc_print::libc_println;

    #[global_allocator]
    static GLOBAL: NtAllocator = NtAllocator;

    /// Test custom allocator with a vector.
    #[test]
    fn test_alloc_vector() {
        let mut vec: Vec<u32> = Vec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);
        libc_println!("Vector: {:?}", vec);
        assert_eq!(vec, [1, 2, 3]);
    }

    /// Test custom allocator with a string.
    #[test]
    fn test_alloc_string() {
        let s: String = "Hello, world!".to_string();
        libc_println!("String: {}", s);
        assert_eq!(s, "Hello, world!");
    }

    /// Test multiple allocations and deallocations.
    #[test]
    fn test_multiple_allocations() {
        let mut vec1: Vec<u32> = Vec::with_capacity(10);
        for i in 0..10 {
            vec1.push(i);
        }
        libc_println!("Vector 1: {:?}", vec1);

        let mut vec2: Vec<u32> = Vec::with_capacity(20);
        for i in 0..20 {
            vec2.push(i);
        }
        libc_println!("Vector 2: {:?}", vec2);

        drop(vec1);
        libc_println!("Dropped Vector 1");

        let s: String = "Another string".to_string();
        libc_println!("String: {}", s);
    }

    /// Test allocation and deallocation larger than a page.
    #[test]
    fn test_large_allocation() {
        let mut vec: Vec<u8> = Vec::with_capacity(50000); // More than one page
        for i in 0..50000 {
            vec.push(i as u8);
        }
        libc_println!("Large Vector: length={}", vec.len());
        assert_eq!(vec.len(), 50000);
        drop(vec);
        libc_println!("Dropped Large Vector");
    }

    /// Test custom allocator with alloc_zeroed.
    #[test]
    fn test_alloc_zeroed() {
        unsafe {
            let layout = Layout::from_size_align(100, 1).unwrap();

            // Allocate memory and write non-zero data
            let ptr = GLOBAL.alloc(layout);
            if !ptr.is_null() {
                for i in 0..100 {
                    ptr.add(i).write_volatile(0xAA);
                }
                libc_println!("Memory allocated and filled with 0xAA:");
                for i in 0..100 {
                    libc_println!("ptr[{}] = {:#X}", i, *ptr.add(i));
                }
                GLOBAL.dealloc(ptr, layout);
            } else {
                libc_println!("Allocation failed.");
                assert!(false);
            }

            // Allocate memory again without zeroing
            let new_ptr = GLOBAL.alloc(layout);
            if !new_ptr.is_null() {
                libc_println!("Memory re-allocated without zeroing:");
                for i in 0..100 {
                    libc_println!("new_ptr[{}] = {:#X}", i, *new_ptr.add(i));
                }
                GLOBAL.dealloc(new_ptr, layout);
            } else {
                libc_println!("Re-allocation failed.");
                assert!(false);
            }

            // Allocate zeroed memory
            let zeroed_ptr = GLOBAL.alloc_zeroed(layout);
            if !zeroed_ptr.is_null() {
                libc_println!("Memory allocated with zeroing:");
                for i in 0..100 {
                    assert_eq!(*zeroed_ptr.add(i), 0);
                    libc_println!("zeroed_ptr[{}] = {:#X}", i, *zeroed_ptr.add(i));
                }
                GLOBAL.dealloc(zeroed_ptr, layout);
                libc_println!("Allocated and deallocated zeroed memory successfully.");
            } else {
                libc_println!("Zeroed allocation failed.");
                assert!(false);
            }
        }
    }

    /// Test custom allocator with realloc.
    #[test]
    fn test_realloc() {
        unsafe {
            let old_layout = Layout::from_size_align(100, 1).unwrap();
            let new_size = 200;

            // Allocate memory and write non-zero data
            let ptr = GLOBAL.alloc(old_layout);
            if !ptr.is_null() {
                for i in 0..100 {
                    ptr.add(i).write_volatile(0xAA);
                }
                libc_println!("Memory allocated and filled with 0xAA:");
                libc_println!("ptr = {:p}, size = {}", ptr, old_layout.size());
                libc_println!("First byte of allocated memory: {:#X}", *ptr);

                // Reallocate memory to a larger size
                let new_ptr = GLOBAL.realloc(ptr, old_layout, new_size);
                if !new_ptr.is_null() {
                    libc_println!("Memory reallocated to a larger size:");
                    libc_println!("new_ptr = {:p}, new_size = {}", new_ptr, new_size);
                    libc_println!("First byte of reallocated memory: {:#X}", *new_ptr);
                    libc_println!(
                        "Last byte of old part of reallocated memory: {:#X}",
                        *new_ptr.add(99)
                    );
                    libc_println!(
                        "First byte of new part of reallocated memory: {:#X}",
                        *new_ptr.add(100)
                    );
                    libc_println!(
                        "Last byte of reallocated memory: {:#X}",
                        *new_ptr.add(new_size - 1)
                    );

                    // Check that the old data was copied correctly
                    for i in 0..100 {
                        assert_eq!(*new_ptr.add(i), 0xAA);
                    }
                    // Check that the new part is zeroed
                    for i in 100..200 {
                        assert_eq!(*new_ptr.add(i), 0);
                    }
                    GLOBAL.dealloc(
                        new_ptr,
                        Layout::from_size_align_unchecked(new_size, old_layout.align()),
                    );
                } else {
                    libc_println!("Reallocation failed.");
                    assert!(false);
                }
            } else {
                libc_println!("Allocation failed.");
                assert!(false);
            }
        }
    }
}
