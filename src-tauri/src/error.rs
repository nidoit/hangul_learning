use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Directory not found: {0}")]
    DirNotFound(String),

    #[error("Directory not readable: {0}")]
    DirNotReadable(String),

    #[error("Output directory not writable: {0}")]
    OutputNotWritable(String),

    #[error("Scan failed: {0}")]
    ScanFailed(String),

    #[error("PDF conversion error: {0}")]
    PdfError(String),

    #[error("XLSX conversion error: {0}")]
    XlsxError(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Dialog failed: {0}")]
    DialogFailed(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
