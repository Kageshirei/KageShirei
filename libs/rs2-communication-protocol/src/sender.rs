use std::error::Error;

use bytes::Bytes;

use crate::metadata::Metadata;

pub mod terminal_sender;

/// Define the sender trait responsible for sending data.
pub trait Sender {
    /// Set whether the request is a checkin.
    fn set_is_checkin(&mut self, is_checkin: bool) -> &Self;

    /// Send some data as raw bytes.
    ///
    /// # Arguments
    ///
    /// * `data` - The raw bytes to send.
    /// * `metadata` - The metadata to send with the data.
    ///
    /// # Returns
    ///
    /// A result indicating success or failure.
    fn send(&mut self, data: Bytes, metadata: Metadata) -> impl std::future::Future<Output=Result<Bytes, Box<dyn Error>>> + Send;
}