use alloc::{sync::Arc, vec::Vec};
use core::cmp::Ordering;

use hkdf::{hmac::SimpleHmac, Hkdf};
use k256::{elliptic_curve::rand_core::RngCore as _, FieldBytes, PublicKey, SecretKey};
use rand::rngs::OsRng;
use sha3::Sha3_512;

use crate::{
    encryption_algorithm::{EncryptionAlgorithm, WithKeyDerivation},
    symmetric_encryption_algorithm::SymmetricEncryptionAlgorithm,
    CryptError,
};

pub struct KeyPair {
    pub secret_key: Option<Arc<SecretKey>>,
    pub public_key: Option<Arc<PublicKey>>,
}

/// An asymmetric encryption algorithm that uses a symmetric encryption algorithm for encryption and decryption
pub struct AsymmetricAlgorithm<T> {
    /// The sender keypair
    sender:             Arc<KeyPair>,
    /// The receiver keypair
    receiver:           Option<Arc<KeyPair>>,
    /// The implementation of a symmetric algorithm to use with this asymmetric algorithm
    algorithm_instance: T,
}

/// The size of the salt used for the HKDF key derivation function (128 bytes)
const HKDF_SALT_SIZE: usize = 0x80;

// Safety: AsymmetricAlgorithm is Send
unsafe impl<T> Send for AsymmetricAlgorithm<T> where T: Send {}

impl<T> Default for AsymmetricAlgorithm<T>
where
    T: SymmetricEncryptionAlgorithm + EncryptionAlgorithm + WithKeyDerivation,
{
    fn default() -> Self { Self::new() }
}

impl<T> AsymmetricAlgorithm<T>
where
    T: SymmetricEncryptionAlgorithm + EncryptionAlgorithm + WithKeyDerivation,
{
    /// Create a temporary secret key and return it as a bytes representation
    pub fn make_temporary_secret_key() -> Vec<u8> {
        let mut rng = OsRng;
        let secret_key = SecretKey::random(&mut rng).to_bytes();
        secret_key.to_vec()
    }

    /// Crate a bytes representation of the sender's public key
    pub fn serialize_public_key(&self) -> Result<Vec<u8>, CryptError> {
        if self.sender.public_key.is_none() {
            return Err(CryptError::MissingOrInvalidPublicKey);
        }

        Ok(Vec::from(
            self.sender.public_key.clone().unwrap().to_sec1_bytes(),
        ))
    }

    /// Crate a bytes representation of the sender's secret key
    pub fn serialize_secret_key(&self) -> Result<Vec<u8>, CryptError> {
        if self.sender.secret_key.is_none() {
            return Err(CryptError::MissingOrInvalidSecretKey);
        }

        let bytes = self.sender.secret_key.clone().unwrap().to_bytes();
        Ok(bytes.to_vec())
    }

    /// Set the receiver public key
    ///
    /// # Arguments
    ///
    /// * `public_key` - The public key of the receiver
    ///
    /// # Returns
    ///
    /// The updated current instance
    pub fn set_receiver(&mut self, receiver: Arc<KeyPair>) -> &mut Self {
        self.receiver = Some(receiver);
        self
    }

    /// Derive a shared secret from a given public key and return a new key derivation function instance for key
    /// generation
    ///
    /// # Arguments
    ///
    /// * `public_key` - The public key to derive the shared secret from
    ///
    /// # Returns
    ///
    /// A new key derivation function instance for secure-key generation
    pub fn derive_shared_secret(
        sender: Arc<KeyPair>,
        receiver: Option<Arc<KeyPair>>,
    ) -> Result<Hkdf<Sha3_512, SimpleHmac<Sha3_512>>, CryptError> {
        if sender.secret_key.is_none() {
            return Err(CryptError::MissingOrInvalidSecretKey);
        }

        let is_receiver_none = if let Some(ref keypair) = receiver &&
            keypair.public_key.is_none()
        {
            true
        }
        else {
            false
        };
        if receiver.is_none() || is_receiver_none {
            return Err(CryptError::MissingOrInvalidPublicKey);
        }

        // derive the shared secret
        let shared_secret = k256::ecdh::diffie_hellman(
            &sender.secret_key.clone().unwrap().to_nonzero_scalar(),
            receiver.unwrap().public_key.clone().unwrap().as_affine(),
        );

        // compute the salt
        let mut rng = OsRng;
        let mut salt = [0u8; HKDF_SALT_SIZE];
        rng.fill_bytes(&mut salt);

        Ok(shared_secret.extract::<Sha3_512>(None /* Some(&salt) */))
    }
}

impl<T> Clone for AsymmetricAlgorithm<T>
where
    T: SymmetricEncryptionAlgorithm + EncryptionAlgorithm + WithKeyDerivation,
{
    fn clone(&self) -> Self {
        Self {
            sender:             self.sender.clone(),
            receiver:           self.receiver.clone(),
            algorithm_instance: T::new(),
        }
    }
}

impl<T> AsymmetricAlgorithm<T>
where
    T: EncryptionAlgorithm + SymmetricEncryptionAlgorithm + WithKeyDerivation,
{
    /// Get the key from the provided key bytes
    ///
    /// if the key is less than 32 bytes, it will be padded with zeros
    /// if the key is more than 32 bytes, it will be truncated
    ///
    /// # Arguments
    ///
    /// * `key` - The key to get
    ///
    /// # Returns
    ///
    /// The key as a 32 bytes array
    fn get_key(key: &[u8]) -> Vec<u8> {
        let mut key = Vec::from(key);
        match key.len().cmp(&32) {
            Ordering::Less => {
                // Pad the key with zeros to reach the required length of 32 bytes, this is not
                // secure, but it's better than panicking
                key.resize(32, 0);
                key
            },
            Ordering::Equal => key,
            Ordering::Greater => {
                // Truncate the key to the required length of 32 bytes if it's longer
                key.truncate(32);
                key
            },
        }
    }
}

impl<T> TryFrom<&[u8]> for AsymmetricAlgorithm<T>
where
    T: EncryptionAlgorithm + SymmetricEncryptionAlgorithm + WithKeyDerivation,
{
    type Error = CryptError;

    fn try_from(key: &[u8]) -> Result<Self, Self::Error> {
        let key = Self::get_key(key);

        let field_bytes = FieldBytes::from_slice(&key);

        let secret_key = Arc::new(SecretKey::from_bytes(field_bytes).unwrap());
        let public_key = Some(Arc::new(secret_key.public_key()));
        let secret_key = Some(secret_key);

        Ok(Self {
            sender:             Arc::new(KeyPair {
                secret_key,
                public_key,
            }),
            algorithm_instance: T::new(),
            receiver:           None,
        })
    }
}

impl<T> EncryptionAlgorithm for AsymmetricAlgorithm<T>
where
    T: SymmetricEncryptionAlgorithm + EncryptionAlgorithm + WithKeyDerivation,
{
    fn encrypt(&mut self, data: &[u8]) -> Result<Vec<u8>, CryptError> {
        // proxy the encryption to the symmetric algorithm
        self.algorithm_instance.encrypt(data)
    }

    fn decrypt(&self, data: &[u8], key: Option<&[u8]>) -> Result<Vec<u8>, CryptError> {
        // proxy the decryption to the symmetric algorithm
        self.algorithm_instance.decrypt(data, key)
    }

    /// Create a new key pair
    fn new() -> Self {
        let mut rng = OsRng;

        let secret_key = Arc::new(SecretKey::random(&mut rng));
        let public_key = Some(Arc::new(secret_key.public_key()));
        let secret_key = Some(secret_key);

        Self {
            sender:             Arc::new(KeyPair {
                secret_key,
                public_key,
            }),
            algorithm_instance: T::new(),
            receiver:           None,
        }
    }

    fn make_key(&mut self) -> Result<&mut Self, CryptError> {
        if self.receiver.is_none() {
            return Err(CryptError::MissingOrInvalidPublicKey);
        }

        let derived_key = Self::derive_shared_secret(self.sender.clone(), self.receiver.clone())?;

        let algorithm_instance = self.algorithm_instance.clone();
        self.algorithm_instance = T::derive_key(algorithm_instance, derived_key)?;

        Ok(self)
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::encryption_algorithm::xchacha20poly1305_algorithm::XChaCha20Poly1305Algorithm;

    #[test]
    fn test_private_key() {
        let algorithm = AsymmetricAlgorithm::<XChaCha20Poly1305Algorithm>::new();
        let serialized = algorithm.serialize_secret_key().unwrap();
        // no_std_println!(
        //     "Secret key (length: {}): {:?}",
        //     serialized.len(),
        //     serialized
        // );

        let serialized = algorithm.serialize_public_key().unwrap();
        // no_std_println!(
        //     "Public key (length: {}): {:?}",
        //     serialized.len(),
        //     serialized
        // );
    }

    #[test]
    fn test_asymmetric_algorithm() {
        let mut bob = AsymmetricAlgorithm::<XChaCha20Poly1305Algorithm>::new();
        let mut alice = AsymmetricAlgorithm::<XChaCha20Poly1305Algorithm>::new();

        let data = b"Hello, world!".to_vec();

        // bob sends a message to alice
        let bob = bob
            .set_receiver(Arc::new(KeyPair {
                secret_key: alice.sender.secret_key.clone(),
                public_key: alice.sender.public_key.clone(),
            }))
            .make_key()
            .expect("Cannot make key");
        let encrypted = bob.encrypt(data.as_slice()).unwrap();

        // no_std_println!(
        //     "Encrypted: {:?} (\"{}\")",
        //     encrypted,
        //     encrypted.iter().map(|&c| c as char).collect::<String>()
        // );

        // alice receives the message from bob
        let alice = alice
            .set_receiver(Arc::new(KeyPair {
                secret_key: bob.sender.secret_key.clone(),
                public_key: bob.sender.public_key.clone(),
            }))
            .make_key()
            .expect("Cannot make key");
        let decrypted = alice.decrypt(encrypted.as_slice(), None).unwrap();

        // no_std_println!(
        //     "Decrypted: {:?} (\"{}\")",
        //     decrypted,
        //     decrypted.iter().map(|&c| c as char).collect::<String>()
        // );

        assert_eq!(data, decrypted);
    }
}
