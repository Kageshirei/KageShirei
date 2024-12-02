#![no_std]
/// The `kageshirei-runtime` library provides an abstract interface for runtime environments.
/// It defines a `Runtime` trait that allows for different runtime implementations,
/// such as a Tokio-based runtime or a custom thread pool runtime.
pub mod runtime;

pub use runtime::Runtime;
