//! CSV to JSON conversion transform.
//!
//! Converts CSV with a header row into a JSON array of objects.
//! Simple RFC 4180 parsing — handles quoted fields with embedded commas
//! and escaped quotes. No external dependencies.

use std::fmt::Write;

use super::common::check_size;
use crate::error::StringKnifeError;

#[cfg(test)]
use crate::MAX_INPUT_BYTES;

/// Converts CSV text (with a header row) into a JSON array of objects.
///
/// The first row is used as field names. Each subsequent row becomes a JSON
/// object with those field names as keys.
///
/// Handles quoted fields (RFC 4180): fields may be enclosed in double quotes,
/// allowing embedded commas and escaped quotes (`""`).
///
/// # Errors
///
/// Returns [`StringKnifeError::InvalidInput`] if the input has fewer than 2 rows
/// or no columns.
/// Returns [`StringKnifeError::InputTooLarge`] if input exceeds the size limit.
pub fn csv_to_json(input: &str) -> Result<String, StringKnifeError> {
    check_size(input)?;
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(StringKnifeError::InvalidInput {
            operation: "csv_to_json".to_string(),
            reason: "empty CSV input".to_string(),
        });
    }

    let rows = parse_csv_rows(trimmed);
    if rows.len() < 2 {
        return Err(StringKnifeError::InvalidInput {
            operation: "csv_to_json".to_string(),
            reason: "CSV must have a header row and at least one data row".to_string(),
        });
    }

    let headers = &rows[0];
    if headers.is_empty() || (headers.len() == 1 && headers[0].is_empty()) {
        return Err(StringKnifeError::InvalidInput {
            operation: "csv_to_json".to_string(),
            reason: "CSV header row has no columns".to_string(),
        });
    }

    let mut result = String::from("[\n");

    for (row_idx, row) in rows[1..].iter().enumerate() {
        if row_idx > 0 {
            result.push_str(",\n");
        }
        result.push_str("  {");

        for (col_idx, header) in headers.iter().enumerate() {
            if col_idx > 0 {
                result.push_str(", ");
            }
            let value = row.get(col_idx).map_or("", String::as_str);
            let _ = write!(
                result,
                "\"{}\": \"{}\"",
                json_escape_str(header),
                json_escape_str(value)
            );
        }

        result.push('}');
    }

    result.push_str("\n]");
    Ok(result)
}

/// Parse CSV into a vector of rows, each row a vector of field strings.
///
/// Handles RFC 4180 quoting: fields enclosed in double quotes can contain
/// commas, newlines, and escaped quotes (`""`).
fn parse_csv_rows(input: &str) -> Vec<Vec<String>> {
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut current_row: Vec<String> = Vec::new();
    let mut current_field = String::new();
    let mut in_quotes = false;
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        let c = chars[i];

        if in_quotes {
            if c == '"' {
                // Check for escaped quote ("")
                if i + 1 < len && chars[i + 1] == '"' {
                    current_field.push('"');
                    i += 2;
                } else {
                    // End of quoted field
                    in_quotes = false;
                    i += 1;
                }
            } else {
                current_field.push(c);
                i += 1;
            }
        } else if c == '"' && current_field.is_empty() {
            // Start of quoted field
            in_quotes = true;
            i += 1;
        } else if c == ',' {
            current_row.push(current_field.clone());
            current_field.clear();
            i += 1;
        } else if c == '\n' {
            current_row.push(current_field.clone());
            current_field.clear();
            rows.push(current_row.clone());
            current_row.clear();
            i += 1;
        } else if c == '\r' {
            // Skip CR, handle LF next
            i += 1;
        } else {
            current_field.push(c);
            i += 1;
        }
    }

    // Don't forget the last field/row
    if !current_field.is_empty() || !current_row.is_empty() {
        current_row.push(current_field);
        rows.push(current_row);
    }

    rows
}

/// Escape a string for JSON string values (minimal: quotes and backslashes).
fn json_escape_str(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            _ => result.push(c),
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_csv() {
        let input = "name,age,city\nAlice,30,NYC\nBob,25,LA";
        let result = csv_to_json(input).unwrap();
        assert!(result.contains("\"name\": \"Alice\""));
        assert!(result.contains("\"age\": \"30\""));
        assert!(result.contains("\"city\": \"NYC\""));
        assert!(result.contains("\"name\": \"Bob\""));
    }

    #[test]
    fn quoted_fields() {
        let input = "name,description\nAlice,\"Has a, comma\"\nBob,\"Says \"\"hello\"\"\"";
        let result = csv_to_json(input).unwrap();
        assert!(result.contains("Has a, comma"));
        assert!(result.contains(r#"Says \"hello\""#));
    }

    #[test]
    fn single_column() {
        let input = "name\nAlice\nBob";
        let result = csv_to_json(input).unwrap();
        assert!(result.contains("\"name\": \"Alice\""));
        assert!(result.contains("\"name\": \"Bob\""));
    }

    #[test]
    fn missing_fields() {
        let input = "a,b,c\n1,2\n4,5,6";
        let result = csv_to_json(input).unwrap();
        // Row with missing field should have empty string
        assert!(result.contains("\"c\": \"\""));
    }

    #[test]
    fn crlf_line_endings() {
        let input = "name,age\r\nAlice,30\r\nBob,25";
        let result = csv_to_json(input).unwrap();
        assert!(result.contains("\"name\": \"Alice\""));
        assert!(result.contains("\"name\": \"Bob\""));
    }

    #[test]
    fn trailing_newline() {
        let input = "name\nAlice\n";
        let result = csv_to_json(input).unwrap();
        assert!(result.contains("\"name\": \"Alice\""));
        // Should not create an extra empty row
        assert_eq!(result.matches('{').count(), 1);
    }

    #[test]
    fn empty_input() {
        assert!(csv_to_json("").is_err());
    }

    #[test]
    fn header_only() {
        assert!(csv_to_json("name,age").is_err());
    }

    #[test]
    fn json_array_structure() {
        let input = "x\n1\n2";
        let result = csv_to_json(input).unwrap();
        assert!(result.starts_with("[\n"));
        assert!(result.ends_with("\n]"));
    }

    #[test]
    fn special_chars_in_values() {
        let input = "msg\nhello\\world\nline1\\nline2";
        let result = csv_to_json(input).unwrap();
        assert!(result.contains("hello\\\\world"));
    }

    #[test]
    fn input_too_large() {
        let big = "a\n".repeat(MAX_INPUT_BYTES / 2 + 1);
        assert!(matches!(
            csv_to_json(&big).unwrap_err(),
            StringKnifeError::InputTooLarge { .. }
        ));
    }
}
