//! This module contains the dependency injection structs for the application.

use std::sync::Arc;

mod agent_deps;
mod gui_deps;
mod server_deps;

pub use agent_deps::AgentDependencies;
pub use gui_deps::GuiDependencies;
pub use server_deps::ServerDependencies;

pub struct DependencyInjector {
    /// The dependencies for the agent
    agent_deps:  Arc<Box<AgentDependencies>>,
    /// The dependencies for the GUI
    gui_deps:    Arc<Box<GuiDependencies>>,
    /// The dependencies for the server
    server_deps: Arc<Box<ServerDependencies>>,
}

impl DependencyInjector {
    /// Create a new dependency injector
    pub fn new(agent_deps: AgentDependencies, gui_deps: GuiDependencies, server_deps: ServerDependencies) -> Self {
        Self {
            agent_deps:  Arc::new(Box::new(agent_deps)),
            gui_deps:    Arc::new(Box::new(gui_deps)),
            server_deps: Arc::new(Box::new(server_deps)),
        }
    }

    /// Get the agent dependencies
    pub fn agent_deps(&self) -> Arc<Box<AgentDependencies>> { self.agent_deps.clone() }

    /// Get the GUI dependencies
    pub fn gui_deps(&self) -> Arc<Box<GuiDependencies>> { self.gui_deps.clone() }

    /// Get the server dependencies
    pub fn server_deps(&self) -> Arc<Box<ServerDependencies>> { self.server_deps.clone() }
}
