//! Decrypt the provided body using the asymmetric encryption scheme or fail if the decryption fails

use srv_mod_config::handlers::EncryptionAlgorithm;

use crate::response::BaseHandlerResponse;

/// Decrypt the provided body using the asymmetric encryption scheme or fail if the decryption fails
pub const fn decrypt_asymmetric_or_fail(
    algorithm: Option<&EncryptionAlgorithm>,
    body: Vec<u8>,
) -> Result<Vec<u8>, BaseHandlerResponse> {
    // TODO: Implement the asymmetric decryption
    Ok(body)
}
