use anyhow::Result;
use bytes::Bytes;

use crate::encoder::Encoder;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HexEncoder;

impl Encoder for HexEncoder {
    fn encode(&self, data: Bytes) -> String { base16ct::lower::encode_string(data.as_ref()) }

    fn decode(&self, data: &str) -> Result<Bytes> {
        Ok(Bytes::from(
            base16ct::lower::decode_vec(data).map_err(|e| anyhow::anyhow!(e))?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoder::Encoder;

    #[test]
    fn test_encode() {
        let encoder = HexEncoder;
        let data = Bytes::from_static(b"Hello, World!");
        let encoded = encoder.encode(data);
        assert_eq!(encoded, "48656c6c6f2c20576f726c6421");
    }

    #[test]
    fn test_decode() {
        let encoder = HexEncoder;
        let data = "48656c6c6f2c20576f726c6421";
        let decoded = encoder.decode(data).unwrap();
        assert_eq!(decoded, Bytes::from_static(b"Hello, World!"));
    }
}
