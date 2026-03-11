---
type: pattern
tags: [pattern]
---

# Dependency Budget

## Hard Rules

- **< 150 transitive crates** at v1.0 (across all workspace crates)
- **No wildcard versions**: Pin all versions in `Cargo.toml`
- **New deps require justification** in PR description
- **`cargo-deny` must pass**: license + advisory checks

## License Allowlist

| License | Status |
|---------|--------|
| MIT | Allowed |
| Apache-2.0 | Allowed |
| BSD-2-Clause | Allowed |
| BSD-3-Clause | Allowed |
| ISC | Allowed |
| Zlib | Allowed |

Anything else requires explicit approval from the product owner.

## Approved Libraries

| Crate | Purpose | Budget Impact |
|-------|---------|--------------|
| `zed_extension_api` | Zed extension WASM API | WASM crate only |
| `tower-lsp` | LSP protocol implementation | LSP crate |
| `tokio` | Async runtime for LSP | LSP crate |
| `serde` + `serde_json` | Serialization | Shared |

## Process for Adding New Dependencies

1. Check if the functionality can be implemented without a dep
2. Check the crate's license is on the allowlist
3. Check transitive dep count impact: `cargo tree -p <crate>`
4. Run `cargo deny check` to verify
5. Document justification in PR description
