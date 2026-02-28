use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileType {
    Pdf,
    Xlsx,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: String,
    pub relative_path: String,
    pub file_type: FileType,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub pdf_count: u32,
    pub xlsx_count: u32,
    pub total_size_bytes: u64,
    pub files: Vec<FileEntry>,
}
