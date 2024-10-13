use std::sync::Arc;

use anyhow::Result;
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
    /// A result indicating success or failure with the response data.
    fn send(&mut self, data: Bytes, metadata: Arc<Metadata>)
        -> impl std::future::Future<Output = Result<Bytes>> + Send;
}
