use alloc::vec::Vec;
use core::future::Future;

use serde::Serialize;

use crate::{error::Format as FormatError, metadata::WithMetadata};

/// Define the protocol trait responsible for sending and receiving data.
pub trait Format: Send {
    /// Receive some data as raw bytes and deserialize it into a type.
    ///
    /// # Arguments
    ///
    /// * `data` - The raw bytes to deserialize.
    ///
    /// # Returns
    ///
    /// A result containing the deserialized data or an error.
    fn read(&self, data: &[u8]) -> impl Future<Output = Result<Vec<u8>, FormatError>> + Send;

    /// Serialize some data into raw bytes and send it.
    ///
    /// # Arguments
    ///
    /// * `data` - The data to serialize.
    ///
    /// # Returns
    ///
    /// A result indicating success or failure.
    fn write<D>(&mut self, data: D) -> impl Future<Output = Result<Vec<u8>, FormatError>> + Send
    where
        D: Serialize + WithMetadata + Send;
}
