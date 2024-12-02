#![feature(let_chains)]
#![no_std]
//! # Kageshirei Crypt
//!
//! This library provides a set of cryptographic algorithms and utilities for use in Kageshirei
//! agent and server.
//!
//! Most of the features of this library should be protected by feature flags, so you can choose
//! which features to enable.

extern crate alloc;

#[cfg(feature = "sha3")]
pub use sha3;
pub mod crypt_error;
pub mod encoder;
pub mod encryption_algorithm;
pub mod hash;
#[cfg(test)]
pub mod test_util;

pub use crypt_error::CryptError;

pub(crate) mod util;
