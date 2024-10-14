use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Serialize, Deserialize, Debug, Validate, Clone, Default)]
pub struct JwtConfig {
    /// The JWT secret key for encoding and decoding, base64 encoded so it must be 88 characters long (64 bytes once
    /// decoded)
    #[validate(length(equal = 88))]
    pub secret: String,
}
