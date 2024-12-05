//! The extension manager is responsible for loading and managing extensions.

use std::sync::Arc;

use libloading::Library;

use crate::{dependency_injection::DependencyInjector, KageshireiExtension};

pub struct ExtensionManager {
    libraries:           Vec<Library>,
    extensions:          Vec<Box<dyn KageshireiExtension>>,
    dependency_injector: Arc<Box<DependencyInjector>>,
}

impl ExtensionManager {
    /// Create a new extension manager
    pub fn new(dependency_injector: DependencyInjector) -> Self {
        Self {
            libraries:           Vec::new(),
            extensions:          Vec::new(),
            dependency_injector: Arc::new(Box::new(dependency_injector)),
        }
    }

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
            let lib = self.libraries.get(index).unwrap();
            let get_extension = lib
                .get::<extern "C" fn() -> *mut dyn KageshireiExtension>(b"get_extension")
                .map_err(|e| e.to_string())?;
            Box::from_raw(get_extension())
        };
        self.extensions.push(extension);

        Ok(())
    }

    /// Initialize all extensions
    pub fn initialize(&self) {
        for extension in &self.extensions {
            extension.initialize(self.dependency_injector.clone());
        }
    }

    /// Unload an extension
    pub fn unload(&mut self, index: usize) {
        // remove the extension from the vector
        let extension = self.extensions.remove(index);
        // then terminate and drop it
        extension.terminate();
        drop(extension);

        // Unload the library
        let library = self.libraries.remove(index);
        drop(library);
    }
}
