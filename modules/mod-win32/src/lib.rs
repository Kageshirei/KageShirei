#![no_std]
//! # mod-win32
//!
//! This module provides low-level Win32 API bindings and utilities for Rust. It is designed to work
//! seamlessly with `no_std` environments, offering functionality to interact with various Windows
//! subsystems, such as process management, registry access, networking, and system information.
//!
//! ## Features
//! - Lightweight and modular design.
//! - Low-level access to essential Windows APIs.
//! - Optimized for `no_std` environments.
//!
//! ## Modules
//! The `mod-win32` crate is organized into modular components, each focusing on specific
//! areas of Win32 API functionality:
//! - `nt_get_adapters_info`: Network adapter information retrieval using custom `GetAdaptersInfo`.
//! - `nt_get_computer_name_ex`: Computer name retrieval using custom `GetComputerNameEx`.
//! - `nt_path`: Path manipulation utilities. (Unstable)
//! - `nt_peb`: Access and manipulation of the Process Environment Block (PEB).
//! - `nt_ps_api`: Process and thread management.
//! - `nt_reg_api`: Windows registry interaction.
//! - `nt_time`: Time and date utilities.
//! - `nt_winhttp`: HTTP communication utilities using WinHTTP.
//! - `nt_ws2`: Networking utilities using Winsock APIs.
//! - `utils`: General helper functions and utilities.
//!
//! ## Safety
//! As this crate interacts with low-level Windows APIs, many functions are marked as `unsafe`.
//! Users are responsible for ensuring the correctness of parameters and the validity of pointers
//! passed to these functions. Improper use can lead to undefined behavior or system instability.

use kageshirei_win32::ntdef::HANDLE;

pub mod nt_get_adapters_info;
pub mod nt_get_computer_name_ex;
pub mod nt_path;
pub mod nt_peb;
pub mod nt_ps_api;
pub mod nt_reg_api;
pub mod nt_time;
pub mod nt_winhttp;
pub mod nt_ws2;
pub mod utils;

extern crate alloc;

/// Returns a handle to the current process.
///
/// In Windows, `-1` is used as a special value to represent the current process handle.
/// This function mimics the behavior of the `NtCurrentProcess` macro in C.
pub const fn nt_current_process() -> HANDLE { (-1isize) as HANDLE }

/// Returns a handle to the current thread.
///
/// Similar to the process handle, `-2` is used as a special value to represent the current thread
/// handle. This function mimics the behavior of the `NtCurrentThread` macro in C.
pub const fn nt_current_thread() -> HANDLE { (-2isize) as HANDLE }
