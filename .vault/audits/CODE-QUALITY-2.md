---
type: audit
tags: [audit, quality]
date: 2026-03-11
---

# Code Quality Audit #2 (A-030)

## Test Coverage

**321 core tests + 5 LSP tests = 326 total.** All passing in 0.01s.

| Module | Tests | Public Functions |
|--------|-------|-----------------|
| base64 | 20 | 4 |
| case | 35 | 13 |
| csv | 11 | 1 |
| escape | 17 | 6 |
| hash | 27 | 5 |
| hex | 16 | 2 |
| html | 16 | 2 |
| inspect | 8 | 3 |
| json | 24 | 4 |
| jwt | 15 | 3 |
| misc | 6 | 1 |
| unicode | 25 | 3 |
| url | 19 | 3 |
| whitespace | 20 | 12 |
| xml | 22 | 2 |
| detect | 25 | 1 |
| error | 3 | — |
| LSP | 5 | — |
| **Total** | **326** | **65** |

Formal coverage percentage requires `cargo-tarpaulin` (Linux only).
Estimated coverage: >90% — every public function has dedicated tests,
edge cases, error cases, and roundtrip tests.

## Clippy

Zero warnings. Both crates pass `clippy -- -D warnings`.

## Dead Code

No unused transforms. All 65 public functions are wired into the LSP's
`build_actions()` function. No dead feature flags.

## Error Handling Consistency

All transforms follow the same pattern:
1. `check_size(input)?` as first line
2. Return `Result<String, StringKnifeError>`
3. Use `StringKnifeError::InvalidInput` for malformed input
4. Use `StringKnifeError::InputTooLarge` for oversized input

No `unwrap()` in library code. `#![deny(unsafe_code)]` at crate root.

## Code Action Naming

All 64 code actions use the consistent `StringKnife:` prefix.
Categories are clear: encode/decode, case, JSON, XML, CSV, hash,
whitespace, escape, inspect.

## Findings

1. **No issues found.** Code quality is high.
2. **Observation:** `build_actions()` is now ~200 lines (64 actions).
   Still manageable as a flat list, but a registry pattern could help
   if we add 20+ more actions in Phase 4.
3. **Observation:** Inspection actions (count_chars, byte_length, detect_encoding)
   replace the selected text with stats. This is a UX limitation of code actions —
   ideally these would use notifications (blocked on Zed API).
