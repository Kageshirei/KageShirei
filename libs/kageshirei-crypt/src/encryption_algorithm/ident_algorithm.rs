use std::sync::Arc;

use bytes::Bytes;
#[cfg(feature = "hkdf")]
use hkdf::hmac::digest::OutputSizeUser;
#[cfg(feature = "hkdf")]
use hkdf::{Hkdf, HmacImpl};

use super::EncryptionAlgorithm;
#[cfg(feature = "hkdf")]
use super::WithKeyDerivation;
use crate::symmetric_encryption_algorithm::SymmetricEncryptionAlgorithm;

/// An encryptor that does not encrypt or decrypt data.
#[derive(Clone, Copy)]
pub struct IdentEncryptor;

impl SymmetricEncryptionAlgorithm for IdentEncryptor {
    fn set_nonce(&mut self, _nonce: Bytes) -> Result<&mut Self, String> { Ok(self) }

    fn set_key(&mut self, _key: Bytes) -> Result<&mut Self, String> { Ok(self) }

    fn make_nonce(&mut self) -> &mut Self { self }

    fn make_key(&mut self) -> &mut Self { self }

    fn get_nonce(&self) -> Arc<Bytes> { Arc::new(Bytes::new()) }

    fn get_key(&self) -> Arc<Bytes> { Arc::new(Bytes::new()) }
}

impl From<Bytes> for IdentEncryptor {
    fn from(_value: Bytes) -> Self { Self }
}

impl EncryptionAlgorithm for IdentEncryptor {
    fn encrypt(&mut self, data: Bytes) -> Result<Bytes, String> { Ok(data.clone()) }

    fn decrypt(&self, data: Bytes, _key: Option<Bytes>) -> Result<Bytes, String> { Ok(data) }

    fn new() -> Self { Self }

    fn make_key(&mut self) -> Result<&mut Self, String> { Ok(self) }
}

#[cfg(feature = "hkdf")]
impl WithKeyDerivation for IdentEncryptor {
    fn derive_key<H, I>(&mut self, _hkdf: Hkdf<H, I>) -> Result<&Self, String>
    where
        H: OutputSizeUser,
        I: HmacImpl<H>,
    {
        Ok(self)
    }
}
