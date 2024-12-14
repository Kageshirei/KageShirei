//! Contains the context for the different hooks.

use std::sync::Arc;

use crate::cli::base::CliArguments;

/// Context for the `OnServerStart` hook.
pub struct OnServerStart {
    /// Mutable reference to the CLI arguments, this allows for modification of the arguments coming
    /// from the command line.
    arguments: Arc<CliArguments>,
}

impl OnServerStart {
    /// Create a new `OnServerStart` context.
    pub fn new(arguments: Arc<CliArguments>) -> Arc<Self> {
        Arc::new(Self {
            arguments,
        })
    }
}
