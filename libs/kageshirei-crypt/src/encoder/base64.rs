use alloc::{borrow::ToOwned, string::String, vec::Vec};

use base64ct::Encoding as _;

use crate::{
    encoder::{Encoder, EncodingPadding, EncodingVariant},
    CryptError,
};

pub enum Base64Variant {
    UrlUnpadded,
    Url,
    Standard,
    StandardUnpadded,
}

impl EncodingVariant for Base64Variant {
    fn get_alphabet(&self) -> &'static [u8] {
        match self {
            Self::Url | Self::UrlUnpadded => b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_",
            Self::Standard | Self::StandardUnpadded => {
                b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/"
            },
        }
    }
}

impl EncodingPadding for Base64Variant {
    fn get_padding(&self) -> Option<u8> {
        match self {
            Self::UrlUnpadded | Self::StandardUnpadded => None,
            Self::Url | Self::Standard => Some(b'='),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Base64Encoder<T>
where
    T: EncodingVariant + EncodingPadding,
{
    variant: T,
}

impl Base64Encoder<Base64Variant> {
    pub fn new(variant: Base64Variant) -> Self {
        Self {
            variant,
        }
    }
}

impl<T> Encoder for Base64Encoder<T>
where
    T: EncodingVariant + EncodingPadding,
{
    fn encode(&self, data: &[u8]) -> String {
        let alphabet = self.variant.get_alphabet();
        let padding = self.variant.get_padding();
        let mut output = Vec::with_capacity((data.len() * 4 + 2) / 3);

        let mut chunks = data.chunks_exact(3);

        // Process each chunk of 3 bytes
        for chunk in &mut chunks {
            let n = ((chunk[0] as u32) << 16) | ((chunk[1] as u32) << 8) | (chunk[2] as u32);

            output.push(alphabet[((n >> 18) & 0x3f) as usize]);
            output.push(alphabet[((n >> 12) & 0x3f) as usize]);
            output.push(alphabet[((n >> 6) & 0x3f) as usize]);
            output.push(alphabet[(n & 0x3f) as usize]);
        }

        // Handle remaining bytes (if any)
        let rem = chunks.remainder();
        if !rem.is_empty() {
            let n = match rem.len() {
                1 => (rem[0] as u32) << 16,
                2 => ((rem[0] as u32) << 16) | ((rem[1] as u32) << 8),
                _ => unreachable!(),
            };

            output.push(alphabet[((n >> 18) & 0x3f) as usize]);
            output.push(alphabet[((n >> 12) & 0x3f) as usize]);

            if rem.len() == 2 {
                output.push(alphabet[((n >> 6) & 0x3f) as usize]);
            }
            else if let Some(pad) = padding {
                output.push(pad);
            }

            if let Some(pad) = padding {
                output.push(pad);
            }
        }

        String::from_utf8(output).unwrap()
    }

    fn decode(&self, data: &str) -> Result<Vec<u8>, CryptError> {
        let alphabet = self.variant.get_alphabet();
        let padding = self.variant.get_padding();
        let bytes = data.as_bytes();
        let mut output = Vec::with_capacity((bytes.len() * 3) / 4);

        let mut buffer = [0u32; 4];
        let mut index = 0;

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

            buffer[index] = value;
            index += 1;

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
    use alloc::format;

    use super::*;
    use crate::no_std_println;

    #[test]
    fn test_base64_encode_url_unpadded() {
        let data = b"Hello, World!".to_vec();

        let encoder = Base64Encoder::new(Base64Variant::UrlUnpadded);
        let encoded = encoder.encode(data.as_slice());
        assert_eq!(
            encoded,
            base64ct::Base64UrlUnpadded::encode_string(data.as_slice())
        );
        assert_eq!(encoded, "SGVsbG8sIFdvcmxkIQ");

        // no_std_println!("Base64 URL Unpadded: {}", encoded);
    }

    #[test]
    fn test_base64_decode_url_unpadded() {
        let data = "SGVsbG8sIFdvcmxkIQ";

        let encoder = Base64Encoder::new(Base64Variant::UrlUnpadded);
        let decoded = encoder.decode(data).unwrap();
        assert_eq!(
            decoded,
            base64ct::Base64UrlUnpadded::decode_vec(data).unwrap()
        );
        assert_eq!(decoded, b"Hello, World!".to_vec());
    }

    #[test]
    fn test_base64_encode_url() {
        let data = b"Hello, World!".to_vec();

        let encoder = Base64Encoder::new(Base64Variant::Url);
        let encoded = encoder.encode(data.as_slice());
        assert_eq!(encoded, base64ct::Base64Url::encode_string(data.as_slice()));
        assert_eq!(encoded, "SGVsbG8sIFdvcmxkIQ==");

        // no_std_println!("Base64 URL: {}", encoded);
    }

    #[test]
    fn test_base64_decode_url() {
        let data = "SGVsbG8sIFdvcmxkIQ==";

        let encoder = Base64Encoder::new(Base64Variant::Url);
        let decoded = encoder.decode(data).unwrap();
        assert_eq!(decoded, base64ct::Base64Url::decode_vec(data).unwrap());
        assert_eq!(decoded, b"Hello, World!".to_vec());
    }

    #[test]
    fn test_base64_encode_unpadded() {
        let data = b"Hello, World!".to_vec();

        let encoder = Base64Encoder::new(Base64Variant::StandardUnpadded);
        let encoded = encoder.encode(data.as_slice());
        assert_eq!(
            encoded,
            base64ct::Base64Unpadded::encode_string(data.as_slice())
        );
        assert_eq!(encoded, "SGVsbG8sIFdvcmxkIQ");

        // no_std_println!("Base64 Unpadded: {}", encoded);
    }

    #[test]
    fn test_base64_decode_unpadded() {
        let data = "SGVsbG8sIFdvcmxkIQ";

        let encoder = Base64Encoder::new(Base64Variant::StandardUnpadded);
        let decoded = encoder.decode(data).unwrap();
        assert_eq!(decoded, base64ct::Base64Unpadded::decode_vec(data).unwrap());
        assert_eq!(decoded, b"Hello, World!".to_vec());
    }

    #[test]
    fn test_base64_encode() {
        let data = b"Hello, World!".to_vec();

        let encoder = Base64Encoder::new(Base64Variant::Standard);
        let encoded = encoder.encode(data.as_slice());
        assert_eq!(encoded, base64ct::Base64::encode_string(data.as_slice()));
        assert_eq!(encoded, "SGVsbG8sIFdvcmxkIQ==");

        // no_std_println!("Base64: {}", encoded);
    }

    #[test]
    fn test_base64_decode() {
        let data = "SGVsbG8sIFdvcmxkIQ==";

        let encoder = Base64Encoder::new(Base64Variant::Standard);
        let decoded = encoder.decode(data).unwrap();
        assert_eq!(decoded, base64ct::Base64::decode_vec(data).unwrap());
        assert_eq!(decoded, b"Hello, World!".to_vec());
    }
}
