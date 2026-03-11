---
type: audit
id: A-010
date: 2026-03-11
status: Partial (coverage pending)
tags: [audit, quality]
---

# Code Quality Audit #1 (A-010)

## Clippy

`cargo clippy -- -D warnings` passes with **zero warnings** across both crates.

## Test Coverage

**Blocked:** `cargo-tarpaulin` is not installed and only supports Linux.
Manual assessment: all 17 public transform functions have corresponding tests
(129 tests in stringknife-core). Every encode/decode pair has roundtrip tests,
edge case tests (empty string, Unicode, input too large), and error path tests.

**Estimated coverage of `transforms/`:** >90% (all branches tested, including
error paths, but measurement requires cargo-tarpaulin on Linux CI).

## Code Duplication

**Found and fixed:**
- `check_size()` was duplicated 6 times across transform modules
- Extracted to `transforms/common.rs` as shared helper
- All 6 modules now use `common::check_size()`

**Noted but not extracted (intentional):**
- `hex_char()` / `hex_digit()` — differ in case (lowercase in hex.rs, uppercase in url.rs)
- `from_hex()` / `from_hex_digit()` — identical logic but different error messages
- UTF-8 validation pattern — 4 occurrences, but each has a unique operation name

These are small, module-specific helpers where extraction would add indirection
without meaningful clarity improvement.

## Rustdoc Comments

All 17 public functions have complete `///` rustdoc comments including:
- Description of what the function does
- `# Errors` section listing possible error variants
- Parameter/return documentation where non-obvious

## Findings Summary

| Check | Status |
|-------|--------|
| Clippy zero warnings | Pass |
| Test coverage >= 80% | Estimated pass (measurement blocked) |
| Code duplication audit | Fixed (check_size extracted) |
| Public fn rustdoc | Pass |

## Action Items

- [ ] Set up cargo-tarpaulin in Linux CI for measured coverage
- [ ] Consider extracting `from_hex()` if a third module needs it
