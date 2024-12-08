//! Utility functions to decode the body of a request or return a failed response

use std::num::NonZeroU16;

use axum::http::StatusCode;
use kageshirei_crypt::encoder::{
    base32::Encoder as Base32Encoder,
    base64::{Encoder as Base64Encoder, Variant as Base64Variant},
    hex::Encoder as HexEncoder,
    Encoder as _,
};
use srv_mod_config::handlers::Encoder;
use tracing::warn;

use crate::response::BaseHandlerResponse;

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
        return Err(BaseHandlerResponse {
            status:    NonZeroU16::try_from(StatusCode::OK.as_u16()).unwrap_or(NonZeroU16::new(200).unwrap()),
            body:      vec![],
            formatter: None,
        });
    }

    Ok(decoded.unwrap())
}

#[cfg(test)]
mod test {
    use std::num::NonZeroU16;

    use axum::http::StatusCode;

    use super::*;

    #[test]
    fn test_bytes_to_string() {
        let bytes = b"Hello, World!";
        let result = bytes_to_string(bytes);
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn test_decode_or_fail_response() {
        let encoder = Encoder::Hex;
        let body = b"48656c6c6f2c20576f726c6421".to_vec();

        let result = decode_or_fail_response(&encoder, body.clone());
        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(result, b"Hello, World!".to_vec());
    }

    #[test]
    fn test_decode_or_fail_response_unknown_format() {
        let encoder = Encoder::Base32;
        let body = b"Hello, World!".to_vec();

        let result = decode_or_fail_response(&encoder, body.clone());
        assert!(result.is_err());

        let result = result.err().unwrap();
        assert_eq!(
            result.status,
            NonZeroU16::try_from(StatusCode::OK.as_u16()).unwrap_or(NonZeroU16::new(200).unwrap())
        );
        assert_eq!(result.body, Vec::<u8>::new());
        assert_eq!(result.formatter, None);
    }
}
