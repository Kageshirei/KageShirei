use alloc::vec::Vec;
use core::any::Any;

use crate::CryptError;

/// A trait to abstract the encryption and decryption mechanism.
pub trait EncryptionAlgorithm: Send + Any + Clone {
    /// Encrypts a slice of bytes and returns the encrypted data.
    ///
    /// # Arguments
    ///
    /// * `data` - A slice of bytes representing the data to be encrypted.
    ///
    /// # Returns
    ///
    /// * `Result<Vec<u8>, Box<dyn Error>>` - A result containing the encrypted data or an error.
    fn encrypt(&mut self, data: &[u8]) -> Result<Vec<u8>, CryptError>;

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
    fn decrypt(&self, data: &[u8], key: Option<&[u8]>) -> Result<Vec<u8>, CryptError>;

    /// Create a new instance
    fn new() -> Self;

    /// Create a new key
    ///
    /// # Returns
    ///
    /// The updated current instance
    fn make_key(&mut self) -> Result<&mut Self, CryptError>;
}
