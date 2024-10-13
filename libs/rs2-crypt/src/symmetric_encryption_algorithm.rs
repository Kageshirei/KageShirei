use std::{
    error::Error,
    fmt::{Debug, Display, Formatter},
    sync::Arc,
};

use anyhow::Result;
use bytes::Bytes;

/// Abstract empty trait used to refer to symmetric encryption algorithms.
pub trait SymmetricEncryptionAlgorithm {
    /// Set the nonce
    ///
    /// # Arguments
    ///
    /// * `nonce` - The nonce to set
    ///
    /// # Returns
    ///
    /// The updated current instance
    fn set_nonce(&mut self, nonce: Bytes) -> Result<&mut Self>;

    /// Set the key
    ///
    /// # Arguments
    ///
    /// * `key` - The key to set
    ///
    /// # Returns
    ///
    /// The updated current instance
    fn set_key(&mut self, key: Bytes) -> Result<&mut Self>;

    /// Create a new nonce
    ///
    /// # Returns
    ///
    /// The updated current instance
    fn make_nonce(&mut self) -> &mut Self;

    /// Create a new key
    ///
    /// # Returns
    ///
    /// The updated current instance
    fn make_key(&mut self) -> &mut Self;

    /// Get the nonce
    ///
    /// # Returns
    ///
    /// The nonce
    fn get_nonce(&self) -> Arc<Bytes>;

    /// Get the key
    ///
    /// # Returns
    ///
    /// The key
    fn get_key(&self) -> Arc<Bytes>;
}

pub enum SymmetricEncryptionAlgorithmError {
    /// The key length is invalid (expected, received)
    InvalidKeyLength(u8, usize),
    /// The nonce length is invalid (expected, received)
    InvalidNonceLength(u8, usize),
}

impl Debug for SymmetricEncryptionAlgorithmError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidKeyLength(bytes, received) => {
                write!(
                    f,
                    "Invalid key length, expected {} bytes, got {}",
                    bytes, received
                )
            },
            Self::InvalidNonceLength(bytes, received) => {
                write!(
                    f,
                    "Invalid nonce length, expected {} bytes, got {}",
                    bytes, received
                )
            },
        }
    }
}

impl Display for SymmetricEncryptionAlgorithmError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // Delegate to Debug
        write!(f, "{:?}", self)
    }
}

impl Error for SymmetricEncryptionAlgorithmError {}
