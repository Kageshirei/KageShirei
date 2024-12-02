use alloc::{borrow::ToOwned as _, string::String, vec::Vec};

use crate::{
    encoder::{Encoder as EncoderTrait, EncodingPadding, EncodingVariant},
    util::{check_capacity, checked_push, make_invalid_encoding_length, make_invalid_padding_bytes_encoding_length},
    CryptError,
};

pub enum Variant {
    UrlUnpadded,
    Url,
    Standard,
    StandardUnpadded,
}

impl EncodingVariant for Variant {
    fn get_alphabet(&self) -> &'static [u8] {
        match *self {
            Self::Url | Self::UrlUnpadded => b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_",
            Self::Standard | Self::StandardUnpadded => {
                b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/"
            },
        }
    }
}

impl EncodingPadding for Variant {
    fn get_padding(&self) -> Option<u8> {
        match *self {
            Self::UrlUnpadded | Self::StandardUnpadded => None,
            Self::Url | Self::Standard => Some(b'='),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Encoder<T>
where
    T: EncodingVariant + EncodingPadding,
{
    /// Which variant of base64 to use
    variant: T,
}

impl Encoder<Variant> {
    pub const fn new(variant: Variant) -> Self {
        Self {
            variant,
        }
    }
}

impl<T> EncoderTrait for Encoder<T>
where
    T: EncodingVariant + EncodingPadding,
{
    fn encode(&self, data: &[u8]) -> Result<String, CryptError> {
        let alphabet = self.variant.get_alphabet();
        let padding = self.variant.get_padding();

        let capacity = data.len().overflowing_mul(4);
        check_capacity(capacity)?;

        let capacity = capacity.0.overflowing_add(2);
        check_capacity(capacity)?;

        let capacity = capacity.0.overflowing_div(3);
        check_capacity(capacity)?;

        let mut output = Vec::with_capacity(capacity.0);

        let mut chunks = data.chunks_exact(3);

        // Process each chunk of 3 bytes
        for chunk in &mut chunks {
            let n = chunk.first().map_or_else(
                || Err(make_invalid_encoding_length(chunk.len())),
                |first| {
                    chunk.get(1).map_or_else(
                        || Err(make_invalid_encoding_length(chunk.len())),
                        |second| {
                            chunk.get(2).map_or_else(
                                || Err(make_invalid_encoding_length(chunk.len())),
                                |third| Ok(((*first as u32) << 16) | ((*second as u32) << 8) | (*third as u32)),
                            )
                        },
                    )
                },
            )?;

            checked_push(alphabet, &mut output, (n >> 18) & 0x3f)?;
            checked_push(alphabet, &mut output, (n >> 12) & 0x3f)?;
            checked_push(alphabet, &mut output, (n >> 6) & 0x3f)?;
            checked_push(alphabet, &mut output, n & 0x3f)?;
        }

        // Handle remaining bytes (if any)
        let rem = chunks.remainder();
        if !rem.is_empty() {
            if rem.len() > 2 {
                return Err(make_invalid_padding_bytes_encoding_length(rem.len()));
            }

            let n = if rem.len() == 1 {
                rem.first().map_or_else(
                    || Err(make_invalid_padding_bytes_encoding_length(rem.len())),
                    |first| Ok((*first as u32) << 16),
                )
            }
            else {
                rem.first().map_or_else(
                    || Err(make_invalid_padding_bytes_encoding_length(rem.len())),
                    |first| {
                        rem.get(1).map_or_else(
                            || Err(make_invalid_padding_bytes_encoding_length(rem.len())),
                            |second| Ok(((*first as u32) << 16) | ((*second as u32) << 8)),
                        )
                    },
                )
            }?;

            checked_push(alphabet, &mut output, (n >> 18) & 0x3f)?;
            checked_push(alphabet, &mut output, (n >> 12) & 0x3f)?;

            if rem.len() == 2 {
                checked_push(alphabet, &mut output, (n >> 6) & 0x3f)?;
            }
            else if let Some(pad) = padding {
                output.push(pad);
            }

            if let Some(pad) = padding {
                output.push(pad);
            }
        }

        Ok(String::from_utf8(output).unwrap())
    }

    fn decode(&self, data: &str) -> Result<Vec<u8>, CryptError> {
        let alphabet = self.variant.get_alphabet();
        let padding = self.variant.get_padding();
        let bytes = data.as_bytes();

        let capacity = bytes.len().overflowing_mul(3);
        if capacity.1 {
            return Err(CryptError::DataTooLong(capacity.0));
        }

        let capacity = capacity.0.overflowing_div(4);
        if capacity.1 {
            return Err(CryptError::InvalidEncodingLength(
                "base64".to_owned(),
                bytes.len(),
            ));
        }
        let mut output = Vec::with_capacity(capacity.0);

        let mut buffer = [0u32; 4];
        let mut index = 0u8;

        for &byte in bytes {
            if Some(byte) == padding || byte == b'=' {
                break;
            }

            let value = alphabet
                .iter()
                .position(|&c| c == byte)
                .ok_or(CryptError::InvalidEncodingCharacter(
                    "base64".to_owned(),
                    byte as char,
                ))? as u32;

            if let Some(buffer_position) = buffer.get_mut(index as usize) {
                *buffer_position = value;
            }
            else {
                return Err(CryptError::InvalidEncodingLength(
                    "base64".to_owned(),
                    bytes.len(),
                ));
            }
            index = index.saturating_add(1);

            if index == 4 {
                let n = (buffer[0] << 18) | (buffer[1] << 12) | (buffer[2] << 6) | buffer[3];
                output.push(((n >> 16) & 0xff) as u8);
                output.push(((n >> 8) & 0xff) as u8);
                output.push((n & 0xff) as u8);
                index = 0;
            }
        }

        // Handle remaining bytes
        if index > 1 {
            let n = (buffer[0] << 18) | (buffer[1] << 12);
            output.push(((n >> 16) & 0xff) as u8);

            if index == 3 {
                let n = n | (buffer[2] << 6);
                output.push(((n >> 8) & 0xff) as u8);
            }
        }

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base64_encode_url_unpadded() {
        let data = b"Hello, World!".to_vec();

        let encoder = Encoder::new(Variant::UrlUnpadded);
        let encoded = encoder.encode(data.as_slice()).unwrap();
        assert_eq!(encoded, "SGVsbG8sIFdvcmxkIQ");

        // no_std_println!("Base64 URL Unpadded: {}", encoded);
    }

    #[test]
    fn test_base64_decode_url_unpadded() {
        let data = "SGVsbG8sIFdvcmxkIQ";

        let encoder = Encoder::new(Variant::UrlUnpadded);
        let decoded = encoder.decode(data).unwrap();
        assert_eq!(decoded, b"Hello, World!".to_vec());
    }

    #[test]
    fn test_base64_encode_url() {
        let data = b"Hello, World!".to_vec();

        let encoder = Encoder::new(Variant::Url);
        let encoded = encoder.encode(data.as_slice()).unwrap();
        assert_eq!(encoded, "SGVsbG8sIFdvcmxkIQ==");

        // no_std_println!("Base64 URL: {}", encoded);
    }

    #[test]
    fn test_base64_decode_url() {
        let data = "SGVsbG8sIFdvcmxkIQ==";

        let encoder = Encoder::new(Variant::Url);
        let decoded = encoder.decode(data).unwrap();
        assert_eq!(decoded, b"Hello, World!".to_vec());
    }

    #[test]
    fn test_base64_encode_unpadded() {
        let data = b"Hello, World!".to_vec();

        let encoder = Encoder::new(Variant::StandardUnpadded);
        let encoded = encoder.encode(data.as_slice()).unwrap();
        assert_eq!(encoded, "SGVsbG8sIFdvcmxkIQ");

        // no_std_println!("Base64 Unpadded: {}", encoded);
    }

    #[test]
    fn test_base64_decode_unpadded() {
        let data = "SGVsbG8sIFdvcmxkIQ";

        let encoder = Encoder::new(Variant::StandardUnpadded);
        let decoded = encoder.decode(data).unwrap();
        assert_eq!(decoded, b"Hello, World!".to_vec());
    }

    #[test]
    fn test_base64_encode() {
        let data = b"Hello, World!".to_vec();

        let encoder = Encoder::new(Variant::Standard);
        let encoded = encoder.encode(data.as_slice()).unwrap();
        assert_eq!(encoded, "SGVsbG8sIFdvcmxkIQ==");

        // no_std_println!("Base64: {}", encoded);
    }

    #[test]
    fn test_base64_decode() {
        let data = "SGVsbG8sIFdvcmxkIQ==";

        let encoder = Encoder::new(Variant::Standard);
        let decoded = encoder.decode(data).unwrap();
        assert_eq!(decoded, b"Hello, World!".to_vec());
    }
}
