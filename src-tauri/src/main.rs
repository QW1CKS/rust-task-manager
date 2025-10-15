// Prevents additional console window on Windows in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Module declarations
mod commands;
mod error;
mod models;
mod services;
mod utils;

// Import Tauri commands for registration
use commands::{get_performance_data, get_system_info};

fn main() {
  tauri::Builder::default()
    .plugin(tauri_plugin_shell::init())
    .invoke_handler(tauri::generate_handler![
      get_system_info,
      get_performance_data,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
