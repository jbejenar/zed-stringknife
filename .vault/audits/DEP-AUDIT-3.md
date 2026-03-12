---
type: audit
audit_id: DEP-AUDIT-3
phase: 4
date: 2026-03-12
tags: [audit, dependencies]
---

# Dependency Audit #3 (A-041)

## Transitive Dependency Count

**83 crates** (budget: < 150)

Budget utilization: 55% — substantial headroom remaining.

## Direct Dependencies

### stringknife-core (Layer 3)
- **Zero production dependencies** (confirmed)
- Dev-only: `criterion = "0.5"` (benchmarks)

### stringknife-lsp (Layer 2)
| Crate | Version | Purpose |
|-------|---------|---------|
| `stringknife-core` | 0.0.1 (path) | Transform engine |
| `tower-lsp` | 0.20.0 | LSP framework |
| `tokio` | 1.43.0 | Async runtime (macros, rt-multi-thread, io-std, time) |
| `serde` | 1.0.217 | Config deserialization |
| `serde_json` | 1.0.138 | JSON parsing |
| `tracing` | 0.1.44 | Structured logging |
| `tracing-subscriber` | 0.3.19 | Log output formatting (fmt, registry) |

**7 direct dependencies** (1 local, 6 external)

## New Crates Since Last Audit

| Crate | Type | Source | Justification |
|-------|------|--------|---------------|
| `tracing` | Direct | Was already transitive via tower-lsp | Made explicit for structured logging (T-423) |
| `tracing-subscriber` | Direct | New | Log output to stderr with reloadable filter (T-423/T-424) |
| `lazy_static` | Transitive | Via tracing-subscriber | Required by tracing internals |
| `sharded-slab` | Transitive | Via tracing-subscriber | Subscriber registry slab allocator |
| `thread_local` | Transitive | Via tracing-subscriber | Thread-local subscriber storage |

**Net new crates: 4** (tracing-subscriber, lazy_static, sharded-slab, thread_local)
`tracing` was already in the tree via `tower-lsp`.

## Tracing Justification

`tracing` + `tracing-subscriber` were chosen over simpler alternatives because:

1. `tracing` is already a transitive dependency (zero cost to make direct)
2. Structured fields (operation, input_size, duration_ms) are first-class
3. `tracing-subscriber::reload` enables runtime log level changes without restart (T-424)
4. Industry-standard Rust logging — familiar to contributors
5. Only 4 net new crates, minimal `default-features = false`

Alternative considered: manual `Client::log_message()` calls. Rejected because it lacks structured fields, filtering, and stderr output for debugging.

## cargo deny

**Not available in this environment.** Should be run in CI.

## Summary

| Metric | Value | Gate | Status |
|--------|-------|------|--------|
| Transitive crates | 83 | < 150 | PASS |
| Core prod deps | 0 | = 0 | PASS |
| Net new crates | 4 | Justified | PASS |
| cargo deny | N/A | CI gate | DEFERRED |

**AUDIT RESULT: PASS**
