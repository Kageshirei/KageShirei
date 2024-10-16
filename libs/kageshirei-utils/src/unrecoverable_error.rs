use alloc::{borrow::ToOwned as _, string::String};

/// Utility error for server components to use when an unrecoverable error is detected and shutdown
/// is required.
pub fn unrecoverable_error() -> Result<(), String> { Err("Unrecoverable error(s) detected, exiting.".to_owned()) }
