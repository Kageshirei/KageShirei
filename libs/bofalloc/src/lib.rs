#![no_std]

use core::alloc::{GlobalAlloc, Layout};
use core::arch::asm;
use core::ptr::null_mut;
use core::sync::atomic::{AtomicIsize, Ordering};

use windows_sys::Win32::{
	Foundation::{BOOL, HANDLE},
	System::Memory::{HEAP_GROWABLE, HEAP_ZERO_MEMORY},
};

use bofhelper::import_function;

// Import various heap functions from NTDLL.
import_function!(NTDLL!RtlCreateHeap(Flags: u32, HeapBase: *mut u8, ReserveSize: usize, CommitSize: usize, Lock: *mut u8, Parameters: *mut u8) -> HANDLE);
import_function!(NTDLL!RtlAllocateHeap(hHeap: HANDLE, dwFlags: u32, dwByes: usize) -> *mut u8);
import_function!(NTDLL!RtlFreeHeap(hHeap: HANDLE, dwFlags: u32, lpMem: *mut u8) -> BOOL);
import_function!(NTDLL!RtlReAllocateHeap(hHeap: HANDLE, dwFlags: u32, lpMem: *mut u8, dwBytes: usize) -> *mut u8);
import_function!(NTDLL!RtlDestroyHeap(hHeap: HANDLE) -> HANDLE);

// Define a global allocator using the BofAlloc struct.
// This macro now generates the __rust_* functions
#[global_allocator]
// This is important to prevent the linker from removing the symbol
// Place this variable in the .data section.
#[link_section = ".data"]
// Initialize the allocator as uninitialized.
pub static ALLOCATOR: BofAlloc = BofAlloc::new_uninitialized();

#[no_mangle]
// Place this variable in the .data section.
#[link_section = ".data"]
// Define a static variable to use for a workaround.
static mut __rust_no_alloc_shim_is_unstable: u8 = 0;

extern "C" {
	// Declare an external C function to avoid creating additional symbols.

	// hack: \x01 tells llvm not to add the _ on 32 bit
	// thanks alexchrichton: https://github.com/rust-lang/rust/issues/35052#issuecomment-235420755
	// though, the no alloc shim still has 3 underscores because reasons -_-
	#[cfg_attr(target_arch = "x86", link_name = "\x01.refptr.___rust_no_alloc_shim_is_unstable")]
	#[cfg_attr(target_arch = "x86_64", link_name = "\x01.refptr.__rust_no_alloc_shim_is_unstable")]
	// again we have to declare this as a function to prevent another .refptr symbol from being generated
	fn __refptr__rust_no_alloc_shim_is_unstable();
}

// Define the BofAlloc struct to manage heap allocation.
pub struct BofAlloc(AtomicIsize);

// Implement Send for thread safety.
unsafe impl Send for BofAlloc {}

// Implement Sync for thread safety.
unsafe impl Sync for BofAlloc {}

#[no_mangle]
unsafe fn __rust_alloc_error_handler() -> ! {
	// Trigger an undefined instruction exception on allocation error.
	asm!( "ud2", options(noreturn) )
}

#[no_mangle]
unsafe fn rust_oom() -> ! {
	// Trigger an undefined instruction exception on out of memory.
	asm!( "ud2", options(noreturn) )
}

impl BofAlloc {
	/// Create a new, uninitialized BofAlloc.
	pub const fn new_uninitialized() -> BofAlloc {
		BofAlloc(AtomicIsize::new(0))
	}

	/// Get the raw handle of the heap.
	#[inline]
	fn raw_handle(&self) -> HANDLE {
		self.0.load(Ordering::Relaxed)
	}

	/// Initialize the heap allocator.
	#[inline]
	pub fn initialize(&self) {
		let hh = unsafe { RtlCreateHeap(HEAP_GROWABLE, null_mut(), 0, 0, null_mut(), null_mut()) };
		// hack: you want that .refptr. symbol to point to __rust_no_alloc_shim_is_unstable? OK.
		unsafe {
			core::ptr::write_unaligned(
				__refptr__rust_no_alloc_shim_is_unstable as usize as *mut usize,
				core::ptr::addr_of!(__rust_no_alloc_shim_is_unstable) as *const u8 as usize,
			)
		}
		self.0.store(hh, Ordering::SeqCst);
	}

	/// Check if the allocator is initialized.
	#[inline]
	pub fn is_initialized(&self) -> bool {
		self.0.load(Ordering::Relaxed) != 0
	}

	/// Initialize the allocator if it is not already initialized.
	pub unsafe fn init_if_required(&self) {
		if !self.is_initialized() {
			self.initialize();
		}
	}

	/// Destroy the allocator via `RtlHeapDestroy`
	/// # Safety
	/// This will render all underlying allocations invalid
	#[inline]
	pub unsafe fn destroy(&self) {
		if self.is_initialized() {
			// will return 0 on success
			RtlDestroyHeap(self.0.swap(0, Ordering::SeqCst));
		}
	}
}

/// Implement the GlobalAlloc trait for BofAlloc to handle memory allocation.
unsafe impl GlobalAlloc for BofAlloc {
	/// Allocate memory.
	unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
		RtlAllocateHeap(self.raw_handle(), 0, layout.size())
	}

	/// Allocate zeroed memory.
	unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
		RtlAllocateHeap(self.raw_handle(), HEAP_ZERO_MEMORY, layout.size())
	}

	/// Deallocate memory.
	unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
		RtlFreeHeap(self.raw_handle(), 0, ptr);
	}

	/// Reallocate memory.
	unsafe fn realloc(&self, ptr: *mut u8, _layout: Layout, new_size: usize) -> *mut u8 {
		RtlReAllocateHeap(self.raw_handle(), 0, ptr, new_size)
	}
}
