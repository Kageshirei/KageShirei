#![no_std]

extern crate alloc;
extern crate rs2_winapi;

pub mod syscall;
pub mod syscall_resolve;

pub use syscall::do_syscall;
pub use syscall_resolve::{init_syscall, NtSyscall};
