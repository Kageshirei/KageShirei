//! Define the format trait responsible for serializing and deserializing data.

use alloc::{collections::BTreeMap, vec::Vec};
use core::any::Any;

use serde::{Deserialize, Serialize};

use crate::error::Format as FormatError;

/// Define the format trait responsible for serializing and deserializing data.
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
    fn read<T, V>(&self, data: &[u8], extra: Option<BTreeMap<&str, V>>) -> Result<T, FormatError>
    where
        T: for<'a> Deserialize<'a>,
        V: Sized + Any;

    /// Serialize some data into raw bytes and send it.
    ///
    /// # Arguments
    ///
    /// * `data` - The data to serialize.
    ///
    /// # Returns
    ///
    /// A result indicating success or failure.
    fn write<D, V>(&mut self, data: D, extra: Option<BTreeMap<&str, V>>) -> Result<Vec<u8>, FormatError>
    where
        D: Serialize,
        V: Sized + Any;
}
