//! Utility functions to decode the body of a request or return a failed response

use axum::{
    http::StatusCode,
    response::{IntoResponse as _, Response},
};
use kageshirei_crypt::encoder::{
    base32::Encoder as Base32Encoder,
    base64::{Encoder as Base64Encoder, Variant as Base64Variant},
    hex::Encoder as HexEncoder,
    Encoder as _,
};
use srv_mod_config::handlers::Encoder;
use tracing::warn;

/// Converts a byte array to a string
pub fn bytes_to_string(bytes: &[u8]) -> String { bytes.iter().map(|b| *b as char).collect::<String>() }

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
pub fn decode_or_fail_response(encoder: &Encoder, body: Vec<u8>) -> Result<Vec<u8>, Response> {
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
