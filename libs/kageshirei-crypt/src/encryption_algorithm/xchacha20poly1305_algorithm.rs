use std::sync::Arc;

use bytes::{BufMut, Bytes, BytesMut};
use chacha20poly1305::{
    aead::{Aead, Payload},
    AeadCore,
    Key,
    KeyInit,
    XChaCha20Poly1305,
    XNonce,
};
#[cfg(feature = "hkdf")]
use hkdf::hmac::digest::OutputSizeUser;
#[cfg(feature = "hkdf")]
use hkdf::{Hkdf, HmacImpl};

#[cfg(feature = "hkdf")]
use crate::encryption_algorithm::WithKeyDerivation;
use crate::{
    encryption_algorithm::EncryptionAlgorithm,
    symmetric_encryption_algorithm::{SymmetricEncryptionAlgorithm, SymmetricEncryptionAlgorithmError},
};

pub struct XChaCha20Poly1305Algorithm {
    /// The key used for encryption
    key:   Arc<Bytes>,
    /// The last nonce used for encryption (automatically refreshed before each encryption)
    nonce: Arc<Bytes>,
}

unsafe impl Send for XChaCha20Poly1305Algorithm {}

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
    fn set_nonce(&mut self, nonce: Bytes) -> Result<&mut Self, String> {
        if nonce.len() != 24 {
            return Err(SymmetricEncryptionAlgorithmError::InvalidNonceLength(24, nonce.len()).to_string());
        }

        self.nonce = Arc::new(nonce);

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
    fn set_key(&mut self, key: Bytes) -> Result<&mut Self, String> {
        if key.len() != 32 {
            return Err(SymmetricEncryptionAlgorithmError::InvalidKeyLength(32, key.len()).to_string());
        }

        self.key = Arc::new(key);

        Ok(self)
    }

    fn make_nonce(&mut self) -> &mut Self {
        let mut rng = rand::thread_rng();

        let nonce = XChaCha20Poly1305::generate_nonce(&mut rng);
        self.nonce = Arc::new(Bytes::from(nonce.to_vec()));

        self
    }

    fn make_key(&mut self) -> &mut Self { EncryptionAlgorithm::make_key(self).unwrap() }

    fn get_nonce(&self) -> Arc<Bytes> { self.nonce.clone() }

    fn get_key(&self) -> Arc<Bytes> { self.key.clone() }
}

impl Clone for XChaCha20Poly1305Algorithm {
    fn clone(&self) -> Self {
        Self {
            key:   self.key.clone(),
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
        // Check if the key length is valid, otherwise adapt it, this methodology is used only in the from
        // implementation as it is not fallible by default, it's always better to provide a key larger than one
        // shorter in order to avoid any security issue due to key padding
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
            }
            else {
                // Truncate the key to the required length of 32 bytes if it's longer
                key.truncate(32);
            }
        }

        let mut nonce = BytesMut::with_capacity(24);
        nonce.put_bytes(0, nonce.capacity());

        let mut instance = Self {
            key:   Arc::new(key),
            nonce: Arc::new(nonce.freeze()),
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
    fn encrypt(&mut self, data: Bytes) -> Result<Bytes, String> {
        let cipher = XChaCha20Poly1305::new(Key::from_slice(self.key.as_ref()));

        self.make_nonce();
        let encrypted = cipher
            .encrypt(
                XNonce::from_slice(self.nonce.as_ref()),
                Payload::from(data.as_ref()),
            )
            .map_err(|e| e.to_string())?;

        let encrypted_length = encrypted.len();

        let mut new_encrypted = BytesMut::with_capacity(encrypted_length + 24);
        new_encrypted.put_bytes(0, new_encrypted.capacity());

        for i in 0 .. encrypted_length {
            new_encrypted[i] = encrypted[i];
        }

        // Append the nonce to the encrypted data
        for i in 0 .. 24 {
            new_encrypted[encrypted_length + i] = self.nonce[i];
        }

        Ok(new_encrypted.freeze())
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
    fn decrypt(&self, data: Bytes, key: Option<Bytes>) -> Result<Bytes, String> {
        let (data, nonce) = data.split_at(data.len() - 24);

        // Check if the key is provided, otherwise use the instance key
        let key = key.unwrap_or_else(|| self.key.as_ref().clone());

        let cipher = XChaCha20Poly1305::new(Key::from_slice(key.as_ref()));

        let decrypted = cipher
            .decrypt(XNonce::from_slice(nonce), Payload::from(data))
            .map_err(|e| e.to_string())?;

        Ok(Bytes::from(decrypted))
    }

    fn new() -> Self {
        let mut instance = Self {
            key:   Arc::new(Bytes::new()),
            nonce: Arc::new(Bytes::new()),
        };

        EncryptionAlgorithm::make_key(&mut instance)
            .unwrap()
            .make_nonce();

        instance
    }

    /// Create a new key
    ///
    /// # Returns
    ///
    /// The updated current instance
    fn make_key(&mut self) -> Result<&mut Self, String> {
        let mut rng = rand::thread_rng();

        let key = XChaCha20Poly1305::generate_key(&mut rng);
        self.key = Arc::new(Bytes::from(key.to_vec()));

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
    fn derive_key<H, I>(&mut self, hkdf: Hkdf<H, I>) -> Result<&Self, String>
    where
        H: OutputSizeUser,
        I: HmacImpl<H>,
    {
        let mut key = [0u8; 32];
        hkdf.expand(&[], &mut key).map_err(|e| e.to_string())?;

        self.key = Arc::new(Bytes::from(key.to_vec()));

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
