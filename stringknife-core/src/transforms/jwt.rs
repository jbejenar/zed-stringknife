//! JWT (JSON Web Token) read-only decode transforms.
//!
//! Decodes JWT header, payload, and full structure without signature
//! verification. For debugging OAuth flows, API tokens, and auth issues.

use std::fmt::Write;

use super::common::check_size;
use crate::error::StringKnifeError;
use crate::transforms::base64;
use crate::transforms::json;

#[cfg(test)]
use crate::MAX_INPUT_BYTES;

/// Decodes the JWT header (first segment) and returns pretty-printed JSON.
///
/// # Errors
///
/// Returns [`StringKnifeError::InvalidInput`] if the input is not a valid JWT structure.
pub fn jwt_decode_header(input: &str) -> Result<String, StringKnifeError> {
    check_size(input)?;
    let parts = split_jwt(input)?;
    let decoded = decode_segment(parts[0], "header")?;
    json::json_pretty_print(&decoded)
}

/// Decodes the JWT payload (second segment) and returns pretty-printed JSON.
///
/// Timestamp fields (`exp`, `iat`, `nbf`) are annotated with human-readable dates.
///
/// # Errors
///
/// Returns [`StringKnifeError::InvalidInput`] if the input is not a valid JWT structure.
pub fn jwt_decode_payload(input: &str) -> Result<String, StringKnifeError> {
    check_size(input)?;
    let parts = split_jwt(input)?;
    let decoded = decode_segment(parts[1], "payload")?;
    let pretty = json::json_pretty_print(&decoded)?;
    Ok(annotate_timestamps(&pretty))
}

/// Decodes the full JWT (header + payload + signature hex) as formatted output.
///
/// # Errors
///
/// Returns [`StringKnifeError::InvalidInput`] if the input is not a valid JWT structure.
pub fn jwt_decode_full(input: &str) -> Result<String, StringKnifeError> {
    check_size(input)?;
    let parts = split_jwt(input)?;

    let header = decode_segment(parts[0], "header")?;
    let header_pretty = json::json_pretty_print(&header)?;

    let payload = decode_segment(parts[1], "payload")?;
    let payload_pretty = json::json_pretty_print(&payload)?;
    let payload_annotated = annotate_timestamps(&payload_pretty);

    // Signature: show as hex bytes
    let sig_bytes = base64::base64url_decode(parts[2]).unwrap_or_default();
    let sig_hex = sig_bytes.bytes().fold(String::new(), |mut acc, b| {
        let _ = write!(acc, "{b:02x}");
        acc
    });

    Ok(format!(
        "=== JWT Header ===\n{header_pretty}\n\n=== JWT Payload ===\n{payload_annotated}\n\n=== Signature ===\n{sig_hex}"
    ))
}

/// Split a JWT into exactly 3 dot-separated parts.
fn split_jwt(input: &str) -> Result<Vec<&str>, StringKnifeError> {
    let trimmed = input.trim();
    let parts: Vec<&str> = trimmed.split('.').collect();
    if parts.len() != 3 {
        return Err(StringKnifeError::InvalidInput {
            operation: "jwt_decode".to_string(),
            reason: format!("expected 3 dot-separated segments, found {}", parts.len()),
        });
    }
    if parts.iter().any(|p| p.is_empty()) {
        return Err(StringKnifeError::InvalidInput {
            operation: "jwt_decode".to_string(),
            reason: "JWT segment is empty".to_string(),
        });
    }
    Ok(parts)
}

/// Base64URL-decode a JWT segment and ensure it's valid UTF-8.
fn decode_segment(segment: &str, name: &str) -> Result<String, StringKnifeError> {
    base64::base64url_decode(segment).map_err(|_| StringKnifeError::InvalidInput {
        operation: "jwt_decode".to_string(),
        reason: format!("invalid Base64URL in {name} segment"),
    })
}

/// Annotate timestamp fields (exp, iat, nbf) with human-readable dates.
///
/// Looks for lines like `"exp": 1234567890` and appends a comment with the date.
fn annotate_timestamps(json: &str) -> String {
    let mut result = String::with_capacity(json.len());
    for line in json.lines() {
        result.push_str(line);
        // Check if this line has a timestamp field
        if let Some(ts) = extract_timestamp_field(line) {
            let date_str = unix_timestamp_to_string(ts);
            let _ = write!(result, "  // {date_str}");
        }
        result.push('\n');
    }
    // Remove trailing newline
    if result.ends_with('\n') {
        result.pop();
    }
    result
}

/// Extract a timestamp value from a JSON line like `  "exp": 1234567890`
fn extract_timestamp_field(line: &str) -> Option<i64> {
    let trimmed = line.trim().trim_end_matches(',');
    for key in &["\"exp\"", "\"iat\"", "\"nbf\""] {
        if let Some(rest) = trimmed.strip_prefix(key) {
            let value_str = rest.trim().strip_prefix(':')?.trim();
            return value_str.parse::<i64>().ok();
        }
    }
    None
}

/// Convert a Unix timestamp to a human-readable UTC date string.
///
/// Format: `YYYY-MM-DD HH:MM:SS UTC`
fn unix_timestamp_to_string(ts: i64) -> String {
    // Simple conversion without external deps
    // Days since epoch -> year/month/day using a civil calendar algorithm
    let secs_per_day: i64 = 86400;
    let total_secs = ts;
    let day_seconds = total_secs.rem_euclid(secs_per_day);
    let hours = day_seconds / 3600;
    let minutes = (day_seconds % 3600) / 60;
    let seconds = day_seconds % 60;

    let mut days = total_secs.div_euclid(secs_per_day);
    // Shift to March 1, 2000 epoch for easier month calculation
    days += 719_468; // days from 0000-03-01 to 1970-01-01

    let era = if days >= 0 {
        days / 146_097
    } else {
        (days - 146_096) / 146_097
    };
    let doe = days - era * 146_097; // day of era [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if m <= 2 { y + 1 } else { y };

    format!("{year:04}-{m:02}-{d:02} {hours:02}:{minutes:02}:{seconds:02} UTC")
}

#[cfg(test)]
mod tests {
    use super::*;

    // Sample JWT tokens for testing (these are well-known test tokens, not secrets)

    // HS256 token: {"alg":"HS256","typ":"JWT"}.{"sub":"1234567890","name":"John Doe","iat":1516239022}
    const HS256_TOKEN: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";

    // RS256 token: {"alg":"RS256","typ":"JWT"}.{"sub":"1234567890","name":"Jane Doe","admin":true,"iat":1516239022}
    const RS256_TOKEN: &str = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkphbmUgRG9lIiwiYWRtaW4iOnRydWUsImlhdCI6MTUxNjIzOTAyMn0.POstGetfAytaZS82wHcjoTyoqhMyxXiWdR7Nn7A29DNSl0EiXLdwJ6xC6AfgZWF1bOsS_TuYI3OG85AmiExREkrS6tDfTQ2B3WXlrr-wp5AokiRbz3_oB4OxG-W9KcEEbDRcZc0nH3L7LzYptiy1PtAylQGxHTWZXtGz4ht0bAecBgmpdgXMguEIcoqPJ1n3pIWk_dUZegpqx0Lka21H6XxUTxiy8OcaarA8zdnPUnV6AmNP3ecFawIFYdvJB_cm-GvpCSbr8G8y_Mllj8f4x9nBH8pQux89_6gUY618iYv7tuPWBFfEbLxtF2pZS6YC1aSfLQxaOoaBSTN2Pew";

    // Token with exp field: {"alg":"HS256"}.{"sub":"user","exp":1893456000,"iat":1609459200}
    const EXPIRED_TOKEN: &str = "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJ1c2VyIiwiZXhwIjoxODkzNDU2MDAwLCJpYXQiOjE2MDk0NTkyMDB9.signature";

    // === JWT Decode Header ===

    #[test]
    fn decode_header_hs256() {
        let result = jwt_decode_header(HS256_TOKEN).unwrap();
        assert!(result.contains("\"alg\""));
        assert!(result.contains("HS256"));
        assert!(result.contains("\"typ\""));
        assert!(result.contains("JWT"));
    }

    #[test]
    fn decode_header_rs256() {
        let result = jwt_decode_header(RS256_TOKEN).unwrap();
        assert!(result.contains("RS256"));
    }

    // === JWT Decode Payload ===

    #[test]
    fn decode_payload_hs256() {
        let result = jwt_decode_payload(HS256_TOKEN).unwrap();
        assert!(result.contains("\"sub\""));
        assert!(result.contains("1234567890"));
        assert!(result.contains("\"name\""));
        assert!(result.contains("John Doe"));
    }

    #[test]
    fn decode_payload_rs256() {
        let result = jwt_decode_payload(RS256_TOKEN).unwrap();
        assert!(result.contains("Jane Doe"));
        assert!(result.contains("\"admin\""));
        assert!(result.contains("true"));
    }

    #[test]
    fn decode_payload_timestamp_annotation() {
        let result = jwt_decode_payload(HS256_TOKEN).unwrap();
        // iat: 1516239022 should be annotated with a date
        assert!(result.contains("// "));
        assert!(result.contains("2018"));
    }

    #[test]
    fn decode_payload_exp_timestamp() {
        let result = jwt_decode_payload(EXPIRED_TOKEN).unwrap();
        // exp: 1893456000 = 2030-01-01
        assert!(result.contains("2030"));
        // iat: 1609459200 = 2021-01-01
        assert!(result.contains("2021"));
    }

    // === JWT Decode Full ===

    #[test]
    fn decode_full_structure() {
        let result = jwt_decode_full(HS256_TOKEN).unwrap();
        assert!(result.contains("=== JWT Header ==="));
        assert!(result.contains("=== JWT Payload ==="));
        assert!(result.contains("=== Signature ==="));
        assert!(result.contains("HS256"));
        assert!(result.contains("John Doe"));
    }

    // === Error handling ===

    #[test]
    fn decode_malformed_no_dots() {
        let err = jwt_decode_header("notajwt").unwrap_err();
        assert!(matches!(err, StringKnifeError::InvalidInput { .. }));
    }

    #[test]
    fn decode_malformed_one_dot() {
        let err = jwt_decode_header("part1.part2").unwrap_err();
        assert!(matches!(err, StringKnifeError::InvalidInput { .. }));
    }

    #[test]
    fn decode_malformed_four_dots() {
        let err = jwt_decode_header("a.b.c.d").unwrap_err();
        assert!(matches!(err, StringKnifeError::InvalidInput { .. }));
    }

    #[test]
    fn decode_malformed_empty_segment() {
        let err = jwt_decode_header("a..c").unwrap_err();
        assert!(matches!(err, StringKnifeError::InvalidInput { .. }));
    }

    #[test]
    fn decode_malformed_invalid_base64() {
        let err = jwt_decode_header("!!!.@@@.###").unwrap_err();
        assert!(matches!(err, StringKnifeError::InvalidInput { .. }));
    }

    // === Input too large ===

    #[test]
    fn decode_input_too_large() {
        let big = "a".repeat(MAX_INPUT_BYTES + 1);
        assert!(matches!(
            jwt_decode_header(&big).unwrap_err(),
            StringKnifeError::InputTooLarge { .. }
        ));
    }

    // === Timestamp conversion ===

    #[test]
    fn timestamp_epoch() {
        let result = unix_timestamp_to_string(0);
        assert_eq!(result, "1970-01-01 00:00:00 UTC");
    }

    #[test]
    fn timestamp_known_date() {
        // 2024-01-01 00:00:00 UTC = 1704067200
        let result = unix_timestamp_to_string(1_704_067_200);
        assert_eq!(result, "2024-01-01 00:00:00 UTC");
    }
}
