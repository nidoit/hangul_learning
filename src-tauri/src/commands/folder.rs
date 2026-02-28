use tauri_plugin_dialog::DialogExt;

use crate::error::AppError;
use crate::scanner::{self, ScanResult};

#[tauri::command]
pub async fn select_folder(app: tauri::AppHandle) -> Result<Option<String>, AppError> {
    let result = app
        .dialog()
        .file()
        .blocking_pick_folder();

    Ok(result.map(|p| p.to_string()))
}

#[tauri::command]
pub async fn scan_files(source_dir: String) -> Result<ScanResult, AppError> {
    let result = scanner::scan_directory(&source_dir)?;
    Ok(result)
}
