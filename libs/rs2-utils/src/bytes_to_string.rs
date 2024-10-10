use bytes::Bytes;

/// Converts a byte array to a string
pub fn bytes_to_string(bytes: Bytes) -> String {
	bytes.iter()
	     .map(|b| *b as char)
	     .collect::<String>()
}