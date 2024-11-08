//! Decrypt the provided body using the symmetric encryption scheme or fail if the decryption fails

use axum::response::Response;
use srv_mod_config::handlers::EncryptionAlgorithm;

/// Decrypt the provided body using the symmetric encryption scheme or fail if the decryption fails
pub const fn decrypt_symmetric_or_fail(
    algorithm: Option<&EncryptionAlgorithm>,
    body: Vec<u8>,
) -> Result<Vec<u8>, Response> {
    // TODO: Implement the symmetric decryption
    Ok(body)
}
