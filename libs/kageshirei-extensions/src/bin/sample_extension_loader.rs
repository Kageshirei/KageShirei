//! Sample code to load extensions from a folder named `./extensions`

#![allow(clippy::print_stdout, reason = "This is a sample code")]
#![allow(clippy::print_stderr, reason = "This is a sample code")]
#![allow(clippy::expect_used, reason = "This is a sample code")]

use std::sync::Arc;

use glob::glob;
use kageshirei_extensions::{
    dependency_injection::{AgentDependencies, DependencyInjector, GuiDependencies, ServerDependencies},
    hook_system::HookRegistry,
    ExtensionManager,
};

#[tokio::main]
async fn main() {
    println!("[::main] Example of loading extensions from a folder named `./extensions`");

    let mut manager = ExtensionManager::new(DependencyInjector::default());

    let suffix = if cfg!(windows) { ".dll" } else { ".so" };
    for path in glob(format!("./extensions/*{}", suffix).as_str())
        .expect("Failed to read glob pattern")
        .flatten()
    {
        println!("[::main] Loading extension from: {}", path.display());
        manager
            .load(path.to_str().unwrap())
            .expect("Failed to load extension");
        println!(
            "[::main] Extension loaded, extra info below:\n{}",
            manager
                .get_by_path(path.to_str().unwrap())
                .unwrap()
                .describe()
        );
    }

    if manager.is_empty() {
        eprintln!("[::main] No extensions loaded");
    }
    else {
        println!("[::main] Loaded {} extensions", manager.len());
    }

    manager.initialize().await;

    println!("[::main] Extensions initialized");
    println!("[::main] Triggering sample hooks");

    let context = Arc::new(Vec::<String>::new());

    let result = manager
        .get_server_deps()
        .registry
        .trigger("on_server_start", context.clone())
        .await;
    println!("[::main] on_server_start hook result: {:?}", result);

    let result = manager
        .get_agent_deps()
        .registry
        .trigger("on_agent_start", context.clone())
        .await;
    println!("[::main] on_agent_start hook result: {:?}", result);

    let result = manager
        .get_gui_deps()
        .registry
        .trigger("on_gui_start", context.clone())
        .await;
    println!("[::main] on_gui_start hook result: {:?}", result);

    manager.terminate().await;
    println!("[::main] Done");
}
