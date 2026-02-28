pub mod types;

pub use types::{FileEntry, FileType, ScanResult};

use std::path::Path;
use walkdir::WalkDir;

use crate::error::AppError;

pub fn scan_directory(source_dir: &str) -> Result<ScanResult, AppError> {
    let source_path = Path::new(source_dir);

    if !source_path.exists() {
        return Err(AppError::DirNotFound(source_dir.to_string()));
    }
    if !source_path.is_dir() {
        return Err(AppError::DirNotFound(format!(
            "{} is not a directory",
            source_dir
        )));
    }
    if std::fs::read_dir(source_path).is_err() {
        return Err(AppError::DirNotReadable(source_dir.to_string()));
    }

    let mut files = Vec::new();
    let mut pdf_count: u32 = 0;
    let mut xlsx_count: u32 = 0;
    let mut total_size_bytes: u64 = 0;

    for entry in WalkDir::new(source_path)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            // Skip hidden directories
            e.file_name()
                .to_str()
                .map(|s| !s.starts_with('.'))
                .unwrap_or(true)
        })
    {
        let entry = match entry {
            Ok(e) => e,
            Err(err) => {
                log::warn!("Skipping inaccessible entry: {}", err);
                continue;
            }
        };

        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase());

        let file_type = match extension.as_deref() {
            Some("pdf") => FileType::Pdf,
            Some("xlsx") => FileType::Xlsx,
            _ => continue,
        };

        let size_bytes = entry.metadata().map(|m| m.len()).unwrap_or(0);

        let relative_path = path
            .strip_prefix(source_path)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();

        match file_type {
            FileType::Pdf => pdf_count += 1,
            FileType::Xlsx => xlsx_count += 1,
        }
        total_size_bytes += size_bytes;

        files.push(FileEntry {
            path: path.to_string_lossy().to_string(),
            relative_path,
            file_type,
            size_bytes,
        });
    }

    Ok(ScanResult {
        pdf_count,
        xlsx_count,
        total_size_bytes,
        files,
    })
}
