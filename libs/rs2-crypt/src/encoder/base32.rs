use anyhow::Result;
use base32ct::Encoding;
use bytes::Bytes;

use crate::encoder::Encoder;

pub struct Base32Encoder;

impl Encoder for Base32Encoder {
	fn encode(&self, data: Bytes) -> String {
		base32ct::Base32Unpadded::encode_string(data.as_ref())
	}

	fn decode(&self, data: &str) -> Result<Bytes> {
		Ok(Bytes::from(base32ct::Base32Unpadded::decode_vec(data).map_err(|e| anyhow::anyhow!(e))?))
	}
}

#[cfg(test)]
mod tests {
	use crate::encoder::Encoder;

	use super::*;

	#[test]
	fn test_encode() {
		let encoder = Base32Encoder;
		let data = Bytes::from_static(b"Hello, World!");
		let encoded = encoder.encode(data);
		assert_eq!(encoded, "jbswy3dpfqqfo33snrscc");
	}

	#[test]
	fn test_decode() {
		let encoder = Base32Encoder;
		let data = "jbswy3dpfqqfo33snrscc";
		let decoded = encoder.decode(data).unwrap();
		assert_eq!(decoded, Bytes::from_static(b"Hello, World!"));
	}
}