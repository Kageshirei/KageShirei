//! Encoding variant traits
//!
//! This module contains the traits responsible for the definition of enconding variants.
//! Refer to the Base64 implementation file for an example on how to use them
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
