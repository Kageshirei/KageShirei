//! Define the protocol trait responsible for sending data.

use alloc::{sync::Arc, vec::Vec};
use core::future::Future;

use crate::{error::Protocol as ProtocolError, metadata::Metadata};

/// Define the sender trait responsible for sending data.
pub trait Protocol: Send {
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
    fn send(
        &mut self,
        data: Vec<u8>,
        metadata: Option<Arc<Metadata>>,
    ) -> impl Future<Output = Result<Vec<u8>, ProtocolError>> + Send;

    /// Receive some data as raw bytes.
    ///
    /// # Arguments
    ///
    /// * `metadata` - The metadata needed to receive the data.
    ///
    /// # Returns
    ///
    /// A future that resolves to the received data.
    fn receive(
        &mut self,
        metadata: Option<Arc<Metadata>>,
    ) -> impl Future<Output = Result<Vec<u8>, ProtocolError>> + Send;
}
