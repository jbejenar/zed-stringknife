//! T-422: No-panic tests for all decode/unescape paths.
//!
//! These tests exercise decode functions with adversarial inputs to verify
//! that no panics occur. All decode functions must return `Result`, never panic.
//!
//! This supplements (but does not replace) proper `cargo-fuzz` testing.
//! When `cargo-fuzz` is available, run:
//!   `cargo fuzz run fuzz_decode -- -max_total_time=1800`

use stringknife_core::transforms::{base64, escape, hex, html, json, jwt, unicode, url};

/// Adversarial byte patterns that commonly trigger panics in decoders.
const ADVERSARIAL_INPUTS: &[&str] = &[
    "",
    "\0",
    "\0\0\0\0",
    "\u{FF}",
    "\u{FEFF}",   // BOM
    "\u{FFFD}",   // replacement character
    "\u{10FFFF}", // max unicode
    "====",       // all padding
    "===",
    "==",
    "=",
    "%",
    "%%",
    "%zz",
    "%0",
    "%0g",
    "&#;",
    "&#x;",
    "&amp", // unterminated entity
    "\\u",
    "\\u{",
    "\\u{}",
    "\\u{FFFFFFFF}",
    "\\uD800",        // lone surrogate
    "\\uDFFF",        // lone surrogate
    "\\uD800\\uD800", // two high surrogates
    "\\",
    "\\\\\\",
    "\"",
    "a.b.c", // dots
    "a..b",
    "..",
    "...",
    "a]b[c",
    "{",
    "}",
    "{{}",
    "[]",
    "[[[",
    r#"{"a":}"#,
    r#"{"a":"#,
    "eyJ.eyJ.sig", // JWT-like
    "eyJ.sig",     // JWT-like with 1 dot
    ".....",
    "0x",
    "0xZZ",
    "GG",
    // Null bytes in middle
    "hello\0world",
    "SGVsbG8\0=",
    // Mixed valid/invalid
    "SGVsbG8=!!!",
    "48656c6c6fZZ",
    "%E4%B8%96%ZZ",
];

/// Additional dynamically-generated adversarial inputs.
fn dynamic_adversarial_inputs() -> Vec<String> {
    vec![
        "A".repeat(100),
        "=".repeat(100),
        "%20".repeat(100),
        "\\u0041".repeat(100),
    ]
}

/// Helper: call a decode fn with all adversarial inputs, assert no panic.
fn assert_no_panic(name: &str, f: fn(&str) -> Result<String, stringknife_core::StringKnifeError>) {
    for input in ADVERSARIAL_INPUTS {
        // We don't care about the result — only that it doesn't panic.
        let _result = f(input);
    }
    for input in &dynamic_adversarial_inputs() {
        let _result = f(input);
    }
    // Also test with a very long ASCII string (just under limit).
    let long = "A".repeat(1_000_000);
    let _result = f(&long);

    // And a string of all bytes 0x01..0x7F.
    let all_ascii: String = (1u8..128).map(|b| b as char).collect();
    let _result = f(&all_ascii);

    let _ = name; // used for diagnostics if needed
}

#[test]
fn base64_decode_no_panic() {
    assert_no_panic("base64_decode", base64::base64_decode);
}

#[test]
fn base64url_decode_no_panic() {
    assert_no_panic("base64url_decode", base64::base64url_decode);
}

#[test]
fn hex_decode_no_panic() {
    assert_no_panic("hex_decode", hex::hex_decode);
}

#[test]
fn url_decode_no_panic() {
    assert_no_panic("url_decode", url::url_decode);
}

#[test]
fn html_decode_no_panic() {
    assert_no_panic("html_decode", html::html_decode);
}

#[test]
fn unicode_unescape_no_panic() {
    assert_no_panic("unicode_unescape", unicode::unicode_unescape);
}

#[test]
fn json_unescape_no_panic() {
    assert_no_panic("json_unescape", json::json_unescape);
}

#[test]
fn unescape_backslashes_no_panic() {
    assert_no_panic("unescape_backslashes", escape::unescape_backslashes);
}

#[test]
fn jwt_decode_header_no_panic() {
    assert_no_panic("jwt_decode_header", jwt::jwt_decode_header);
}

#[test]
fn jwt_decode_payload_no_panic() {
    assert_no_panic("jwt_decode_payload", jwt::jwt_decode_payload);
}

#[test]
fn jwt_decode_full_no_panic() {
    assert_no_panic("jwt_decode_full", jwt::jwt_decode_full);
}

#[test]
fn json_pretty_print_no_panic() {
    assert_no_panic("json_pretty_print", json::json_pretty_print);
}

#[test]
fn json_minify_no_panic() {
    assert_no_panic("json_minify", json::json_minify);
}

/// Test that all encode functions also handle adversarial inputs without panic.
#[test]
fn encode_functions_no_panic() {
    let encode_fns: Vec<(
        &str,
        fn(&str) -> Result<String, stringknife_core::StringKnifeError>,
    )> = vec![
        ("base64_encode", base64::base64_encode),
        ("base64url_encode", base64::base64url_encode),
        ("hex_encode", hex::hex_encode),
        ("url_encode", url::url_encode),
        ("html_encode", html::html_encode),
        ("unicode_escape", unicode::unicode_escape),
        ("json_escape", json::json_escape),
        ("escape_backslashes", escape::escape_backslashes),
    ];

    for (name, f) in &encode_fns {
        for input in ADVERSARIAL_INPUTS {
            let _result = f(input);
        }
        let _ = name;
    }
}
