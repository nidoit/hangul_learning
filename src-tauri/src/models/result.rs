use serde::{Deserialize, Serialize};

use crate::scanner::FileEntry;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConversionStatus {
    Success,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionResult {
    pub file: FileEntry,
    pub status: ConversionStatus,
    pub output_path: Option<String>,
    pub error: Option<String>,
    pub duration_ms: u64,
}
