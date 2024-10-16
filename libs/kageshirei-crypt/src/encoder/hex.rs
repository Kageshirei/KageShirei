use alloc::{borrow::ToOwned, string::String, vec::Vec};

use crate::{encoder::Encoder, CryptError};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HexEncoder;

impl HexEncoder {
    /// Converts a hex character (0-9, a-f) into its corresponding value.
    fn hex_char_to_value(c: char) -> Result<u8, CryptError> {
        match c {
            '0' ..= '9' => Ok(c as u8 - b'0'),
            'a' ..= 'f' => Ok(c as u8 - b'a' + 10),
            _ => Err(CryptError::InvalidEncodingCharacter("hex".to_owned(), c)),
        }
    }
}

impl Encoder for HexEncoder {
    fn encode(&self, data: &[u8]) -> String {
        const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";
        let mut output = String::with_capacity(data.len() * 2); // each byte becomes 2 hex chars

        for &byte in data {
            let high = HEX_CHARS[(byte >> 4) as usize];
            let low = HEX_CHARS[(byte & 0x0f) as usize];
            output.push(high as char);
            output.push(low as char);
        }

        output
    }

    fn decode(&self, data: &str) -> Result<Vec<u8>, CryptError> {
        if data.len() % 2 != 0 {
            return Err(CryptError::InvalidEncodingLength(
                "hex".to_owned(),
                data.len(),
            ));
        }

        let mut output = Vec::with_capacity(data.len() / 2);
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
        let encoded = HexEncoder.encode(data.as_slice());

        let library_encoded = base16ct::lower::encode_string(data.as_slice());
        assert_eq!(library_encoded, "48656c6c6f2c20576f726c6421");
        assert_eq!(encoded, library_encoded);
    }

    #[test]
    fn test_decode() {
        let data = "48656c6c6f2c20576f726c6421";
        let decoded = HexEncoder.decode(data).unwrap();

        let library_decoded = base16ct::lower::decode_vec(data).unwrap();
        assert_eq!(decoded, b"Hello, World!".to_vec());
        assert_eq!(decoded, library_decoded);
    }
}
