pub mod ident_algorithm;

#[cfg(feature = "asymmetric-encryption")]
mod asymmetric_encryption_algorithm;
#[cfg(feature = "asymmetric-encryption")]
pub use asymmetric_encryption_algorithm::AsymmetricAlgorithm;

#[cfg(any(feature = "symmetric-encryption", feature = "xchacha20poly1305"))]
pub mod xchacha20poly1305_algorithm;

#[cfg(feature = "hkdf")]
mod with_key_derivation;

#[cfg(feature = "hkdf")]
pub use with_key_derivation::WithKeyDerivation;

pub mod algorithms;
