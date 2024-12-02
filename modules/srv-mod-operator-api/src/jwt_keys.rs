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

#[cfg(test)]
mod tests {
    use std::{ops::Add, time::Duration};

    use chrono::{format::OffsetPrecision::Seconds, TimeDelta, Utc};
    use jsonwebtoken::{DecodingKey, EncodingKey};
    use serde::{Deserialize, Serialize};

    use super::*;

    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
    struct Sample {
        exp: i32,
    }

    // Test that the Keys struct can be created correctly
    #[test]
    #[serial_test::serial]
    fn test_keys_creation() {
        let secret: &[u8] = b"my_secret_key";

        let keys = Keys::new(secret);

        let encoding = EncodingKey::from_secret(secret);
        let decoding = DecodingKey::from_secret(secret);

        let sample = Sample {
            exp: Utc::now().add(TimeDelta::minutes(30)).timestamp() as i32,
        };

        let encoded = jsonwebtoken::encode(&jsonwebtoken::Header::default(), &sample, &keys.encoding).unwrap();
        let encoded_check = jsonwebtoken::encode(&jsonwebtoken::Header::default(), &sample, &encoding).unwrap();

        // Check that the encoding key is created correctly
        assert_eq!(encoded, encoded_check);

        let decoded = jsonwebtoken::decode::<Sample>(
            &encoded,
            &keys.decoding,
            &jsonwebtoken::Validation::default(),
        )
        .unwrap();
        let decoded_check = jsonwebtoken::decode::<Sample>(
            &encoded_check,
            &decoding,
            &jsonwebtoken::Validation::default(),
        )
        .unwrap();

        // Check that the decoding key is created correctly
        assert_eq!(decoded.claims, decoded_check.claims);
    }

    // Test the initialization of the API_SERVER_JWT_KEYS static variable
    #[test]
    #[serial_test::serial]
    fn test_api_server_jwt_keys_initialization() {
        let secret: &[u8] = b"my_secret_key";

        // Initialize the API_SERVER_JWT_KEYS with the secret
        let keys = Keys::new(secret);
        let _ = API_SERVER_JWT_KEYS.set(keys);

        // Test that the OnceCell contains the correct keys
        let stored_keys = API_SERVER_JWT_KEYS.get().unwrap();

        let encoding = EncodingKey::from_secret(secret);
        let decoding = DecodingKey::from_secret(secret);

        let sample = Sample {
            exp: Utc::now().add(TimeDelta::minutes(30)).timestamp() as i32,
        };

        let encoded = jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &sample,
            &stored_keys.encoding,
        )
        .unwrap();
        let encoded_check = jsonwebtoken::encode(&jsonwebtoken::Header::default(), &sample, &encoding).unwrap();

        // Check that the encoding key is created correctly
        assert_eq!(encoded, encoded_check);

        let decoded = jsonwebtoken::decode::<Sample>(
            &encoded,
            &stored_keys.decoding,
            &jsonwebtoken::Validation::default(),
        )
        .unwrap();
        let decoded_check = jsonwebtoken::decode::<Sample>(
            &encoded_check,
            &decoding,
            &jsonwebtoken::Validation::default(),
        )
        .unwrap();

        // Check that the decoding key is created correctly
        assert_eq!(decoded.claims, decoded_check.claims);
    }

    // Test that the OnceCell can only be set once
    #[test]
    #[serial_test::serial]
    fn test_once_cell_single_set() {
        let secret: &[u8] = b"my_secret_key";

        // First initialization should succeed
        let keys = Keys::new(secret);
        let _ = API_SERVER_JWT_KEYS.set(keys); // comment out this line and uncomment the next one in isolation
                                               // assert!(API_SERVER_JWT_KEYS.set(keys).is_ok()); this works in isolation only

        // Second initialization should fail
        let second_keys = Keys::new(secret);
        assert!(API_SERVER_JWT_KEYS.set(second_keys).is_err());
    }
}
