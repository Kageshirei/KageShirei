//! # C2 Command and Control GUI
//!
//! This crate provides the graphical user interface (GUI) for the Command and Control (C2) system.
//! It enables operators to interact with the C2 framework, manage agents, execute tasks, and more.
// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
