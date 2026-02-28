use std::path::Path;
use std::sync::Mutex;
use std::time::Instant;

use tauri::{AppHandle, Emitter, Manager};

use crate::converter::pdf::convert_pdf_to_markdown;
use crate::converter::xlsx::convert_xlsx_to_csv;
use crate::error::AppError;
use crate::models::result::ConversionStatus;
use crate::models::{
    CompletionEvent, ConversionJob, ConversionResult, FileErrorEvent, ProgressEvent,
};
use crate::scanner::FileType;

pub struct AppState {
    pub last_results: Mutex<Option<Vec<ConversionResult>>>,
}

#[tauri::command]
pub async fn start_conversion(
    app: AppHandle,
    job: ConversionJob,
) -> Result<Vec<ConversionResult>, AppError> {
    // Validate output directory
    let output_dir = Path::new(&job.output_dir);
    if output_dir.exists() && !output_dir.is_dir() {
        return Err(AppError::OutputNotWritable(format!(
            "{} is not a directory",
            job.output_dir
        )));
    }

    // Create output directory if needed
    std::fs::create_dir_all(output_dir)?;

    let total = job.files.len() as u32;
    let job_start = Instant::now();

    let app_handle = app.clone();
    let results = std::thread::spawn(move || {
        let mut results = Vec::new();

        for (i, file) in job.files.iter().enumerate() {
            let file_start = Instant::now();

            // Emit progress event
            let _ = app_handle.emit(
                "conversion-progress",
                ProgressEvent {
                    current: (i + 1) as u32,
                    total,
                    current_file: file.relative_path.clone(),
                    status: "processing".to_string(),
                },
            );

            // Compute output path preserving relative directory structure
            let relative = Path::new(&file.relative_path);
            let output_subdir =
                Path::new(&job.output_dir).join(relative.parent().unwrap_or(Path::new("")));

            let result = match file.file_type {
                FileType::Pdf => {
                    let stem = Path::new(&file.relative_path)
                        .file_stem()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    let output_path = output_subdir.join(format!("{}.md", stem));
                    let output_str = output_path.to_string_lossy().to_string();

                    if !job.overwrite && output_path.exists() {
                        Ok(output_str)
                    } else {
                        convert_pdf_to_markdown(&file.path, &output_str).map(|_| output_str)
                    }
                }
                FileType::Xlsx => {
                    let stem = Path::new(&file.relative_path)
                        .file_stem()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    let out_dir = output_subdir.to_string_lossy().to_string();

                    convert_xlsx_to_csv(&file.path, &out_dir, &stem).map(|paths| paths.join(", "))
                }
            };

            let duration_ms = file_start.elapsed().as_millis() as u64;

            match result {
                Ok(output_path) => {
                    results.push(ConversionResult {
                        file: file.clone(),
                        status: ConversionStatus::Success,
                        output_path: Some(output_path),
                        error: None,
                        duration_ms,
                    });
                }
                Err(e) => {
                    let error_msg = e.to_string();

                    let _ = app_handle.emit(
                        "conversion-error",
                        FileErrorEvent {
                            file: file.relative_path.clone(),
                            error: error_msg.clone(),
                        },
                    );

                    results.push(ConversionResult {
                        file: file.clone(),
                        status: ConversionStatus::Failed,
                        output_path: None,
                        error: Some(error_msg),
                        duration_ms,
                    });
                }
            }
        }

        results
    })
    .join()
    .map_err(|_| AppError::ScanFailed("Conversion thread panicked".to_string()))?;

    let total_duration = job_start.elapsed().as_millis() as u64;
    let succeeded = results
        .iter()
        .filter(|r| matches!(r.status, ConversionStatus::Success))
        .count() as u32;
    let failed = results
        .iter()
        .filter(|r| matches!(r.status, ConversionStatus::Failed))
        .count() as u32;

    // Emit completion event
    let _ = app.emit(
        "conversion-complete",
        CompletionEvent {
            total,
            succeeded,
            failed,
            duration_ms: total_duration,
        },
    );

    // Store results in app state
    let state = app.state::<AppState>();
    *state.last_results.lock().unwrap() = Some(results.clone());

    Ok(results)
}

#[tauri::command]
pub async fn get_summary(
    state: tauri::State<'_, AppState>,
) -> Result<Option<Vec<ConversionResult>>, AppError> {
    Ok(state.last_results.lock().unwrap().clone())
}
