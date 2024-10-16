#![cfg_attr(not(feature = "server"), no_std)]
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
pub use network_interface::NetworkInterface;
pub use protocol::Protocol;
