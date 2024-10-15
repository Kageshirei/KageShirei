use axum::response::Response;
use bytes::Bytes;
use srv_mod_config::handlers::EncryptionAlgorithm;

/// Decrypt the provided body using the asymmetric encryption scheme or fail if the decryption fails
pub const fn decrypt_asymmetric_or_fail(
    algorithm: Option<&EncryptionAlgorithm>,
    body: Bytes,
) -> Result<Bytes, Response> {
    // TODO: Implement the asymmetric decryption
    Ok(body)
}
