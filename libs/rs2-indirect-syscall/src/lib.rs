#![no_std]

pub mod syscall;

pub use syscall::do_syscall;
