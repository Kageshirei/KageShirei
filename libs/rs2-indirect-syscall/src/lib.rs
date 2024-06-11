#![no_std]
#![feature(error_in_core)]

extern crate alloc;

pub mod ntdll_config;
pub mod syscall;
pub mod syscall_resolve;

mod errors;
mod headers;
mod peb;
mod utils;

pub use ntdll_config::NtdllConfig;
pub use syscall::do_syscall;
pub use syscall_resolve::{init_syscall, NtSyscall};
