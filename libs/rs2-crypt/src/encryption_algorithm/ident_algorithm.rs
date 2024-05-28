use anyhow::Result;
use bytes::Bytes;
#[cfg(feature = "hkdf")]
use hkdf::{Hkdf, HmacImpl};
#[cfg(feature = "hkdf")]
use hkdf::hmac::digest::OutputSizeUser;

use crate::symmetric_encryption_algorithm::SymmetricEncryptionAlgorithm;

use super::EncryptionAlgorithm;
#[cfg(feature = "hkdf")]
use super::WithKeyDerivation;

/// An encryptor that does not encrypt or decrypt data.
#[derive(Clone)]
pub struct IdentEncryptor;

impl SymmetricEncryptionAlgorithm for IdentEncryptor {
	fn set_nonce(&mut self, _nonce: Bytes) -> Result<&mut Self> {
		Ok(self)
	}

	fn set_key(&mut self, _key: Bytes) -> Result<&mut Self> {
		Ok(self)
	}

	fn make_nonce(&mut self) -> &mut Self {
		self
	}

	fn make_key(&mut self) -> &mut Self {
		self
	}

	fn get_nonce(&self) -> Bytes {
		Bytes::new()
	}

	fn get_key(&self) -> Bytes {
		Bytes::new()
	}
}

impl From<Bytes> for IdentEncryptor {
	fn from(_value: Bytes) -> Self {
		Self
	}
}

impl EncryptionAlgorithm for IdentEncryptor {
	fn encrypt(&mut self, data: Bytes) -> Result<Bytes> {
		Ok(data.clone())
	}

	fn decrypt(&self, data: Bytes, _key: Option<Bytes>) -> Result<Bytes> {
		Ok(data)
	}

	fn new() -> Self {
		Self
	}

	fn make_key(&mut self) -> Result<&mut Self> {
		Ok(self)
	}
}

#[cfg(feature = "hkdf")]
impl WithKeyDerivation for IdentEncryptor {
	fn derive_key<H, I>(&mut self, _hkdf: Hkdf<H, I>) -> Result<&Self>
		where H: OutputSizeUser,
		      I: HmacImpl<H> {
		Ok(self)
	}
}