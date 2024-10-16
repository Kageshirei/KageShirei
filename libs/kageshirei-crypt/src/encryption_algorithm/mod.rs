mod encryption_algorithm;
pub use encryption_algorithm::EncryptionAlgorithm;
pub mod ident_algorithm;

#[cfg(feature = "asymmetric-encryption")]
pub mod asymmetric_algorithm;

#[cfg(any(feature = "symmetric-encryption", feature = "xchacha20poly1305"))]
pub mod xchacha20poly1305_algorithm;

#[cfg(feature = "hkdf")]
mod with_key_derivation;
#[cfg(feature = "hkdf")]
pub use with_key_derivation::WithKeyDerivation;
