//! Smart detection heuristics for identifying encoded content.
//!
//! These functions analyze selected text and return which decode operations
//! are likely relevant, enabling the LSP to surface the most useful actions first.

/// Detected encoding types, used to prioritise decode actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetectedEncoding {
    /// Text looks like Base64 (A-Za-z0-9+/= or URL-safe variant)
    Base64,
    /// Text contains percent-encoded sequences (%XX)
    UrlEncoded,
    /// Text contains HTML entities (&amp; &#123; &#xAB;)
    HtmlEntity,
    /// Text looks like a hex string (even-length hex chars, optional 0x prefix)
    Hex,
    /// Text contains Unicode escape sequences (\uXXXX or \U{XXXXXX})
    UnicodeEscape,
    /// Text looks like a JWT (three dot-separated `Base64URL` segments)
    Jwt,
}

/// Analyse the selected text and return all detected encoding types.
///
/// Multiple encodings may be detected simultaneously (e.g., a valid hex
/// string might also look like Base64). The caller should present decode
/// actions for all detected types.
#[must_use]
pub fn detect_encodings(input: &str) -> Vec<DetectedEncoding> {
    let mut detected = Vec::new();

    if looks_like_base64(input) {
        detected.push(DetectedEncoding::Base64);
    }
    if looks_like_url_encoded(input) {
        detected.push(DetectedEncoding::UrlEncoded);
    }
    if looks_like_html_entity(input) {
        detected.push(DetectedEncoding::HtmlEntity);
    }
    if looks_like_hex(input) {
        detected.push(DetectedEncoding::Hex);
    }
    if looks_like_unicode_escape(input) {
        detected.push(DetectedEncoding::UnicodeEscape);
    }
    if looks_like_jwt(input) {
        detected.push(DetectedEncoding::Jwt);
    }

    detected
}

/// Check if text looks like Base64-encoded content.
///
/// Heuristic: at least 4 chars, only valid Base64 charset (A-Za-z0-9+/=
/// or URL-safe A-Za-z0-9-_=), optional whitespace. Must not be trivially
/// plain text (requires at least one uppercase + one lowercase or digit).
fn looks_like_base64(input: &str) -> bool {
    let trimmed = input.trim();
    if trimmed.len() < 4 {
        return false;
    }

    let is_url_safe = trimmed.contains('-') || trimmed.contains('_');

    let valid = trimmed.chars().all(|c| {
        c.is_ascii_alphanumeric()
            || c == '+'
            || c == '/'
            || c == '='
            || c == '-'
            || c == '_'
            || c.is_ascii_whitespace()
    });

    if !valid {
        return false;
    }

    // Must have a mix suggesting encoding (not just a plain word)
    let has_upper = trimmed.chars().any(|c| c.is_ascii_uppercase());
    let has_lower = trimmed.chars().any(|c| c.is_ascii_lowercase());
    let has_digit = trimmed.chars().any(|c| c.is_ascii_digit());
    let has_padding = trimmed.contains('=');
    let has_special = trimmed.contains('+') || trimmed.contains('/') || is_url_safe;

    // Strong signals: has padding, has Base64-specific chars, or has mixed case/digits
    has_padding || has_special || (has_digit && (has_upper || has_lower))
}

/// Check if text contains percent-encoded sequences.
fn looks_like_url_encoded(input: &str) -> bool {
    let bytes = input.as_bytes();
    // Look for %XX patterns where X is a hex digit
    for i in 0..bytes.len().saturating_sub(2) {
        if bytes[i] == b'%' {
            let h1 = bytes.get(i + 1).copied().unwrap_or(0);
            let h2 = bytes.get(i + 2).copied().unwrap_or(0);
            if is_hex_digit(h1) && is_hex_digit(h2) {
                return true;
            }
        }
    }
    false
}

/// Check if text contains HTML entities.
fn looks_like_html_entity(input: &str) -> bool {
    // Look for &....; patterns (named, decimal, or hex numeric entities)
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'&' {
            // Check for named entity or numeric entity followed by ;
            if let Some(semi_pos) = input[i..].find(';') {
                let entity = &input[i + 1..i + semi_pos];
                if !entity.is_empty() && is_valid_entity_body(entity) {
                    return true;
                }
            }
        }
        i += 1;
    }
    false
}

/// Validate the body of an HTML entity (between & and ;).
fn is_valid_entity_body(body: &str) -> bool {
    if body.is_empty() {
        return false;
    }

    // Numeric decimal: #123
    if let Some(digits) = body.strip_prefix('#') {
        if let Some(hex_digits) = digits
            .strip_prefix('x')
            .or_else(|| digits.strip_prefix('X'))
        {
            // Hex numeric: #x7B
            return !hex_digits.is_empty() && hex_digits.chars().all(|c| c.is_ascii_hexdigit());
        }
        return !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit());
    }

    // Named entity: only ASCII alpha
    body.chars().all(|c| c.is_ascii_alphabetic())
}

/// Check if text looks like a hex-encoded string.
///
/// Heuristic: all chars are hex digits (optionally with 0x prefix, spaces,
/// or colon separators), even total hex-digit count, minimum 2 hex digits.
fn looks_like_hex(input: &str) -> bool {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return false;
    }

    // Strip optional 0x prefix
    let content = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
        .unwrap_or(trimmed);

    if content.is_empty() {
        return false;
    }

    // Count hex digits, allowing spaces and colons as separators
    let mut hex_count = 0;
    for c in content.chars() {
        if c.is_ascii_hexdigit() {
            hex_count += 1;
        } else if c == ' ' || c == ':' {
            // separator — ok
        } else {
            return false;
        }
    }

    // Need at least 2 hex digits, and an even count
    hex_count >= 2 && hex_count % 2 == 0
}

/// Check if text contains Unicode escape sequences.
fn looks_like_unicode_escape(input: &str) -> bool {
    // \uXXXX or \U{XXXXXX} or \u{XXXXXX}
    (input.contains("\\u") || input.contains("\\U"))
        && (has_backslash_u_escape(input) || has_backslash_u_brace_escape(input))
}

/// Check for \uXXXX pattern.
fn has_backslash_u_escape(input: &str) -> bool {
    let bytes = input.as_bytes();
    for i in 0..bytes.len().saturating_sub(5) {
        if bytes[i] == b'\\' && bytes[i + 1] == b'u' && bytes[i + 2] != b'{' {
            // Need exactly 4 hex digits
            let hex_part = &input[i + 2..];
            if hex_part.len() >= 4 && hex_part[..4].chars().all(|c| c.is_ascii_hexdigit()) {
                return true;
            }
        }
    }
    false
}

/// Check for \U{XXXXXX} pattern.
fn has_backslash_u_brace_escape(input: &str) -> bool {
    input.contains("\\U{") || input.contains("\\u{")
}

/// Check if text looks like a JWT (three dot-separated `Base64URL` segments).
fn looks_like_jwt(input: &str) -> bool {
    let trimmed = input.trim();
    let parts: Vec<&str> = trimmed.split('.').collect();
    if parts.len() != 3 {
        return false;
    }
    // Each part must be non-empty and contain only Base64URL chars
    parts.iter().all(|p| {
        !p.is_empty()
            && p.chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '=')
    })
}

/// Helper: is a byte an ASCII hex digit?
fn is_hex_digit(b: u8) -> bool {
    b.is_ascii_hexdigit()
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Base64 detection ---

    #[test]
    fn detect_base64_standard() {
        let detected = detect_encodings("SGVsbG8gV29ybGQ=");
        assert!(detected.contains(&DetectedEncoding::Base64));
    }

    #[test]
    fn detect_base64_url_safe() {
        let detected = detect_encodings("SGVsbG8t_29ybGQ");
        assert!(detected.contains(&DetectedEncoding::Base64));
    }

    #[test]
    fn detect_base64_not_short_word() {
        // Short plain word shouldn't trigger
        let detected = detect_encodings("Hi");
        assert!(!detected.contains(&DetectedEncoding::Base64));
    }

    #[test]
    fn detect_base64_plain_word_no_false_positive() {
        // A plain English word without mixed case+digits or special chars
        let detected = detect_encodings("hello");
        assert!(!detected.contains(&DetectedEncoding::Base64));
    }

    // --- URL encoding detection ---

    #[test]
    fn detect_url_encoded() {
        let detected = detect_encodings("hello%20world");
        assert!(detected.contains(&DetectedEncoding::UrlEncoded));
    }

    #[test]
    fn detect_url_encoded_special_chars() {
        let detected = detect_encodings("%3Cscript%3E");
        assert!(detected.contains(&DetectedEncoding::UrlEncoded));
    }

    #[test]
    fn detect_url_not_plain() {
        let detected = detect_encodings("hello world");
        assert!(!detected.contains(&DetectedEncoding::UrlEncoded));
    }

    #[test]
    fn detect_url_percent_but_not_hex() {
        // % followed by non-hex
        let detected = detect_encodings("100% natural");
        assert!(!detected.contains(&DetectedEncoding::UrlEncoded));
    }

    // --- HTML entity detection ---

    #[test]
    fn detect_html_named_entity() {
        let detected = detect_encodings("&amp; hello");
        assert!(detected.contains(&DetectedEncoding::HtmlEntity));
    }

    #[test]
    fn detect_html_decimal_entity() {
        let detected = detect_encodings("&#123;");
        assert!(detected.contains(&DetectedEncoding::HtmlEntity));
    }

    #[test]
    fn detect_html_hex_entity() {
        let detected = detect_encodings("&#x7B;");
        assert!(detected.contains(&DetectedEncoding::HtmlEntity));
    }

    #[test]
    fn detect_html_not_bare_ampersand() {
        let detected = detect_encodings("a & b");
        assert!(!detected.contains(&DetectedEncoding::HtmlEntity));
    }

    // --- Hex detection ---

    #[test]
    fn detect_hex_plain() {
        let detected = detect_encodings("48656c6c6f");
        assert!(detected.contains(&DetectedEncoding::Hex));
    }

    #[test]
    fn detect_hex_with_prefix() {
        let detected = detect_encodings("0x48656c6c6f");
        assert!(detected.contains(&DetectedEncoding::Hex));
    }

    #[test]
    fn detect_hex_space_separated() {
        let detected = detect_encodings("48 65 6c 6c 6f");
        assert!(detected.contains(&DetectedEncoding::Hex));
    }

    #[test]
    fn detect_hex_colon_separated() {
        let detected = detect_encodings("48:65:6c:6c:6f");
        assert!(detected.contains(&DetectedEncoding::Hex));
    }

    #[test]
    fn detect_hex_odd_count_not_detected() {
        let detected = detect_encodings("abc");
        assert!(!detected.contains(&DetectedEncoding::Hex));
    }

    // --- Unicode escape detection ---

    #[test]
    fn detect_unicode_escape_basic() {
        let detected = detect_encodings("\\u0048\\u0065\\u006C\\u006C\\u006F");
        assert!(detected.contains(&DetectedEncoding::UnicodeEscape));
    }

    #[test]
    fn detect_unicode_escape_brace() {
        let detected = detect_encodings("\\U{1F600}");
        assert!(detected.contains(&DetectedEncoding::UnicodeEscape));
    }

    #[test]
    fn detect_unicode_not_plain() {
        let detected = detect_encodings("hello");
        assert!(!detected.contains(&DetectedEncoding::UnicodeEscape));
    }

    // --- Multiple detection ---

    #[test]
    fn detect_multiple_encodings() {
        // A valid hex string that is also valid Base64
        let detected = detect_encodings("ABCDEF12");
        assert!(detected.contains(&DetectedEncoding::Hex));
        // It could also look like Base64 (mixed case + digits)
        assert!(detected.contains(&DetectedEncoding::Base64));
    }

    // --- JWT detection ---

    #[test]
    fn detect_jwt_token() {
        let token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U";
        let detected = detect_encodings(token);
        assert!(detected.contains(&DetectedEncoding::Jwt));
    }

    #[test]
    fn detect_jwt_not_two_dots() {
        let detected = detect_encodings("abc.def");
        assert!(!detected.contains(&DetectedEncoding::Jwt));
    }

    // --- Empty / edge cases ---

    #[test]
    fn detect_empty_string() {
        let detected = detect_encodings("");
        assert!(detected.is_empty());
    }

    #[test]
    fn detect_whitespace_only() {
        let detected = detect_encodings("   ");
        assert!(detected.is_empty());
    }
}
