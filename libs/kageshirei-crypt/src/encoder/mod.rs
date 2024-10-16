#[cfg(feature = "base32-encoding")]
pub mod base32;
#[cfg(feature = "base64-encoding")]
pub mod base64;
#[expect(
    clippy::module_inception,
    reason = "The module is kept private while the struct is directly exported here."
)]
mod encoder;
mod encoding_variant;
#[cfg(feature = "hex-encoding")]
pub mod hex;

pub use encoder::Encoder;
pub use encoding_variant::{EncodingPadding, EncodingVariant};
