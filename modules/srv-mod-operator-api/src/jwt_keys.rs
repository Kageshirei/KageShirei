//! JWT keypair for the api server

use jsonwebtoken::{DecodingKey, EncodingKey};
use once_cell::sync::OnceCell;

/// The JWT keypair for the api server
pub static API_SERVER_JWT_KEYS: OnceCell<Keys> = OnceCell::new();

/// JWT keypair for the api server
pub struct Keys {
    /// The key used to encode JWTs
    pub encoding: EncodingKey,
    /// The key used to decode JWTs
    pub decoding: DecodingKey,
}

impl Keys {
    /// Create a new keypair from the given secret
    pub fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}
