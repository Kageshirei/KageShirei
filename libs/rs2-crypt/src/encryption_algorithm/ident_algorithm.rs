use anyhow::Result;
use bytes::Bytes;
#[cfg(feature = "hkdf")]
use hkdf::{Hkdf, HmacImpl};
#[cfg(feature = "hkdf")]
use hkdf::hmac::digest::OutputSizeUser;

use super::EncryptionAlgorithm;

/// An encryptor that does not encrypt or decrypt data.
#[derive(Clone)]
pub struct IdentEncryptor;

impl EncryptionAlgorithm for IdentEncryptor {
	fn encrypt(&mut self, data: Bytes) -> Result<Bytes> {
		Ok(data.clone())
	}

	fn decrypt(&self, data: Bytes) -> Result<Bytes> {
		Ok(data)
	}

	fn new() -> Self {
		Self
	}

	fn from(_key: Bytes) -> Self {
		Self
	}

	fn make_key(&mut self) -> &mut Self {
		self
	}

	#[cfg(feature = "hkdf")]
	fn derive_key<H, I>(&mut self, _hkdf: Hkdf<H, I>) -> Result<&Self>
		where H: OutputSizeUser,
		      I: HmacImpl<H> {
		Ok(self)
	}
}