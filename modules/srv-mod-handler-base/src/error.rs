//! Contains the error type for the command handling module

use std::{
    error::Error,
    fmt::{Debug, Display, Formatter},
};

use kageshirei_communication_protocol::error::Format;
use kageshirei_crypt::CryptError;
use srv_mod_entity::sea_orm::DbErr;

/// Represent the different types of errors that can occur during command handling
#[derive(PartialEq, Eq)]
pub enum CommandHandling {
    /// Represent a formatting error that occurred while trying to parse a command
    Format(Format),
    /// Represent a command that was not found
    NotFound,
    /// Represent an agent that was not found
    AgentNotFound,
    /// Represent a database error
    Database(String, DbErr),
    /// Represent a cryptographic error
    Crypt(CryptError),
    /// A generic error
    Generic(String),
}

impl Debug for CommandHandling {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        // Delegate to Display
        write!(f, "{}", self)
    }
}

impl Display for CommandHandling {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        #[expect(
            clippy::pattern_type_mismatch,
            reason = "Cannot dereference into the Display trait implementation"
        )]
        match self {
            Self::Format(format_err) => {
                write!(
                    f,
                    "A formatting error was detected during command handling: {}",
                    format_err
                )
            },
            Self::NotFound => {
                write!(f, "Command not found")
            },
            Self::AgentNotFound => {
                write!(
                    f,
                    "A command was found but the linked agent does not exists"
                )
            },
            Self::Database(context_info, db_err) => {
                write!(
                    f,
                    "A database error occurred (context = {}): {}",
                    context_info, db_err
                )
            },
            Self::Crypt(crypt_err) => {
                write!(f, "A cryptographic error occurred: {}", crypt_err)
            },
            Self::Generic(message) => {
                write!(f, "{}", message)
            },
        }
    }
}

impl Error for CommandHandling {}
