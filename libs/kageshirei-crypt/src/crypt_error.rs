use alloc::string::{String, ToString};
#[cfg(any(feature = "server", test))]
use core::{
    error::Error as ErrorTrait,
    fmt::Debug,
    fmt::{Display, Formatter},
};

/// The wrapper type for the hkdf::InvalidLength error that does not implement the PartialEq nor Eq
/// traits
#[cfg(feature = "hkdf")]
#[derive(Clone)]
pub struct HkdfInvalidLength;

/// Implement the conversion from hkdf::InvalidLength to HkdfInvalidLength, as the former does not
/// implement the PartialEq nor Eq traits.
///
/// Note that the original type is simply empty, as the error does not provide any additional data,
/// and the conversion is simply a way to wrap the error into a type that can be compared.
#[cfg(feature = "hkdf")]
impl From<hkdf::InvalidLength> for HkdfInvalidLength {
    fn from(_: hkdf::InvalidLength) -> Self { HkdfInvalidLength }
}

/// All `HkdfInvalidLength` instances are considered equal as they do not contain any additional
/// data.
#[cfg(feature = "hkdf")]
impl PartialEq for HkdfInvalidLength {
    fn eq(&self, _: &Self) -> bool { true }
}

#[cfg(feature = "hkdf")]
impl Eq for HkdfInvalidLength {}

/// Implement the Display trait for the HkdfInvalidLength error, delegating to the original error
/// message.
#[cfg(feature = "hkdf")]
impl Display for HkdfInvalidLength {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result { write!(f, "{}", hkdf::InvalidLength) }
}

#[derive(Clone, PartialEq, Eq)]
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
    CannotHashOrDerive(HkdfInvalidLength),
    /// Invalid character in input
    InvalidCharacterInput,
    /// Cannot decode the data
    CannotDecode,
    /// Cannot encode the data
    CannotEncode,
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
    /// The internal encoding bitmask overflowed
    EncodingBitmaskOverflow(usize),
}

#[cfg(any(feature = "server", test))]
impl Debug for CryptError {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        // Delegate to Display
        write!(f, "{}", self)
    }
}

#[cfg(any(feature = "server", test))]
impl Display for CryptError {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        #[expect(
            clippy::pattern_type_mismatch,
            reason = "Cannot dereference into the Display trait implementation"
        )]
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
            Self::CannotHashOrDerive(e) => {
                write!(f, "Cannot hash the provided data: {}", e)
            },
            Self::InvalidCharacterInput => {
                write!(f, "Invalid character in input")
            },
            Self::CannotDecode => {
                write!(f, "Cannot decode the data")
            },
            Self::CannotHashArgon2(e) => {
                write!(f, "Cannot hash the provided data with Argon2: {}", e)
            },
            Self::CannotDeriveArgon2(e) => {
                write!(f, "Cannot derive the key with Argon2: {}", e)
            },
            Self::MissingOrInvalidPublicKey => {
                write!(f, "The receiver public key is missing or invalid")
            },
            Self::MissingOrInvalidSecretKey => {
                write!(f, "The sender secret key is missing or invalid")
            },
            Self::CannotEncryptWithChaCha20Poly1305(e) => {
                write!(
                    f,
                    "Cannot encrypt the provided data with ChaCha20Poly1305: {}",
                    e
                )
            },
            Self::CannotDecryptWithChaCha20Poly1305(e) => {
                write!(
                    f,
                    "Cannot decrypt the provided data with ChaCha20Poly1305: {}",
                    e
                )
            },
            Self::DataTooLong(overflowing_size) => {
                write!(
                    f,
                    "The provided data is too long, overflowing the maximum size resulting in: {}",
                    overflowing_size
                )
            },
            Self::DataTooShort(underflowing_size) => {
                write!(
                    f,
                    "The provided data is too short, underflowing the minimum size resulting in: {}",
                    underflowing_size
                )
            },
            Self::InvalidEncodingCharacter(encoding, char) => {
                write!(
                    f,
                    "Invalid character in input for encoding '{}': '{}'",
                    encoding, char
                )
            },
            Self::InvalidEncodingLength(encoding, size) => {
                write!(
                    f,
                    "Invalid length in input for encoding '{}': '{}'",
                    encoding, size
                )
            },
            Self::CannotEncode => {
                write!(f, "Cannot encode the data")
            },
            Self::EncodingBitmaskOverflow(bitmask) => {
                write!(f, "The internal encoding bitmask overflowed: {}", bitmask)
            },
        }
    }
}

#[cfg(any(feature = "server", test))]
impl ErrorTrait for CryptError {}
