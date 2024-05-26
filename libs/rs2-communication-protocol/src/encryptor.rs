use std::error::Error;

use bytes::Bytes;

pub mod ident_encryptor;

/// A trait to abstract the encryption and decryption mechanism.
pub trait Encryptor {
    /// Encrypts a slice of bytes and returns the encrypted data.
    ///
    /// # Arguments
    ///
    /// * `data` - A slice of bytes representing the data to be encrypted.
    ///
    /// # Returns
    ///
    /// * `Result<Vec<u8>, Box<dyn Error>>` - A result containing the encrypted data or an error.
    fn encrypt(&self, data: Bytes) -> Result<Bytes, Box<dyn Error>>;

    /// Decrypts a slice of bytes and returns the decrypted data.
    ///
    /// # Arguments
    ///
    /// * `data` - A slice of bytes representing the data to be decrypted.
    ///
    /// # Returns
    ///
    /// * `Result<Vec<u8>, Box<dyn Error>>` - A result containing the decrypted data or an error.
    fn decrypt(&self, data: Bytes) -> Result<Bytes, Box<dyn Error>>;
}
