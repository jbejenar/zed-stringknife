//! HTML entity encoding and decoding transforms.

use crate::error::StringKnifeError;
use crate::MAX_INPUT_BYTES;

/// Named HTML entities for the five special characters.
const NAMED_ENTITIES: &[(&str, char)] = &[
    ("&amp;", '&'),
    ("&lt;", '<'),
    ("&gt;", '>'),
    ("&quot;", '"'),
    ("&apos;", '\''),
    ("&nbsp;", '\u{00A0}'),
];

/// Encodes special HTML characters to named entities.
///
/// Encodes `& < > " '` to `&amp; &lt; &gt; &quot; &apos;`.
///
/// # Errors
///
/// Returns [`StringKnifeError::InputTooLarge`] if input exceeds [`MAX_INPUT_BYTES`].
pub fn html_encode(input: &str) -> Result<String, StringKnifeError> {
    check_size(input)?;
    let mut result = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '"' => result.push_str("&quot;"),
            '\'' => result.push_str("&apos;"),
            other => result.push(other),
        }
    }
    Ok(result)
}

/// Decodes HTML entities back to their character equivalents.
///
/// Supports named entities (`&amp;`, `&lt;`, `&gt;`, `&quot;`, `&apos;`, `&nbsp;`),
/// decimal numeric entities (`&#123;`), and hex numeric entities (`&#x7B;`).
/// Malformed entities pass through unchanged.
///
/// # Errors
///
/// Returns [`StringKnifeError::InputTooLarge`] if input exceeds [`MAX_INPUT_BYTES`].
pub fn html_decode(input: &str) -> Result<String, StringKnifeError> {
    check_size(input)?;
    let mut result = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '&' {
            if let Some((decoded_char, advance)) = try_decode_entity(&chars, i) {
                result.push(decoded_char);
                i += advance;
                continue;
            }
        }
        result.push(chars[i]);
        i += 1;
    }
    Ok(result)
}

/// Tries to decode an HTML entity starting at position `start`.
///
/// Returns `Some((decoded_char, chars_consumed))` on success, `None` if not a valid entity.
fn try_decode_entity(chars: &[char], start: usize) -> Option<(char, usize)> {
    // Find the semicolon — entities are at most ~10 chars
    let max_len = (start + 12).min(chars.len());
    let semi_pos = (start + 1..max_len).find(|&j| chars[j] == ';')?;
    let entity_len = semi_pos - start + 1;

    // Build the entity string
    let entity: String = chars[start..=semi_pos].iter().collect();

    // Check named entities
    for &(name, ch) in NAMED_ENTITIES {
        if entity == name {
            return Some((ch, entity_len));
        }
    }

    // Check numeric entities
    let inner = &entity[2..entity.len() - 1]; // strip &# and ;
    if entity.starts_with("&#x") || entity.starts_with("&#X") {
        // Hex numeric entity: &#x7B;
        let hex_str = &inner[1..]; // strip the 'x' or 'X'
        let code_point = u32::from_str_radix(hex_str, 16).ok()?;
        let ch = char::from_u32(code_point)?;
        return Some((ch, entity_len));
    }

    if entity.starts_with("&#") {
        // Decimal numeric entity: &#123;
        let code_point: u32 = inner.parse().ok()?;
        let ch = char::from_u32(code_point)?;
        return Some((ch, entity_len));
    }

    None
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

#[cfg(test)]
mod tests {
    use super::*;

    // === HTML Encode ===

    #[test]
    fn encode_empty() {
        assert_eq!(html_encode("").unwrap(), "");
    }

    #[test]
    fn encode_no_special_chars() {
        assert_eq!(html_encode("hello world").unwrap(), "hello world");
    }

    #[test]
    fn encode_ampersand() {
        assert_eq!(html_encode("a & b").unwrap(), "a &amp; b");
    }

    #[test]
    fn encode_all_special() {
        assert_eq!(
            html_encode("<div class=\"test\">'hello' & world</div>").unwrap(),
            "&lt;div class=&quot;test&quot;&gt;&apos;hello&apos; &amp; world&lt;/div&gt;"
        );
    }

    #[test]
    fn encode_unicode_passthrough() {
        assert_eq!(
            html_encode("cafe\u{0301} \u{1F680}").unwrap(),
            "cafe\u{0301} \u{1F680}"
        );
    }

    // === HTML Decode ===

    #[test]
    fn decode_empty() {
        assert_eq!(html_decode("").unwrap(), "");
    }

    #[test]
    fn decode_named_entities() {
        assert_eq!(
            html_decode("&amp; &lt; &gt; &quot; &apos;").unwrap(),
            "& < > \" '"
        );
    }

    #[test]
    fn decode_nbsp() {
        assert_eq!(
            html_decode("hello&nbsp;world").unwrap(),
            "hello\u{00A0}world"
        );
    }

    #[test]
    fn decode_decimal_numeric() {
        assert_eq!(html_decode("&#123;").unwrap(), "{");
        assert_eq!(html_decode("&#65;").unwrap(), "A");
    }

    #[test]
    fn decode_hex_numeric() {
        assert_eq!(html_decode("&#x7B;").unwrap(), "{");
        assert_eq!(html_decode("&#x41;").unwrap(), "A");
        assert_eq!(html_decode("&#X41;").unwrap(), "A");
    }

    #[test]
    fn decode_malformed_passthrough() {
        // Incomplete entity — pass through unchanged
        assert_eq!(html_decode("&unknown;").unwrap(), "&unknown;");
        assert_eq!(html_decode("& no semicolon").unwrap(), "& no semicolon");
        assert_eq!(html_decode("&#notanumber;").unwrap(), "&#notanumber;");
    }

    #[test]
    fn decode_mixed() {
        assert_eq!(
            html_decode("&lt;p&gt;Hello &#38; &#x26; world&lt;/p&gt;").unwrap(),
            "<p>Hello & & world</p>"
        );
    }

    // === Roundtrip ===

    #[test]
    fn roundtrip() {
        let input = "<div class=\"test\">'hello' & world</div>";
        let encoded = html_encode(input).unwrap();
        let decoded = html_decode(&encoded).unwrap();
        assert_eq!(decoded, input);
    }

    #[test]
    fn roundtrip_nested() {
        // Encoding already-encoded content should be reversible
        let input = "&amp;";
        let encoded = html_encode(input).unwrap();
        assert_eq!(encoded, "&amp;amp;");
        let decoded = html_decode(&encoded).unwrap();
        assert_eq!(decoded, input);
    }

    // === Size limits ===

    #[test]
    fn encode_input_too_large() {
        let big = "a".repeat(MAX_INPUT_BYTES + 1);
        assert!(matches!(
            html_encode(&big).unwrap_err(),
            StringKnifeError::InputTooLarge { .. }
        ));
    }

    #[test]
    fn decode_input_too_large() {
        let big = "a".repeat(MAX_INPUT_BYTES + 1);
        assert!(matches!(
            html_decode(&big).unwrap_err(),
            StringKnifeError::InputTooLarge { .. }
        ));
    }
}
