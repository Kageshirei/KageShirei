//! A trait to abstract the key derivation mechanism.
use hkdf::{hmac::digest::OutputSizeUser, Hkdf, HmacImpl};

use crate::CryptError;

/// A trait to abstract the key derivation mechanism.
pub trait WithKeyDerivation {
    /// Derive a key from a given key derivation function instance
    ///
    /// # Arguments
    ///
    /// * `hkdf` - The key derivation function instance
    ///
    /// # Returns
    ///
    /// The updated current instance
    fn derive_key<H, I>(algorithm: Self, hkdf: Hkdf<H, I>) -> Result<Self, CryptError>
    where
        H: OutputSizeUser,
        I: HmacImpl<H>,
        Self: Sized;
}
