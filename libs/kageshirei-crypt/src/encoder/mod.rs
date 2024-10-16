use alloc::{string::String, vec::Vec};
use core::str::FromStr;

use crate::CryptError;

#[cfg(feature = "base32-encoding")]
pub mod base32;
#[cfg(feature = "base64-encoding")]
pub mod base64;
#[cfg(feature = "hex-encoding")]
pub mod hex;

pub trait Encoder {
    /// Encode the given data
    ///
    /// # Arguments
    ///
    /// * `data` - The data to encode
    ///
    /// # Returns
    ///
    /// The encoded data
    fn encode(&self, data: &[u8]) -> String;

    /// Decode the given data
    ///
    /// # Arguments
    ///
    /// * `data` - The data to decode
    ///
    /// # Returns
    ///
    /// The decoded data
    fn decode(&self, data: &str) -> Result<Vec<u8>, CryptError>;
}

pub trait EncodingVariant {
    /// Get the alphabet for the encoding variant
    ///
    /// # Returns
    ///
    /// The alphabet for the encoding variant
    fn get_alphabet(&self) -> &'static [u8];
}

pub trait EncodingPadding {
    /// Get the padding character for the encoding variant
    ///
    /// # Returns
    ///
    /// The padding character for the encoding variant
    fn get_padding(&self) -> Option<u8>;
}
