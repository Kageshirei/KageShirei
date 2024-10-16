use alloc::{sync::Arc, vec::Vec};
use core::fmt::{Debug, Display};

use crate::{encryption_algorithm::EncryptionAlgorithm, CryptError};

/// Abstract empty trait used to refer to symmetric encryption algorithms.
pub trait SymmetricEncryptionAlgorithm: EncryptionAlgorithm {
    /// Set the nonce
    ///
    /// # Arguments
    ///
    /// * `nonce` - The nonce to set
    ///
    /// # Returns
    ///
    /// The updated current instance
    fn set_nonce(&mut self, nonce: &[u8]) -> Result<&mut Self, CryptError>;

    /// Set the key
    ///
    /// # Arguments
    ///
    /// * `key` - The key to set
    ///
    /// # Returns
    ///
    /// The updated current instance
    fn set_key(&mut self, key: &[u8]) -> Result<&mut Self, CryptError>;

    /// Create a new nonce
    ///
    /// # Returns
    ///
    /// The updated current instance
    fn make_nonce(&mut self) -> &mut Self;

    /// Get the nonce
    ///
    /// # Returns
    ///
    /// The nonce
    fn get_nonce(&self) -> Arc<Vec<u8>>;

    /// Get the key
    ///
    /// # Returns
    ///
    /// The key
    fn get_key(&self) -> Arc<Vec<u8>>;
}
