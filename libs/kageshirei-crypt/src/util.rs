use alloc::{borrow::ToOwned, vec::Vec};

use crate::CryptError;

/// Push a character to the output if it exists in the alphabet
///
/// # Arguments
///
/// * `alphabet` - The alphabet to check the character against
/// * `output` - The output vector to push the character to
/// * `value` - The value to check against the alphabet
///
/// # Returns
///
/// * `Ok(())` if the character was pushed successfully
/// * `Err(CryptError::InvalidCharacterInput)` if the character is not in the alphabet
pub fn checked_push(alphabet: &[u8], output: &mut Vec<u8>, value: u32) -> Result<(), CryptError> {
    alphabet
        .get(value as usize)
        .map_or(Err(CryptError::InvalidCharacterInput), |value| {
            output.push(*value);
            Ok(())
        })
}

/// Check if the capacity is too large
///
/// # Arguments
///
/// * `capacity` - The capacity to check
///
/// # Returns
///
/// * `Ok(())` if the capacity is not too large
/// * `Err(CryptError::DataTooLong(capacity))` if the capacity is too large
pub const fn check_capacity(capacity: (usize, bool)) -> Result<(), CryptError> {
    if capacity.1 {
        Err(CryptError::DataTooLong(capacity.0))
    }
    else {
        Ok(())
    }
}

/// Create an error for an invalid encoding length
///
/// # Arguments
///
/// * `length` - The length of the encoding
///
/// # Returns
///
/// A `CryptError` for an invalid encoding length
pub fn make_invalid_encoding_length(length: usize) -> CryptError {
    CryptError::InvalidEncodingLength("base64".to_owned(), length)
}

/// Create an error for an invalid number of padding bytes
///
/// # Arguments
///
/// * `length` - The length of the padding bytes
///
/// # Returns
///
/// A `CryptError` for an invalid number of padding bytes
pub fn make_invalid_padding_bytes_encoding_length(length: usize) -> CryptError {
    CryptError::InvalidEncodingLength("base64 padding bytes".to_owned(), length)
}
