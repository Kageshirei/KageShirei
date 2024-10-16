use alloc::{borrow::ToOwned, string::String, vec::Vec};

use crate::{encoder::Encoder, CryptError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Base32Encoder;

impl Encoder for Base32Encoder {
    fn encode(&self, data: &[u8]) -> String {
        const ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyz234567";

        let mut bits = 0u32;
        let mut bit_count = 0;
        let mut output = Vec::new();

        for byte in data.to_vec() {
            bits = (bits << 8) | byte as u32;
            bit_count += 8;

            while bit_count >= 5 {
                let index = ((bits >> (bit_count - 5)) & 0x1f) as usize;
                output.push(ALPHABET[index]);
                bit_count -= 5;
            }
        }

        if bit_count > 0 {
            let index = ((bits << (5 - bit_count)) & 0x1f) as usize;
            output.push(ALPHABET[index]);
        }

        output.iter().map(|c| *c as char).collect::<String>()
    }

    fn decode(&self, data: &str) -> Result<Vec<u8>, CryptError> {
        let mut bits = 0u32;
        let mut bit_count = 0;
        let mut output = Vec::new();

        for byte in data.bytes() {
            let value = match byte {
                b'a' ..= b'z' => byte - b'a',
                b'2' ..= b'7' => byte - b'2' + 26,
                v => {
                    return Err(CryptError::InvalidEncodingCharacter(
                        "base32".to_owned(),
                        v as char,
                    ))
                },
            } as u32;

            bits = (bits << 5) | value;
            bit_count += 5;

            if bit_count >= 8 {
                output.push((bits >> (bit_count - 8)) as u8);
                bit_count -= 8;
            }
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
        let encoded = Base32Encoder.encode(data.as_slice());
        assert_eq!(encoded, "jbswy3dpfqqfo33snrscc");
    }

    #[test]
    fn test_decode() {
        let data = "jbswy3dpfqqfo33snrscc";
        let decoded = Base32Encoder.decode(data).unwrap();
        assert_eq!(decoded, b"Hello, World!".to_vec());
    }
}
