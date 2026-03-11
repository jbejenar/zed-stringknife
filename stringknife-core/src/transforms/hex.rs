//! Hex encoding and decoding transforms.

use crate::error::StringKnifeError;
use crate::MAX_INPUT_BYTES;

/// Encodes the input string's UTF-8 bytes as a lowercase hex string.
///
/// Each byte becomes two hex characters (e.g., `"A"` becomes `"41"`).
///
/// # Errors
///
/// Returns [`StringKnifeError::InputTooLarge`] if input exceeds [`MAX_INPUT_BYTES`].
pub fn hex_encode(input: &str) -> Result<String, StringKnifeError> {
    check_size(input)?;
    let mut result = String::with_capacity(input.len() * 2);
    for &byte in input.as_bytes() {
        result.push(hex_char(byte >> 4));
        result.push(hex_char(byte & 0x0F));
    }
    Ok(result)
}

/// Decodes a hex string back to UTF-8 text.
///
/// Accepts uppercase and lowercase hex digits. Ignores spaces and `0x` prefixes.
///
/// # Errors
///
/// Returns [`StringKnifeError::InvalidInput`] if the input contains invalid hex characters,
/// has an odd number of hex digits, or if the decoded bytes are not valid UTF-8.
pub fn hex_decode(input: &str) -> Result<String, StringKnifeError> {
    check_size(input)?;
    // Strip common prefixes and whitespace
    let clean: String = input
        .replace("0x", "")
        .replace("0X", "")
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();

    if !clean.len().is_multiple_of(2) {
        return Err(StringKnifeError::InvalidInput {
            operation: "hex_decode".to_string(),
            reason: "odd number of hex digits".to_string(),
        });
    }

    let mut bytes = Vec::with_capacity(clean.len() / 2);
    let chars: Vec<u8> = clean.bytes().collect();

    let mut i = 0;
    while i < chars.len() {
        let hi = from_hex(chars[i])?;
        let lo = from_hex(chars[i + 1])?;
        bytes.push((hi << 4) | lo);
        i += 2;
    }

    String::from_utf8(bytes).map_err(|_| StringKnifeError::InvalidInput {
        operation: "hex_decode".to_string(),
        reason: "decoded bytes are not valid UTF-8".to_string(),
    })
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

fn hex_char(nibble: u8) -> char {
    match nibble {
        0..=9 => char::from(b'0' + nibble),
        10..=15 => char::from(b'a' + nibble - 10),
        _ => unreachable!(),
    }
}

fn from_hex(byte: u8) -> Result<u8, StringKnifeError> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err(StringKnifeError::InvalidInput {
            operation: "hex_decode".to_string(),
            reason: format!("invalid hex character: '{}'", char::from(byte)),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Hex Encode ===

    #[test]
    fn encode_empty() {
        assert_eq!(hex_encode("").unwrap(), "");
    }

    #[test]
    fn encode_ascii() {
        assert_eq!(hex_encode("Hello").unwrap(), "48656c6c6f");
    }

    #[test]
    fn encode_single_byte() {
        assert_eq!(hex_encode("A").unwrap(), "41");
    }

    #[test]
    fn encode_unicode() {
        // "e\u{0301}" = é (combining), UTF-8: 65 CC 81
        assert_eq!(hex_encode("e\u{0301}").unwrap(), "65cc81");
    }

    // === Hex Decode ===

    #[test]
    fn decode_empty() {
        assert_eq!(hex_decode("").unwrap(), "");
    }

    #[test]
    fn decode_lowercase() {
        assert_eq!(hex_decode("48656c6c6f").unwrap(), "Hello");
    }

    #[test]
    fn decode_uppercase() {
        assert_eq!(hex_decode("48656C6C6F").unwrap(), "Hello");
    }

    #[test]
    fn decode_with_spaces() {
        assert_eq!(hex_decode("48 65 6c 6c 6f").unwrap(), "Hello");
    }

    #[test]
    fn decode_with_0x_prefix() {
        assert_eq!(hex_decode("0x48656c6c6f").unwrap(), "Hello");
    }

    #[test]
    fn decode_space_separated_0x() {
        assert_eq!(hex_decode("0x48 0x65 0x6c 0x6c 0x6f").unwrap(), "Hello");
    }

    #[test]
    fn decode_odd_digits() {
        let err = hex_decode("4865f").unwrap_err();
        assert!(matches!(err, StringKnifeError::InvalidInput { .. }));
    }

    #[test]
    fn decode_invalid_char() {
        let err = hex_decode("ZZZZ").unwrap_err();
        assert!(matches!(err, StringKnifeError::InvalidInput { .. }));
    }

    #[test]
    fn decode_invalid_utf8() {
        let err = hex_decode("FFFE").unwrap_err();
        assert!(matches!(err, StringKnifeError::InvalidInput { .. }));
    }

    // === Roundtrip ===

    #[test]
    fn roundtrip_ascii() {
        let input = "Hello, World!";
        let encoded = hex_encode(input).unwrap();
        let decoded = hex_decode(&encoded).unwrap();
        assert_eq!(decoded, input);
    }

    #[test]
    fn roundtrip_unicode() {
        let input = "cafe\u{0301} \u{1F680}";
        let encoded = hex_encode(input).unwrap();
        let decoded = hex_decode(&encoded).unwrap();
        assert_eq!(decoded, input);
    }

    // === Size limits ===

    #[test]
    fn encode_input_too_large() {
        let big = "a".repeat(MAX_INPUT_BYTES + 1);
        assert!(matches!(
            hex_encode(&big).unwrap_err(),
            StringKnifeError::InputTooLarge { .. }
        ));
    }

    #[test]
    fn decode_input_too_large() {
        let big = "a".repeat(MAX_INPUT_BYTES + 1);
        assert!(matches!(
            hex_decode(&big).unwrap_err(),
            StringKnifeError::InputTooLarge { .. }
        ));
    }
}
