---
type: audit
audit_type: dependency
date: 2026-03-11
scope: stringknife-lsp (all transitive dependencies)
tags: [audit, dependency, phase-0]
---

# Dependency Audit #1

## Summary

| Metric | Value |
|--------|-------|
| Audit date | 2026-03-11 |
| Scope | `stringknife-lsp` and all transitive deps |
| Total unique crates | 79 |
| Budget limit (v1.0) | 150 |
| Headroom | 71 crates |
| `cargo deny check` | Pass (advisories ok, bans ok, licenses ok, sources ok) |
| `cargo audit` (CI) | Pass (no known vulnerabilities) |

## License Compliance

All 79 transitive dependencies use one of:
- MIT
- Apache-2.0
- MIT OR Apache-2.0
- Unicode-3.0

All licenses are compatible with the project's MIT license.

## Duplicate Crates

| Crate | Versions | Reason |
|-------|----------|--------|
| `bitflags` | 1.3.2, 2.11.0 | `lsp-types` depends on v1, `redox_syscall` (via `parking_lot_core` via `dashmap` via `tower-lsp`) depends on v2 |

This is a transitive duplicate and cannot be resolved without upgrading `tower-lsp` or `lsp-types`. Acceptable at this stage.

## Flagged Dependencies

### No Known CVEs

`cargo deny check advisories` and CI `cargo audit` both pass with zero advisories.

### Staleness Review

Major direct dependencies and their status:

| Dependency | Version | Last Release | Status |
|------------|---------|-------------|--------|
| `tower-lsp` | 0.20.0 | 2023-10 | Mature but not actively maintained. Watch for alternatives. |
| `tokio` | 1.43.0 | 2025-01 | Actively maintained |
| `serde` | 1.0.217 | 2025-01 | Actively maintained |
| `serde_json` | 1.0.138 | 2025-01 | Actively maintained |
| `zed_extension_api` | 0.7.0 | Active | Actively maintained by Zed team |

### Notes on `tower-lsp`

`tower-lsp` v0.20.0 is the latest release (Oct 2023). The crate has low maintenance activity. If it becomes abandoned:
- Alternative: `lsp-server` (rust-analyzer's LSP framework)
- Migration effort: Medium (different API surface, but same LSP protocol)
- No urgent action needed — it works correctly for our use case

## Recommendations

1. **No action needed** — all dependencies are clean
2. **Monitor** `tower-lsp` for maintenance status
3. **Budget tracking** — 79/150 crates used (53%), ample headroom for Phase 1-4 additions
4. Re-audit at Phase 2 exit when more transforms (and potential new deps) are added
