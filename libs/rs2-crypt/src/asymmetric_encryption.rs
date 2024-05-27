use bytes::Bytes;
use hkdf::Hkdf;
use hkdf::hmac::SimpleHmac;
use k256::{PublicKey, SecretKey};
use k256::elliptic_curve::rand_core::RngCore;
use sha3::Sha3_512;

pub struct KeyPair {
	pub secret_key: SecretKey,
	pub public_key: PublicKey,
}

impl KeyPair {
	/// The size of the salt used for the HKDF key derivation
	const HKDF_SALT_SIZE: usize = 128;

	/// Create a new key pair
	pub fn new() -> Self {
		let mut rng = rand::thread_rng();

		let secret_key = SecretKey::random(&mut rng);
		let public_key = secret_key.public_key();

		Self {
			public_key,
			secret_key,
		}
	}

	/// Crate a bytes representation of the public key
	pub fn serialize_public_key(&self) -> Bytes {
		Bytes::from(self.public_key.to_sec1_bytes())
	}

	/// Crate a bytes representation of the secret key
	pub fn serialize_secret_key(&self) -> Bytes {
		Bytes::from(self.secret_key.to_bytes().as_slice())
	}

	/// Derive a shared secret from a given public key and return a new key derivation function instance for key generation
	///
	/// # Arguments
	///
	/// * `public_key` - The public key to derive the shared secret from
	///
	/// # Returns
	///
	/// A new key derivation function instance for secure-key generation
	pub fn derive_shared_secret(&self, public_key: &PublicKey) -> Hkdf<Sha3_512, SimpleHmac<Sha3_512>> {
		// derive the shared secret
		let shared_secret = k256::ecdh::diffie_hellman(
			&self.secret_key.to_nonzero_scalar(),
			public_key.as_affine(),
		);

		// compute the salt
		let mut rng = rand::thread_rng();
		let mut salt = [0u8; Self::HKDF_SALT_SIZE];
		rng.fill_bytes(&mut salt);

		shared_secret.extract::<Sha3_512>(Some(&salt))
	}
}

