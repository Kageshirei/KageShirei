#[cfg(any(feature = "server", test))]
use core::{
    error::Error as ErrorTrait,
    fmt::{Debug, Display, Formatter},
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// TODO: This is a placeholder, remove once at least a format error is defined
    _PLACEHOLDER,
}

#[cfg(any(feature = "server", test))]
impl Debug for Format {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        // Delegate to Display
        write!(f, "{}", self)
    }
}

#[cfg(any(feature = "server", test))]
impl Display for Format {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        #[expect(
            clippy::pattern_type_mismatch,
            reason = "Cannot dereference into the Display trait implementation"
        )]
        match self {
            Self::_PLACEHOLDER => {
                write!(f, "Placeholder error",)
            },
        }
    }
}

#[cfg(any(feature = "server", test))]
impl ErrorTrait for Format {}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    // TODO: Check if the error variants are correct, probably they are not
    /// Error when trying to send data.
    SendingError,
    /// Error when trying to receive data.
    ReceivingError,
    /// Error when trying to connect to a server.
    ConnectionError,
    /// Error when trying to disconnect from a server.
    DisconnectionError,
    /// Error when trying to send a message to a server.
    MessageError,
    /// Error when trying to receive a message from a server.
    ReceiveMessageError,
}

#[cfg(any(feature = "server", test))]
impl Debug for Protocol {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        // Delegate to Display
        write!(f, "{}", self)
    }
}

#[cfg(any(feature = "server", test))]
impl Display for Protocol {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        #[expect(
            clippy::pattern_type_mismatch,
            reason = "Cannot dereference into the Display trait implementation"
        )]
        match self {
            // Self::InvalidKeyLength(bytes, received) => {
            //     write!(
            //         f,
            //         "Invalid key length, expected {} bytes, got {}",
            //         bytes, received
            //     )
            // },
            Protocol::SendingError => {
                write!(f, "Error when trying to send data.")
            },
            Protocol::ReceivingError => {
                write!(f, "Error when trying to receive data.")
            },
            Protocol::ConnectionError => {
                write!(f, "Error when trying to connect to a server.")
            },
            Protocol::DisconnectionError => {
                write!(f, "Error when trying to disconnect from a server.")
            },
            Protocol::MessageError => {
                write!(f, "Error when trying to send a message to a server.")
            },
            Protocol::ReceiveMessageError => {
                write!(f, "Error when trying to receive a message from a server.")
            },
        }
    }
}

#[cfg(any(feature = "server", test))]
impl ErrorTrait for Protocol {}
