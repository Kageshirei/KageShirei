//! # Kageshirei Extensions
//! Implements the building blocks for the Kageshirei extension system.

pub mod dependency_injection;
mod extension_def;
pub mod hook_system;
mod manager;

// Re-export async_trait
pub use async_trait::async_trait;
pub use extension_def::KageshireiExtension;
pub use glob;
pub use manager::ExtensionManager;
