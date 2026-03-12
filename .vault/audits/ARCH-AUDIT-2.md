---
type: audit
audit_id: ARCH-AUDIT-2
phase: 4
date: 2026-03-12
tags: [audit, architecture]
---

# Architecture Audit #2 (A-040)

## Configuration Plumbing

**Status: CLEAN**

- Config defined in `config.rs` with clean separation: `Config`, `HashFormat`, `LogLevel`
- Flow: `initializationOptions` → deserialize → `RwLock<Config>` → cloned per `code_action()`
- Live reload via `workspace/didChangeConfiguration` — includes log level updates
- All fields have sensible defaults via `impl Default`
- No spaghetti: config is immutable after deserialization, used only by `build_actions()` for filtering

## Memory Under Sustained Operation

**Status: CLEAN**

- `DocumentStore` uses `Arc<String>` — `get_text()` clones Arc (O(1)), not the document
- Documents properly removed on `didClose`
- `build_actions()` is completely stateless: fresh Vecs per call, no global accumulation
- `sustained_build_actions_no_accumulation` test runs 1000 iterations successfully
- `document_store_churn` test creates/removes 100 × 100KB documents — no leaks
- Only static resource: `LOG_RELOAD_HANDLE` for tracing reload (non-accumulating)

## LSP Lifecycle

**Status: CLEAN**

- `initialize()`: reads config, sets log level, returns capabilities
- `shutdown()`: no-op (correct for stateless server)
- `didOpen`/`didClose`: properly inserts/removes documents from store
- `spawn_blocking` tasks are awaited with timeout; no orphan tasks possible
- No dangling references: all Arc handles scoped to single request

## Performance Contract

**Status: MET**

- Size check at line 177 happens BEFORE `spawn_blocking` — no work for oversized input
- `build_actions()` runs in tokio blocking pool with 5s timeout
- 10 sequential code action computations complete well under 5s (test verified)
- Benchmark suite (criterion) available via `make bench` for 1KB/10KB/100KB/1MB inputs
- `InputTooLarge` returns immediately for > 1MB selections

## Layer Boundaries

**Status: PERFECT ISOLATION**

- `stringknife-core`: zero production dependencies, `#![deny(unsafe_code)]`
- All transforms: `fn(&str) -> Result<String, StringKnifeError>` — no IO, no side effects
- No imports from LSP → core direction
- Config filtering happens in LSP layer only

## Summary

| Area | Finding | Risk |
|------|---------|------|
| Config plumbing | Clean, tested, live-reloadable | None |
| Memory | Stateless, Arc-based, no accumulation | None |
| LSP lifecycle | Clean init/shutdown, proper doc cleanup | None |
| Performance | Size check first, timeout protection, benchmarks available | None |
| Layer boundaries | Core pure, zero deps, no unsafe | None |

**AUDIT RESULT: PASS**
