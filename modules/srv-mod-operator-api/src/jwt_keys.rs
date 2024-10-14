use jsonwebtoken::{DecodingKey, EncodingKey};
use once_cell::sync::OnceCell;

pub static API_SERVER_JWT_KEYS: OnceCell<Keys> = OnceCell::new();

/// JWT keypair for the api server
pub struct Keys {
    pub encoding: EncodingKey,
    pub decoding: DecodingKey,
}

impl Keys {
    pub fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}
