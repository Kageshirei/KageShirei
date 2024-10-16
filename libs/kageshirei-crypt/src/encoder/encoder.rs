use alloc::{string::String, vec::Vec};

use crate::CryptError;

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
    fn encode(&self, data: &[u8]) -> Result<String, CryptError>;

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
