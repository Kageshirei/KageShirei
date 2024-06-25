use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use bytes::Bytes;
use tracing::warn;

use rs2_crypt::encoder::base32::Base32Encoder;
use rs2_crypt::encoder::base64::Base64Encoder;
use rs2_crypt::encoder::Encoder as _;
use rs2_crypt::encoder::hex::HexEncoder;
use rs2_utils::bytes_to_string::bytes_to_string;
use srv_mod_config::handlers::{Encoder, EncryptionScheme};

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
pub(crate) fn decode_or_fail_response(encoder: &Encoder, body: Bytes) -> Result<Bytes, Response> {
	let decoded = match encoder {
		Encoder::Hex => {
			HexEncoder::default().decode(
				bytes_to_string(body).as_str()
			)
		}
		Encoder::Base32 => {
			Base32Encoder::default().decode(
				bytes_to_string(body).as_str()
			)
		}
		Encoder::Base64 => {
			Base64Encoder::default().decode(
				bytes_to_string(body).as_str()
			)
		}
	};

	if decoded.is_err() {
		// if no protocol matches, drop the request
		warn!("Unknown format (not {}), request refused", encoder.to_string());
		warn!("Internal status code: {}", StatusCode::BAD_REQUEST);

		// always return OK to avoid leaking information
		return Err((StatusCode::OK, "").into_response());
	}

	Ok(decoded.unwrap())
}