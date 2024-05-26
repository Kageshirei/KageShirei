use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ProtocolError {
    /// Error when trying to deserialize data.
    DeserializationError,
    /// Error when trying to serialize data.
    SerializationError,
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

impl Display for ProtocolError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ProtocolError::DeserializationError => write!(f, "Error when trying to deserialize data."),
            ProtocolError::SerializationError => write!(f, "Error when trying to serialize data."),
            ProtocolError::SendingError => write!(f, "Error when trying to send data."),
            ProtocolError::ReceivingError => write!(f, "Error when trying to receive data."),
            ProtocolError::ConnectionError => write!(f, "Error when trying to connect to a server."),
            ProtocolError::DisconnectionError => write!(f, "Error when trying to disconnect from a server."),
            ProtocolError::MessageError => write!(f, "Error when trying to send a message to a server."),
            ProtocolError::ReceiveMessageError => write!(f, "Error when trying to receive a message from a server."),
        }
    }
}

impl Error for ProtocolError {}