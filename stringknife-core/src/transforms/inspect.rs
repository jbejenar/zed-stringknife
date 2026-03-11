//! String inspection transforms (non-destructive).
//!
//! These functions analyze the input and return a summary string
//! rather than transforming the content.

use std::fmt::Write;

use super::common::check_size;
use crate::detect::detect_encodings;
use crate::error::StringKnifeError;

#[cfg(test)]
use crate::MAX_INPUT_BYTES;

/// Returns character count, byte count, word count, and line count.
///
/// # Errors
///
/// Returns [`StringKnifeError::InputTooLarge`] if input exceeds the size limit.
pub fn count_chars(input: &str) -> Result<String, StringKnifeError> {
    check_size(input)?;
    let chars = input.chars().count();
    let bytes = input.len();
    let words = input.split_whitespace().count();
    let lines = if input.is_empty() {
        0
    } else {
        input.lines().count()
    };
    Ok(format!(
        "Characters: {chars}\nBytes (UTF-8): {bytes}\nWords: {words}\nLines: {lines}"
    ))
}

/// Returns the UTF-8 byte length of the input.
///
/// # Errors
///
/// Returns [`StringKnifeError::InputTooLarge`] if input exceeds the size limit.
pub fn byte_length(input: &str) -> Result<String, StringKnifeError> {
    check_size(input)?;
    Ok(format!("{} bytes (UTF-8)", input.len()))
}

/// Detects the encoding of the input and returns a human-readable summary.
///
/// Uses the smart detection module to identify Base64, URL-encoded, hex,
/// HTML entity, Unicode escape, and JWT patterns.
///
/// # Errors
///
/// Returns [`StringKnifeError::InputTooLarge`] if input exceeds the size limit.
pub fn detect_encoding(input: &str) -> Result<String, StringKnifeError> {
    check_size(input)?;
    let detected = detect_encodings(input);
    if detected.is_empty() {
        return Ok("No known encoding detected".to_string());
    }
    let mut result = String::from("Detected encodings:\n");
    for enc in &detected {
        let _ = writeln!(result, "• {enc:?}");
    }
    // Remove trailing newline
    if result.ends_with('\n') {
        result.pop();
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn count_ascii() {
        let result = count_chars("hello world").unwrap();
        assert!(result.contains("Characters: 11"));
        assert!(result.contains("Bytes (UTF-8): 11"));
        assert!(result.contains("Words: 2"));
        assert!(result.contains("Lines: 1"));
    }

    #[test]
    fn count_unicode() {
        let result = count_chars("héllo 世界").unwrap();
        assert!(result.contains("Characters: 8"));
        // é = 2 bytes, 世 = 3 bytes, 界 = 3 bytes
        assert!(result.contains("Bytes (UTF-8): 13"));
    }

    #[test]
    fn count_multiline() {
        let result = count_chars("a\nb\nc").unwrap();
        assert!(result.contains("Lines: 3"));
    }

    #[test]
    fn count_empty() {
        let result = count_chars("").unwrap();
        assert!(result.contains("Characters: 0"));
        assert!(result.contains("Lines: 0"));
    }

    #[test]
    fn byte_length_ascii() {
        assert_eq!(byte_length("hello").unwrap(), "5 bytes (UTF-8)");
    }

    #[test]
    fn byte_length_unicode() {
        // 🎉 = 4 bytes
        assert_eq!(byte_length("🎉").unwrap(), "4 bytes (UTF-8)");
    }

    #[test]
    fn detect_base64() {
        let result = detect_encoding("SGVsbG8gV29ybGQ=").unwrap();
        assert!(result.contains("Base64"));
    }

    #[test]
    fn detect_nothing() {
        let result = detect_encoding("hello").unwrap();
        assert!(result.contains("No known encoding"));
    }

    #[test]
    fn detect_jwt() {
        let result = detect_encoding("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c").unwrap();
        assert!(result.contains("Jwt"));
    }

    #[test]
    fn input_too_large() {
        let big = "a".repeat(MAX_INPUT_BYTES + 1);
        assert!(matches!(
            count_chars(&big).unwrap_err(),
            StringKnifeError::InputTooLarge { .. }
        ));
    }
}
