#![cfg_attr(not(feature = "server"), no_std)]

//! Kageshirei communication protocol
//!
//! This crate contains the communication protocol and related structs and traits used by the
//! Kageshirei project.
//!
//! Notice that this crate is designed to be used by both the server and the
//! agent, so it is a no_std crate.

extern crate alloc;

pub mod communication;
pub mod error;
mod format;
pub mod magic_numbers;
mod metadata;
mod network_interface;
mod protocol;

pub use format::Format;
pub use metadata::{Metadata, WithMetadata};
pub use network_interface::{NetworkInterface, NetworkInterfaceArray};
pub use protocol::Protocol;
