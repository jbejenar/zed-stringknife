//! URL percent-encoding and decoding transforms (RFC 3986).

use crate::error::StringKnifeError;
use crate::MAX_INPUT_BYTES;

/// Characters that are unreserved in RFC 3986 and never need encoding.
fn is_unreserved(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b'.' || b == b'~'
}

/// Percent-encodes a string per RFC 3986.
///
/// Unreserved characters (`A-Z a-z 0-9 - _ . ~`) are not encoded.
/// All other bytes are encoded as `%XX`.
///
/// # Errors
///
/// Returns [`StringKnifeError::InputTooLarge`] if input exceeds [`MAX_INPUT_BYTES`].
pub fn url_encode(input: &str) -> Result<String, StringKnifeError> {
    check_size(input)?;
    let mut result = String::with_capacity(input.len());
    for &byte in input.as_bytes() {
        if is_unreserved(byte) {
            result.push(char::from(byte));
        } else {
            result.push('%');
            result.push(hex_digit(byte >> 4));
            result.push(hex_digit(byte & 0x0F));
        }
    }
    Ok(result)
}

/// Decodes a percent-encoded string.
///
/// Handles both `%20` (URI encoding) and `+` (form encoding) as space.
///
/// # Errors
///
/// Returns [`StringKnifeError::InvalidInput`] if a `%` is followed by invalid hex digits
/// or if the decoded bytes are not valid UTF-8.
pub fn url_decode(input: &str) -> Result<String, StringKnifeError> {
    check_size(input)?;
    let bytes = input.as_bytes();
    let mut result = Vec::with_capacity(bytes.len());
    let mut i = 0;

    while i < bytes.len() {
        match bytes[i] {
            b'+' => {
                result.push(b' ');
                i += 1;
            }
            b'%' => {
                if i + 2 >= bytes.len() {
                    return Err(StringKnifeError::InvalidInput {
                        operation: "url_decode".to_string(),
                        reason: "incomplete percent-encoding at end of input".to_string(),
                    });
                }
                let hi = from_hex_digit(bytes[i + 1])?;
                let lo = from_hex_digit(bytes[i + 2])?;
                result.push((hi << 4) | lo);
                i += 3;
            }
            b => {
                result.push(b);
                i += 1;
            }
        }
    }

    String::from_utf8(result).map_err(|_| StringKnifeError::InvalidInput {
        operation: "url_decode".to_string(),
        reason: "decoded bytes are not valid UTF-8".to_string(),
    })
}

/// Percent-encodes a string as a URI component.
///
/// Encodes everything except unreserved characters (RFC 3986).
/// This is the same as [`url_encode`] — both use RFC 3986 unreserved set.
/// Provided as a separate action for clarity in the UI.
///
/// # Errors
///
/// Returns [`StringKnifeError::InputTooLarge`] if input exceeds [`MAX_INPUT_BYTES`].
pub fn url_encode_component(input: &str) -> Result<String, StringKnifeError> {
    url_encode(input)
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

fn hex_digit(nibble: u8) -> char {
    match nibble {
        0..=9 => char::from(b'0' + nibble),
        10..=15 => char::from(b'A' + nibble - 10),
        _ => unreachable!(),
    }
}

fn from_hex_digit(byte: u8) -> Result<u8, StringKnifeError> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err(StringKnifeError::InvalidInput {
            operation: "url_decode".to_string(),
            reason: format!("invalid hex digit: '{}'", char::from(byte)),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === URL Encode ===

    #[test]
    fn encode_empty() {
        assert_eq!(url_encode("").unwrap(), "");
    }

    #[test]
    fn encode_unreserved_passthrough() {
        let unreserved = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_.~";
        assert_eq!(url_encode(unreserved).unwrap(), unreserved);
    }

    #[test]
    fn encode_space() {
        assert_eq!(url_encode("hello world").unwrap(), "hello%20world");
    }

    #[test]
    fn encode_reserved_characters() {
        assert_eq!(url_encode("!#$&").unwrap(), "%21%23%24%26");
        assert_eq!(
            url_encode("'()*+,/:;=?@[]").unwrap(),
            "%27%28%29%2A%2B%2C%2F%3A%3B%3D%3F%40%5B%5D"
        );
    }

    #[test]
    fn encode_unicode() {
        // "cafe\u{0301}" = "café" with combining accent, UTF-8: 63 61 66 65 CC 81
        assert_eq!(url_encode("cafe\u{0301}").unwrap(), "cafe%CC%81");
    }

    #[test]
    fn encode_emoji() {
        // Rocket emoji U+1F680, UTF-8: F0 9F 9A 80
        assert_eq!(url_encode("\u{1F680}").unwrap(), "%F0%9F%9A%80");
    }

    // === URL Decode ===

    #[test]
    fn decode_empty() {
        assert_eq!(url_decode("").unwrap(), "");
    }

    #[test]
    fn decode_percent_20_as_space() {
        assert_eq!(url_decode("hello%20world").unwrap(), "hello world");
    }

    #[test]
    fn decode_plus_as_space() {
        assert_eq!(url_decode("hello+world").unwrap(), "hello world");
    }

    #[test]
    fn decode_reserved_characters() {
        assert_eq!(url_decode("%21%23%24%26").unwrap(), "!#$&");
    }

    #[test]
    fn decode_lowercase_hex() {
        assert_eq!(url_decode("%2f%3a").unwrap(), "/:");
    }

    #[test]
    fn decode_unicode() {
        assert_eq!(url_decode("cafe%CC%81").unwrap(), "cafe\u{0301}");
    }

    #[test]
    fn decode_incomplete_percent() {
        let err = url_decode("hello%2").unwrap_err();
        assert!(matches!(err, StringKnifeError::InvalidInput { .. }));
    }

    #[test]
    fn decode_invalid_hex() {
        let err = url_decode("hello%GG").unwrap_err();
        assert!(matches!(err, StringKnifeError::InvalidInput { .. }));
    }

    // === Roundtrip ===

    #[test]
    fn roundtrip_ascii() {
        let input = "Hello, World! How's it going?";
        let encoded = url_encode(input).unwrap();
        let decoded = url_decode(&encoded).unwrap();
        assert_eq!(decoded, input);
    }

    #[test]
    fn roundtrip_unicode() {
        let input = "cafe\u{0301} \u{1F680} \u{00E9}";
        let encoded = url_encode(input).unwrap();
        let decoded = url_decode(&encoded).unwrap();
        assert_eq!(decoded, input);
    }

    // === URL Encode Component ===

    #[test]
    fn encode_component_same_as_encode() {
        let input = "hello world/path?query=value";
        assert_eq!(
            url_encode_component(input).unwrap(),
            url_encode(input).unwrap()
        );
    }

    // === Size limits ===

    #[test]
    fn encode_input_too_large() {
        let big = "a".repeat(MAX_INPUT_BYTES + 1);
        let err = url_encode(&big).unwrap_err();
        assert!(matches!(err, StringKnifeError::InputTooLarge { .. }));
    }

    #[test]
    fn decode_input_too_large() {
        let big = "a".repeat(MAX_INPUT_BYTES + 1);
        let err = url_decode(&big).unwrap_err();
        assert!(matches!(err, StringKnifeError::InputTooLarge { .. }));
    }
}
