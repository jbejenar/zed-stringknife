//! JSON formatting and string escape transforms.
//!
//! Pretty print, minify, and escape/unescape JSON strings.
//! No external JSON parser — character-level formatting only.

use std::fmt::Write;

use super::common::check_size;
use crate::error::StringKnifeError;

#[cfg(test)]
use crate::MAX_INPUT_BYTES;

/// Pretty-prints JSON with 2-space indentation.
///
/// Re-formats valid JSON regardless of existing formatting. Handles strings
/// (with escaped quotes), numbers, booleans, null, arrays, and objects.
///
/// # Errors
///
/// Returns [`StringKnifeError::InvalidInput`] if the input is empty.
/// Returns [`StringKnifeError::InputTooLarge`] if input exceeds the size limit.
pub fn json_pretty_print(input: &str) -> Result<String, StringKnifeError> {
    check_size(input)?;
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(StringKnifeError::InvalidInput {
            operation: "json_pretty_print".to_string(),
            reason: "empty JSON input".to_string(),
        });
    }

    let mut result = String::with_capacity(trimmed.len() * 2);
    let mut indent: usize = 0;
    let mut in_string = false;
    let mut escape_next = false;
    let chars: Vec<char> = trimmed.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        if escape_next {
            result.push(c);
            escape_next = false;
            i += 1;
            continue;
        }

        if in_string {
            result.push(c);
            if c == '\\' {
                escape_next = true;
            } else if c == '"' {
                in_string = false;
            }
            i += 1;
            continue;
        }

        match c {
            '"' => {
                in_string = true;
                result.push(c);
            }
            '{' | '[' => {
                result.push(c);
                indent += 1;
                // Check if next non-whitespace is closing bracket (empty object/array)
                if let Some(pos) = chars[i + 1..].iter().position(|ch| !ch.is_whitespace()) {
                    let next_char = chars[i + 1 + pos];
                    if (c == '{' && next_char == '}') || (c == '[' && next_char == ']') {
                        indent -= 1;
                        result.push(next_char);
                        i = i + 2 + pos;
                        continue;
                    }
                }
                result.push('\n');
                push_indent(&mut result, indent);
            }
            '}' | ']' => {
                indent = indent.saturating_sub(1);
                result.push('\n');
                push_indent(&mut result, indent);
                result.push(c);
            }
            ',' => {
                result.push(',');
                result.push('\n');
                push_indent(&mut result, indent);
            }
            ':' => {
                result.push(':');
                result.push(' ');
            }
            c if c.is_whitespace() => {}
            _ => {
                result.push(c);
            }
        }

        i += 1;
    }

    Ok(result)
}

/// Minifies JSON by removing all unnecessary whitespace.
///
/// Preserves whitespace inside string values.
///
/// # Errors
///
/// Returns [`StringKnifeError::InvalidInput`] if the input is empty.
/// Returns [`StringKnifeError::InputTooLarge`] if input exceeds the size limit.
pub fn json_minify(input: &str) -> Result<String, StringKnifeError> {
    check_size(input)?;
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(StringKnifeError::InvalidInput {
            operation: "json_minify".to_string(),
            reason: "empty JSON input".to_string(),
        });
    }

    let mut result = String::with_capacity(trimmed.len());
    let mut in_string = false;
    let mut escape_next = false;

    for c in trimmed.chars() {
        if escape_next {
            result.push(c);
            escape_next = false;
            continue;
        }

        if in_string {
            result.push(c);
            if c == '\\' {
                escape_next = true;
            } else if c == '"' {
                in_string = false;
            }
            continue;
        }

        if c == '"' {
            in_string = true;
            result.push(c);
        } else if !c.is_whitespace() {
            result.push(c);
        }
    }

    Ok(result)
}

/// Escapes a string for embedding in a JSON string value.
///
/// Escapes control characters, backslashes, double quotes, and other special
/// characters per the JSON specification (RFC 8259).
///
/// # Errors
///
/// Returns [`StringKnifeError::InputTooLarge`] if input exceeds the size limit.
pub fn json_escape(input: &str) -> Result<String, StringKnifeError> {
    check_size(input)?;
    let mut result = String::with_capacity(input.len() + 16);
    for c in input.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            '\u{08}' => result.push_str("\\b"),
            '\u{0C}' => result.push_str("\\f"),
            c if c.is_control() => {
                let code = c as u32;
                if code <= 0xFFFF {
                    let _ = write!(result, "\\u{code:04x}");
                } else {
                    result.push(c);
                }
            }
            _ => result.push(c),
        }
    }
    Ok(result)
}

/// Unescapes a JSON string value.
///
/// Processes escape sequences: `\"`, `\\`, `\/`, `\n`, `\r`, `\t`, `\b`, `\f`,
/// and `\uXXXX` (including surrogate pairs for characters above BMP).
///
/// # Errors
///
/// Returns [`StringKnifeError::InputTooLarge`] if input exceeds the size limit.
pub fn json_unescape(input: &str) -> Result<String, StringKnifeError> {
    check_size(input)?;
    let mut result = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '\\' && i + 1 < chars.len() {
            match chars[i + 1] {
                '"' => {
                    result.push('"');
                    i += 2;
                }
                '\\' => {
                    result.push('\\');
                    i += 2;
                }
                '/' => {
                    result.push('/');
                    i += 2;
                }
                'n' => {
                    result.push('\n');
                    i += 2;
                }
                'r' => {
                    result.push('\r');
                    i += 2;
                }
                't' => {
                    result.push('\t');
                    i += 2;
                }
                'b' => {
                    result.push('\u{08}');
                    i += 2;
                }
                'f' => {
                    result.push('\u{0C}');
                    i += 2;
                }
                'u' if i + 5 < chars.len() => {
                    let hex: String = chars[i + 2..i + 6].iter().collect();
                    if let Ok(code) = u32::from_str_radix(&hex, 16) {
                        // Handle surrogate pairs
                        if (0xD800..=0xDBFF).contains(&code)
                            && i + 11 < chars.len()
                            && chars[i + 6] == '\\'
                            && chars[i + 7] == 'u'
                        {
                            let low_hex: String = chars[i + 8..i + 12].iter().collect();
                            if let Ok(low) = u32::from_str_radix(&low_hex, 16) {
                                if (0xDC00..=0xDFFF).contains(&low) {
                                    let cp = 0x10000 + ((code - 0xD800) << 10) + (low - 0xDC00);
                                    if let Some(ch) = char::from_u32(cp) {
                                        result.push(ch);
                                        i += 12;
                                        continue;
                                    }
                                }
                            }
                        }
                        if let Some(ch) = char::from_u32(code) {
                            result.push(ch);
                        } else {
                            // Invalid codepoint, pass through
                            result.push('\\');
                            result.push('u');
                            result.push_str(&hex);
                        }
                        i += 6;
                    } else {
                        result.push(chars[i]);
                        i += 1;
                    }
                }
                _ => {
                    // Unknown escape, pass through
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

fn push_indent(s: &mut String, level: usize) {
    for _ in 0..level {
        s.push_str("  ");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === JSON Pretty Print ===

    #[test]
    fn pretty_print_simple_object() {
        let result = json_pretty_print(r#"{"name":"John","age":30}"#).unwrap();
        assert!(result.contains("\"name\": \"John\""));
        assert!(result.contains("\"age\": 30"));
        assert!(result.contains('\n'));
    }

    #[test]
    fn pretty_print_nested_object() {
        let result = json_pretty_print(r#"{"a":{"b":{"c":1}}}"#).unwrap();
        assert!(result.contains("      \"c\": 1")); // 6 spaces = 3 levels
    }

    #[test]
    fn pretty_print_array() {
        let result = json_pretty_print(r#"[1,2,3]"#).unwrap();
        assert!(result.contains("  1,\n  2,\n  3"));
    }

    #[test]
    fn pretty_print_empty_object() {
        assert_eq!(json_pretty_print("{}").unwrap(), "{}");
    }

    #[test]
    fn pretty_print_empty_array() {
        assert_eq!(json_pretty_print("[]").unwrap(), "[]");
    }

    #[test]
    fn pretty_print_already_pretty() {
        let pretty = "{\n  \"a\": 1\n}";
        let result = json_pretty_print(pretty).unwrap();
        assert_eq!(result, "{\n  \"a\": 1\n}");
    }

    #[test]
    fn pretty_print_string_with_special_chars() {
        let result = json_pretty_print(r#"{"msg":"hello \"world\""}"#).unwrap();
        assert!(result.contains(r#"\"world\""#));
    }

    #[test]
    fn pretty_print_booleans_and_null() {
        let result = json_pretty_print(r#"{"a":true,"b":false,"c":null}"#).unwrap();
        assert!(result.contains("true"));
        assert!(result.contains("false"));
        assert!(result.contains("null"));
    }

    #[test]
    fn pretty_print_empty_input() {
        assert!(json_pretty_print("").is_err());
    }

    #[test]
    fn pretty_print_input_too_large() {
        let big = "a".repeat(MAX_INPUT_BYTES + 1);
        assert!(matches!(
            json_pretty_print(&big).unwrap_err(),
            StringKnifeError::InputTooLarge { .. }
        ));
    }

    // === JSON Minify ===

    #[test]
    fn minify_simple() {
        let result = json_minify("{\n  \"a\": 1,\n  \"b\": 2\n}").unwrap();
        assert_eq!(result, r#"{"a":1,"b":2}"#);
    }

    #[test]
    fn minify_preserves_string_spaces() {
        let result = json_minify(r#"{ "msg": "hello world" }"#).unwrap();
        assert_eq!(result, r#"{"msg":"hello world"}"#);
    }

    #[test]
    fn minify_already_minified() {
        let input = r#"{"a":1}"#;
        assert_eq!(json_minify(input).unwrap(), input);
    }

    #[test]
    fn minify_empty_input() {
        assert!(json_minify("").is_err());
    }

    #[test]
    fn minify_input_too_large() {
        let big = "a".repeat(MAX_INPUT_BYTES + 1);
        assert!(matches!(
            json_minify(&big).unwrap_err(),
            StringKnifeError::InputTooLarge { .. }
        ));
    }

    // === JSON Escape ===

    #[test]
    fn escape_quotes() {
        assert_eq!(json_escape(r#"say "hello""#).unwrap(), r#"say \"hello\""#);
    }

    #[test]
    fn escape_backslash() {
        assert_eq!(json_escape(r"path\to\file").unwrap(), r"path\\to\\file");
    }

    #[test]
    fn escape_newline_tab() {
        assert_eq!(
            json_escape("line1\nline2\ttab").unwrap(),
            "line1\\nline2\\ttab"
        );
    }

    #[test]
    fn escape_control_chars() {
        assert_eq!(json_escape("\u{00}\u{01}").unwrap(), "\\u0000\\u0001");
    }

    #[test]
    fn escape_empty() {
        assert_eq!(json_escape("").unwrap(), "");
    }

    #[test]
    fn escape_unicode_passthrough() {
        assert_eq!(json_escape("héllo 世界").unwrap(), "héllo 世界");
    }

    #[test]
    fn escape_input_too_large() {
        let big = "a".repeat(MAX_INPUT_BYTES + 1);
        assert!(matches!(
            json_escape(&big).unwrap_err(),
            StringKnifeError::InputTooLarge { .. }
        ));
    }

    // === JSON Unescape ===

    #[test]
    fn unescape_quotes() {
        assert_eq!(json_unescape(r#"say \"hello\""#).unwrap(), r#"say "hello""#);
    }

    #[test]
    fn unescape_backslash() {
        assert_eq!(json_unescape(r"path\\to\\file").unwrap(), r"path\to\file");
    }

    #[test]
    fn unescape_newline_tab() {
        assert_eq!(
            json_unescape("line1\\nline2\\ttab").unwrap(),
            "line1\nline2\ttab"
        );
    }

    #[test]
    fn unescape_unicode_bmp() {
        assert_eq!(
            json_unescape("\\u0048\\u0065\\u006C\\u006C\\u006F").unwrap(),
            "Hello"
        );
    }

    #[test]
    fn unescape_surrogate_pair() {
        // 🎉 = U+1F389 = surrogate pair \uD83C\uDF89
        assert_eq!(json_unescape("\\uD83C\\uDF89").unwrap(), "🎉");
    }

    #[test]
    fn unescape_slash() {
        assert_eq!(json_unescape(r"a\/b").unwrap(), "a/b");
    }

    #[test]
    fn unescape_empty() {
        assert_eq!(json_unescape("").unwrap(), "");
    }

    #[test]
    fn unescape_no_escapes() {
        assert_eq!(json_unescape("plain text").unwrap(), "plain text");
    }

    #[test]
    fn unescape_input_too_large() {
        let big = "a".repeat(MAX_INPUT_BYTES + 1);
        assert!(matches!(
            json_unescape(&big).unwrap_err(),
            StringKnifeError::InputTooLarge { .. }
        ));
    }

    // === Roundtrip ===

    #[test]
    fn roundtrip_pretty_minify() {
        let input = r#"{"name":"test","items":[1,2,3],"nested":{"a":true}}"#;
        let pretty = json_pretty_print(input).unwrap();
        let minified = json_minify(&pretty).unwrap();
        assert_eq!(minified, input);
    }

    #[test]
    fn roundtrip_escape_unescape() {
        let input = "hello \"world\"\npath\\to\\file\ttab";
        let escaped = json_escape(input).unwrap();
        let unescaped = json_unescape(&escaped).unwrap();
        assert_eq!(unescaped, input);
    }
}
