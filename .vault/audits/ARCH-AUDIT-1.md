---
type: audit
tags: [audit, architecture]
date: 2026-03-11
---

# Architecture Audit #1 (A-020)

## Module Boundaries

**Verdict: Clean separation.**

- `stringknife-core` has **zero dependencies** (`cargo tree` shows no children)
- `#![deny(unsafe_code)]` enforced at crate root
- No LSP types in transforms/ (`grep` for `tower_lsp|lsp_types` returns zero matches)
- All transforms are pure `fn(&str) -> Result<String, StringKnifeError>` — no I/O, no state

## LSP Handler Analysis

**Verdict: Thin dispatch layer.**

`build_actions()` in `main.rs` is a flat list of `try_decode`/`try_encode` calls.
The only logic is:
1. Smart detection (`detect_encodings`) to determine which decode actions to show
2. Building `CodeAction` objects from transform results

No business logic has leaked into the LSP layer. The `extract_range` and
`char_offset_to_byte` helpers are LSP-specific concerns (UTF-16 offset handling).

## Performance Profile

All 239 tests complete in **0.01s** (10,000+ operations including 10KB inputs).
Individual operation latency is sub-millisecond for typical input sizes.

The test suite exercises all operations with realistic inputs, and the entire
suite finishes in ~10ms, well under the 50ms/operation target.

## Dependency Tree

| Crate | Direct Deps | Transitive Deps |
|-------|------------|-----------------|
| stringknife-core | 0 | 0 |
| stringknife-lsp | 4 (serde, serde_json, tokio, tower-lsp) | ~79 |

No new dependencies were added in Phase 2. All hash, JWT, JSON, XML, and CSV
operations are implemented from scratch with zero external dependencies.

## Standalone Crate Assessment

`stringknife-core` could be published as a standalone crate today:
- Zero dependencies
- Public API is stable (`fn(&str) -> Result<String, StringKnifeError>`)
- All modules are `pub`
- Comprehensive test coverage (239 tests)
- No LSP coupling

Potential crate name: `stringknife` or `string-transforms`.

## Transform Module Inventory

| Module | Functions | Tests |
|--------|----------|-------|
| base64 | 4 (encode, decode, url_encode, url_decode) | 20 |
| csv | 1 (csv_to_json) | 11 |
| hash | 5 (md5, sha1, sha256, sha512, crc32) | 27 |
| hex | 2 (encode, decode) | 16 |
| html | 2 (encode, decode) | 16 |
| json | 4 (pretty_print, minify, escape, unescape) | 24 |
| jwt | 3 (decode_header, decode_payload, decode_full) | 15 |
| misc | 1 (reverse_string) | 6 |
| unicode | 3 (escape, unescape, show_codepoints) | 25 |
| url | 3 (encode, decode, encode_component) | 19 |
| xml | 2 (pretty_print, minify) | 22 |
| detect | 1 (detect_encodings) | 25 |
| common | 1 (check_size) | — |
| **Total** | **32 functions** | **226 tests** |

Plus 13 LSP-specific tests = **239 total**.

## Findings

1. **No issues found.** Architecture is clean and well-separated.
2. **Observation:** `build_actions()` is growing (currently ~150 lines). As more
   transforms are added, consider a registry pattern. Not urgent — the flat list
   is still readable and maintainable.
3. **Observation:** The `common` module is `pub(crate)` — correct, since `check_size`
   is an internal utility, not a public API.
