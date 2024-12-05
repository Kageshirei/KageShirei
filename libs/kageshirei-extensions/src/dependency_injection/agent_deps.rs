//! Agent dependencies.

use crate::hook_system::HookRegistry;

/// Container of dependencies for agent DI (dependency injection).
pub struct AgentDependencies {
    /// A registry of agent related hooks
    pub registry: HookRegistry,
}
