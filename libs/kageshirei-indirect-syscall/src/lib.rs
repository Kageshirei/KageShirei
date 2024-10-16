#![no_std]
//! A crate for performing indirect syscalls on x86 and x86_64 architectures.
//! It provides macros and functions to execute syscalls with dynamic syscall numbers and addresses.
pub mod syscall;

pub use syscall::do_syscall;
