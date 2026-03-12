//! Base64 encoding and decoding transforms.
//!
//! Supports both standard (RFC 4648) and URL-safe alphabets.

use super::common::check_size;
use crate::error::StringKnifeError;

#[cfg(test)]
use crate::MAX_INPUT_BYTES;

const STANDARD_ALPHABET: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

const URL_SAFE_ALPHABET: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

/// Encodes the input string to standard Base64 (RFC 4648).
///
/// The input is treated as UTF-8 bytes. Output includes `=` padding.
///
/// # Errors
///
/// Returns [`StringKnifeError::InputTooLarge`] if input exceeds [`MAX_INPUT_BYTES`].
pub fn base64_encode(input: &str) -> Result<String, StringKnifeError> {
    check_size(input)?;
    Ok(encode_bytes(input.as_bytes(), STANDARD_ALPHABET, true))
}

/// Decodes a standard Base64 string back to UTF-8 text.
///
/// Accepts both padded and unpadded input.
///
/// # Errors
///
/// Returns [`StringKnifeError::InvalidInput`] if the input is not valid Base64
/// or if the decoded bytes are not valid UTF-8.
pub fn base64_decode(input: &str) -> Result<String, StringKnifeError> {
    check_size(input)?;
    let bytes = decode_bytes(input, STANDARD_ALPHABET)?;
    String::from_utf8(bytes).map_err(|_| StringKnifeError::InvalidInput {
        operation: "base64_decode".to_string(),
        reason: "decoded bytes are not valid UTF-8".to_string(),
    })
}

/// Encodes the input string to URL-safe Base64 (RFC 4648, section 5).
///
/// Uses `-` and `_` instead of `+` and `/`. No padding.
///
/// # Errors
///
/// Returns [`StringKnifeError::InputTooLarge`] if input exceeds [`MAX_INPUT_BYTES`].
pub fn base64url_encode(input: &str) -> Result<String, StringKnifeError> {
    check_size(input)?;
    Ok(encode_bytes(input.as_bytes(), URL_SAFE_ALPHABET, false))
}

/// Decodes a URL-safe Base64 string back to UTF-8 text.
///
/// Accepts both padded and unpadded input.
///
/// # Errors
///
/// Returns [`StringKnifeError::InvalidInput`] if the input is not valid Base64
/// or if the decoded bytes are not valid UTF-8.
pub fn base64url_decode(input: &str) -> Result<String, StringKnifeError> {
    check_size(input)?;
    let bytes = decode_bytes(input, URL_SAFE_ALPHABET)?;
    String::from_utf8(bytes).map_err(|_| StringKnifeError::InvalidInput {
        operation: "base64url_decode".to_string(),
        reason: "decoded bytes are not valid UTF-8".to_string(),
    })
}

/// Encodes the input string to standard Base64 with MIME-style line wrapping.
///
/// Output is wrapped at 76 characters per line (per RFC 2045) with CRLF line endings.
///
/// # Errors
///
/// Returns [`StringKnifeError::InputTooLarge`] if input exceeds [`MAX_INPUT_BYTES`].
pub fn base64_encode_wrapped(input: &str) -> Result<String, StringKnifeError> {
    check_size(input)?;
    let raw = encode_bytes(input.as_bytes(), STANDARD_ALPHABET, true);
    let mut result = String::with_capacity(raw.len() + raw.len() / 76 * 2);
    for (i, ch) in raw.chars().enumerate() {
        if i > 0 && i % 76 == 0 {
            result.push('\r');
            result.push('\n');
        }
        result.push(ch);
    }
    Ok(result)
}

fn encode_bytes(bytes: &[u8], alphabet: &[u8; 64], pad: bool) -> String {
    let mut result = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b0 = u32::from(chunk[0]);
        let b1 = chunk.get(1).map_or(0, |&b| u32::from(b));
        let b2 = chunk.get(2).map_or(0, |&b| u32::from(b));
        let triple = (b0 << 16) | (b1 << 8) | b2;

        result.push(char::from(alphabet[((triple >> 18) & 0x3F) as usize]));
        result.push(char::from(alphabet[((triple >> 12) & 0x3F) as usize]));

        if chunk.len() > 1 {
            result.push(char::from(alphabet[((triple >> 6) & 0x3F) as usize]));
        } else if pad {
            result.push('=');
        }

        if chunk.len() > 2 {
            result.push(char::from(alphabet[(triple & 0x3F) as usize]));
        } else if pad {
            result.push('=');
        }
    }
    result
}

fn build_decode_table(alphabet: &[u8; 64]) -> [u8; 256] {
    let mut table = [0xFF_u8; 256];
    for (i, &ch) in alphabet.iter().enumerate() {
        // Alphabet has exactly 64 entries, so i always fits in u8.
        #[allow(clippy::cast_possible_truncation)]
        let idx = i as u8;
        table[ch as usize] = idx;
    }
    table
}

fn decode_bytes(input: &str, alphabet: &[u8; 64]) -> Result<Vec<u8>, StringKnifeError> {
    let table = build_decode_table(alphabet);
    // Strip padding and whitespace
    let clean: Vec<u8> = input
        .bytes()
        .filter(|&b| b != b'=' && !b.is_ascii_whitespace())
        .collect();

    for &b in &clean {
        if table[b as usize] == 0xFF {
            return Err(StringKnifeError::InvalidInput {
                operation: "base64_decode".to_string(),
                reason: format!("invalid character: '{}'", char::from(b)),
            });
        }
    }

    let mut result = Vec::with_capacity(clean.len() * 3 / 4);
    for chunk in clean.chunks(4) {
        let vals: Vec<u32> = chunk
            .iter()
            .map(|&b| u32::from(table[b as usize]))
            .collect();
        let len = vals.len();

        if len >= 2 {
            let triple = (vals[0] << 18)
                | (vals[1] << 12)
                | (if len > 2 { vals[2] } else { 0 } << 6)
                | if len > 3 { vals[3] } else { 0 };

            // Bit-shifting a 24-bit value — results always fit in u8.
            #[allow(clippy::cast_possible_truncation)]
            {
                result.push((triple >> 16) as u8);
                if len > 2 {
                    result.push((triple >> 8) as u8);
                }
                if len > 3 {
                    result.push(triple as u8);
                }
            }
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Base64 Standard Encode ===

    #[test]
    fn encode_empty() {
        assert_eq!(base64_encode("").unwrap(), "");
    }

    #[test]
    fn encode_ascii() {
        assert_eq!(
            base64_encode("Hello, World!").unwrap(),
            "SGVsbG8sIFdvcmxkIQ=="
        );
    }

    #[test]
    fn encode_unicode() {
        // "cafe\u{0301}" = "café" (with combining accent)
        assert_eq!(base64_encode("cafe\u{0301}").unwrap(), "Y2FmZcyB");
    }

    #[test]
    fn encode_single_byte() {
        assert_eq!(base64_encode("A").unwrap(), "QQ==");
    }

    #[test]
    fn encode_two_bytes() {
        assert_eq!(base64_encode("AB").unwrap(), "QUI=");
    }

    // === Base64 Standard Decode ===

    #[test]
    fn decode_empty() {
        assert_eq!(base64_decode("").unwrap(), "");
    }

    #[test]
    fn decode_ascii() {
        assert_eq!(
            base64_decode("SGVsbG8sIFdvcmxkIQ==").unwrap(),
            "Hello, World!"
        );
    }

    #[test]
    fn decode_unpadded() {
        assert_eq!(
            base64_decode("SGVsbG8sIFdvcmxkIQ").unwrap(),
            "Hello, World!"
        );
    }

    #[test]
    fn decode_invalid_char() {
        let err = base64_decode("!!!").unwrap_err();
        assert!(matches!(err, StringKnifeError::InvalidInput { .. }));
    }

    #[test]
    fn decode_invalid_utf8() {
        // \xFF\xFE are not valid UTF-8; base64 of those bytes is "//4="
        let err = base64_decode("//4=").unwrap_err();
        assert!(matches!(err, StringKnifeError::InvalidInput { .. }));
    }

    // === Roundtrip ===

    #[test]
    fn roundtrip_standard() {
        let input = "The quick brown fox jumps over the lazy dog";
        let encoded = base64_encode(input).unwrap();
        let decoded = base64_decode(&encoded).unwrap();
        assert_eq!(decoded, input);
    }

    #[test]
    fn roundtrip_unicode() {
        let input = "Hello \u{1F600} World \u{00E9}";
        let encoded = base64_encode(input).unwrap();
        let decoded = base64_decode(&encoded).unwrap();
        assert_eq!(decoded, input);
    }

    // === Base64URL Encode ===

    #[test]
    fn url_encode_empty() {
        assert_eq!(base64url_encode("").unwrap(), "");
    }

    #[test]
    fn url_encode_no_padding() {
        // "A" in standard = "QQ==", in URL-safe = "QQ" (no padding)
        assert_eq!(base64url_encode("A").unwrap(), "QQ");
    }

    #[test]
    fn url_encode_uses_url_chars() {
        // Input that produces +/ in standard should produce -_ in URL-safe
        // bytes [0xFB, 0xFF, 0xBF] encode to "+/+/" in standard and "-_-_" in URL-safe
        // Use a string whose bytes trigger this: "\u{00FB}" is 0xC3 0xBB in UTF-8
        let standard = base64_encode("?>>?>>").unwrap();
        let url_safe = base64url_encode("?>>?>>").unwrap();
        // They should differ only in +/- and /_ characters
        assert!(!url_safe.contains('+'));
        assert!(!url_safe.contains('/'));
        assert_ne!(standard, url_safe);
    }

    // === Base64URL Decode ===

    #[test]
    fn url_decode_empty() {
        assert_eq!(base64url_decode("").unwrap(), "");
    }

    #[test]
    fn url_roundtrip() {
        let input = "Hello, World! \u{1F680}";
        let encoded = base64url_encode(input).unwrap();
        let decoded = base64url_decode(&encoded).unwrap();
        assert_eq!(decoded, input);
    }

    #[test]
    fn url_decode_with_padding() {
        // URL-safe decode should also accept padded input
        let encoded = base64url_encode("AB").unwrap();
        let padded = format!("{encoded}=");
        assert_eq!(base64url_decode(&padded).unwrap(), "AB");
    }

    // === Base64 Wrapped Encode ===

    #[test]
    fn wrapped_encode_short() {
        // Short input should not have line breaks
        let result = base64_encode_wrapped("Hello").unwrap();
        assert!(!result.contains('\n'));
        assert_eq!(result, "SGVsbG8=");
    }

    #[test]
    fn wrapped_encode_long() {
        // 57 bytes of input = 76 base64 chars = exactly one line, no wrap
        let input = "a".repeat(57);
        let result = base64_encode_wrapped(&input).unwrap();
        assert!(!result.contains('\n'));
        assert_eq!(result.len(), 76);
    }

    #[test]
    fn wrapped_encode_wraps_at_76() {
        // 58 bytes of input = 80 base64 chars = should wrap
        let input = "a".repeat(58);
        let result = base64_encode_wrapped(&input).unwrap();
        assert!(result.contains("\r\n"));
        let lines: Vec<&str> = result.split("\r\n").collect();
        assert_eq!(lines[0].len(), 76);
    }

    #[test]
    fn wrapped_roundtrip() {
        let input = "a".repeat(200);
        let wrapped = base64_encode_wrapped(&input).unwrap();
        // base64_decode strips whitespace, so roundtrip should work
        let decoded = base64_decode(&wrapped).unwrap();
        assert_eq!(decoded, input);
    }

    // === Size limits ===

    #[test]
    fn encode_input_too_large() {
        let big = "a".repeat(MAX_INPUT_BYTES + 1);
        let err = base64_encode(&big).unwrap_err();
        assert!(matches!(err, StringKnifeError::InputTooLarge { .. }));
    }

    #[test]
    fn decode_input_too_large() {
        let big = "a".repeat(MAX_INPUT_BYTES + 1);
        let err = base64_decode(&big).unwrap_err();
        assert!(matches!(err, StringKnifeError::InputTooLarge { .. }));
    }
}
