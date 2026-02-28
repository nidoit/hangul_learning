/// Detects tabular structures in extracted text lines and converts them to Markdown tables.
///
/// The algorithm:
/// 1. Splits text into lines.
/// 2. Groups consecutive lines that have consistent column separators
///    (multiple consecutive spaces or tab characters).
/// 3. If a group has >= 2 columns and >= 2 rows, it is treated as a table.
/// 4. Outputs Markdown table syntax with a header separator after the first row.

/// Attempt to detect and convert tables in a block of text.
/// Returns the text with detected tables converted to Markdown table format.
pub fn detect_and_convert_tables(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    if lines.is_empty() {
        return String::new();
    }

    let mut result = String::new();
    let mut i = 0;

    while i < lines.len() {
        // Try to detect a table starting at this line
        if let Some((table_md, consumed)) = try_parse_table(&lines[i..]) {
            result.push_str(&table_md);
            result.push('\n');
            i += consumed;
        } else {
            result.push_str(lines[i]);
            result.push('\n');
            i += 1;
        }
    }

    result
}

/// Try to parse a table from the given lines. Returns the Markdown table string
/// and the number of lines consumed, or None if no table was detected.
fn try_parse_table(lines: &[&str]) -> Option<(String, usize)> {
    if lines.len() < 2 {
        return None;
    }

    // Find column boundaries by looking for consistent separator patterns
    // We look for lines that can be split into multiple columns by 2+ spaces
    let mut table_rows: Vec<Vec<String>> = Vec::new();
    let mut consumed = 0;

    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            // End of potential table on blank line
            break;
        }

        // Skip separator lines (----, ====, etc.) but count them
        if is_separator_line(trimmed) {
            consumed += 1;
            continue;
        }

        // Try to split into columns by 2+ consecutive spaces
        let columns = split_into_columns(trimmed);

        if columns.len() >= 2 {
            // Check if column count is consistent with previous rows
            if !table_rows.is_empty() && columns.len() != table_rows[0].len() {
                // Inconsistent column count — stop here
                break;
            }
            table_rows.push(columns);
            consumed += 1;
        } else {
            break;
        }
    }

    // Need at least 2 rows (header + data) and 2 columns to be a table
    if table_rows.len() < 2 || table_rows[0].len() < 2 {
        return None;
    }

    // Build Markdown table
    let mut md = String::new();
    let num_cols = table_rows[0].len();

    // Compute column widths for alignment
    let mut widths: Vec<usize> = vec![3; num_cols]; // minimum width 3
    for row in &table_rows {
        for (j, cell) in row.iter().enumerate() {
            if j < num_cols {
                widths[j] = widths[j].max(cell.len());
            }
        }
    }

    // Header row
    md.push('|');
    for (j, cell) in table_rows[0].iter().enumerate() {
        let w = widths.get(j).copied().unwrap_or(3);
        md.push_str(&format!(" {:<width$} |", cell, width = w));
    }
    md.push('\n');

    // Separator row
    md.push('|');
    for &w in &widths {
        md.push_str(&format!(" {} |", "-".repeat(w)));
    }
    md.push('\n');

    // Data rows
    for row in &table_rows[1..] {
        md.push('|');
        for j in 0..num_cols {
            let cell = row.get(j).map(|s| s.as_str()).unwrap_or("");
            let w = widths.get(j).copied().unwrap_or(3);
            md.push_str(&format!(" {:<width$} |", cell, width = w));
        }
        md.push('\n');
    }

    Some((md, consumed))
}

/// Split a line into columns based on 2+ consecutive whitespace characters.
fn split_into_columns(line: &str) -> Vec<String> {
    let mut columns = Vec::new();
    let mut current = String::new();
    let mut space_count = 0;

    for ch in line.chars() {
        if ch == ' ' || ch == '\t' {
            space_count += 1;
        } else {
            if space_count >= 2 && !current.is_empty() {
                columns.push(current.trim().to_string());
                current = String::new();
            } else if space_count > 0 {
                current.push(' ');
            }
            space_count = 0;
            current.push(ch);
        }
    }

    if !current.trim().is_empty() {
        columns.push(current.trim().to_string());
    }

    columns
}

/// Check if a line is a separator (e.g., "----", "====", "____")
fn is_separator_line(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.len() < 3 {
        return false;
    }
    trimmed.chars().all(|c| c == '-' || c == '=' || c == '_' || c == '+' || c == ' ')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_simple_table() {
        let text = "Name    Age    City\nAlice   30     Seoul\nBob     25     Tokyo";
        let result = detect_and_convert_tables(text);
        assert!(result.contains("|"));
        assert!(result.contains("Name"));
        assert!(result.contains("Alice"));
        assert!(result.contains("---"));
    }

    #[test]
    fn test_no_table_in_plain_text() {
        let text = "This is just a regular paragraph.\nWith two lines.";
        let result = detect_and_convert_tables(text);
        assert!(!result.contains("|"));
    }

    #[test]
    fn test_separator_line_detection() {
        assert!(is_separator_line("----------"));
        assert!(is_separator_line("=========="));
        assert!(!is_separator_line("hello"));
    }
}
