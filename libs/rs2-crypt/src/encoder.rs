use bytes::Bytes;

#[cfg(feature = "hex-encoding")]
pub mod hex;
#[cfg(feature = "base32-encoding")]
pub mod base32;
#[cfg(feature = "base64-encoding")]
pub mod base64;

pub trait Encoder {
	/// Encode the given data
	///
	/// # Arguments
	///
	/// * `data` - The data to encode
	///
	/// # Returns
	///
	/// The encoded data
	fn encode(&self, data: Bytes) -> String;

	/// Decode the given data
	///
	/// # Arguments
	///
	/// * `data` - The data to decode
	///
	/// # Returns
	///
	/// The decoded data
	fn decode(&self, data: &str) -> anyhow::Result<Bytes>;
}