use base64ct::Encoding;
use bytes::Bytes;

use crate::encoder::Encoder;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Base64Encoder;

impl Encoder for Base64Encoder {
	fn encode(&self, data: Bytes) -> String {
		base64ct::Base64UrlUnpadded::encode_string(data.as_ref())
	}

	fn decode(&self, data: &str) -> anyhow::Result<Bytes> {
		Ok(Bytes::from(base64ct::Base64UrlUnpadded::decode_vec(data).map_err(|e| anyhow::anyhow!(e))?))
	}
}

#[cfg(test)]
mod tests {
	use bytes::Bytes;

	use super::*;

	#[test]
	fn test_base64_encode() {
		let encoder = Base64Encoder;
		let data = Bytes::from_static(b"Hello, World!");
		let encoded = encoder.encode(data);
		assert_eq!(encoded, "SGVsbG8sIFdvcmxkIQ");
	}

	#[test]
	fn test_base64_decode() {
		let encoder = Base64Encoder;
		let data = "SGVsbG8sIFdvcmxkIQ";
		let decoded = encoder.decode(data).unwrap();
		assert_eq!(decoded, Bytes::from_static(b"Hello, World!"));
	}
}