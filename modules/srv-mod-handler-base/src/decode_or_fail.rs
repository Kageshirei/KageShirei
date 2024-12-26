use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use bytes::Bytes;
use kageshirei_crypt::encoder::{base32::Base32Encoder, base64::Base64Encoder, hex::HexEncoder, Encoder as _};
use kageshirei_utils::bytes_to_string::bytes_to_string;
use srv_mod_config::handlers::{Encoder, EncryptionScheme};
use tracing::warn;

/// Decodes the body of the request based on the encoder or return a failed response
///
/// # Arguments
///
/// * `encoder` - The encoder to use to decode the body
/// * `body` - The body to decode
///
/// # Returns
///
/// The decoded body or a failed response
#[allow(
    clippy::module_name_repetitions,
    reason = "The name repetition clarifies the purpose of the function."
)]
pub fn decode_or_fail_response(encoder: &Encoder, body: Vec<u8>) -> Result<Vec<u8>, BaseHandlerResponse> {
    let decoded = match *encoder {
        Encoder::Hex => HexEncoder.decode(bytes_to_string(body.as_slice()).as_str()),
        Encoder::Base32 => Base32Encoder.decode(bytes_to_string(body.as_slice()).as_str()),
        Encoder::Base64 => {
            Base64Encoder::new(Base64Variant::UrlUnpadded).decode(bytes_to_string(body.as_slice()).as_str())
        },
    };

    if decoded.is_err() {
        // if no protocol matches, drop the request
        warn!(
            "Unknown format (not {}), request refused",
            encoder.to_string()
        );
        warn!("Internal status code: {}", StatusCode::BAD_REQUEST);

        // always return OK to avoid leaking information
        return Err((StatusCode::OK, "").into_response());
    }

    Ok(decoded.unwrap())
}
