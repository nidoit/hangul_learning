use serde::{Deserialize, Serialize};

use crate::scanner::FileEntry;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionJob {
    pub source_dir: String,
    pub output_dir: String,
    #[serde(default)]
    pub overwrite: bool,
    pub files: Vec<FileEntry>,
}
