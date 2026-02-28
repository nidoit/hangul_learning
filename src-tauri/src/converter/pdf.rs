use std::fs;
use std::path::Path;

use crate::converter::table_detect::detect_and_convert_tables;
use crate::error::AppError;

/// Convert a PDF file to Markdown.
///
/// Uses pdf-extract to pull text from each page, then applies table detection
/// to convert tabular structures into Markdown table syntax.
pub fn convert_pdf_to_markdown(pdf_path: &str, output_path: &str) -> Result<(), AppError> {
    let pdf_bytes =
        fs::read(pdf_path).map_err(|e| AppError::PdfError(format!("Cannot read {}: {}", pdf_path, e)))?;

    let text = pdf_extract::extract_text_from_mem(&pdf_bytes)
        .map_err(|e| AppError::PdfError(format!("Failed to extract text from {}: {}", pdf_path, e)))?;

    if text.trim().is_empty() {
        return Err(AppError::PdfError(format!(
            "No extractable text in {}",
            pdf_path
        )));
    }

    // Process the text: detect tables and format as Markdown
    let markdown = process_text_to_markdown(&text);

    // Ensure output directory exists
    if let Some(parent) = Path::new(output_path).parent() {
        fs::create_dir_all(parent)
            .map_err(|e| AppError::Io(e))?;
    }

    fs::write(output_path, markdown.as_bytes())
        .map_err(|e| AppError::PdfError(format!("Cannot write {}: {}", output_path, e)))?;

    Ok(())
}

/// Process extracted PDF text into Markdown format.
/// - Normalize line endings
/// - Detect and convert tables
/// - Clean up excessive whitespace
fn process_text_to_markdown(text: &str) -> String {
    // Normalize line endings
    let text = text.replace("\r\n", "\n").replace('\r', "\n");

    // Split into pages (pdf-extract uses form-feed \x0C between pages)
    let pages: Vec<&str> = text.split('\x0C').collect();

    let mut markdown = String::new();

    for (i, page) in pages.iter().enumerate() {
        let page = page.trim();
        if page.is_empty() {
            continue;
        }

        // Apply table detection
        let processed = detect_and_convert_tables(page);

        // Clean up: collapse 3+ consecutive newlines into 2
        let cleaned = collapse_newlines(&processed);

        markdown.push_str(&cleaned);

        // Add page break between pages
        if i < pages.len() - 1 {
            markdown.push_str("\n\n---\n\n");
        }
    }

    markdown.trim().to_string() + "\n"
}

/// Collapse runs of 3+ newlines into exactly 2 (one blank line).
fn collapse_newlines(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut newline_count = 0;

    for ch in text.chars() {
        if ch == '\n' {
            newline_count += 1;
            if newline_count <= 2 {
                result.push(ch);
            }
        } else {
            newline_count = 0;
            result.push(ch);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collapse_newlines() {
        let input = "Hello\n\n\n\n\nWorld";
        let result = collapse_newlines(input);
        assert_eq!(result, "Hello\n\nWorld");
    }

    #[test]
    fn test_process_text_basic() {
        let input = "Hello World\nThis is a test.";
        let result = process_text_to_markdown(input);
        assert!(result.contains("Hello World"));
        assert!(result.contains("This is a test."));
    }
}
