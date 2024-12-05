//! GUI dependencies

use crate::hook_system::HookRegistry;

/// Container of dependencies for GUI DI (dependency injection).
pub struct GuiDependencies {
    /// A registry of GUI related hooks
    pub registry: HookRegistry,
}
