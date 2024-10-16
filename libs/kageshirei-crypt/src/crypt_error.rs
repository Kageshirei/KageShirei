use alloc::{
    boxed::Box,
    string::{String, ToString},
};
#[cfg(any(feature = "server", test))]
use core::fmt::Debug;
use core::{
    error::Error as ErrorTrait,
    fmt::{Display, Formatter},
};

pub enum CryptError {
    /// The key length is invalid (expected, received)
    InvalidKeyLength(u8, usize),
    /// The nonce length is invalid (expected, received)
    InvalidNonceLength(u8, usize),
    /// Cannot hash the provided data with Argon2
    #[cfg(feature = "argon2")]
    CannotHashArgon2(argon2::password_hash::Error),
    /// Cannot derive the key with Argon2
    #[cfg(feature = "argon2")]
    CannotDeriveArgon2(argon2::Error),
    /// The provided data cannot be hashed
    #[cfg(feature = "hkdf")]
    CannotHashOrDerive(hkdf::InvalidLength),
    /// Invalid character in input
    InvalidCharacterInput,
    /// Cannot decode the data
    CannotDecode,
    /// The public key is missing for or invalid for the operation
    MissingOrInvalidPublicKey,
    /// The secret key is missing for or invalid for the operation
    MissingOrInvalidSecretKey,
    /// The provided data cannot be encrypted
    #[cfg(feature = "xchacha20poly1305")]
    CannotEncryptWithChaCha20Poly1305(chacha20poly1305::Error),
    /// The provided data cannot be decrypted
    #[cfg(feature = "xchacha20poly1305")]
    CannotDecryptWithChaCha20Poly1305(chacha20poly1305::Error),
    /// The provided data is too long, overflowing the maximum size
    DataTooLong(usize),
    /// The provided data is too short, underflowing the minimum size
    DataTooShort(usize),
    /// Invalid encoding character (encoding, character)
    InvalidEncodingCharacter(String, char),
    /// Invalid encoding length (encoding, length)
    InvalidEncodingLength(String, usize),
}

#[cfg(any(feature = "server", test))]
impl Debug for CryptError {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        // Delegate to Display
        write!(f, "{}", self.to_string())
    }
}

impl Display for CryptError {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        match self {
            Self::InvalidKeyLength(bytes, received) => {
                write!(
                    f,
                    "Invalid key length, expected {} bytes, got {}",
                    bytes, received
                )
            },
            Self::InvalidNonceLength(bytes, received) => {
                write!(
                    f,
                    "Invalid nonce length, expected {} bytes, got {}",
                    bytes, received
                )
            },
            CryptError::CannotHashOrDerive(e) => {
                write!(f, "Cannot hash the provided data: {}", e.to_string())
            },
            CryptError::InvalidCharacterInput => {
                write!(f, "Invalid character in input")
            },
            CryptError::CannotDecode => {
                write!(f, "Cannot decode the data")
            },
            CryptError::CannotHashArgon2(e) => {
                write!(
                    f,
                    "Cannot hash the provided data with Argon2: {}",
                    e.to_string()
                )
            },
            CryptError::CannotDeriveArgon2(e) => {
                write!(f, "Cannot derive the key with Argon2: {}", e.to_string())
            },
            Self::MissingOrInvalidPublicKey => {
                write!(f, "The receiver public key is missing or invalid")
            },
            Self::MissingOrInvalidSecretKey => {
                write!(f, "The sender secret key is missing or invalid")
            },
            CryptError::CannotEncryptWithChaCha20Poly1305(e) => {
                write!(
                    f,
                    "Cannot encrypt the provided data with ChaCha20Poly1305: {}",
                    e.to_string()
                )
            },
            CryptError::CannotDecryptWithChaCha20Poly1305(e) => {
                write!(
                    f,
                    "Cannot decrypt the provided data with ChaCha20Poly1305: {}",
                    e.to_string()
                )
            },
            CryptError::DataTooLong(overflowing_size) => {
                write!(
                    f,
                    "The provided data is too long, overflowing the maximum size resulting in: {}",
                    overflowing_size
                )
            },
            CryptError::DataTooShort(underflowing_size) => {
                write!(
                    f,
                    "The provided data is too short, underflowing the minimum size resulting in: {}",
                    underflowing_size
                )
            },
            CryptError::InvalidEncodingCharacter(encoding, char) => {
                write!(
                    f,
                    "Invalid character in input for encoding '{}': '{}'",
                    encoding, char
                )
            },
            CryptError::InvalidEncodingLength(encoding, size) => {
                write!(
                    f,
                    "Invalid length in input for encoding '{}': '{}'",
                    encoding, size
                )
            },
        }
    }
}

#[cfg(any(feature = "server", test))]
impl ErrorTrait for CryptError {}
