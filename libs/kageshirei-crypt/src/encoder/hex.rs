use alloc::{borrow::ToOwned as _, string::String, vec::Vec};

use crate::{encoder::Encoder as EncoderTrait, CryptError};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Encoder;

impl Encoder {
    /// Converts a hex character (0-9, a-f) into its corresponding value.
    fn hex_char_to_value(c: char) -> Result<u8, CryptError> {
        match c {
            '0' ..= '9' => Ok((c as u8).saturating_sub(b'0')),
            'a' ..= 'f' => Ok((c as u8).saturating_sub(b'a').saturating_add(10)),
            _ => Err(CryptError::InvalidEncodingCharacter("hex".to_owned(), c)),
        }
    }
}

impl EncoderTrait for Encoder {
    fn encode(&self, data: &[u8]) -> Result<String, CryptError> {
        /// Hex alphabet
        const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";

        // each byte becomes 2 hex chars
        let capacity = data.len().overflowing_mul(2);
        if capacity.1 {
            return Err(CryptError::DataTooLong(capacity.0));
        }

        let mut output = String::with_capacity(capacity.0);

        for &byte in data {
            let high = HEX_CHARS.get((byte >> 4) as usize);
            if high.is_none() {
                return Err(CryptError::InvalidCharacterInput);
            }
            let high = *high.unwrap();

            let low = HEX_CHARS.get((byte & 0x0f) as usize);
            if low.is_none() {
                return Err(CryptError::InvalidCharacterInput);
            }
            let low = *low.unwrap();

            output.push(high as char);
            output.push(low as char);
        }

        Ok(output)
    }

    fn decode(&self, data: &str) -> Result<Vec<u8>, CryptError> {
        // constant time check for the length (this is the same as data.len() % 2 != 0)
        if data.len() & 1 != 0 {
            return Err(CryptError::InvalidEncodingLength(
                "hex".to_owned(),
                data.len(),
            ));
        }

        // constant time calculation of the capacity
        let capacity = data.len() >> 1;
        let mut output = Vec::with_capacity(capacity);
        let mut chars = data.chars();

        while let (Some(high), Some(low)) = (chars.next(), chars.next()) {
            let byte = (Self::hex_char_to_value(high)? << 4) | Self::hex_char_to_value(low)?;
            output.push(byte);
        }

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoder::Encoder as _;

    #[test]
    fn test_encode() {
        let data = b"Hello, World!".to_vec();
        let encoded = Encoder.encode(data.as_slice()).unwrap();

        assert_eq!(encoded, "48656c6c6f2c20576f726c6421");
    }

    #[test]
    fn test_decode() {
        let data = "48656c6c6f2c20576f726c6421";
        let decoded = Encoder.decode(data).unwrap();

        assert_eq!(decoded, b"Hello, World!".to_vec());
    }
}
