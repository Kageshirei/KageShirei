//! Common traits to define the minimum requirements for an extension (aka collection of plugins)

use std::sync::Arc;

use crate::dependency_injection::DependencyInjector;

pub trait KageshireiExtension {
    /// Get the name of the extension
    fn name(&self) -> &'static str;

    /// Get the version of the extension
    fn version(&self) -> &'static str;

    /// Get the author of the extension
    fn author(&self) -> &'static str;

    /// Get the description of the extension
    fn description(&self) -> &'static str;

    /// Get the compatibility range of the extension
    fn compatibility(&self) -> &'static str;

    /// Initialize the extension
    fn initialize(&self, dependencies: Arc<Box<DependencyInjector>>);

    /// Terminate the extension
    fn terminate(&self);
}
