#![cfg_attr(not(feature = "std"), no_std)]

//! The HTTP protocol module provides the HTTP protocol implementation for the agent.
//!
//! The module provides the [`HttpProtocol`] struct which implements the [`Protocol`] trait.
//!
//! Many implementation can be defined and chosen using simple feature flags.
//! Available implementations are:
//! - `std` (default): The default HTTP implementation using the `reqwest` crate. This
//!   implementation is feature-rich and provides a lot of configuration options, but it is not
//!   available in  `no_std` environments.
//! - `winhttp`: An implementation using the Windows HTTP client library. This implementation is
//!   only available on Windows. It is less feature-rich than the `std` implementation, but it is
//!   available in `no_std` environments and should come with minimal dependencies.

// Enable the stdout capture feature for tests.
#![cfg_attr(test, feature(internal_output_capture, proc_macro_hygiene))]
extern crate core;

#[cfg(feature = "std")]
mod std;
#[cfg(feature = "std")]
pub use std::HttpProtocol;
