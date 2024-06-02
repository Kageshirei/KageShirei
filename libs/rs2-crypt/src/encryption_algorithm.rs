use std::any::Any;

use anyhow::Result;
use bytes::Bytes;
#[cfg(feature = "hkdf")]
use hkdf::{Hkdf, HmacImpl};
#[cfg(feature = "hkdf")]
use hkdf::hmac::digest::OutputSizeUser;

pub mod ident_algorithm;
#[cfg(any(feature = "symmetric-encryption", feature = "xchacha20poly1305"))]
pub mod xchacha20poly1305_algorithm;
#[cfg(feature = "asymmetric-encryption")]
pub mod asymmetric_algorithm;

/// A trait to abstract the encryption and decryption mechanism.
pub trait EncryptionAlgorithm: Send + Any + Clone + From<Bytes> {
	/// Encrypts a slice of bytes and returns the encrypted data.
	///
	/// # Arguments
	///
	/// * `data` - A slice of bytes representing the data to be encrypted.
	///
	/// # Returns
	///
	/// * `Result<Vec<u8>, Box<dyn Error>>` - A result containing the encrypted data or an error.
	fn encrypt(&mut self, data: Bytes) -> Result<Bytes>;

	/// Decrypts a slice of bytes and returns the decrypted data.
	///
	/// # Arguments
	///
	/// * `data` - A slice of bytes representing the data to be decrypted suffixed with the nonce
	/// * `key` - An optional key to use for decryption, if not provided the instance key will be used
	///
	/// # Returns
	///
	/// * `Result<Vec<u8>, Box<dyn Error>>` - A result containing the decrypted data or an error.
	fn decrypt(&self, data: Bytes, key: Option<Bytes>) -> Result<Bytes>;

	/// Create a new instance
	fn new() -> Self;

	/// Create a new key
	///
	/// # Returns
	///
	/// The updated current instance
	fn make_key(&mut self) -> Result<&mut Self>;
}

#[cfg(feature = "hkdf")]
/// A trait to abstract the key derivation mechanism.
pub trait WithKeyDerivation {
	/// Derive a key from a given key derivation function instance
	///
	/// # Arguments
	///
	/// * `hkdf` - The key derivation function instance
	///
	/// # Returns
	///
	/// The updated current instance
	fn derive_key<H, I>(&mut self, hkdf: Hkdf<H, I>) -> Result<&Self>
		where
			H: OutputSizeUser,
			I: HmacImpl<H>;
}