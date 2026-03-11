# Zed StringKnife — Technical Architecture Reference (Archived)

> This document was extracted from the main roadmap to reduce line count.
> It contains the full technical architecture, dependency budget, performance model,
> security model, ARI integration, PM governance, audit schedule, and CI gate policy.
> Active roadmap: `roadmap/roadmap.md`

---

## Technical Architecture

### Why LSP? The Architecture Decision Record

Zed's extension API (`zed_extension_api` v0.7.x) supports themes, languages, slash commands, MCP servers, debuggers, and icon themes — but **does not expose direct editor text manipulation**. There is no `editor.replaceSelection()`, no code action registration, no context menu hook available to extension authors via the WASM API. This is a known limitation with active community discussion but no current resolution.

The only surface in Zed that provides right-click context menu integration with text replacement capabilities is the **Language Server Protocol**. Specifically, `textDocument/codeAction` responses appear in Zed's context menu when text is selected, and `WorkspaceEdit` payloads can replace that selection. This is the architectural bet: we build a custom LSP that requires zero semantic analysis — it receives selected text, transforms it, and returns the result.

**Alternatives considered and rejected:**

| Approach | Why Rejected |
|----------|-------------|
| Slash Commands | Only available in the Assistant panel. Cannot modify editor text. |
| MCP Server | Designed for AI context injection, not editor text manipulation. |
| Tasks + CLI | Works but no context menu. User must configure tasks manually. Poor discoverability. |
| Wait for Editor API | Indefinite timeline. Zed's extension API roadmap does not commit to text manipulation hooks. |
| Fork Zed / Contribute upstream | Disproportionate effort for a utility extension. Not sustainable for a side project. |

**Decision:** Ship as a Zed extension (Rust WASM) that bundles and manages a custom Language Server binary. The LSP registers against broad file types so code actions are universally available regardless of what the user is editing.

---

### System Context

```
┌──────────────────────────────────────────────────────────────────┐
│                         Zed Editor                               │
│                                                                  │
│  ┌─────────────┐    ┌──────────────────────────────────────────┐ │
│  │   Editor     │    │        Extension Host (WASM Sandbox)     │ │
│  │   Buffer     │    │  ┌────────────────────────────────────┐  │ │
│  │             │    │  │   stringknife extension (WASM)     │  │ │
│  │  [selected  │    │  │                                    │  │ │
│  │   text]     │    │  │  • language_server_command()        │  │ │
│  │             │    │  │  • language_server_init_options()   │  │ │
│  │             │    │  │  • Downloads/locates LSP binary    │  │ │
│  └──────┬──────┘    │  └──────────────┬─────────────────────┘  │ │
│         │           └─────────────────┼────────────────────────┘ │
│         │ LSP Protocol (stdio)        │ manages lifecycle        │
│         │                             │                          │
│  ┌──────▼─────────────────────────────▼──────────────────────┐   │
│  │              stringknife-lsp (native binary)              │   │
│  │                                                           │   │
│  │  ┌─────────────────┐    ┌──────────────────────────────┐  │   │
│  │  │  LSP Protocol    │    │     Transform Engine         │  │   │
│  │  │  Handler         │    │                              │  │   │
│  │  │                  │    │  transforms/base64.rs        │  │   │
│  │  │  • initialize    │───▶│  transforms/url.rs           │  │   │
│  │  │  • codeAction    │    │  transforms/html.rs          │  │   │
│  │  │  • didOpen       │◀───│  transforms/hex.rs           │  │   │
│  │  │  • didChange     │    │  transforms/case.rs          │  │   │
│  │  │  • shutdown      │    │  transforms/json.rs          │  │   │
│  │  │                  │    │  transforms/hash.rs          │  │   │
│  │  └─────────────────┘    │  transforms/jwt.rs           │  │   │
│  │                          │  transforms/escape.rs        │  │   │
│  │  ┌─────────────────┐    │  transforms/...              │  │   │
│  │  │  Document Store  │    │                              │  │   │
│  │  │  HashMap<Url,    │    │  fn(input: &str)             │  │   │
│  │  │    String>       │    │    -> Result<String,         │  │   │
│  │  └─────────────────┘    │         StringKnifeError>     │  │   │
│  │                          └──────────────────────────────┘  │   │
│  └───────────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────────┘
```

---

### Component Architecture

**Layer 1: Zed Extension (WASM)** — `src/lib.rs`

The thinnest possible shim. Lifecycle management of the LSP binary only.

**Layer 2: LSP Server** — `lsp/src/`

A thin dispatch layer that speaks LSP over stdio. Maintains document text state and dispatches to the Transform Engine.

| Component | File | Responsibility |
|-----------|------|----------------|
| Server bootstrap | `main.rs` | Tokio runtime, tower-lsp setup, stdio transport |
| Protocol handlers | `handlers.rs` | `initialize`, `didOpen`, `didChange`, `codeAction`, `shutdown` |
| Document store | `document_store.rs` | `HashMap<Url, String>` — full text sync |
| Action builder | `actions.rs` | Builds `CodeAction` + `WorkspaceEdit` from transform results |
| Smart detection | `detection.rs` | Pattern matching to suggest relevant decode operations |
| Configuration | `config.rs` | Deserialise `initializationOptions`, handle `didChangeConfiguration` |
| Error mapping | `error.rs` | Maps `StringKnifeError` -> LSP diagnostics / `window/showMessage` |

**Layer 3: Transform Engine** — `transforms/`

Pure functions with zero dependencies on LSP types, I/O, or side effects:

```rust
pub fn transform_name(input: &str) -> Result<String, StringKnifeError>
```

---

### Data Flow: Code Action Request

```
User selects text "SGVsbG8gV29ybGQ=" in editor
                    │
                    ▼
    Zed sends textDocument/codeAction
                    │ stdio (JSON-RPC)
                    ▼
    LSP Handler: codeAction()
    1. Look up document text from DocumentStore
    2. Extract selected text using range coordinates
    3. Run smart detection on selected text
    4. Build list of applicable CodeActions
                    │
                    ▼
    Smart Detection → [Base64Decode] + [all encode actions]
                    │
                    ▼
    Response: CodeAction list (detected first, then all encodes)
                    │
                    ▼
    User selects action → Transform Engine executes
                    │
                    ▼
    WorkspaceEdit replaces selection (Undo-able via Cmd+Z)
```

---

### Repository Structure

```
zed-stringknife/
├── extension.toml              # Zed extension manifest
├── Cargo.toml                  # Workspace root
├── Cargo.lock                  # Committed for build determinism
├── rust-toolchain.toml         # Pins stable Rust channel
├── deny.toml                   # cargo-deny configuration
├── src/lib.rs                  # WASM extension shim (Layer 1)
├── lsp/src/                    # LSP server (Layer 2)
├── transforms/src/             # Transform engine (Layer 3)
├── CLAUDE.md                   # Agent entry point
├── .vault/                     # Obsidian-compatible knowledge vault
└── .github/workflows/          # CI/CD pipelines
```

---

### Key Design Principles

1. **Pure function supremacy.** Every transform is `fn(&str) -> Result<String, StringKnifeError>`.
2. **The LSP is a router, not a processor.**
3. **Detection is heuristic, not authoritative.**
4. **Errors are values, not exceptions.**
5. **Cross-platform is a constraint, not a feature.**
6. **Zero network, zero telemetry.**

---

### Dependency Budget

| Crate | Purpose | Layer |
|-------|---------|-------|
| `zed_extension_api` | Zed WASM extension trait | root |
| `tower-lsp` | LSP protocol implementation | lsp |
| `tokio` | Async runtime for LSP | lsp |
| `serde` + `serde_json` | JSON serialization | lsp, transforms |
| `tracing` | Structured logging | lsp |
| `base64` | Base64 encode/decode | transforms |
| `percent-encoding` | URL encode/decode | transforms |
| `sha2` + `md-5` | SHA and MD5 hashing | transforms |
| `crc32fast` | CRC32 checksum | transforms |
| `uuid` | UUID generation | transforms |
| `flate2` | Gzip/Deflate compression | transforms |
| `similar` | Text diffing | transforms |
| `chrono` | Timestamp operations | transforms |

**Hard rules:** No `unsafe` in `transforms/`. No system library links. < 150 transitive crates at v1.0.

---

### Performance Model

Every code action response must complete in under **100ms for 100KB input**. Selections > 1MB return `InputTooLarge` error.

### Security Model

No network access, no file system writes/reads beyond LSP protocol, no code execution, no `unsafe` Rust, no credential handling. All decode operations validate input before transformation.

---

## AI-Agent Readiness: `ariscan` Integration

| # | Pillar | v1.0 Target | Strategy |
|---|--------|-------------|----------|
| 1 | Test Isolation | >= 9 | Pure function transforms |
| 2 | Build Determinism | >= 9 | `rust-toolchain.toml` + `Cargo.lock` |
| 3 | Type Safety | >= 9 | Rust + strict clippy |
| 4 | Modular Coherence | >= 9 | Standalone pure functions per module |
| 5 | Documentation Density | >= 8 | `.vault/` + rustdoc |
| 6 | Dependency Transparency | >= 9 | Minimal deps + `cargo-deny` |
| 7 | Error Explicitness | >= 9 | `Result<T, E>` everywhere |
| 8 | Security (Gate) | Pass | `cargo-audit` + no `unsafe` |

---

## PM Review Cadence

| Review | When | Scope |
|--------|------|-------|
| PMR-0 | End of Phase 0 | Validate architecture bet |
| PMR-1 | Mid-Phase 1 | MVP scope review |
| PMR-2 | End of Phase 2 | Feature velocity check |
| PMR-3 | End of Phase 4 | Pre-launch scope lock |
| PMR-4 | 2 weeks post-publish | Post-launch retrospective |
| PMR-5 | Mid-Phase 6 | v1.0 readiness |

---

## Audit Schedule

| Audit | Focus | Cadence |
|-------|-------|---------|
| Code Quality | Clippy, dead code, coverage | Every 2 phases |
| Security | cargo-audit, fuzzing, unsafe | Every 2 phases + pre-publish |
| Architecture | Module coherence, performance | Phase 2 and Phase 4 |
| Dependency | Transitive deps, licenses | Every phase |
| UX | Discoverability, error messages | Phase 3 and pre-publish |

---

## PR & CI Gate Policy

### Required CI Checks

| Check | Command | Blocking |
|-------|---------|----------|
| Build (WASM) | `cargo check -p stringknife-ext --target wasm32-wasip1` | Yes |
| Build (LSP) | `cargo check -p stringknife-lsp` | Yes |
| Unit Tests | `cargo test --workspace` | Yes |
| Lint (Clippy) | `cargo clippy --workspace -- -D warnings` | Yes |
| Format | `cargo fmt --all -- --check` | Yes |
| License/Advisory | `cargo deny check` | Yes |
| Security Audit | `cargo audit` | Yes |
| ARI Score | `ariscan --format pr-comment` | Blocking (Phase 2+) |

### Branch Protection Rules

- Require PR reviews (minimum 1)
- Require status checks to pass
- Require branch up-to-date
- No direct pushes to `main`
- Require linear history (squash merge)
- Require signed commits (Phase 2+)
- Dismiss stale reviews
- Require conversation resolution

### CI Gate Escalation by Phase

| Phase | ARI Blocking | Benchmark Gate | Coverage Gate |
|-------|-------------|----------------|---------------|
| 0-1 | Advisory only | None | None |
| 2-3 | Pillar scores >= phase target | Advisory | >= 70% |
| 4 | Pillar scores >= phase target | Warn on >10% regression | >= 80% |
| 5-6 | Pillar scores >= phase target | Block on >20% regression | >= 85% |
