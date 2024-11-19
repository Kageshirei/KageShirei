use alloc::{sync::Arc, vec::Vec};
use core::mem;

use chacha20poly1305::{
    aead::{Aead as _, Payload},
    AeadCore as _,
    Key,
    KeyInit as _,
    XChaCha20Poly1305,
    XNonce,
};
#[cfg(feature = "hkdf")]
use hkdf::{hmac::digest::OutputSizeUser, Hkdf, HmacImpl};
use rand::rngs::OsRng;

#[cfg(feature = "hkdf")]
use crate::encryption_algorithm::WithKeyDerivation;
use crate::{
    crypt_error::HkdfInvalidLength,
    encryption_algorithm::algorithms::{BasicAlgorithm, SymmetricAlgorithm},
    CryptError,
};

#[derive(Eq, PartialEq)]
#[cfg_attr(any(feature = "server", test), derive(Debug))]
pub struct XChaCha20Poly1305Algorithm {
    /// The key used for encryption
    key:   Arc<Vec<u8>>,
    /// The last nonce used for encryption (automatically refreshed before each encryption)
    nonce: Arc<Vec<u8>>,
}

// Safety: XChaCha20Poly1305Algorithm is Send
unsafe impl Send for XChaCha20Poly1305Algorithm {}

impl SymmetricAlgorithm for XChaCha20Poly1305Algorithm {
    /// Set the nonce
    ///
    /// # Arguments
    ///
    /// * `nonce` - The nonce to set (24 bytes)
    ///
    /// # Returns
    ///
    /// The updated current instance
    fn set_nonce(&mut self, nonce: &'_ [u8]) -> Result<&mut Self, CryptError> {
        if nonce.len() != 24 {
            return Err(CryptError::InvalidNonceLength(24, nonce.len()));
        }

        self.nonce = Arc::new(Vec::from(nonce));

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
    fn set_key(&mut self, key: &'_ [u8]) -> Result<&mut Self, CryptError> {
        if key.len() != 32 {
            return Err(CryptError::InvalidKeyLength(32, key.len()));
        }

        self.key = Arc::new(Vec::from(key));

        Ok(self)
    }

    fn make_nonce(&mut self) -> &mut Self {
        let mut rng = OsRng;

        let nonce = XChaCha20Poly1305::generate_nonce(&mut rng);
        self.nonce = Arc::new(nonce.to_vec());

        self
    }

    fn get_nonce(&self) -> Arc<Vec<u8>> { self.nonce.clone() }

    fn get_key(&self) -> Arc<Vec<u8>> { self.key.clone() }
}

impl Clone for XChaCha20Poly1305Algorithm {
    fn clone(&self) -> Self {
        Self {
            key:   self.key.clone(),
            nonce: self.nonce.clone(),
        }
    }
}

impl Default for XChaCha20Poly1305Algorithm {
    fn default() -> Self {
        Self {
            key:   Arc::new(Vec::new()),
            nonce: Arc::new(Vec::new()),
        }
    }
}

impl BasicAlgorithm for XChaCha20Poly1305Algorithm {
    /// Encrypt the given data
    ///
    /// # Arguments
    ///
    /// * `data` - The data to encrypt
    ///
    /// # Returns
    ///
    /// The encrypted data
    fn encrypt(&mut self, data: &'_ [u8]) -> Result<Vec<u8>, CryptError> {
        let cipher = XChaCha20Poly1305::new(Key::from_slice(self.key.as_slice()));

        self.make_nonce();
        let mut encrypted = cipher
            .encrypt(XNonce::from_slice(self.nonce.as_ref()), Payload::from(data))
            .map_err(CryptError::CannotEncryptWithChaCha20Poly1305)?;

        let full_length = encrypted.len().overflowing_add(24);
        if full_length.1 {
            return Err(CryptError::DataTooLong(full_length.0));
        }

        // Append the nonce to the encrypted data
        for i in 0 .. 24 {
            encrypted.push(
                if let Some(value) = self.nonce.get(i) {
                    *value
                }
                else {
                    return Err(CryptError::InvalidNonceLength(24, i));
                },
            );
        }

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
    fn decrypt(&self, data: &[u8], key: Option<&[u8]>) -> Result<Vec<u8>, CryptError> {
        let data_length = data.len();
        if data_length < 24 {
            return Err(CryptError::DataTooShort(data_length));
        }

        let key = key.map_or_else(
            || Key::from_slice(self.key.as_slice()),
            |k| Key::from_slice(k),
        );

        let (data, nonce) = data.split_at(data_length.saturating_sub(24));
        let cipher = XChaCha20Poly1305::new(key);

        let decrypted = cipher
            .decrypt(XNonce::from_slice(nonce), Payload::from(data))
            .map_err(CryptError::CannotDecryptWithChaCha20Poly1305)?;

        Ok(decrypted)
    }

    fn new() -> Self {
        let mut instance = Self {
            key:   Arc::new(Vec::new()),
            nonce: Arc::new(Vec::new()),
        };
        let mut fallback_instance = instance.clone();

        let mut instance = instance.make_key().unwrap_or(&mut fallback_instance);

        instance = instance.make_nonce();

        mem::take(instance)
    }

    /// Create a new key
    ///
    /// # Returns
    ///
    /// The updated current instance
    fn make_key(&mut self) -> Result<&mut Self, CryptError> {
        let mut rng = OsRng;

        let key = XChaCha20Poly1305::generate_key(&mut rng);
        self.key = Arc::new(key.to_vec());

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
    fn derive_key<H, I>(mut algorithm: Self, hkdf: Hkdf<H, I>) -> Result<Self, CryptError>
    where
        H: OutputSizeUser,
        I: HmacImpl<H>,
    {
        let mut key = [0u8; 32];
        hkdf.expand(&[], &mut key)
            .map_err(|e| CryptError::CannotHashOrDerive(HkdfInvalidLength::from(e)))?;

        algorithm.key = Arc::new(key.to_vec());

        Ok(algorithm)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_xchacha20poly1305() {
        let mut algorithm = XChaCha20Poly1305Algorithm::new();
        let data = Vec::from(b"Hello, world!");

        let encrypted = algorithm.encrypt(data.as_slice()).unwrap();

        // no_std_println!(
        //     "Encrypted: {:?} (\"{}\")",
        //     encrypted,
        //     encrypted.iter().map(|&c| c as char).collect::<String>()
        // );

        let decrypted = algorithm.decrypt(encrypted.as_slice(), None).unwrap();
        assert_eq!(data, decrypted);

        // no_std_println!(
        //     "Decrypted: {:?} (\"{}\")",
        //     decrypted,
        //     decrypted.iter().map(|&c| c as char).collect::<String>()
        // );
    }
}
