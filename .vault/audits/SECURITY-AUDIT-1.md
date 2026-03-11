---
type: audit
tags: [audit, security]
date: 2026-03-11
---

# Security Audit #1 (A-021)

## cargo deny check advisories

```
stringknife-core: advisories ok
stringknife-lsp:  advisories ok
```

Zero advisories.

## cargo audit

Not installed (`cargo-audit` not available). Covered by `cargo deny` above.

## Unsafe Code

```
$ grep -r "unsafe" stringknife-core/src/transforms/
(zero matches)
```

`#![deny(unsafe_code)]` is enforced at the crate root (`lib.rs:8`).

## Hash Implementation Review

All hash algorithms (MD5, SHA-1, SHA-256, SHA-512, CRC32) are implemented
from scratch in `transforms/hash.rs`. No external crate dependencies.

- Verified against NIST/RFC test vectors (27 tests)
- MD5: RFC 1321 test suite (empty, "a", "abc", "message digest", alphabet)
- SHA-1: FIPS 180-4 test vectors
- SHA-256: FIPS 180-4 test vectors
- SHA-512: FIPS 180-4 test vectors
- CRC32: ISO 3309 check value ("123456789" → 0xCBF43926)

Note: These are for display/checksum purposes only, not cryptographic security.
The extension description should clarify this if users might assume otherwise.

## Supply Chain

stringknife-core has **zero dependencies** — no supply chain risk.
stringknife-lsp depends on well-established crates (serde, tokio, tower-lsp)
with millions of downloads.

## Input Validation

- All transforms enforce `MAX_INPUT_BYTES` (1MB) via `check_size()`
- Invalid input returns `StringKnifeError`, never panics
- No `unwrap()` in library code (enforced by convention + review)

## Fuzz Testing

Deferred — requires `cargo-fuzz` setup (needs nightly toolchain + Linux CI).
Recommended targets: base64_decode, url_decode, json_pretty_print, xml_pretty_print.

## Findings

1. **No security issues found.**
2. **Recommendation:** Add fuzz testing in CI when nightly toolchain is available.
3. **Recommendation:** Document that hash functions are for display, not crypto.
