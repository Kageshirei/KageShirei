use anyhow::{anyhow, Result};
use bytes::{Buf, Bytes};
use chacha20poly1305::{AeadCore, Key, KeyInit, XNonce};
use chacha20poly1305::aead::{Aead, Payload};
use chacha20poly1305::XChaCha20Poly1305;
#[cfg(feature = "hkdf")]
use hkdf::{Hkdf, HmacImpl};
#[cfg(feature = "hkdf")]
use hkdf::hmac::digest::OutputSizeUser;

use crate::encryption_algorithm::{EncryptionAlgorithm, WithKeyDerivation};
use crate::symmetric_encryption_algorithm::{SymmetricEncryptionAlgorithm, SymmetricEncryptionAlgorithmError};

pub struct XChaCha20Poly1305Algorithm {
	/// The key used for encryption
	key: Bytes,
	/// The last nonce used for encryption (automatically refreshed before each encryption)
	nonce: Bytes,
}

impl SymmetricEncryptionAlgorithm for XChaCha20Poly1305Algorithm {
	/// Set the nonce
	///
	/// # Arguments
	///
	/// * `nonce` - The nonce to set (24 bytes)
	///
	/// # Returns
	///
	/// The updated current instance
	fn set_nonce(&mut self, nonce: Bytes) -> Result<&mut Self> {
		if nonce.len() != 24 {
			return Err(anyhow::anyhow!(SymmetricEncryptionAlgorithmError::InvalidNonceLength(24, nonce.len())));
		}

		self.nonce = nonce;

		Ok(self)
	}

	/// Set the key
	///
	/// # Arguments
	///
	/// * `key` - The key to set (32 bytes)
	///
	/// # Returns
	///
	/// The updated current instance
	fn set_key(&mut self, key: Bytes) -> Result<&mut Self> {
		if key.len() != 32 {
			return Err(anyhow::anyhow!(SymmetricEncryptionAlgorithmError::InvalidKeyLength(32, key.len())));
		}

		self.key = key;

		Ok(self)
	}

	fn make_nonce(&mut self) -> &mut Self {
		let mut rng = rand::thread_rng();

		let nonce = XChaCha20Poly1305::generate_nonce(&mut rng);
		self.nonce = Bytes::from(nonce.to_vec());

		self
	}

	fn make_key(&mut self) -> &mut Self {
		EncryptionAlgorithm::make_key(self).unwrap()
	}

	fn get_nonce(&self) -> Bytes {
		self.nonce.clone()
	}

	fn get_key(&self) -> Bytes {
		self.key.clone()
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

impl From<Bytes> for XChaCha20Poly1305Algorithm {
	/// Create a new instance with a given key
	///
	/// # Arguments
	///
	/// * `key` - The key to use for encryption (32 bytes)
	///
	/// # Returns
	///
	/// The new instance
	fn from(mut key: Bytes) -> Self {
		// Check if the key length is valid
		let key_length = key.len();
		// Check if the key length is valid, otherwise adapt it, this methodology is used only in the from implementation
		// as it is not fallible by default, it's always better to provide a key larger than one shorter in order to avoid
		// any security issue due to key padding
		if key_length != 32 {
			if key_length < 32 {
				// Pad the key with 0s to reach the required length of 32 bytes, this is not secure, but it's better
				// than panicking
				let mut new_key = vec![0u8; 32];
				new_key.fill(0);

				// chain the original key with the all-zero key truncating to 32 bytes
				key = key.chain(Bytes::from(new_key)).copy_to_bytes(32);
			} else {
				// Truncate the key to the required length of 32 bytes if it's longer
				key.truncate(32);
			}
		}

		let mut instance = Self {
			key,
			nonce: Bytes::new(),
		};

		instance.make_nonce();

		instance
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
	fn decrypt(&self, data: Bytes, key: Option<Bytes>) -> Result<Bytes> {
		let (data, nonce) = data.split_at(data.len() - 24);

		// Check if the key is provided, otherwise use the instance key
		let key = key.unwrap_or_else(|| self.key.clone());

		let cipher = XChaCha20Poly1305::new(Key::from_slice(key.as_ref()));

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

		EncryptionAlgorithm::make_key(&mut instance).unwrap().make_nonce();

		instance
	}

	/// Create a new key
	///
	/// # Returns
	///
	/// The updated current instance
	fn make_key(&mut self) -> Result<&mut Self> {
		let mut rng = rand::thread_rng();

		let key = XChaCha20Poly1305::generate_key(&mut rng);
		self.key = Bytes::from(key.to_vec());

		Ok(self)
	}
}

#[cfg(feature = "hkdf")]
impl WithKeyDerivation for XChaCha20Poly1305Algorithm {
	/// Derive a key from a given key derivation function instance
	///
	/// # Arguments
	///
	/// * `hkdf` - The key derivation function instance
	///
	/// # Returns
	///
	/// The updated current instance
	fn derive_key<H, I>(&mut self, hkdf: Hkdf<H, I>) -> anyhow::Result<&Self>
		where
			H: OutputSizeUser,
			I: HmacImpl<H>,
	{
		let mut key = [0u8; 32];
		hkdf.expand(&[], &mut key).map_err(|e| anyhow!(e))?;

		self.key = Bytes::from(key.to_vec());

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

		let decrypted = algorithm.decrypt(encrypted, None).unwrap();
		assert_eq!(data, decrypted);
	}
}