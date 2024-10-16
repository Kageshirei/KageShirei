use alloc::{sync::Arc, vec::Vec};

#[cfg(feature = "hkdf")]
use hkdf::hmac::digest::OutputSizeUser;
#[cfg(feature = "hkdf")]
use hkdf::{Hkdf, HmacImpl};

use super::EncryptionAlgorithm;
#[cfg(feature = "hkdf")]
use super::WithKeyDerivation;
use crate::{symmetric_encryption_algorithm::SymmetricEncryptionAlgorithm, CryptError};

/// An encryptor that does not encrypt or decrypt data.
#[derive(Clone, Copy)]
pub struct IdentEncryptor;

impl SymmetricEncryptionAlgorithm for IdentEncryptor {
    fn set_nonce(&mut self, _nonce: &[u8]) -> Result<&mut Self, CryptError> { Ok(self) }

    fn set_key(&mut self, _key: &[u8]) -> Result<&mut Self, CryptError> { Ok(self) }

    fn make_nonce(&mut self) -> &mut Self { self }

    fn get_nonce(&self) -> Arc<Vec<u8>> { Arc::new(Vec::new()) }

    fn get_key(&self) -> Arc<Vec<u8>> { Arc::new(Vec::new()) }
}

impl EncryptionAlgorithm for IdentEncryptor {
    fn encrypt(&mut self, data: &[u8]) -> Result<Vec<u8>, CryptError> { Ok(Vec::from(data)) }

    fn decrypt(&self, data: &[u8], _key: Option<&[u8]>) -> Result<Vec<u8>, CryptError> { Ok(Vec::from(data)) }

    fn new() -> Self { Self }

    fn make_key(&mut self) -> Result<&mut Self, CryptError> { Ok(self) }
}

#[cfg(feature = "hkdf")]
impl WithKeyDerivation for IdentEncryptor {
    fn derive_key<H, I>(algorithm: Self, _hkdf: Hkdf<H, I>) -> Result<Self, CryptError>
    where
        H: OutputSizeUser,
        I: HmacImpl<H>,
    {
        Ok(algorithm)
    }
}
