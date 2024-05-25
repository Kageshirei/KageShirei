#![no_std]

extern crate alloc;

use core::ffi::c_void;

mod ntdll;
mod peb;

struct NtSyscall {
	/// The number of the syscall
	number: u32,
	/// The hash of the syscall (used for lookup)
	hash: u128,
	/// The address of the syscall
	address: *const c_void,
	/// The address of a jmp instruction to the syscall asm instruction, not in the current syscall definition
	jmp_address: *const c_void,
}

impl NtSyscall {
	/// Create a new syscall definition
	///
	/// # Arguments
	///
	/// * `hash` - The hash of the syscall to match during the lookup
	pub const fn new(hash: u128) -> Self {
		Self {
			number,
			hash,
			address,
			jmp_address,
		}
	}
}