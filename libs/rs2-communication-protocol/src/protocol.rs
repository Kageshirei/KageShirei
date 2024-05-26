use std::error::Error;

use bytes::Bytes;
use serde::Serialize;

use crate::encryptor::Encryptor;
use crate::metadata::WithMetadata;
use crate::sender::Sender;

/// Define the protocol trait responsible for sending and receiving data.
pub trait Protocol: Sender + Send {
    /// Receive some data as raw bytes and deserialize it into a type.
    ///
    /// # Arguments
    ///
    /// * `data` - The raw bytes to deserialize.
    /// * `encryptor` - The encryptor to use to decrypt the data.
    ///
    /// # Returns
    ///
    /// A result containing the deserialized data or an error.
    fn read<S, E>(&self, data: Bytes, encryptor: Option<E>) -> Result<S, Box<dyn Error>>
        where
            S: serde::de::DeserializeOwned,
            E: Encryptor;

    /// Serialize some data into raw bytes and send it.
    ///
    /// # Arguments
    ///
    /// * `data` - The data to serialize.
    /// * `sender` - The sender to use to send the data.
    /// * `encryptor` - The encryptor to use to encrypt the data.
    ///
    /// # Returns
    ///
    /// A result indicating success or failure.
    fn write<D, E>(&mut self, data: D, encryptor: Option<E>) -> impl std::future::Future<Output=Result<Bytes, Box<dyn Error>>> + Send
        where
            D: Serialize + WithMetadata + Send,
            E: Encryptor + Send;
}