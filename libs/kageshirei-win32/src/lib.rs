#![no_std]
//! # Kageshirei Win32
//!
//! This crate is a collection of Win32 API bindings for Rust. It is intended to be used in
//! conjunction with other crates to provide a more complete Windows API experience.

pub mod kernel32;
pub mod macros;
pub mod ntapi;
pub mod ntdef;
pub mod ntstatus;
pub mod utils;
pub mod winhttp;
pub mod ws2_32;
