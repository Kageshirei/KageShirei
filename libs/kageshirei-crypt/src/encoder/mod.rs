use alloc::{string::String, vec::Vec};

use crate::CryptError;

#[cfg(feature = "base32-encoding")]
pub mod base32;
#[cfg(feature = "base64-encoding")]
pub mod base64;
mod encoder;
mod encoding_variant;
#[cfg(feature = "hex-encoding")]
pub mod hex;

pub use encoder::Encoder;
pub use encoding_variant::{EncodingPadding, EncodingVariant};
