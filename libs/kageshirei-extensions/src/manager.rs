//! The extension manager is responsible for loading and managing extensions.

use std::sync::Arc;

use libloading::Library;

use crate::{
    dependency_injection::{AgentDependencies, DependencyInjector, GuiDependencies, ServerDependencies},
    KageshireiExtension,
};

/// A loaded extension
struct LoadedExtension {
    /// Path to the shared library
    path:      String,
    /// The extension
    extension: Box<dyn KageshireiExtension>,
}

impl LoadedExtension {
    /// Get the library the extension was **ORIGINALLY** loaded from
    fn path(&self) -> String { self.path.clone() }

    /// Get the extension
    #[expect(
        clippy::borrowed_box,
        reason = "The type is dynamic and cannot be referenced directly"
    )]
    fn extension(&self) -> &Box<dyn KageshireiExtension> { &self.extension }
}

/// The extension manager is responsible for loading and managing extensions.
//#[expect(clippy::module_name_repetitions, reason = "The name is descriptive and is exported from the crate root")]
pub struct ExtensionManager {
    /// Shared libraries loaded by the extension manager
    libraries:           Vec<Library>,
    /// Extensions loaded by the extension manager
    extensions:          Vec<LoadedExtension>,
    /// Dependency injector used to inject dependencies into extensions
    dependency_injector: Arc<DependencyInjector>,
}

impl ExtensionManager {
    /// Create a new extension manager
    pub fn new(dependency_injector: DependencyInjector) -> Self {
        Self {
            libraries:           Vec::new(),
            extensions:          Vec::new(),
            dependency_injector: Arc::new(dependency_injector),
        }
    }

    /// Get the server hook registry
    pub fn get_server_deps(&self) -> Arc<ServerDependencies> { self.dependency_injector.server_deps() }

    /// Get the agent hook registry
    pub fn get_agent_deps(&self) -> Arc<AgentDependencies> { self.dependency_injector.agent_deps() }

    /// Get the GUI hook registry
    pub fn get_gui_deps(&self) -> Arc<GuiDependencies> { self.dependency_injector.gui_deps() }

    /// Get the number of extensions loaded by the extension manager
    pub fn len(&self) -> usize { self.extensions.len() }

    /// Check if the extension manager is empty
    pub fn is_empty(&self) -> bool { self.len() == 0 }

    /// Load an extension from a shared library
    ///
    /// The shared library **must** contain a function named `get_extension` that returns a pointer
    /// to a `KageshireiExtension`.
    ///
    /// # Example
    /// ```rust ignore
    /// struct MyPlugin;
    ///
    /// impl KageshireiExtension for MyPlugin {
    ///    // ...
    /// }
    ///
    /// #[no_mangle]
    /// pub extern "C" fn get_extension() -> *mut dyn KageshireiExtension {
    ///     let plugin = MyPlugin;
    ///     Box::into_raw(Box::new(plugin)) as *mut dyn KageshireiExtension
    /// }
    /// ```
    pub fn load(&mut self, path: &str) -> Result<(), String> {
        // Load the shared library
        let lib = unsafe { Library::new(path).map_err(|e| e.to_string())? };

        // Track the library in its vector
        let index = self.libraries.len();
        self.libraries.push(lib);

        // Get the extension from the shared library and track it
        let extension = unsafe {
            #[expect(clippy::indexing_slicing, reason = "The library was just added")]
            let lib = &self.libraries[index];
            let get_extension = lib
                .get::<extern "C" fn() -> *mut dyn KageshireiExtension>(b"get_extension")
                .map_err(|e| e.to_string())?;
            Box::from_raw(get_extension())
        };

        self.extensions.push(LoadedExtension {
            path: path.to_owned(),
            extension,
        });

        Ok(())
    }

    /// Initialize all extensions
    pub async fn initialize(&self) {
        for ext in &self.extensions {
            ext.extension()
                .initialize(self.dependency_injector.clone())
                .await;
        }
    }

    /// Unload an extension
    pub async fn unload(&mut self, index: usize) {
        // remove the extension from the vector
        let extension = self.extensions.remove(index);
        // then terminate and drop it
        extension.extension().terminate().await;
        drop(extension);

        // Unload the library
        let library = self.libraries.remove(index);
        drop(library);
    }

    /// Terminate all extensions and unload their shared libraries
    pub async fn terminate(&mut self) {
        for i in 0 .. self.extensions.len() {
            self.unload(i).await;
        }
    }

    /// Get an extension by its name
    #[expect(
        clippy::borrowed_box,
        reason = "The type is dynamic and cannot be referenced directly"
    )]
    pub fn get_by_name(&self, name: &str) -> Option<&Box<dyn KageshireiExtension>> {
        self.extensions
            .iter()
            .find(|ext| ext.extension().name() == name)
            .map(|ext| ext.extension())
    }

    /// Get an extension by its path
    #[expect(
        clippy::borrowed_box,
        reason = "The type is dynamic and cannot be referenced directly"
    )]
    pub fn get_by_path(&self, path: &str) -> Option<&Box<dyn KageshireiExtension>> {
        self.extensions
            .iter()
            .find(|ext| ext.path() == path)
            .map(|ext| ext.extension())
    }
}
