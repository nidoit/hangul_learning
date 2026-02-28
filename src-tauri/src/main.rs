// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod converter;
mod error;
mod models;
mod scanner;

use commands::convert::AppState;
use std::sync::Mutex;

fn main() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            last_results: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            commands::folder::select_folder,
            commands::folder::scan_files,
            commands::convert::start_conversion,
            commands::convert::get_summary,
        ])
        .run(tauri::generate_context!())
        .expect("error while running DocConvert");
}
