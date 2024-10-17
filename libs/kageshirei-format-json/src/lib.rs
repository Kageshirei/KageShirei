#![feature(let_chains)]
#![no_std]

//! # Kageshirei JSON Format
//!
//! This crate provides a JSON format implementation for the Kageshirei communication protocol.

extern crate alloc;

use alloc::{boxed::Box, collections::BTreeMap, vec::Vec};
use core::any::Any;

use kageshirei_communication_protocol::{error::Format as FormatError, magic_numbers, Format};
use serde::{Deserialize, Serialize};

pub struct FormatJson;

impl Format for FormatJson {
    fn read<T, V>(&self, data: &[u8], _extra: Option<BTreeMap<&str, V>>) -> Result<T, FormatError>
    where
        T: for<'a> Deserialize<'a>,
        V: Sized + Any,
    {
        if data.is_empty() {
            return Err(FormatError::EmptyData);
        }

        let magic_number_len = magic_numbers::JSON.len();
        let magic_bytes = data.get(.. magic_number_len).ok_or(FormatError::InvalidData)?;
        if data.len() < magic_number_len || magic_bytes != magic_numbers::JSON {
            return Err(FormatError::InvalidData);
        }

        let data_chunk = data.get(magic_number_len ..);
        if data_chunk.is_none() {
            return Err(FormatError::InvalidData);
        }
        let data_chunk = data_chunk.unwrap();

        serde_json::from_slice::<T>(data_chunk).map_err(|e| FormatError::Generic(Box::new(e)))
    }

    fn write<D, V>(&mut self, data: D, _extra: Option<BTreeMap<&str, V>>) -> Result<Vec<u8>, FormatError>
    where
        D: Serialize,
        V: Sized + Any,
    {
        let mut buffer = Vec::new();
        buffer.extend_from_slice(&magic_numbers::JSON);

        let data = serde_json::to_vec(&data).map_err(|e| FormatError::Generic(Box::new(e)))?;
        buffer.extend_from_slice(&data);

        Ok(buffer)
    }
}

#[cfg(test)]
mod test {
    use alloc::borrow::ToOwned;
    use alloc::string::String;
    use super::*;

    #[test]
    fn test_read() {
        let mut data = Vec::new();
        data.extend_from_slice(&magic_numbers::JSON);
        data.extend_from_slice(b"{\"test\":42}");

        let result: Result<BTreeMap<String, u8>, FormatError> = FormatJson.read(data.as_slice(), None::<BTreeMap<&str, &str>>);
        assert!(result.is_ok());

        let result = unsafe { result.unwrap_unchecked() };
        assert_eq!(result.get("test"), Some(&42));
    }

    #[test]
    fn test_write() {
        let mut data = BTreeMap::new();
        data.insert("test".to_owned(), 42);

        let result: Result<Vec<u8>, FormatError> = FormatJson.write(data, None::<BTreeMap<&str, &str>>);
        assert!(result.is_ok());

        let result = unsafe { result.unwrap_unchecked() };

        let mut expected_result = Vec::new();
        expected_result.extend_from_slice(&magic_numbers::JSON);
        expected_result.extend_from_slice(b"{\"test\":42}");
        assert_eq!(result, expected_result.as_slice());
    }
}