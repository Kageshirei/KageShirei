use alloc::{boxed::Box, string::String};
use core::{
    error::Error as ErrorTrait,
    fmt::{Debug, Display, Formatter},
};

#[derive(Clone, PartialEq, Eq)]
pub enum Format {
    /// No data have been provided.
    EmptyData,
    /// The data provided is invalid, it does not match the expected format.
    InvalidData,
    /// A generic error occurred.
    Generic(Box<dyn ErrorTrait>),
}

impl Debug for Format {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        // Delegate to Display
        write!(f, "{}", self)
    }
}

impl Display for Format {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        #[expect(
            clippy::pattern_type_mismatch,
            reason = "Cannot dereference into the Display trait implementation"
        )]
        match self {
            Self::EmptyData => {
                write!(f, "No data have been provided.")
            },
            Self::InvalidData => {
                write!(
                    f,
                    "The data provided is invalid, it does not match the expected format."
                )
            },
            Self::Generic(e) => {
                write!(f, "A generic error occurred: {}", e)
            },
        }
    }
}

impl ErrorTrait for Format {}

#[derive(Clone, PartialEq, Eq)]
pub enum Protocol {
    // TODO: Check if the error variants are correct, probably they are not
    /// Error when trying to send data. Takes a parameter to indicate the reason.
    SendingError(Option<String>),
    /// Error when trying to receive data. Takes a parameter to indicate the reason.
    ReceivingError(Option<String>),
    /// Error when trying to initialize the protocol.
    InitializationError(String),
    /// A generic error occurred.
    Generic(String),
    /// Error when trying to connect to a server.
    ConnectionError,
    /// Error when trying to disconnect from a server.
    DisconnectionError,
    /// Error when trying to send a message to a server.
    MessageError,
    /// Error when trying to receive a message from a server.
    ReceiveMessageError,
}

impl Debug for Protocol {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        // Delegate to Display
        write!(f, "{}", self)
    }
}

impl Display for Protocol {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        #[expect(
            clippy::pattern_type_mismatch,
            reason = "Cannot dereference into the Display trait implementation"
        )]
        match self {
            Self::SendingError(reason) => {
                if let Some(reason) = reason {
                    write!(f, "Error when trying to send data: {}", reason)
                }
                else {
                    write!(f, "Error when trying to send data.")
                }
            },
            Self::ReceivingError(reason) => {
                if let Some(reason) = reason {
                    write!(f, "Error when trying to receive data: {}", reason)
                }
                else {
                    write!(f, "Error when trying to receive data.")
                }
            },
            Self::ConnectionError => {
                write!(f, "Error when trying to connect to a server.")
            },
            Self::DisconnectionError => {
                write!(f, "Error when trying to disconnect from a server.")
            },
            Self::MessageError => {
                write!(f, "Error when trying to send a message to a server.")
            },
            Self::ReceiveMessageError => {
                write!(f, "Error when trying to receive a message from a server.")
            },
            Self::InitializationError(reason_or_errored_fragment) => {
                write!(
                    f,
                    "Error when trying to initialize the protocol: {}",
                    reason_or_errored_fragment
                )
            },
            Self::Generic(e) => {
                write!(f, "A generic error occurred: {}", e)
            },
        }
    }
}

impl ErrorTrait for Protocol {}
