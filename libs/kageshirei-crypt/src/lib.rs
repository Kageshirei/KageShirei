#![feature(let_chains)]
#![no_std]
extern crate alloc;

#[cfg(feature = "sha3")]
pub use sha3;

#[cfg(feature = "argon2")]
pub mod argon;
pub mod crypt_error;
pub mod encoder;
pub mod encryption_algorithm;
pub mod symmetric_encryption_algorithm;
#[cfg(test)]
pub mod test_util;

pub use crypt_error::CryptError;
