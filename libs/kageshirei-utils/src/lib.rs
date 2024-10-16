#![no_std]
//! A collection of utilities for Kageshirei.
//!
//! This crate provides a collection of utilities that can be used in Kageshirei projects.
//!
//! All utilities are optional and can be enabled or disabled using feature flags.

extern crate alloc;

#[cfg(feature = "duration-extension")]
pub mod duration_extension;
#[cfg(feature = "print-validation-error")]
pub mod print_validation_error;
#[cfg(feature = "unrecoverable-error")]
pub mod unrecoverable_error;
#[cfg(feature = "unwrap-infallible")]
pub mod unwrap_infallible;
