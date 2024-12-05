//! Agent dependencies.

use std::sync::Arc;

use crate::hook_system::HookRegistry;

/// Container of dependencies for agent DI (dependency injection).
///
/// All dependencies except for the hook registry **must** be `Option`s to allow for optional
/// dependencies and lazy initialization.
///
/// All dependencies must also be `Arc`s to allow for shared ownership.
#[derive(Debug, Default)]
pub struct AgentDependencies {
    /// A registry of agent related hooks
    pub registry: Arc<HookRegistry>,
}
