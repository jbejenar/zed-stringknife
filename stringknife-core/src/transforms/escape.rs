//! Escape and unescape transforms for common contexts.
//!
//! Backslash escaping, regex metacharacter escaping, SQL string escaping,
//! shell string escaping, and CSV field escaping.

use super::common::check_size;
use crate::error::StringKnifeError;

#[cfg(test)]
use crate::MAX_INPUT_BYTES;

/// Escapes backslashes (`\` → `\\`).
///
/// # Errors
///
/// Returns [`StringKnifeError::InputTooLarge`] if input exceeds the size limit.
pub fn escape_backslashes(input: &str) -> Result<String, StringKnifeError> {
    check_size(input)?;
    Ok(input.replace('\\', "\\\\"))
}

/// Unescapes backslashes (`\\` → `\`).
///
/// # Errors
///
/// Returns [`StringKnifeError::InputTooLarge`] if input exceeds the size limit.
pub fn unescape_backslashes(input: &str) -> Result<String, StringKnifeError> {
    check_size(input)?;
    Ok(input.replace("\\\\", "\\"))
}

/// Escapes regex special characters so the string can be used as a literal
/// in a regular expression pattern.
///
/// Characters escaped: `.` `^` `$` `*` `+` `?` `(` `)` `[` `]` `{` `}` `|` `\`
///
/// # Errors
///
/// Returns [`StringKnifeError::InputTooLarge`] if input exceeds the size limit.
pub fn escape_regex(input: &str) -> Result<String, StringKnifeError> {
    check_size(input)?;
    let mut result = String::with_capacity(input.len() * 2);
    for c in input.chars() {
        if ".^$*+?()[]{}|\\".contains(c) {
            result.push('\\');
        }
        result.push(c);
    }
    Ok(result)
}

/// Escapes a string for use in a SQL single-quoted string.
///
/// Doubles single quotes (`'` → `''`) per SQL standard.
///
/// # Errors
///
/// Returns [`StringKnifeError::InputTooLarge`] if input exceeds the size limit.
pub fn escape_sql(input: &str) -> Result<String, StringKnifeError> {
    check_size(input)?;
    Ok(input.replace('\'', "''"))
}

/// Escapes a string for safe use in a shell (POSIX `sh`).
///
/// Wraps the string in single quotes and escapes any existing single quotes
/// with the `'\''` idiom (end quote, escaped quote, start quote).
///
/// # Errors
///
/// Returns [`StringKnifeError::InputTooLarge`] if input exceeds the size limit.
pub fn escape_shell(input: &str) -> Result<String, StringKnifeError> {
    check_size(input)?;
    let escaped = input.replace('\'', "'\\''");
    Ok(format!("'{escaped}'"))
}

/// Escapes a field for CSV output per RFC 4180.
///
/// If the field contains commas, double quotes, or newlines, it is wrapped
/// in double quotes. Existing double quotes are doubled (`"` → `""`).
///
/// # Errors
///
/// Returns [`StringKnifeError::InputTooLarge`] if input exceeds the size limit.
pub fn escape_csv(input: &str) -> Result<String, StringKnifeError> {
    check_size(input)?;
    if input.contains(',') || input.contains('"') || input.contains('\n') || input.contains('\r') {
        let escaped = input.replace('"', "\"\"");
        Ok(format!("\"{escaped}\""))
    } else {
        Ok(input.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Backslash escape/unescape ===

    #[test]
    fn escape_backslash() {
        assert_eq!(escape_backslashes(r"a\b\c").unwrap(), r"a\\b\\c");
    }

    #[test]
    fn unescape_backslash() {
        assert_eq!(unescape_backslashes(r"a\\b\\c").unwrap(), r"a\b\c");
    }

    #[test]
    fn roundtrip_backslash() {
        let input = r"path\to\file";
        let escaped = escape_backslashes(input).unwrap();
        let unescaped = unescape_backslashes(&escaped).unwrap();
        assert_eq!(unescaped, input);
    }

    // === Regex escape ===

    #[test]
    fn escape_regex_metacharacters() {
        assert_eq!(escape_regex("foo.bar[0]").unwrap(), r"foo\.bar\[0\]");
    }

    #[test]
    fn escape_regex_all_special() {
        let result = escape_regex(".^$*+?()[]{}|\\").unwrap();
        assert_eq!(result, r"\.\^\$\*\+\?\(\)\[\]\{\}\|\\");
    }

    #[test]
    fn escape_regex_plain() {
        assert_eq!(escape_regex("hello").unwrap(), "hello");
    }

    // === SQL escape ===

    #[test]
    fn escape_sql_quotes() {
        assert_eq!(escape_sql("it's a test").unwrap(), "it''s a test");
    }

    #[test]
    fn escape_sql_no_quotes() {
        assert_eq!(escape_sql("hello").unwrap(), "hello");
    }

    // === Shell escape ===

    #[test]
    fn escape_shell_simple() {
        assert_eq!(escape_shell("hello world").unwrap(), "'hello world'");
    }

    #[test]
    fn escape_shell_with_quotes() {
        assert_eq!(escape_shell("it's here").unwrap(), "'it'\\''s here'");
    }

    #[test]
    fn escape_shell_special_chars() {
        assert_eq!(escape_shell("$HOME;rm -rf /").unwrap(), "'$HOME;rm -rf /'");
    }

    // === CSV escape ===

    #[test]
    fn escape_csv_with_comma() {
        assert_eq!(escape_csv("a,b").unwrap(), "\"a,b\"");
    }

    #[test]
    fn escape_csv_with_quotes() {
        assert_eq!(escape_csv("say \"hi\"").unwrap(), "\"say \"\"hi\"\"\"");
    }

    #[test]
    fn escape_csv_with_newline() {
        assert_eq!(escape_csv("a\nb").unwrap(), "\"a\nb\"");
    }

    #[test]
    fn escape_csv_plain() {
        assert_eq!(escape_csv("hello").unwrap(), "hello");
    }

    // === Edge cases ===

    #[test]
    fn empty_input() {
        assert_eq!(escape_backslashes("").unwrap(), "");
        assert_eq!(escape_regex("").unwrap(), "");
        assert_eq!(escape_sql("").unwrap(), "");
        assert_eq!(escape_csv("").unwrap(), "");
    }

    #[test]
    fn input_too_large() {
        let big = "a".repeat(MAX_INPUT_BYTES + 1);
        assert!(matches!(
            escape_backslashes(&big).unwrap_err(),
            StringKnifeError::InputTooLarge { .. }
        ));
        assert!(matches!(
            escape_regex(&big).unwrap_err(),
            StringKnifeError::InputTooLarge { .. }
        ));
    }
}
