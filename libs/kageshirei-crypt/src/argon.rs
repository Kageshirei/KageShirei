use argon2::{password_hash::SaltString, PasswordHash, PasswordHasher as _, PasswordVerifier as _};
use bytes::Bytes;
use rand::thread_rng;

pub struct Argon2;

impl Argon2 {
    /// Hash a password using Argon2
    ///
    /// Returns a PHC string ($argon2id$v=19$...)
    ///
    /// # Arguments
    ///
    /// * `password` - The password to hash
    ///
    /// # Returns
    ///
    /// The hashed password
    pub fn hash_password(password: &str) -> Result<String, String> {
        // initialize the SRNG
        let rng = thread_rng();

        // generate a random salt
        let salt = SaltString::generate(rng);

        // configure the hashing algorithm Argon2 with default params (Argon2id v19)
        let config = argon2::Argon2::default();

        // Hash password to PHC string ($argon2id$v=19$...)
        let hash = config
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| e.to_string())?
            .to_string();

        Ok(hash)
    }

    /// Verify a password against a hash
    ///
    /// # Arguments
    ///
    /// * `password` - The password to verify
    /// * `hash` - The hash to verify against
    ///
    /// # Returns
    ///
    /// True if the password matches the hash, false otherwise
    pub fn verify_password(password: &str, hash: &str) -> bool {
        let parsed_hash = PasswordHash::new(hash).unwrap();

        argon2::Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok()
    }

    /// Derive a key from a password
    ///
    /// # Arguments
    ///
    /// * `password` - The password to derive the key from
    /// * `salt` - The salt to use for the key derivation
    /// * `output_length` - The length of the derived key
    ///
    /// # Returns
    ///
    /// The derived key as a byte array
    pub fn derive_key(password: &str, salt: Option<Vec<u8>>, output_length: u32) -> Result<Bytes, String> {
        // initialize the salt if not provided
        let salt = salt.unwrap_or_else(|| {
            let rng = thread_rng();
            SaltString::generate(rng).to_string().as_bytes().to_vec()
        });

        let mut result = vec![0u8; output_length as usize];
        argon2::Argon2::default()
            .hash_password_into(password.as_bytes(), &salt, &mut result)
            .map_err(|e| format!("{:?}", e))?;

        Ok(Bytes::from(result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_password() {
        let password = "password";
        let hash = Argon2::hash_password(password).unwrap();
        assert!(Argon2::verify_password(password, &hash));
        println!("Hashed password: {}", hash)
    }

    #[test]
    fn test_derive_key() {
        let password = "password";

        let mut salt = vec![0u8; 16];
        salt.fill(0u8);

        let output_length = 32;
        let key = Argon2::derive_key(password, Some(salt), output_length).unwrap();
        assert_eq!(key.len(), output_length as usize);
        println!("Derived key: {:?}", key)
    }
}
