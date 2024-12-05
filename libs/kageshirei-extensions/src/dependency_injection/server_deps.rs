//! Server dependencies

use crate::hook_system::HookRegistry;

/// Container of dependencies for server DI (dependency injection).
pub struct ServerDependencies {
    /// A registry of server related hooks
    pub registry: HookRegistry,
}
