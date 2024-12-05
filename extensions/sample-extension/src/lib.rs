use std::{pin::Pin, sync::Arc};

use kageshirei_extensions::{
    async_trait,
    dependency_injection::DependencyInjector,
    hook_system::HookMetadata,
    KageshireiExtension,
};
use kageshirei_extensions_macros::registerable_hook;

struct SampleExtension;

#[registerable_hook]
async fn on_server_start(_context: Arc<Vec<String>>) -> Result<(), String> {
    println!("[::sample_extension::on_server_start] Triggered");
    Ok(())
}

#[registerable_hook]
async fn on_agent_start(_context: Arc<Vec<String>>) -> Result<(), String> {
    println!("[::sample_extension::on_agent_start] Triggered");
    Ok(())
}

#[registerable_hook]
async fn on_gui_start(_context: Arc<Vec<String>>) -> Result<(), String> {
    println!("[::sample_extension::on_gui_start] Triggered");
    Ok(())
}

#[async_trait]
impl KageshireiExtension for SampleExtension {
    fn name(&self) -> &'static str { "Sample Extension" }

    fn version(&self) -> &'static str { "0.1.0" }

    fn author(&self) -> &'static str { "Ebalo" }

    fn description(&self) -> &'static str { "A sample extension for Kageshirei" }

    fn compatibility(&self) -> &'static str { "unstable" }

    async fn initialize(&self, dependencies: Arc<DependencyInjector>) {
        println!("[::sample_extension::initialize] Initializing sample extension");

        println!("[::sample_extension::initialize] Registering on_server_start hook");
        dependencies
            .server_deps()
            .registry
            .register(
                "on_server_start",
                HookMetadata {
                    priority:    u8::MAX,
                    description: "A sample hook that triggers when the server starts".to_owned(),
                },
                on_server_start::register(),
            )
            .await;

        println!("[::sample_extension::initialize] Registering on_agent_start hook");
        dependencies
            .agent_deps()
            .registry
            .register(
                "on_agent_start",
                HookMetadata {
                    priority:    u8::MAX,
                    description: "A sample hook that triggers when the server starts".to_owned(),
                },
                on_agent_start::register(),
            )
            .await;

        println!("[::sample_extension::initialize] Registering on_gui_start hook");
        dependencies
            .gui_deps()
            .registry
            .register(
                "on_gui_start",
                HookMetadata {
                    priority:    u8::MAX,
                    description: "A sample hook that triggers when the server starts".to_owned(),
                },
                on_gui_start::register(),
            )
            .await;

        println!("[::sample_extension::initialize] Sample extension initialized");
    }

    async fn terminate(&self) {
        println!("[::sample_extension::terminate] Terminating sample extension");
    }
}

#[no_mangle]
pub extern "C" fn get_extension() -> *mut dyn KageshireiExtension {
    let extension = SampleExtension;
    Box::into_raw(Box::new(extension)) as *mut dyn KageshireiExtension
}
