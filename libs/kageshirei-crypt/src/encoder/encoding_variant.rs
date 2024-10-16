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
