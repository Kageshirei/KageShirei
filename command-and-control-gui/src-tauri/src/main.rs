// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(clippy::multiple_crate_versions, reason = "required by tauri")]
#![allow(clippy::print_stderr, reason = "required for error handling")]

fn main() {
    let result = tauri::Builder::default().run(tauri::generate_context!());

    if let Err(e) = result {
        eprintln!("An error occurred while running tauri application: {} ", e);
        std::process::exit(1);
    }
}
