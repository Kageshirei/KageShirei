use std::error::Error;

use bytes::Bytes;

use super::Encryptor;

/// An encryptor that does not encrypt or decrypt data.
pub struct IdentEncryptor;

impl Encryptor for IdentEncryptor {
    fn encrypt(&self, data: Bytes) -> Result<Bytes, Box<dyn Error>> {
        Ok(data)
    }

    fn decrypt(&self, data: Bytes) -> Result<Bytes, Box<dyn Error>> {
        Ok(data)
    }
}