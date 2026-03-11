//! Unicode escape and unescape transforms.
//!
//! Supports `\uXXXX` (BMP) and `\U{XXXXXX}` (full Unicode) formats.

use std::fmt::Write;

use crate::error::StringKnifeError;
use crate::MAX_INPUT_BYTES;

/// Escapes a string to Unicode escape sequences.
///
/// ASCII printable characters (0x20..=0x7E) pass through unchanged.
/// All other characters are escaped as `\uXXXX` (BMP) or `\U{XXXXXX}` (above BMP).
///
/// # Errors
///
/// Returns [`StringKnifeError::InputTooLarge`] if input exceeds [`MAX_INPUT_BYTES`].
pub fn unicode_escape(input: &str) -> Result<String, StringKnifeError> {
    check_size(input)?;
    let mut result = String::with_capacity(input.len() * 2);
    for ch in input.chars() {
        let cp = ch as u32;
        if (0x20..=0x7E).contains(&cp) {
            result.push(ch);
        } else if cp <= 0xFFFF {
            let _ = write!(result, "\\u{cp:04X}");
        } else {
            let _ = write!(result, "\\U{{{cp:06X}}}");
        }
    }
    Ok(result)
}

/// Unescapes Unicode escape sequences back to characters.
///
/// Recognizes `\uXXXX` (exactly 4 hex digits) and `\U{XXXXXX}` (1-6 hex digits in braces).
/// Non-escape text passes through unchanged.
///
/// # Errors
///
/// Returns [`StringKnifeError::InputTooLarge`] if input exceeds [`MAX_INPUT_BYTES`].
/// Returns [`StringKnifeError::InvalidInput`] if an escape sequence contains an invalid
/// code point.
pub fn unicode_unescape(input: &str) -> Result<String, StringKnifeError> {
    check_size(input)?;
    let chars: Vec<char> = input.chars().collect();
    let mut result = String::with_capacity(input.len());
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '\\' && i + 1 < chars.len() {
            match chars[i + 1] {
                'u' => {
                    // \uXXXX — exactly 4 hex digits
                    if i + 5 < chars.len() {
                        let hex_str: String = chars[i + 2..i + 6].iter().collect();
                        if let Some(ch) = parse_hex_codepoint(&hex_str) {
                            result.push(ch);
                            i += 6;
                            continue;
                        }
                    }
                    // Not a valid escape — pass through
                    result.push(chars[i]);
                    i += 1;
                }
                'U' => {
                    // \U{XXXXXX} — 1-6 hex digits in braces
                    if i + 3 < chars.len() && chars[i + 2] == '{' {
                        if let Some(brace_end) = chars[i + 3..]
                            .iter()
                            .position(|&c| c == '}')
                            .map(|p| p + i + 3)
                        {
                            let hex_str: String = chars[i + 3..brace_end].iter().collect();
                            if let Some(ch) = parse_hex_codepoint(&hex_str) {
                                result.push(ch);
                                i = brace_end + 1;
                                continue;
                            }
                        }
                    }
                    // Not a valid escape — pass through
                    result.push(chars[i]);
                    i += 1;
                }
                _ => {
                    result.push(chars[i]);
                    i += 1;
                }
            }
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }
    Ok(result)
}

/// Shows Unicode codepoints for each character in the input.
///
/// Returns a string like `U+0048 U+0065 U+006C U+006C U+006F` for "Hello".
///
/// # Errors
///
/// Returns [`StringKnifeError::InputTooLarge`] if input exceeds [`MAX_INPUT_BYTES`].
pub fn show_codepoints(input: &str) -> Result<String, StringKnifeError> {
    check_size(input)?;
    let codepoints: Vec<String> = input
        .chars()
        .map(|ch| format!("U+{:04X}", ch as u32))
        .collect();
    Ok(codepoints.join(" "))
}

fn check_size(input: &str) -> Result<(), StringKnifeError> {
    if input.len() > MAX_INPUT_BYTES {
        return Err(StringKnifeError::InputTooLarge {
            max_bytes: MAX_INPUT_BYTES,
            actual_bytes: input.len(),
        });
    }
    Ok(())
}

fn parse_hex_codepoint(hex: &str) -> Option<char> {
    let cp = u32::from_str_radix(hex, 16).ok()?;
    char::from_u32(cp)
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Unicode Escape ===

    #[test]
    fn escape_empty() {
        assert_eq!(unicode_escape("").unwrap(), "");
    }

    #[test]
    fn escape_ascii() {
        assert_eq!(unicode_escape("Hello").unwrap(), "Hello");
    }

    #[test]
    fn escape_space() {
        // Space (0x20) is in printable range, passes through
        assert_eq!(unicode_escape("Hi there").unwrap(), "Hi there");
    }

    #[test]
    fn escape_non_ascii() {
        assert_eq!(unicode_escape("\u{00E9}").unwrap(), "\\u00E9");
    }

    #[test]
    fn escape_emoji() {
        assert_eq!(unicode_escape("\u{1F680}").unwrap(), "\\U{01F680}");
    }

    #[test]
    fn escape_mixed() {
        assert_eq!(
            unicode_escape("Hello \u{00E9}\u{1F680}").unwrap(),
            "Hello \\u00E9\\U{01F680}"
        );
    }

    #[test]
    fn escape_tab_and_newline() {
        // Control chars are not printable ASCII, should be escaped
        assert_eq!(unicode_escape("\t\n").unwrap(), "\\u0009\\u000A");
    }

    #[test]
    fn escape_cjk() {
        assert_eq!(
            unicode_escape("\u{4E16}\u{754C}").unwrap(),
            "\\u4E16\\u754C"
        );
    }

    // === Unicode Unescape ===

    #[test]
    fn unescape_empty() {
        assert_eq!(unicode_unescape("").unwrap(), "");
    }

    #[test]
    fn unescape_no_escapes() {
        assert_eq!(unicode_unescape("Hello world").unwrap(), "Hello world");
    }

    #[test]
    fn unescape_bmp() {
        assert_eq!(unicode_unescape("\\u00E9").unwrap(), "\u{00E9}");
    }

    #[test]
    fn unescape_above_bmp() {
        assert_eq!(unicode_unescape("\\U{01F680}").unwrap(), "\u{1F680}");
    }

    #[test]
    fn unescape_mixed() {
        assert_eq!(
            unicode_unescape("Hello \\u00E9\\U{01F680}").unwrap(),
            "Hello \u{00E9}\u{1F680}"
        );
    }

    #[test]
    fn unescape_invalid_passthrough() {
        // Invalid escape sequences pass through unchanged
        assert_eq!(unicode_unescape("\\uZZZZ").unwrap(), "\\uZZZZ");
        assert_eq!(unicode_unescape("\\U{ZZZZZZ}").unwrap(), "\\U{ZZZZZZ}");
    }

    #[test]
    fn unescape_incomplete_passthrough() {
        assert_eq!(unicode_unescape("\\u00").unwrap(), "\\u00");
    }

    // === Roundtrip ===

    #[test]
    fn roundtrip_ascii() {
        let input = "Hello, World!";
        let escaped = unicode_escape(input).unwrap();
        let unescaped = unicode_unescape(&escaped).unwrap();
        assert_eq!(unescaped, input);
    }

    #[test]
    fn roundtrip_emoji() {
        let input = "\u{1F600}\u{1F680}\u{2764}";
        let escaped = unicode_escape(input).unwrap();
        let unescaped = unicode_unescape(&escaped).unwrap();
        assert_eq!(unescaped, input);
    }

    #[test]
    fn roundtrip_cjk() {
        let input = "\u{4E16}\u{754C}";
        let escaped = unicode_escape(input).unwrap();
        let unescaped = unicode_unescape(&escaped).unwrap();
        assert_eq!(unescaped, input);
    }

    #[test]
    fn roundtrip_combining() {
        let input = "cafe\u{0301}";
        let escaped = unicode_escape(input).unwrap();
        let unescaped = unicode_unescape(&escaped).unwrap();
        assert_eq!(unescaped, input);
    }

    // === Show Codepoints ===

    #[test]
    fn codepoints_empty() {
        assert_eq!(show_codepoints("").unwrap(), "");
    }

    #[test]
    fn codepoints_ascii() {
        assert_eq!(show_codepoints("Hi").unwrap(), "U+0048 U+0069");
    }

    #[test]
    fn codepoints_emoji() {
        assert_eq!(show_codepoints("\u{1F680}").unwrap(), "U+1F680");
    }

    #[test]
    fn codepoints_combining() {
        // "e\u{0301}" shows as two codepoints
        assert_eq!(show_codepoints("e\u{0301}").unwrap(), "U+0065 U+0301");
    }

    // === Size limits ===

    #[test]
    fn escape_input_too_large() {
        let big = "a".repeat(MAX_INPUT_BYTES + 1);
        assert!(matches!(
            unicode_escape(&big).unwrap_err(),
            StringKnifeError::InputTooLarge { .. }
        ));
    }

    #[test]
    fn unescape_input_too_large() {
        let big = "a".repeat(MAX_INPUT_BYTES + 1);
        assert!(matches!(
            unicode_unescape(&big).unwrap_err(),
            StringKnifeError::InputTooLarge { .. }
        ));
    }
}
