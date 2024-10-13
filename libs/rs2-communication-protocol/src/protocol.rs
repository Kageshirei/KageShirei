use anyhow::Result;
use bytes::Bytes;
use rs2_crypt::encryption_algorithm::EncryptionAlgorithm;
use serde::Serialize;

use crate::{metadata::WithMetadata, sender::Sender};

/// Define the protocol trait responsible for sending and receiving data.
pub trait Protocol<E>: Sender + Send
where
    E: EncryptionAlgorithm,
{
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
    fn read<S>(&self, data: Bytes, encryptor: Option<E>) -> Result<S, String>
    where
        S: serde::de::DeserializeOwned;

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
    fn write<D>(
        &mut self,
        data: D,
        encryptor: Option<E>,
    ) -> impl std::future::Future<Output = Result<Bytes, String>> + Send
    where
        D: Serialize + WithMetadata + Send;
}
