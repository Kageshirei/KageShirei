use axum::response::Response;
use bytes::Bytes;
use srv_mod_config::handlers::EncryptionAlgorithm;

/// Decrypt the provided body using the symmetric encryption scheme or fail if the decryption fails
pub fn decrypt_symmetric_or_fail(algorithm: Option<&EncryptionAlgorithm>, body: Bytes) -> Result<Bytes, Response> {
	// TODO: Implement the symmetric decryption
	Ok(body)
}