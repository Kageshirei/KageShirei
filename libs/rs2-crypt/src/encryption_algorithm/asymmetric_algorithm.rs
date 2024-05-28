use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;

use anyhow::{anyhow, Result};
use bytes::{BufMut, Bytes, BytesMut};
use hkdf::Hkdf;
use hkdf::hmac::SimpleHmac;
use k256::{FieldBytes, PublicKey, SecretKey};
use k256::elliptic_curve::rand_core::RngCore;
use sha3::Sha3_512;

use crate::encryption_algorithm::{EncryptionAlgorithm, WithKeyDerivation};
use crate::symmetric_encryption_algorithm::SymmetricEncryptionAlgorithm;

/// An asymmetric encryption algorithm that uses a symmetric encryption algorithm for encryption and decryption
pub struct AsymmetricAlgorithm<T> {
	/// The secret key of the pair
	secret_key: Arc<SecretKey>,
	/// The public key of the pair
	public_key: Arc<PublicKey>,
	/// The implementation of a symmetric algorithm to use with this asymmetric algorithm
	algorithm_instance: T,
	/// The number of encrypted messages, this is a counter used to internally rotate the keys after a certain number
	/// of messages have been encrypted
	encrypted_messages: u16,
	/// The public key of the receiver of the encrypted text
	receiver: Option<Arc<PublicKey>>,
	/// The last used key for encryption, useful to retrieve the last key used for encryption in order to decrypt the
	/// message if the key has been rotated
	last_used_key: Option<Bytes>,
}

/// The size of the salt used for the HKDF key derivation function (128 bytes)
const HKDF_SALT_SIZE: usize = 0x80;
/// The threshold of encrypted messages before rotating the key (1024 messages)
const KEY_ROTATION_THRESHOLD: u16 = 0x400;

unsafe impl<T> Send for AsymmetricAlgorithm<T> {}

impl<T> AsymmetricAlgorithm<T>
	where T: SymmetricEncryptionAlgorithm + EncryptionAlgorithm + WithKeyDerivation {

	/// Create a new key pair
	pub fn new() -> Self {
		let mut rng = rand::thread_rng();

		let secret_key = Arc::new(SecretKey::random(&mut rng));
		let public_key = Arc::new(secret_key.public_key());

		Self {
			public_key,
			secret_key,
			algorithm_instance: T::new(),
			encrypted_messages: 0,
			receiver: None,
			last_used_key: None,
		}
	}

	/// Crate a bytes representation of the public key
	pub fn serialize_public_key(&self) -> Bytes {
		Bytes::from(self.public_key.to_sec1_bytes())
	}

	/// Crate a bytes representation of the secret key
	pub fn serialize_secret_key(&self) -> Bytes {
		let bytes = self.secret_key.to_bytes();
		Bytes::from(bytes.to_vec())
	}

	/// Set the receiver public key
	///
	/// # Arguments
	///
	/// * `public_key` - The public key of the receiver
	///
	/// # Returns
	///
	/// The updated current instance
	pub fn set_receiver(&mut self, public_key: Arc<PublicKey>) -> &mut Self {
		self.receiver = Some(public_key);
		self
	}

	/// Derive a shared secret from a given public key and return a new key derivation function instance for key generation
	///
	/// # Arguments
	///
	/// * `public_key` - The public key to derive the shared secret from
	///
	/// # Returns
	///
	/// A new key derivation function instance for secure-key generation
	pub fn derive_shared_secret(&mut self, public_key: Arc<PublicKey>) -> Hkdf<Sha3_512, SimpleHmac<Sha3_512>> {
		// set the receiver public key to easily reuse it later
		self.receiver = Some(public_key.clone());

		// derive the shared secret
		let shared_secret = k256::ecdh::diffie_hellman(
			&self.secret_key.to_nonzero_scalar(),
			public_key.as_affine(),
		);

		// compute the salt
		let mut rng = rand::thread_rng();
		let mut salt = [0u8; HKDF_SALT_SIZE];
		rng.fill_bytes(&mut salt);

		shared_secret.extract::<Sha3_512>(Some(&salt))
	}
}

impl<T> Clone for AsymmetricAlgorithm<T>
	where T: SymmetricEncryptionAlgorithm + EncryptionAlgorithm + WithKeyDerivation {
	fn clone(&self) -> Self {
		Self {
			public_key: self.public_key.clone(),
			secret_key: self.secret_key.clone(),
			algorithm_instance: T::new(),
			encrypted_messages: 0,
			receiver: self.receiver.clone(),
			last_used_key: self.last_used_key.clone(),
		}
	}
}

impl<T> From<Bytes> for AsymmetricAlgorithm<T>
	where T: EncryptionAlgorithm + SymmetricEncryptionAlgorithm + WithKeyDerivation {
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
				let mut new_key = BytesMut::with_capacity(32);
				// Fill the new key with zeros
				new_key.put_bytes(0, new_key.capacity());
				// Copy the original key to the new key (overriding the zeros)
				new_key.copy_from_slice(&key[..]);

				// Freeze the new key
				key = new_key.freeze();
			} else {
				// Truncate the key to the required length of 32 bytes if it's longer
				key.truncate(32);
			}
		}

		let field_bytes = FieldBytes::from_slice(key.get(..).unwrap());

		let secret_key = Arc::new(SecretKey::from_bytes(&field_bytes).unwrap());
		let public_key = Arc::new(secret_key.public_key());

		Self {
			public_key,
			secret_key,
			algorithm_instance: T::new(),
			encrypted_messages: 0,
			receiver: None,
			last_used_key: None,
		}
	}
}

impl<T> EncryptionAlgorithm for AsymmetricAlgorithm<T>
	where T: SymmetricEncryptionAlgorithm + EncryptionAlgorithm + WithKeyDerivation {
	fn encrypt(&mut self, data: Bytes) -> Result<Bytes> {
		self.encrypted_messages += 1;

		// Rotate the key if the threshold is reached
		if self.encrypted_messages >= KEY_ROTATION_THRESHOLD {
			self.make_key()?;
			self.encrypted_messages = 0;
		}

		self.last_used_key = Some(self.algorithm_instance.get_key());

		// proxy the encryption to the symmetric algorithm
		self.algorithm_instance.encrypt(data)
	}

	fn decrypt(&self, data: Bytes, key: Option<Bytes>) -> Result<Bytes> {
		// proxy the decryption to the symmetric algorithm
		self.algorithm_instance.decrypt(data, key)
	}

	fn new() -> Self {
		// proxy to the new method of the symmetric algorithm
		Self::new()
	}

	fn make_key(&mut self) -> Result<&mut Self> {
		if self.receiver.is_none() {
			return Err(anyhow!(AsymmetricEncryptionAlgorithmError::MissingReceiverPublicKey));
		}

		let derived_key = self.derive_shared_secret(self.receiver.clone().unwrap());
		self.algorithm_instance.derive_key(derived_key)?;

		Ok(self)
	}
}

pub enum AsymmetricEncryptionAlgorithmError {
	/// The receiver public key is missing
	MissingReceiverPublicKey,
}

impl Debug for AsymmetricEncryptionAlgorithmError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::MissingReceiverPublicKey => {
				write!(f, "The receiver public key is missing")
			}
		}
	}
}

impl Display for AsymmetricEncryptionAlgorithmError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		// Delegate to Debug
		write!(f, "{:?}", self)
	}
}

impl Error for AsymmetricEncryptionAlgorithmError {}

#[cfg(test)]
mod test {
	use crate::encryption_algorithm::xchacha20poly1305_algorithm::XChaCha20Poly1305Algorithm;

	use super::*;

	#[test]
	fn test_private_key() {
		let algorithm = AsymmetricAlgorithm::<XChaCha20Poly1305Algorithm>::new();
		let serialized = algorithm.serialize_secret_key();
		println!("Secret key length: {}", serialized.len());
		println!("Serialized: {:?}", serialized);

		let serialized = algorithm.serialize_public_key();
		println!("Public key length: {}", serialized.len());
		println!("Serialized: {:?}", serialized);
	}

	#[test]
	fn test_asymmetric_algorithm() {
		let mut bob = AsymmetricAlgorithm::<XChaCha20Poly1305Algorithm>::new();
		let mut alice = AsymmetricAlgorithm::<XChaCha20Poly1305Algorithm>::new();

		let data = Bytes::from("Hello, world!");

		// bob sends a message to alice
		bob.set_receiver(alice.public_key.clone());

		let encrypted = bob.encrypt(data.clone()).unwrap();
		// this is not memory safe, it should be... how?
		let used_key = bob.last_used_key.clone();
		println!("Encrypted: {:?}", encrypted);

		// alice receives the message from bob
		alice.set_receiver(bob.public_key.clone());

		let decrypted = alice.decrypt(encrypted, used_key).unwrap();
		println!("Decrypted: {:?}", decrypted);

		assert_eq!(data, decrypted);
	}

	#[test]
	fn test_key_rotation() {
		let mut bob = AsymmetricAlgorithm::<XChaCha20Poly1305Algorithm>::new();
		bob.set_receiver(bob.public_key.clone());

		let key = bob.make_key().unwrap().algorithm_instance.get_key();
		let new_key = bob.make_key().unwrap().algorithm_instance.get_key();

		println!("Key: {:?}", key);
		println!("New key: {:?}", new_key);

		assert_ne!(key, new_key);
	}

	#[test]
	fn test_key_automatically_rotate() {
		let mut bob = AsymmetricAlgorithm::<XChaCha20Poly1305Algorithm>::new();
		let mut alice = AsymmetricAlgorithm::<XChaCha20Poly1305Algorithm>::new();

		let data = Bytes::from("Hello, world!");

		let mut original_key = None;
		let mut last_used_key = None;
		let mut previous_round_encrypted;
		let mut last_encrypted = None;

		for _ in 0..=KEY_ROTATION_THRESHOLD + 1 {
			// bob sends a message to alice
			bob.set_receiver(alice.public_key.clone());

			let encrypted = bob.encrypt(data.clone()).unwrap();
			// this is not memory safe, it should be... how?
			last_used_key = bob.last_used_key.clone();

			// shift the last encrypted and the previous round encrypted, after the check that the decryption is successful
			// we will ensure that they differ
			previous_round_encrypted = last_encrypted.clone();
			last_encrypted = Some(encrypted.clone());

			// alice receives the message from bob
			alice.set_receiver(bob.public_key.clone());

			let decrypted = alice.decrypt(encrypted, last_used_key.clone()).unwrap();
			assert_eq!(data, decrypted);
			assert_ne!(previous_round_encrypted, last_encrypted);

			if original_key.is_none() {
				original_key = Some(bob.algorithm_instance.get_key());
			}
		}

		println!("Original key: {:?}", original_key);
		println!("Last used key: {:?}", last_used_key);
		assert_ne!(original_key, last_used_key);
	}
}