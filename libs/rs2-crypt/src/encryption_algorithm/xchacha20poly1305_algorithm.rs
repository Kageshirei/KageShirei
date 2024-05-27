use std::io::Read;

use bytes::{Buf, Bytes};
use chacha20poly1305::{AeadCore, Key, KeyInit, XNonce};
use chacha20poly1305::aead::{Aead, Payload};
use chacha20poly1305::XChaCha20Poly1305;
#[cfg(feature = "hkdf")]
use hkdf::{Hkdf, HmacImpl};
#[cfg(feature = "hkdf")]
use hkdf::hmac::digest::OutputSizeUser;

use crate::encryption_algorithm::EncryptionAlgorithm;

pub struct XChaCha20Poly1305Algorithm {
	/// The key used for encryption
	key: Bytes,
	/// The last nonce used for encryption (automatically refreshed before each encryption)
	nonce: Bytes,
}

impl XChaCha20Poly1305Algorithm {
	/// Create a new nonce
	///
	/// # Returns
	///
	/// The updated current instance
	fn make_nonce(&mut self) -> &mut Self {
		let mut rng = rand::thread_rng();

		let nonce = XChaCha20Poly1305::generate_nonce(&mut rng);
		self.nonce = Bytes::from(nonce.to_vec());

		self
	}
}

impl Clone for XChaCha20Poly1305Algorithm {
	fn clone(&self) -> Self {
		Self {
			key: self.key.clone(),
			nonce: self.nonce.clone(),
		}
	}
}

impl EncryptionAlgorithm for XChaCha20Poly1305Algorithm {
	/// Encrypt the given data
	///
	/// # Arguments
	///
	/// * `data` - The data to encrypt
	///
	/// # Returns
	///
	/// The encrypted data
	fn encrypt(&mut self, data: Bytes) -> anyhow::Result<Bytes> {
		let cipher = XChaCha20Poly1305::new(Key::from_slice(self.key.as_ref()));

		self.make_nonce();
		let encrypted = cipher.encrypt(
			XNonce::from_slice(self.nonce.as_ref()),
			Payload::from(data.as_ref()),
		).map_err(|e| anyhow::anyhow!(e))?;

		let encrypted_length = encrypted.len();

		let encrypted = Bytes::from(encrypted).chain(self.nonce.clone()).copy_to_bytes(encrypted_length + self.nonce.len());

		Ok(encrypted)
	}

	/// Decrypt the given data
	///
	/// # Arguments
	///
	/// * `data` - The data to decrypt
	/// * `nonce` - The nonce to use for decryption
	///
	/// # Returns
	///
	/// The decrypted data
	fn decrypt(&self, data: Bytes) -> anyhow::Result<Bytes> {
		let (data, nonce) = data.split_at(data.len() - 24);

		let cipher = XChaCha20Poly1305::new(Key::from_slice(self.key.as_ref()));

		let decrypted = cipher.decrypt(
			XNonce::from_slice(nonce.as_ref()),
			Payload::from(data.as_ref()),
		).map_err(|e| anyhow::anyhow!(e))?;

		Ok(Bytes::from(decrypted))
	}

	fn new() -> Self {
		let mut instance = Self {
			key: Bytes::new(),
			nonce: Bytes::new(),
		};

		instance.make_key().make_nonce();

		instance
	}

	/// Create a new instance with a given key
	///
	/// # Arguments
	///
	/// * `key` - The key to use for encryption (32 bytes)
	///
	/// # Returns
	///
	/// The new instance
	fn from(key: Bytes) -> Self {
		let mut instance = Self {
			key,
			nonce: Bytes::new(),
		};

		instance.make_nonce();

		instance
	}

	/// Create a new key
	///
	/// # Returns
	///
	/// The updated current instance
	fn make_key(&mut self) -> &mut Self {
		let mut rng = rand::thread_rng();

		let key = XChaCha20Poly1305::generate_key(&mut rng);
		self.key = Bytes::from(key.to_vec());

		self
	}

	/// Derive a key from a given key derivation function instance
	///
	/// # Arguments
	///
	/// * `hkdf` - The key derivation function instance
	///
	/// # Returns
	///
	/// The updated current instance
	#[cfg(feature = "hkdf")]
	fn derive_key<H, I>(&mut self, hkdf: Hkdf<H, I>) -> anyhow::Result<&Self>
		where
			H: OutputSizeUser,
			I: HmacImpl<H>,
	{
		let mut key = [0u8, 32];
		hkdf.expand(&[], &mut key)?;

		self.key = Bytes::from(key.as_slice());

		Ok(self)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_xchacha20poly1305() {
		let mut algorithm = XChaCha20Poly1305Algorithm::new();
		let data = Bytes::from("Hello, world!");

		let encrypted = algorithm.encrypt(data.clone()).unwrap();
		println!("Encrypted: {:?}", encrypted);

		let decrypted = algorithm.decrypt(encrypted).unwrap();
		assert_eq!(data, decrypted);
	}
}