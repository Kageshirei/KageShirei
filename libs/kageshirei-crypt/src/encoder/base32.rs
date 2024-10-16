use alloc::{borrow::ToOwned as _, string::String, vec::Vec};

use crate::{encoder::Encoder as EncoderTrait, util::checked_push, CryptError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Encoder;

impl EncoderTrait for Encoder {
    fn encode(&self, data: &[u8]) -> Result<String, CryptError> {
        /// Base32 alphabet
        const ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyz234567";

        let mut bits = 0u32;
        let mut bit_count: i32 = 0;
        let mut output = Vec::new();

        for byte in data.iter().copied() {
            bits = (bits << 8) | byte as u32;
            bit_count = bit_count.saturating_add(8);

            while bit_count >= 5 {
                let index = ((bits >> bit_count.saturating_sub(5)) & 0x1f) as usize;

                #[expect(
                    clippy::map_err_ignore,
                    reason = "The default function uses a generic error, as we can use a specific one we opt into it \
                              without changing the original implementation"
                )]
                checked_push(ALPHABET, &mut output, index as u32)
                    .map_err(|_| CryptError::EncodingBitmaskOverflow(index))?;

                bit_count = bit_count.saturating_sub(5);
            }
        }

        if bit_count > 0 {
            let index = ((bits << 5i32.saturating_sub(bit_count)) & 0x1f) as usize;

            #[expect(
                clippy::map_err_ignore,
                reason = "The default function uses a generic error, as we can use a specific one we opt into it \
                          without changing the original implementation"
            )]
            checked_push(ALPHABET, &mut output, index as u32)
                .map_err(|_| CryptError::EncodingBitmaskOverflow(index))?;
        }

        Ok(output.iter().map(|c| *c as char).collect::<String>())
    }

    fn decode(&self, data: &str) -> Result<Vec<u8>, CryptError> {
        let mut bits = 0u32;
        let mut bit_count: i32 = 0;
        let mut output = Vec::new();

        for byte in data.bytes() {
            let value = match byte {
                b'a' ..= b'z' => byte.saturating_sub(b'a'),
                b'2' ..= b'7' => byte.saturating_sub(b'2').saturating_add(26),
                v => {
                    return Err(CryptError::InvalidEncodingCharacter(
                        "base32".to_owned(),
                        v as char,
                    ))
                },
            } as u32;

            bits = (bits << 5) | value;
            bit_count = bit_count.saturating_add(5);

            if bit_count >= 8 {
                output.push((bits >> (bit_count.saturating_sub(8))) as u8);
                bit_count = bit_count.saturating_sub(8);
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
        let encoded = Encoder.encode(data.as_slice()).unwrap();
        assert_eq!(encoded, "jbswy3dpfqqfo33snrscc");
    }

    #[test]
    fn test_decode() {
        let data = "jbswy3dpfqqfo33snrscc";
        let decoded = Encoder.decode(data).unwrap();
        assert_eq!(decoded, b"Hello, World!".to_vec());
    }
}
