use std::fs;
use std::path::Path;

use calamine::{open_workbook_auto, Data, Reader};

use crate::error::AppError;

/// Convert an XLSX file to one or more CSV files.
///
/// - Single-sheet workbooks produce `<basename>.csv`
/// - Multi-sheet workbooks produce `<basename>_<sheetname>.csv`
///
/// Returns the list of output file paths created.
pub fn convert_xlsx_to_csv(
    xlsx_path: &str,
    output_dir: &str,
    base_name: &str,
) -> Result<Vec<String>, AppError> {
    let mut workbook = open_workbook_auto(xlsx_path)
        .map_err(|e| AppError::XlsxError(format!("Cannot open {}: {}", xlsx_path, e)))?;

    let sheet_names: Vec<String> = workbook.sheet_names().to_vec();

    if sheet_names.is_empty() {
        return Err(AppError::XlsxError(format!(
            "No sheets found in {}",
            xlsx_path
        )));
    }

    // Ensure output directory exists
    fs::create_dir_all(output_dir).map_err(AppError::Io)?;

    let single_sheet = sheet_names.len() == 1;
    let mut output_paths = Vec::new();

    for sheet_name in &sheet_names {
        let range = workbook.worksheet_range(sheet_name).map_err(|e| {
            AppError::XlsxError(format!(
                "Cannot read sheet '{}' in {}: {}",
                sheet_name, xlsx_path, e
            ))
        })?;

        // Determine output filename
        let csv_filename = if single_sheet {
            format!("{}.csv", base_name)
        } else {
            let safe_name: String = sheet_name
                .chars()
                .map(|c| {
                    if c.is_alphanumeric() || c == '-' || c == '_' {
                        c
                    } else {
                        '_'
                    }
                })
                .collect();
            format!("{}_{}.csv", base_name, safe_name)
        };

        let output_path = Path::new(output_dir).join(&csv_filename);
        let output_path_str = output_path.to_string_lossy().to_string();

        let mut wtr = csv::Writer::from_path(&output_path)
            .map_err(|e| AppError::XlsxError(format!("Cannot create {}: {}", output_path_str, e)))?;

        for row in range.rows() {
            let fields: Vec<String> = row.iter().map(format_cell).collect();
            wtr.write_record(&fields)
                .map_err(|e| AppError::XlsxError(format!("Write error: {}", e)))?;
        }

        wtr.flush()
            .map_err(|e| AppError::XlsxError(format!("Flush error: {}", e)))?;

        output_paths.push(output_path_str);
    }

    Ok(output_paths)
}

/// Format a calamine Data cell value to a string.
fn format_cell(cell: &Data) -> String {
    match cell {
        Data::Empty => String::new(),
        Data::String(s) => s.clone(),
        Data::Int(i) => i.to_string(),
        Data::Float(f) => {
            if f.fract() == 0.0 {
                format!("{}", *f as i64)
            } else {
                format!("{}", f)
            }
        }
        Data::Bool(b) => {
            if *b {
                "true".to_string()
            } else {
                "false".to_string()
            }
        }
        Data::Error(e) => format!("#ERR:{:?}", e),
        Data::DateTime(dt) => {
            excel_date_to_iso(dt.as_f64()).unwrap_or_else(|| format!("{}", dt.as_f64()))
        }
        Data::DateTimeIso(s) => s.clone(),
        Data::DurationIso(s) => s.clone(),
    }
}

/// Convert an Excel serial date number to ISO 8601 date string.
fn excel_date_to_iso(serial: f64) -> Option<String> {
    if serial < 0.0 {
        return None;
    }

    let days = serial as i64;
    let frac = serial - days as f64;

    let date = chrono::NaiveDate::from_ymd_opt(1899, 12, 30)?
        .checked_add_signed(chrono::Duration::days(days))?;

    if frac > 0.001 {
        let total_seconds = (frac * 86400.0).round() as u32;
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;
        let time = chrono::NaiveTime::from_hms_opt(hours, minutes, seconds)?;
        let dt = chrono::NaiveDateTime::new(date, time);
        Some(dt.format("%Y-%m-%dT%H:%M:%S").to_string())
    } else {
        Some(date.format("%Y-%m-%d").to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_cell_string() {
        let cell = Data::String("Hello 한국어".to_string());
        assert_eq!(format_cell(&cell), "Hello 한국어");
    }

    #[test]
    fn test_format_cell_int() {
        let cell = Data::Int(42);
        assert_eq!(format_cell(&cell), "42");
    }

    #[test]
    fn test_format_cell_float_whole() {
        let cell = Data::Float(100.0);
        assert_eq!(format_cell(&cell), "100");
    }

    #[test]
    fn test_format_cell_empty() {
        let cell = Data::Empty;
        assert_eq!(format_cell(&cell), "");
    }

    #[test]
    fn test_excel_date_to_iso() {
        let result = excel_date_to_iso(45292.0);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "2024-01-01");
    }
}
