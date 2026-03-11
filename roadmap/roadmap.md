# Zed StringKnife — Product Roadmap

> **A surgical string transformation toolkit for the Zed editor, delivered as a Language Server Protocol extension with context-menu code actions.**

**Product Owner:** Dragos Ionut Bejenariu
**Repository:** `zed-stringknife`
**License:** MIT
**Extension ID:** `stringknife`
**Target Zed API:** `zed_extension_api` v0.7.x+

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

The system is composed of three distinct layers with enforced boundaries:

**Layer 1: Zed Extension (WASM)** — `src/lib.rs`

The thinnest possible shim. Its sole responsibilities are lifecycle management of the LSP binary: telling Zed where to find the binary, passing initialization options, and handling download for published releases. This layer contains zero business logic. It compiles to WebAssembly and runs inside Zed's sandboxed extension host.

| Responsibility | Implementation |
|---------------|----------------|
| Register extension with Zed | `register_extension!(StringKnifeExtension)` |
| Provide LSP binary path | `language_server_command()` → path to `stringknife-lsp` |
| Pass configuration | `language_server_initialization_options()` → JSON config |
| Download binary on install | `zed::download_file()` from GitHub Releases |
| Verify binary checksum | SHA256 verification post-download |

**Layer 2: LSP Server** — `lsp/src/`

A thin dispatch layer that speaks the Language Server Protocol over stdio. It maintains document text state (required by the protocol for `textDocument/codeAction` to know what text is selected) and dispatches to the Transform Engine. The LSP handler should never contain transformation logic — it is a router, not a processor.

| Component | File | Responsibility |
|-----------|------|----------------|
| Server bootstrap | `main.rs` | Tokio runtime, tower-lsp setup, stdio transport |
| Protocol handlers | `handlers.rs` | `initialize`, `didOpen`, `didChange`, `codeAction`, `shutdown` |
| Document store | `document_store.rs` | `HashMap<Url, String>` — full text sync |
| Action builder | `actions.rs` | Builds `CodeAction` + `WorkspaceEdit` from transform results |
| Smart detection | `detection.rs` | Pattern matching to suggest relevant decode operations |
| Configuration | `config.rs` | Deserialise `initializationOptions`, handle `didChangeConfiguration` |
| Error mapping | `error.rs` | Maps `StringKnifeError` → LSP diagnostics / `window/showMessage` |

**Layer 3: Transform Engine** — `transforms/`

The heart. A library of pure functions with zero dependencies on LSP types, I/O, or side effects. Every transform has the same signature:

```rust
pub fn transform_name(input: &str) -> Result<String, StringKnifeError>
```

This uniformity is deliberate. It makes every transform trivially testable (no mocking, no setup, no teardown), trivially composable, and trivially portable — the `transforms/` crate could be published independently for reuse in CLIs, other editors, or web APIs.

| Module | Transforms |
|--------|-----------|
| `transforms/base64.rs` | encode, decode, url_encode, url_decode |
| `transforms/url.rs` | encode, decode, encode_component |
| `transforms/html.rs` | encode, decode |
| `transforms/hex.rs` | encode, decode |
| `transforms/unicode.rs` | escape, unescape, codepoints |
| `transforms/hash.rs` | md5, sha1, sha256, sha512, crc32 |
| `transforms/jwt.rs` | decode_header, decode_payload, decode_full |
| `transforms/json.rs` | pretty_print, minify, escape, unescape, to_yaml |
| `transforms/xml.rs` | pretty_print, minify |
| `transforms/case.rs` | upper, lower, title, sentence, camel, pascal, snake, screaming_snake, kebab, dot, path, toggle |
| `transforms/whitespace.rs` | trim, trim_leading, trim_trailing, collapse, remove_blank_lines, remove_duplicates, sort_asc, sort_desc, sort_length, reverse_lines, shuffle, number_lines |
| `transforms/inspect.rs` | count_chars, byte_length, detect_encoding |
| `transforms/escape.rs` | backslash, regex, sql, shell, csv |
| `transforms/misc.rs` | reverse_string |
| `transforms/timestamp.rs` | epoch_to_iso, iso_to_epoch, epoch_to_human |
| `transforms/numbers.rs` | dec_to_hex, hex_to_dec, dec_to_bin, bin_to_dec, dec_to_oct, oct_to_dec |
| `transforms/uuid.rs` | generate_v4, generate_v7, validate |
| `transforms/extract.rs` | emails, urls, ips, mask_sensitive |
| `transforms/diff.rs` | line_diff, char_diff |
| `transforms/compress.rs` | gzip_to_base64, base64_to_gzip, deflate_to_base64, base64_to_deflate |

---

### Data Flow: Code Action Request

```
User selects text "SGVsbG8gV29ybGQ=" in editor
                    │
                    ▼
    ┌───────────────────────────────┐
    │  Zed sends LSP request:       │
    │  textDocument/codeAction      │
    │  {                            │
    │    range: { start, end },     │
    │    context: { ... }           │
    │  }                            │
    └───────────────┬───────────────┘
                    │ stdio (JSON-RPC)
                    ▼
    ┌───────────────────────────────┐
    │  LSP Handler: codeAction()    │
    │                               │
    │  1. Look up document text     │
    │     from DocumentStore        │
    │  2. Extract selected text     │
    │     using range coordinates   │
    │  3. Run smart detection on    │
    │     selected text             │
    │  4. Build list of applicable  │
    │     CodeActions               │
    └───────────────┬───────────────┘
                    │
                    ▼
    ┌───────────────────────────────┐
    │  Smart Detection              │
    │                               │
    │  "SGVsbG8gV29ybGQ="          │
    │   ├─ Base64? ✓ (charset +    │
    │   │           padding match)  │
    │   ├─ URL-encoded? ✗          │
    │   ├─ Hex? ✗ (odd length)     │
    │   ├─ JWT? ✗ (no dots)        │
    │   └─ HTML entity? ✗          │
    │                               │
    │  Result: [Base64Decode] +     │
    │          [all encode actions]  │
    └───────────────┬───────────────┘
                    │
                    ▼
    ┌───────────────────────────────┐
    │  Response to Zed:             │
    │  [                            │
    │    { title: "StringKnife:     │
    │       Base64 Decode",  ◄──────┤── detected (shown first)
    │      kind: "refactor",        │
    │      edit: null (lazy)  },    │
    │    { title: "StringKnife:     │
    │       Base64 Encode",         │
    │      ... },                   │
    │    { title: "StringKnife:     │
    │       URL Encode",            │
    │      ... },                   │
    │    ...                        │
    │  ]                            │
    └───────────────┬───────────────┘
                    │
                    ▼
    ┌───────────────────────────────┐
    │  User selects "StringKnife:   │
    │  Base64 Decode" from context  │
    │  menu                         │
    └───────────────┬───────────────┘
                    │
                    ▼
    ┌───────────────────────────────┐
    │  Zed sends:                   │
    │  codeAction/resolve           │
    │  (or inline edit was eager)   │
    └───────────────┬───────────────┘
                    │
                    ▼
    ┌───────────────────────────────┐
    │  Transform Engine             │
    │                               │
    │  base64::decode(              │
    │    "SGVsbG8gV29ybGQ="        │
    │  )                            │
    │  → Ok("Hello World")         │
    └───────────────┬───────────────┘
                    │
                    ▼
    ┌───────────────────────────────┐
    │  WorkspaceEdit response:      │
    │  {                            │
    │    changes: {                  │
    │      "file:///path": [{       │
    │        range: { start, end }, │
    │        newText: "Hello World" │
    │      }]                       │
    │    }                          │
    │  }                            │
    └───────────────┬───────────────┘
                    │
                    ▼
    ┌───────────────────────────────┐
    │  Zed replaces selection:      │
    │  "SGVsbG8gV29ybGQ="          │
    │        becomes                │
    │  "Hello World"                │
    │                               │
    │  (Undo-able via Cmd+Z)        │
    └───────────────────────────────┘
```

---

### Repository Structure

```
zed-stringknife/
├── extension.toml              # Zed extension manifest
├── Cargo.toml                  # Workspace root (members: ".", "lsp", "transforms")
├── Cargo.lock                  # Committed for build determinism
├── rust-toolchain.toml         # Pins stable Rust channel
├── deny.toml                   # cargo-deny configuration
├── LICENSE                     # MIT
├── README.md                   # User-facing documentation
├── CHANGELOG.md                # Release history
├── CONTRIBUTING.md             # Developer onboarding
├── HINTS.md                    # AI-agent context (LCI-compatible)
├── SECURITY.md                 # Responsible disclosure
├── ROADMAP.md                  # This document
│
├── src/
│   └── lib.rs                  # WASM extension shim (Layer 1)
│
├── lsp/
│   ├── Cargo.toml              # LSP binary crate
│   └── src/
│       ├── main.rs             # Entry point, tokio runtime, stdio transport
│       ├── handlers.rs         # LSP protocol handlers
│       ├── document_store.rs   # Full-text document sync
│       ├── actions.rs          # CodeAction + WorkspaceEdit builder
│       ├── detection.rs        # Smart encoding detection
│       ├── config.rs           # Extension configuration
│       └── error.rs            # StringKnifeError → LSP error mapping
│
├── transforms/
│   ├── Cargo.toml              # Pure library crate (zero LSP deps)
│   └── src/
│       ├── lib.rs              # Public API, re-exports
│       ├── error.rs            # StringKnifeError enum
│       ├── base64.rs           # Base64 encode/decode
│       ├── url.rs              # URL percent-encoding
│       ├── html.rs             # HTML entity encode/decode
│       ├── hex.rs              # Hex encode/decode
│       ├── unicode.rs          # Unicode escape/unescape
│       ├── hash.rs             # MD5, SHA-1, SHA-256, SHA-512, CRC32
│       ├── jwt.rs              # JWT decode (header, payload, full)
│       ├── json.rs             # Pretty print, minify, escape, YAML conversion
│       ├── xml.rs              # Pretty print, minify
│       ├── case.rs             # Case conversions (12 variants)
│       ├── whitespace.rs       # Trim, collapse, sort, dedupe, etc.
│       ├── inspect.rs          # Count, length, detect encoding
│       ├── escape.rs           # Backslash, regex, SQL, shell, CSV
│       ├── timestamp.rs        # Epoch ↔ ISO 8601 ↔ human
│       ├── numbers.rs          # Base conversions (dec/hex/bin/oct)
│       ├── uuid.rs             # UUID v4/v7 generation, validation
│       ├── extract.rs          # Email, URL, IP extraction, masking
│       ├── diff.rs             # Line and character diff
│       ├── compress.rs         # Gzip/Deflate ↔ Base64
│       └── misc.rs             # Reverse string, other one-offs
│
├── docs/
│   ├── ari/                    # ARI checkpoint reports
│   │   ├── ARI-BASELINE.md
│   │   ├── ARI-0.md ... ARI-4.md
│   ├── pm-reviews/             # PM review decision records
│   │   ├── PMR-0.md ... PMR-5.md
│   └── audits/                 # Audit reports
│       ├── CODE-QUALITY-{N}.md
│       ├── SECURITY-AUDIT-{N}.md
│       ├── ARCH-AUDIT-{N}.md
│       ├── DEP-AUDIT-{N}.md
│       └── UX-AUDIT-{N}.md
│
└── .github/
    └── workflows/
        ├── ci.yml              # Build, test, lint, deny, audit
        ├── release.yml         # Cross-compile + publish binaries
        ├── ariscan.yml         # ARI score on every PR
        └── dependabot.yml      # Dependency update automation
```

---

### Workspace Crate Graph

```
┌─────────────────────────────────────┐
│  Workspace Root (Cargo.toml)        │
│  members = [".", "lsp", "transforms"]│
└──────────┬──────────┬───────────────┘
           │          │
     ┌─────▼────┐  ┌──▼──────────────┐
     │   root    │  │      lsp        │
     │  (cdylib) │  │    (binary)     │
     │           │  │                 │
     │ zed_ext   │  │  tower-lsp     │
     │ _api      │  │  tokio         │
     │           │  │  serde         │
     │           │  │  serde_json    │
     │           │  │  tracing       │
     │           │  │                 │
     │           │  │  depends on:   │
     │           │  │  └─ transforms │
     └───────────┘  └────────┬───────┘
                             │
                    ┌────────▼───────┐
                    │   transforms   │
                    │   (lib crate)  │
                    │                │
                    │  base64 (std)  │
                    │  percent-enc.  │
                    │  sha2          │
                    │  md-5          │
                    │  crc32fast     │
                    │  serde_json    │
                    │  serde_yaml    │
                    │  toml          │
                    │  uuid          │
                    │  flate2        │
                    │  similar       │
                    │                │
                    │  ZERO LSP deps │
                    │  ZERO I/O      │
                    │  ZERO side fx  │
                    └────────────────┘
```

The critical boundary: **`transforms` has no dependency on `lsp`**, and `lsp` has no dependency on `root` (the WASM extension). The arrows point downward only. The `transforms` crate is publishable independently to crates.io for reuse in CLIs, other editors, or web services.

---

### Key Design Principles

**1. Pure function supremacy.** Every string operation is `fn(&str) -> Result<String, StringKnifeError>`. No hidden state, no environment reads, no file system, no network. This is not a stylistic preference — it is the structural guarantee that makes the codebase agent-friendly (ARI Test Isolation pillar), trivially testable, and immune to the class of bugs that emerge from shared mutable state.

**2. The LSP is a router, not a processor.** The `codeAction` handler's job is to extract selected text, ask the detection module which transforms are relevant, build a menu of `CodeAction` responses, and — when one is selected — call the transform function and wrap the result in a `WorkspaceEdit`. If the handler grows beyond ~200 lines, it is accumulating logic that belongs in `transforms/` or `detection.rs`.

**3. Detection is heuristic, not authoritative.** Smart detection uses pattern matching to *suggest* relevant decode operations (e.g., if the selection looks like Base64, surface "Base64 Decode" at the top). It does not guarantee correctness — a hex string of even length also matches Base64 charset. The user always sees all encode actions regardless. False positives in detection are acceptable; false negatives (failing to suggest an obvious decode) are bugs.

**4. Errors are values, not exceptions.** `StringKnifeError` is a first-class enum carried through every code path. The LSP layer maps errors to either `window/showMessage` notifications (user-visible) or silently omits an action from the menu (if detection suggests a decode that fails validation). A transform function that panics is a P0 bug.

**5. Cross-platform is a constraint, not a feature.** The LSP binary must compile for macOS (Intel + ARM), Linux (x86_64 + ARM), and Windows (x86_64). This constrains dependency choices — no platform-specific crates in `transforms/`, no FFI, no system library links. Pure Rust, all the way down.

**6. Zero network, zero telemetry.** StringKnife makes no outbound network calls, ever. No usage analytics, no crash reporting, no update checks (Zed handles extension updates). This is both a privacy commitment and an architectural simplification — the extension works identically offline and online.

---

### Dependency Budget

Minimal dependencies are a first-class constraint, not an afterthought. Every crate added to `transforms/` increases supply chain risk, compile time, and binary size. The following is the target dependency set at v1.0:

| Crate | Purpose | Layer | Justification |
|-------|---------|-------|---------------|
| `zed_extension_api` | Zed WASM extension trait | root | Required. No alternative. |
| `tower-lsp` | LSP protocol implementation | lsp | Industry standard. Alternatives (lsp-server) are less maintained. |
| `tokio` | Async runtime for LSP | lsp | Required by tower-lsp. Use `rt` + `io-std` features only. |
| `serde` + `serde_json` | JSON serialization | lsp, transforms | Universal. Required for LSP protocol and JSON transforms. |
| `tracing` | Structured logging | lsp | Lightweight. Better than ad-hoc `eprintln!`. |
| `base64` | Base64 encode/decode | transforms | Tiny, well-maintained, pure Rust. |
| `percent-encoding` | URL encode/decode | transforms | From the `url` ecosystem. Battle-tested. |
| `sha2` + `md-5` | SHA and MD5 hashing | transforms | RustCrypto ecosystem. Audited, no unsafe. |
| `crc32fast` | CRC32 checksum | transforms | SIMD-accelerated, pure Rust fallback. |
| `serde_yaml` | YAML serialization | transforms | For JSON ↔ YAML conversion. |
| `toml` | TOML serialization | transforms | For JSON ↔ TOML conversion. |
| `uuid` | UUID generation | transforms | Feature-gated: `v4`, `v7`. |
| `flate2` | Gzip/Deflate compression | transforms | Pure Rust backend (`miniz_oxide`). |
| `similar` | Text diffing | transforms | Line and character-level diffs. |
| `chrono` | Timestamp operations | transforms | For epoch ↔ ISO 8601 ↔ human readable. |

**Hard rules:**
- No crate with `unsafe` in `transforms/`
- No crate requiring system libraries (OpenSSL, libz, etc.)
- No crate with fewer than 1,000 downloads/week unless handwritten
- Total transitive dependency count target: < 150 crates at v1.0
- All dependencies must pass `cargo-deny` license check (MIT, Apache-2.0, BSD, ISC, Zlib)

---

### Performance Model

The performance contract is simple: every code action response must complete in under **100ms for 100KB of selected text**. This budget breaks down as:

| Step | Budget | Notes |
|------|--------|-------|
| Document text lookup | < 1ms | HashMap lookup by URL |
| Text extraction from range | < 1ms | String slicing |
| Smart detection (all patterns) | < 5ms | Regex/charset scanning on selection |
| Transform execution | < 80ms | The actual encode/decode/hash/format |
| WorkspaceEdit construction | < 5ms | JSON serialization |
| LSP response serialization | < 5ms | JSON-RPC framing |
| **Total** | **< 100ms** | |

For selections > 1MB, the LSP returns an `InputTooLarge` error via `window/showMessage` rather than attempting the operation. This limit is configurable via `stringknife.maxInputSize`.

---

### Security Model

StringKnife operates under a minimal-privilege security model:

| Property | Guarantee |
|----------|-----------|
| **No network access** | Zero outbound connections. No DNS, no HTTP, no sockets. |
| **No file system writes** | Transforms operate on in-memory strings only. The only "write" is the LSP response. |
| **No file system reads** | Beyond what Zed provides via `textDocument/didOpen`. |
| **No code execution** | No `eval`, no shell spawning, no subprocess creation. |
| **No `unsafe` Rust** | Enforced by Clippy deny lint. |
| **No credential handling** | JWT decode is read-only. Signatures are not verified — no secret keys are ever processed. |
| **Input sanitization** | All decode operations validate input before transformation. Invalid input → `Err`, never partial output. |
| **Fuzz-tested surfaces** | All decode/parse functions are fuzz-tested with `cargo-fuzz` before each release. |

---

## AI-Agent Readiness: `ariscan` Integration

This repository is built agent-first from commit zero. **Prontiq's `ariscan` CLI** scores repository AI-agent readiness across the 8-pillar ARI index. Rather than retrofitting agent-readiness after the fact, StringKnife treats ARI score as a first-class engineering metric — measured at every phase gate, with regressions treated as blocking.

### ARI Pillars & StringKnife Targets

| # | Pillar | Phase 0 Target | v1.0 Target | Strategy |
|---|--------|---------------|-------------|----------|
| 1 | **Test Isolation** | ≥ 8 | ≥ 9 | Pure function transforms = trivially isolated tests. No shared state between test cases. |
| 2 | **Build Determinism** | ≥ 8 | ≥ 9 | `rust-toolchain.toml` pins channel. `Cargo.lock` committed. Reproducible WASM + binary builds. |
| 3 | **Type Safety** | ≥ 9 | ≥ 9 | Rust. Enough said. Strict clippy lints, no `unwrap()` in library code. |
| 4 | **Modular Coherence** | ≥ 7 | ≥ 9 | Each transform is a standalone pure function in its own module. LSP wiring is separate from logic. |
| 5 | **Documentation Density** | ≥ 6 | ≥ 8 | HINTS.md, CONTRIBUTING.md, inline rustdoc on all public APIs. LCI-compatible doc structure. |
| 6 | **Dependency Transparency** | ≥ 8 | ≥ 9 | Minimal deps. `cargo-deny` for license/advisory audit. No transitive wildcards. |
| 7 | **Error Explicitness** | ≥ 8 | ≥ 9 | All transforms return `Result<T, E>` with structured error types. No panics. No silent failures. |
| 8 | **Security (Gate)** | Pass | Pass | `cargo-audit` in CI. No `unsafe`. No network calls. No file system access in transforms. |

### `ariscan` Checkpoint Schedule

| Checkpoint | When | Minimum ARI | Action on Fail |
|-----------|------|-------------|----------------|
| **ARI-0** | End of Phase 0 | ≥ 7.0 composite | Block Phase 1 entry. Fix pillar deficiencies. |
| **ARI-1** | End of Phase 1 | ≥ 7.5 composite | Block Phase 2 entry. Remediation sprint. |
| **ARI-2** | End of Phase 3 | ≥ 8.0 composite | Block Phase 4 entry. Architectural review if below. |
| **ARI-3** | Pre-publish (Phase 5) | ≥ 8.5 composite | Block store submission. Final hardening sprint. |
| **ARI-4** | Post v1.0 (Phase 6) | ≥ 9.0 composite | Continuous. Regressions flagged in CI. |

---

## Product Management Governance

### PM Review Cadence

Roadmaps rot. Features that seemed essential at conception become irrelevant after the first user touches the product. The following PM reviews are scheduled as **mandatory phase gates** — not optional retrospectives.

| Review | When | Scope | Outputs |
|--------|------|-------|---------|
| **PMR-0: Foundation Review** | End of Phase 0 | Validate architecture bet (LSP code actions), confirm Zed API compatibility, review Phase 1 scope against real dev experience | Go/No-Go for Phase 1. Scope adjustments. Kill list. |
| **PMR-1: MVP Scope Review** | Mid-Phase 1 (after EPIC-1.2) | Are the right encodings prioritised? User-test with 3 developers. Check Zed extension store landscape for competitors. | Reprioritise remaining Phase 1 EPICs. Promote/demote from backlog. |
| **PMR-2: Feature Velocity Check** | End of Phase 2 | Review velocity. Is Phase 3 scope realistic? Are hashing/JWT features actually used or speculative? | Cut, defer, or accelerate Phase 3 items. Adjust release cadence. |
| **PMR-3: Pre-Launch Review** | End of Phase 4 | Full feature audit. What ships in v0.5.0? What gets cut? Review README, demo assets, store listing. | Final v0.5.0 scope lock. Marketing checklist. |
| **PMR-4: Post-Launch Retrospective** | 2 weeks after Phase 5 store publish | User feedback synthesis. Download/install metrics. GitHub issues triage. Community sentiment. | Phase 6 priority stack rank. Backlog grooming. Kill underperforming features. |
| **PMR-5: v1.0 Readiness Review** | Mid-Phase 6 | Is v1.0 warranted? Stability, completeness, community health. | Ship v1.0 or continue iterating as 0.x. |

### PM Review Process

Each PM Review produces a **written decision record** (committed to `docs/pm-reviews/PMR-{N}.md`) containing:
1. **Decisions made** — what was added, cut, reprioritised, deferred
2. **Evidence basis** — user feedback, metrics, competitive intel, ariscan scores
3. **Next review trigger** — what conditions trigger the next review
4. **Backlog mutations** — tickets moved in/out of phases with justification

---

## Audit Schedule

### Audit Types

| Audit | Focus | Cadence |
|-------|-------|---------|
| **Code Quality Audit** | Clippy compliance, dead code, code duplication, module boundaries, test coverage % | Every 2 phases |
| **Security Audit** | `cargo-audit` advisories, `cargo-deny` license check, unsafe blocks, input fuzzing results | Every 2 phases + pre-publish |
| **Architecture Audit** | Module coherence, LSP protocol compliance, separation of concerns, performance profiling | Phase 2 and Phase 4 |
| **Dependency Audit** | Transitive dep count, license compatibility, version currency, supply chain risk | Every phase |
| **UX Audit** | Code action discoverability, naming consistency, error message clarity, multi-cursor behavior | Phase 3 and pre-publish |

### Audit Tickets (Embedded in Phases)

These are woven into the phase structure below with `A-` prefix ticket numbers.

---

## Phase 0 — Project Bootstrap

> **Goal:** Repository scaffolded, CI green, dev extension installable in Zed with a single no-op code action proving the full pipeline works end-to-end. ARI foundations laid from first commit.

### EPIC-0.1: Repository & Toolchain Setup

- [ ] **T-001** — Initialise Git repository with `main` branch protection rules
- [ ] **T-002** — Create `extension.toml` manifest
  - [ ] Set `id = "stringknife"`, `name = "StringKnife"`, `schema_version = 1`
  - [ ] Add `description`, `authors`, `repository` fields
  - [ ] Register language server entry: `[language_servers.stringknife-lsp]`
  - [ ] Map language server to broad file types: `["Rust", "TypeScript", "JavaScript", "Python", "Go", "Ruby", "HTML", "CSS", "JSON", "TOML", "YAML", "Markdown", "Plain Text", "C", "C++", "Java", "Kotlin", "Swift", "Shell Script", "SQL", "Elixir", "PHP"]`
- [ ] **T-003** — Create `Cargo.toml` for the Zed extension WASM crate
  - [ ] Set `crate-type = ["cdylib"]`
  - [ ] Add `zed_extension_api = "0.7.0"` dependency
- [ ] **T-004** — Create `src/lib.rs` with minimal `Extension` trait implementation
  - [ ] Implement `language_server_command()` to return path to bundled LSP binary
  - [ ] Implement `language_server_initialization_options()` returning empty config
  - [ ] Call `register_extension!` macro
- [ ] **T-005** — Add `LICENSE` (MIT) at repository root
- [ ] **T-006** — Create `.gitignore` (target/, node_modules/, *.wasm)
- [ ] **T-007** — Create `README.md` with project overview, installation instructions, and feature list placeholder
- [ ] **T-008** — Create `CHANGELOG.md` with `## [Unreleased]` section
- [ ] **T-009** — Create `CONTRIBUTING.md` with dev setup instructions

### EPIC-0.2: ARI Foundations (Agent-Readiness from Day One)

- [ ] **T-025** — Create `HINTS.md` at repository root
  - [ ] Document repo structure and purpose of each directory
  - [ ] Document the LSP ↔ WASM extension architecture
  - [ ] Document how to add a new string operation (step-by-step)
  - [ ] Document test patterns and conventions
- [ ] **T-026** — Create `rust-toolchain.toml` pinning stable channel (Build Determinism)
- [ ] **T-027** — Commit `Cargo.lock` to version control (Build Determinism)
- [ ] **T-028** — Configure strict Clippy lints in workspace `Cargo.toml` or `.clippy.toml`
  - [ ] `#![deny(clippy::unwrap_used)]` in library code
  - [ ] `#![deny(clippy::panic)]` in library code
  - [ ] `#![warn(clippy::pedantic)]`
- [ ] **T-029** — Define `StringKnifeError` enum with structured error variants (Error Explicitness)
  - [ ] `InvalidInput { operation: String, reason: String }`
  - [ ] `UnsupportedEncoding { encoding: String }`
  - [ ] `InputTooLarge { max_bytes: usize, actual_bytes: usize }`
  - [ ] Implement `Display` and `std::error::Error`
- [ ] **T-030** — Create `transforms/` module directory with `mod.rs` (Modular Coherence)
  - [ ] Each transform category gets its own submodule file
  - [ ] All transforms are pure functions: `fn(input: &str) -> Result<String, StringKnifeError>`
  - [ ] No LSP types, no I/O, no side effects in transform modules
- [ ] **T-031** — Add `cargo-deny` configuration (`deny.toml`)
  - [ ] License allowlist: MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause, ISC, Zlib
  - [ ] Advisory database check enabled
  - [ ] Duplicate crate detection enabled
- [ ] **T-032** — Add `cargo-audit` to CI pipeline (Security gate)
- [ ] **T-033** — Add rustdoc comments on all public types and functions (Documentation Density)
- [ ] **T-034** — Install and run `ariscan` against the repo — establish **ARI-BASELINE** score
  - [ ] Record baseline scores per pillar in `docs/ari/ARI-BASELINE.md`
  - [ ] Identify any pillar below 6.0 and create remediation tickets

### EPIC-0.3: Language Server Skeleton

- [ ] **T-010** — Create `lsp/` directory for the LSP binary crate
- [ ] **T-011** — Create `lsp/Cargo.toml`
  - [ ] Add dependencies: `tower-lsp`, `tokio`, `serde`, `serde_json`
  - [ ] Set binary name: `stringknife-lsp`
- [ ] **T-012** — Implement minimal LSP server in `lsp/src/main.rs`
  - [ ] Implement `initialize` handler returning server capabilities
  - [ ] Declare `codeActionProvider = true` in capabilities
  - [ ] Declare `textDocumentSync` as `Full` (needed to access document text)
  - [ ] Implement `textDocument/didOpen` handler to store document text
  - [ ] Implement `textDocument/didChange` handler to update stored text
  - [ ] Implement `textDocument/codeAction` handler returning empty actions list
  - [ ] Implement `shutdown` handler
- [ ] **T-013** — Add document text store (HashMap<Url, String>) to server state
- [ ] **T-014** — Verify LSP binary compiles and runs standalone with `--stdio` flag
- [ ] **T-015** — Wire extension WASM to download/locate the LSP binary
  - [ ] For dev: point to local `target/release/stringknife-lsp`
  - [ ] For published: implement binary download from GitHub Releases via `zed::download_file()`

### EPIC-0.4: End-to-End Proof of Life

- [ ] **T-016** — Add a single hardcoded code action: "StringKnife: Reverse String"
  - [ ] Implement as a pure function in `transforms/misc.rs`
  - [ ] Wire into LSP code action handler
  - [ ] Extract selected text range from `CodeActionParams`
  - [ ] Return `CodeAction` with `WorkspaceEdit` replacing the selection range
- [ ] **T-017** — Add unit test for reverse string transform (isolated, no LSP dependency)
- [ ] **T-018** — Install as dev extension in Zed (`zed: install dev extension`)
- [ ] **T-019** — Verify code action appears in context menu when text is selected
- [ ] **T-020** — Verify selecting the action replaces text correctly
- [ ] **T-035** — Document the dev install workflow in `CONTRIBUTING.md`

### EPIC-0.5: CI/CD Pipeline

- [ ] **T-021** — Create `.github/workflows/ci.yml`
  - [ ] Run `cargo check` on both WASM crate and LSP crate
  - [ ] Run `cargo test` on LSP crate
  - [ ] Run `cargo clippy` with `-D warnings`
  - [ ] Run `cargo fmt --check`
  - [ ] Run `cargo deny check` (license + advisory)
  - [ ] Run `cargo audit` (security)
  - [ ] Run `ariscan` and output ARI score summary (informational, non-blocking initially)
- [ ] **T-022** — Create `.github/workflows/release.yml`
  - [ ] Trigger on Git tag `v*`
  - [ ] Cross-compile LSP binary for `x86_64-apple-darwin`, `aarch64-apple-darwin`, `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, `x86_64-pc-windows-msvc`
  - [ ] Upload binaries as GitHub Release assets
  - [ ] Generate checksums (SHA256)
- [ ] **T-023** — Create `.github/workflows/ariscan.yml`
  - [ ] Run `ariscan` on every PR
  - [ ] Post ARI score as PR comment (per-pillar breakdown)
  - [ ] Fail PR if any pillar drops below its phase target threshold
  - [ ] Cache previous score for delta comparison
- [ ] **T-024** — Add Dependabot config for Cargo dependency updates

### 🔒 GATE: ARI-0 Checkpoint

- [ ] **ARI-0** — Run `ariscan` — **minimum composite score ≥ 7.0**
  - [ ] Record scores in `docs/ari/ARI-0.md`
  - [ ] All 8 pillars individually ≥ 6.0
  - [ ] Security pillar: **Pass** (no advisories, no unsafe, no panics in lib)
  - [ ] If below threshold: create remediation tickets, block Phase 1 entry

### 🔍 AUDIT: Dependency Audit #1

- [ ] **A-001** — Run `cargo deny check` and review all transitive dependencies
  - [ ] Document total dependency count in `docs/audits/DEP-AUDIT-1.md`
  - [ ] Flag any dependency with > 6 months since last release
  - [ ] Flag any dependency with known CVE (even if not directly exploitable)
  - [ ] Confirm all licenses compatible with MIT

### 📋 PM REVIEW: PMR-0 — Foundation Review

- [ ] **PMR-0** — Conduct Foundation Review
  - [ ] Validate LSP code action architecture works reliably in Zed
  - [ ] Confirm no Zed API changes that invalidate the approach
  - [ ] Review Phase 1 scope: are these the right encoding operations?
  - [ ] Check Zed extension store for any competing string utility extensions
  - [ ] Decision: Go/No-Go for Phase 1
  - [ ] Document decisions in `docs/pm-reviews/PMR-0.md`

**Phase 0 Exit Criteria:** Dev extension installs in Zed. Selecting text → right-click → "StringKnife: Reverse String" works. CI is green. ARI ≥ 7.0. PMR-0 complete.

---

## Phase 1 — Core Encoding & Decoding

> **Goal:** Ship the essential encoding/decoding operations that cover 90% of daily string manipulation needs.

### EPIC-1.1: Base64 Operations

- [ ] **T-100** — Implement `Base64 Encode` code action
  - [ ] Pure function in `transforms/base64.rs`
  - [ ] Standard Base64 (RFC 4648)
  - [ ] Handle UTF-8 input correctly
  - [ ] Preserve line selection range for replacement
- [ ] **T-101** — Implement `Base64 Decode` code action
  - [ ] Return `StringKnifeError::InvalidInput` for invalid Base64 (no panics, no crashes)
  - [ ] Support padded and unpadded input
- [ ] **T-102** — Implement `Base64URL Encode` code action (URL-safe alphabet, no padding)
- [ ] **T-103** — Implement `Base64URL Decode` code action
- [ ] **T-104** — Unit tests for all Base64 variants
  - [ ] Empty string
  - [ ] ASCII input
  - [ ] Unicode/UTF-8 multi-byte input
  - [ ] Roundtrip encode→decode identity
  - [ ] Invalid input error paths (returns `Err`, never panics)

### EPIC-1.2: URL Encoding Operations

- [ ] **T-110** — Implement `URL Encode` code action (percent-encoding, RFC 3986)
  - [ ] Pure function in `transforms/url.rs`
- [ ] **T-111** — Implement `URL Decode` code action
  - [ ] Handle `+` as space (form encoding) and `%20` as space (URI encoding)
- [ ] **T-112** — Implement `URL Encode (Component)` code action (encodes everything except unreserved chars)
- [ ] **T-113** — Unit tests for URL encoding
  - [ ] Reserved characters: `! # $ & ' ( ) * + , / : ; = ? @ [ ]`
  - [ ] Unicode characters
  - [ ] Already-encoded input (double-encoding prevention awareness — document behavior)
  - [ ] Roundtrip identity

### 📋 PM REVIEW: PMR-1 — MVP Scope Check (Mid-Phase)

- [ ] **PMR-1** — Conduct MVP Scope Review
  - [ ] User-test with 2–3 developers: are Base64 and URL the right first operations?
  - [ ] Review Zed extension store: any new competitors since PMR-0?
  - [ ] Assess: should HTML entities be cut in favour of something more requested?
  - [ ] Assess: is hex encoding worth its priority slot or should it move to Phase 2?
  - [ ] Reprioritise remaining Phase 1 EPICs if needed
  - [ ] Promote any backlog items that early users are requesting
  - [ ] Document decisions in `docs/pm-reviews/PMR-1.md`

### EPIC-1.3: HTML Entity Operations

- [ ] **T-120** — Implement `HTML Encode` code action
  - [ ] Pure function in `transforms/html.rs`
  - [ ] Encode `& < > " '` to named entities
  - [ ] Option: encode all non-ASCII to numeric entities
- [ ] **T-121** — Implement `HTML Decode` code action
  - [ ] Support named entities (`&amp;`, `&lt;`, `&gt;`, `&quot;`, `&apos;`, `&nbsp;`)
  - [ ] Support decimal numeric entities (`&#123;`)
  - [ ] Support hex numeric entities (`&#x7B;`)
- [ ] **T-122** — Unit tests for HTML entities
  - [ ] Nested/compound encoding
  - [ ] Malformed entities (pass through unchanged)

### EPIC-1.4: Hex Operations

- [ ] **T-130** — Implement `Hex Encode` code action (UTF-8 bytes → hex string)
  - [ ] Pure function in `transforms/hex.rs`
- [ ] **T-131** — Implement `Hex Decode` code action (hex string → UTF-8 text)
  - [ ] Support with/without `0x` prefix
  - [ ] Support with/without space-separated bytes
  - [ ] Error on invalid hex characters
- [ ] **T-132** — Unit tests for hex operations

### EPIC-1.5: Unicode Operations

- [ ] **T-140** — Implement `Unicode Escape` code action (`Hello` → `\u0048\u0065\u006C\u006C\u006F`)
  - [ ] Pure function in `transforms/unicode.rs`
  - [ ] Support `\uXXXX` format (JavaScript/Java style)
  - [ ] Support `\UXXXXXXXX` for chars above BMP
- [ ] **T-141** — Implement `Unicode Unescape` code action
  - [ ] Parse `\uXXXX` and `\UXXXXXXXX` sequences
  - [ ] Leave non-escape text unchanged
- [ ] **T-142** — Implement `Show Unicode Codepoints` code action (informational — shows codepoints as a comment/diagnostic, doesn't replace text)
- [ ] **T-143** — Unit tests for Unicode operations
  - [ ] Emoji (multi-codepoint sequences)
  - [ ] CJK characters
  - [ ] Combining characters

### EPIC-1.6: Code Action Categorisation & UX

- [ ] **T-150** — Group code actions under `"StringKnife"` category in the code action response
  - [ ] Use `CodeActionKind::REFACTOR` as the base kind
  - [ ] Prefix all action titles with `StringKnife:` for discoverability
- [ ] **T-151** — Only return relevant decode actions when selected text looks like encoded content
  - [ ] Detect Base64 pattern (charset + optional padding)
  - [ ] Detect URL-encoded pattern (contains `%XX`)
  - [ ] Detect HTML entity pattern (contains `&...;`)
  - [ ] Detect hex pattern (valid hex chars, even length)
  - [ ] Always show all encode actions
- [ ] **T-152** — Order code actions by relevance (detected decodes first, then all encodes)
- [ ] **T-153** — Handle multi-line selections correctly
- [ ] **T-154** — Handle empty selection (no code actions returned)

### 🔒 GATE: ARI-1 Checkpoint

- [ ] **ARI-1** — Run `ariscan` — **minimum composite score ≥ 7.5**
  - [ ] Record scores in `docs/ari/ARI-1.md`
  - [ ] Test Isolation pillar ≥ 8.0 (pure function transforms must be trivially testable)
  - [ ] Modular Coherence pillar ≥ 7.0 (transforms cleanly separated from LSP wiring)
  - [ ] Compare delta against ARI-0 — no pillar should have regressed
  - [ ] If below threshold: remediation sprint before Phase 2 entry

### 🔍 AUDIT: Code Quality Audit #1

- [ ] **A-010** — Code Quality Audit
  - [ ] Run `cargo clippy` — zero warnings
  - [ ] Measure test coverage with `cargo-tarpaulin` — target ≥ 80% on `transforms/` module
  - [ ] Check for code duplication across transform modules (extract shared patterns)
  - [ ] Verify all public functions have rustdoc comments
  - [ ] Document findings in `docs/audits/CODE-QUALITY-1.md`

**Phase 1 Exit Criteria:** All encoding/decoding actions work. Smart detection suggests relevant decode operations. Full unit test coverage. ARI ≥ 7.5. PMR-1 complete.

---

## Phase 2 — Hashing, Cryptographic & Data Format Operations

> **Goal:** Expand into hashing, JWT inspection, and data format conversions that developers reach for daily.

### EPIC-2.1: Hash Operations (One-Way)

- [ ] **T-200** — Implement `MD5 Hash` code action
  - [ ] Pure function in `transforms/hash.rs`
  - [ ] Replaces selected text with its MD5 hex digest
  - [ ] Add informational note: not for security use
- [ ] **T-201** — Implement `SHA-1 Hash` code action
- [ ] **T-202** — Implement `SHA-256 Hash` code action
- [ ] **T-203** — Implement `SHA-512 Hash` code action
- [ ] **T-204** — Implement `CRC32 Checksum` code action
- [ ] **T-205** — Unit tests for all hash operations
  - [ ] Known test vectors (RFC / NIST)
  - [ ] Empty string hash
  - [ ] Unicode input

### EPIC-2.2: JWT Operations (Read-Only Decode)

- [ ] **T-210** — Implement `JWT Decode Header` code action
  - [ ] Pure function in `transforms/jwt.rs`
  - [ ] Parse JWT structure (header.payload.signature)
  - [ ] Pretty-print JSON header
  - [ ] Replace selection with decoded header JSON
- [ ] **T-211** — Implement `JWT Decode Payload` code action
  - [ ] Decode payload section
  - [ ] Pretty-print JSON
  - [ ] Highlight `exp`/`iat`/`nbf` timestamps as human-readable dates in output
- [ ] **T-212** — Implement `JWT Decode (Full)` code action
  - [ ] Show header + payload + signature (hex) as formatted multi-line output
- [ ] **T-213** — Graceful handling of invalid JWT format
- [ ] **T-214** — Unit tests with sample JWTs
  - [ ] HS256 token
  - [ ] RS256 token
  - [ ] Expired token (still decodes, just shows expired date)
  - [ ] Malformed token (missing sections)

### EPIC-2.3: JSON Operations

- [ ] **T-220** — Implement `JSON Pretty Print` code action
  - [ ] Pure function in `transforms/json.rs`
  - [ ] 2-space indent
  - [ ] Handle already-pretty JSON (no-op or re-format)
- [ ] **T-221** — Implement `JSON Minify` code action
- [ ] **T-222** — Implement `JSON Escape String` code action (escape special chars for embedding in JSON string values)
- [ ] **T-223** — Implement `JSON Unescape String` code action
- [ ] **T-224** — Implement `JSON → YAML` code action
- [ ] **T-225** — Implement `YAML → JSON` code action
- [ ] **T-226** — Unit tests for JSON operations
  - [ ] Nested objects and arrays
  - [ ] Special characters and escape sequences
  - [ ] Large payloads (performance)
  - [ ] Invalid JSON error handling

### EPIC-2.4: XML/HTML Operations

- [ ] **T-230** — Implement `XML Pretty Print` code action
  - [ ] Pure function in `transforms/xml.rs`
- [ ] **T-231** — Implement `XML Minify` code action
- [ ] **T-232** — Unit tests for XML operations

### EPIC-2.5: TOML/CSV Utility Operations

- [ ] **T-240** — Implement `TOML → JSON` code action
- [ ] **T-241** — Implement `JSON → TOML` code action
- [ ] **T-242** — Implement `CSV → JSON Array` code action (first row as headers)
- [ ] **T-243** — Unit tests for format conversion operations

### 🔍 AUDIT: Architecture Audit #1

- [ ] **A-020** — Architecture Audit
  - [ ] Review module boundaries: are transforms fully decoupled from LSP types?
  - [ ] Review LSP handler: is it a thin dispatch layer or accumulating logic?
  - [ ] Profile code action response latency for each operation (target < 50ms for 10KB input)
  - [ ] Review dependency tree: any unnecessary transitive deps introduced by hash/JWT crates?
  - [ ] Assess: could `transforms/` be published as a standalone crate for reuse?
  - [ ] Document findings in `docs/audits/ARCH-AUDIT-1.md`

### 🔍 AUDIT: Security Audit #1

- [ ] **A-021** — Security Audit
  - [ ] Run `cargo audit` — zero advisories
  - [ ] Run `cargo deny check advisories`
  - [ ] Verify no `unsafe` blocks in entire codebase
  - [ ] Review hash crate dependencies for known supply chain issues
  - [ ] Fuzz test Base64 decode, URL decode, and JSON parse with `cargo-fuzz` (minimum 10 minutes per target)
  - [ ] Document findings in `docs/audits/SECURITY-AUDIT-1.md`

### 🔍 AUDIT: Dependency Audit #2

- [ ] **A-022** — Dependency Audit
  - [ ] Review all new dependencies added in Phase 2
  - [ ] Document total transitive dependency count delta from Phase 1
  - [ ] Verify no new license incompatibilities
  - [ ] Flag any dep with fewer than 100 downloads/week (supply chain risk)
  - [ ] Document in `docs/audits/DEP-AUDIT-2.md`

### 📋 PM REVIEW: PMR-2 — Feature Velocity Check

- [ ] **PMR-2** — Conduct Feature Velocity Check
  - [ ] Review actual velocity: how long did Phase 2 take vs. estimate?
  - [ ] Are hashing features actually useful or speculative? (check: would you use them?)
  - [ ] Is JWT decode a differentiator or bloat?
  - [ ] Review Phase 3 scope: is the full case conversion list necessary or should we ship fewer, better?
  - [ ] Re-examine backlog: anything from B-001–B-015 that should be promoted?
  - [ ] Adjust release cadence if velocity differs from plan
  - [ ] Decision: cut, defer, or accelerate Phase 3 items
  - [ ] Document decisions in `docs/pm-reviews/PMR-2.md`

**Phase 2 Exit Criteria:** All hash, JWT, and format conversion operations functional. Error handling is graceful across all actions. Architecture audit passed. Security audit passed.

---

## Phase 3 — Text Transformation & Case Conversion

> **Goal:** The string manipulation operations developers use when refactoring — case conversions, whitespace operations, text analysis.

### EPIC-3.1: Case Conversions

- [ ] **T-300** — Implement `To UPPERCASE` code action
  - [ ] Pure function in `transforms/case.rs`
- [ ] **T-301** — Implement `To lowercase` code action
- [ ] **T-302** — Implement `To Title Case` code action (capitalize first letter of each word)
- [ ] **T-303** — Implement `To Sentence Case` code action (capitalize first letter of each sentence)
- [ ] **T-304** — Implement `To camelCase` code action
- [ ] **T-305** — Implement `To PascalCase` code action
- [ ] **T-306** — Implement `To snake_case` code action
- [ ] **T-307** — Implement `To SCREAMING_SNAKE_CASE` code action
- [ ] **T-308** — Implement `To kebab-case` code action
- [ ] **T-309** — Implement `To dot.case` code action
- [ ] **T-310** — Implement `To path/case` code action
- [ ] **T-311** — Implement `To CONSTANT_CASE` code action (alias for SCREAMING_SNAKE)
- [ ] **T-312** — Implement `Toggle Case` code action (swap upper↔lower per character)
- [ ] **T-313** — Unit tests for all case conversions
  - [ ] Single word
  - [ ] Multi-word with various separators (space, underscore, hyphen, camelCase boundaries)
  - [ ] Acronyms (`HTTPSConnection` → `https_connection` → `httpsConnection`)
  - [ ] Unicode case mapping (ß → SS, İ → i)
  - [ ] Numbers in identifiers (`myVar2Name` → `my_var_2_name`)

### EPIC-3.2: Whitespace & Line Operations

- [ ] **T-320** — Implement `Trim Whitespace` code action (leading + trailing)
  - [ ] Pure function in `transforms/whitespace.rs`
- [ ] **T-321** — Implement `Trim Leading Whitespace` code action
- [ ] **T-322** — Implement `Trim Trailing Whitespace` code action
- [ ] **T-323** — Implement `Collapse Whitespace` code action (multiple spaces/tabs → single space)
- [ ] **T-324** — Implement `Remove Blank Lines` code action
- [ ] **T-325** — Implement `Remove Duplicate Lines` code action (preserve order)
- [ ] **T-326** — Implement `Sort Lines (A→Z)` code action
- [ ] **T-327** — Implement `Sort Lines (Z→A)` code action
- [ ] **T-328** — Implement `Sort Lines (by length)` code action
- [ ] **T-329** — Implement `Reverse Lines` code action (reverse line order, not characters)
- [ ] **T-330** — Implement `Shuffle Lines` code action (random order)
- [ ] **T-331** — Implement `Number Lines` code action (prefix each line with its number)
- [ ] **T-332** — Unit tests for whitespace and line operations

### EPIC-3.3: String Inspection (Non-Destructive)

- [ ] **T-340** — Implement `Count Characters` code action
  - [ ] Pure function in `transforms/inspect.rs`
  - [ ] Show total characters, bytes (UTF-8), words, lines as a Zed notification/diagnostic
  - [ ] Do NOT replace the selected text
- [ ] **T-341** — Implement `String Length (bytes)` code action (show UTF-8 byte count)
- [ ] **T-342** — Implement `Detect Encoding` code action (attempt to identify if selection is Base64, URL-encoded, hex, JWT, etc.)
- [ ] **T-343** — Unit tests for inspection operations

### EPIC-3.4: Escape/Unescape Operations

- [ ] **T-350** — Implement `Escape Backslashes` code action (`\` → `\\`)
  - [ ] Pure function in `transforms/escape.rs`
- [ ] **T-351** — Implement `Unescape Backslashes` code action (`\\` → `\`)
- [ ] **T-352** — Implement `Escape Regex` code action (escape regex special characters)
- [ ] **T-353** — Implement `Escape SQL String` code action (single quotes)
- [ ] **T-354** — Implement `Escape Shell String` code action
- [ ] **T-355** — Implement `Escape CSV Field` code action
- [ ] **T-356** — Unit tests for escape operations

### 🔒 GATE: ARI-2 Checkpoint

- [ ] **ARI-2** — Run `ariscan` — **minimum composite score ≥ 8.0**
  - [ ] Record scores in `docs/ari/ARI-2.md`
  - [ ] Test Isolation ≥ 8.5 (extensive pure function test suite by now)
  - [ ] Modular Coherence ≥ 8.0 (7+ transform modules, clean boundaries)
  - [ ] Documentation Density ≥ 7.5 (rustdoc on all public APIs, HINTS.md current)
  - [ ] Delta report against ARI-1 — flag any regressions
  - [ ] If below 8.0: **architectural review required** before Phase 4

### 🔍 AUDIT: Code Quality Audit #2

- [ ] **A-030** — Code Quality Audit
  - [ ] Test coverage ≥ 85% on `transforms/` module
  - [ ] Zero clippy warnings
  - [ ] Review for dead code (any unused transforms? any dead feature flags?)
  - [ ] Check for consistent error handling patterns across all modules
  - [ ] Review code action naming: is the `StringKnife:` prefix consistent?
  - [ ] Document findings in `docs/audits/CODE-QUALITY-2.md`

### 🔍 AUDIT: UX Audit #1

- [ ] **A-031** — UX Audit
  - [ ] Install extension on a clean Zed instance
  - [ ] Test code action discoverability: can a new user find encode/decode in < 3 seconds?
  - [ ] Count total code actions shown when selecting arbitrary text — is it overwhelming?
  - [ ] Review smart detection: does it correctly identify Base64 vs. hex vs. URL-encoded?
  - [ ] Review error messages: are they helpful to a developer who doesn't know StringKnife internals?
  - [ ] Test with multi-line selections, single character, entire file selected
  - [ ] Document findings and recommendations in `docs/audits/UX-AUDIT-1.md`

**Phase 3 Exit Criteria:** All case, whitespace, and escape operations functional. Inspection actions return info without modifying text. ARI ≥ 8.0. UX audit complete.

---

## Phase 4 — Configuration, Performance & Polish

> **Goal:** User-configurable behavior, performant operation on large selections, and production-quality error handling.

### EPIC-4.1: Extension Configuration

- [ ] **T-400** — Define LSP configuration schema (`initializationOptions`)
  - [ ] `stringknife.enabledCategories`: array of enabled categories (encoding, hashing, case, json, etc.)
  - [ ] `stringknife.maxCodeActions`: max number of code actions shown (default: 20)
  - [ ] `stringknife.smartDetection`: boolean to enable/disable smart decode suggestions (default: true)
  - [ ] `stringknife.hashOutputFormat`: `"lowercase"` | `"uppercase"` (default: lowercase)
  - [ ] `stringknife.jsonIndent`: number of spaces for pretty print (default: 2)
  - [ ] `stringknife.base64LineBreaks`: boolean for 76-char line wrapping (default: false)
- [ ] **T-401** — Read configuration from Zed settings via `initializationOptions`
- [ ] **T-402** — Handle `workspace/didChangeConfiguration` for live config updates
- [ ] **T-403** — Document all configuration options in README
- [ ] **T-404** — Add example Zed `settings.json` snippet to README

### EPIC-4.2: Performance & Large Input Handling

- [ ] **T-410** — Benchmark code action response time for 1KB, 10KB, 100KB, 1MB selections
- [ ] **T-411** — Set maximum input size limit (default: 1MB) with clear error message
- [ ] **T-412** — Ensure document sync doesn't hold full document copies unnecessarily
- [ ] **T-413** — Profile memory usage under sustained operation
- [ ] **T-414** — Add timeout handling for code action computation (5 second max)

### EPIC-4.3: Error Handling & User Feedback

- [ ] **T-420** — Define error response strategy: return `Diagnostic` for decode errors vs. silent skip
- [ ] **T-421** — Implement `window/showMessage` notifications for operations that fail on invalid input
- [ ] **T-422** — Ensure no panics in LSP binary under any input (fuzz test critical paths)
- [ ] **T-423** — Add structured logging to LSP (`tracing` crate, configurable log level)
- [ ] **T-424** — Log level configurable via `stringknife.logLevel` setting

### EPIC-4.4: Multi-Selection Support

- [ ] **T-430** — Handle multiple selection ranges in a single `codeAction` request
- [ ] **T-431** — Return `WorkspaceEdit` with multiple `TextEdit` entries (one per selection)
- [ ] **T-432** — Test multi-cursor encode/decode operations
- [ ] **T-433** — Ensure edits don't conflict when ranges overlap (reject with message)

### 🔍 AUDIT: Architecture Audit #2

- [ ] **A-040** — Architecture Audit
  - [ ] Review configuration plumbing: is it clean or spaghetti?
  - [ ] Profile memory under 1000 sequential code actions (leak test)
  - [ ] Review LSP lifecycle: clean shutdown, no orphan processes
  - [ ] Benchmark: all operations < 100ms for 100KB input (hard requirement)
  - [ ] Document findings in `docs/audits/ARCH-AUDIT-2.md`

### 🔍 AUDIT: Dependency Audit #3

- [ ] **A-041** — Dependency Audit
  - [ ] Full transitive dependency audit
  - [ ] Check for any new crates added for config/logging
  - [ ] Verify `tracing` dependency is justified vs. simpler logging
  - [ ] Document in `docs/audits/DEP-AUDIT-3.md`

### 📋 PM REVIEW: PMR-3 — Pre-Launch Review

- [ ] **PMR-3** — Conduct Pre-Launch Review
  - [ ] Full feature inventory: what's shipping in v0.5.0?
  - [ ] Kill list: any features that are half-baked and should be cut rather than shipped broken?
  - [ ] Review README: is it compelling for a developer discovering StringKnife in the store?
  - [ ] Review demo assets: do the GIFs clearly show the value proposition?
  - [ ] Review store listing: description, icon, metadata
  - [ ] Review CHANGELOG: is it coherent and useful?
  - [ ] Competitive check: has anyone published a similar extension since PMR-1?
  - [ ] Decision: **final v0.5.0 scope lock**
  - [ ] Marketing checklist: where to announce, who to tell
  - [ ] Document decisions in `docs/pm-reviews/PMR-3.md`

**Phase 4 Exit Criteria:** Extension is configurable, performant on large inputs, handles errors gracefully, and supports multi-cursor. Architecture audit passed. Pre-launch scope locked.

---

## Phase 5 — Publish, Distribute & Community

> **Goal:** Extension published to the Zed Extension Store, discoverable, documented, and ready for community contributions.

### EPIC-5.1: Publication Preparation

- [ ] **T-500** — Verify extension ID `stringknife` is available in the Zed extension registry
- [ ] **T-501** — Ensure `extension.toml` passes all Zed validation rules
  - [ ] ID does not contain "zed"
  - [ ] Version is semver
  - [ ] License file present and accepted (MIT)
  - [ ] Repository URL is HTTPS
- [ ] **T-502** — Write comprehensive `README.md`
  - [ ] Feature list with GIF/video demos
  - [ ] Installation instructions
  - [ ] Configuration reference table
  - [ ] Supported file types list
  - [ ] Contributing guidelines link
  - [ ] Changelog link
- [ ] **T-503** — Create extension icon/logo (SVG, follows Zed extension store guidelines)
- [ ] **T-504** — Create demo GIFs showing key workflows
  - [ ] Base64 encode/decode
  - [ ] JWT decode
  - [ ] Case conversion
  - [ ] Smart detection in action
- [ ] **T-505** — Update `HINTS.md` with final architecture, contributor onboarding, and "how to add a new operation" guide

### EPIC-5.2: Publish to Zed Extension Store

- [ ] **T-510** — Fork `zed-industries/extensions` to personal GitHub account
- [ ] **T-511** — Add `stringknife` as a Git submodule in `extensions/` directory
- [ ] **T-512** — Add entry to top-level `extensions.toml`
- [ ] **T-513** — Run `pnpm sort-extensions` to sort entries
- [ ] **T-514** — Open PR to `zed-industries/extensions`
- [ ] **T-515** — Respond to review feedback and iterate
- [ ] **T-516** — Verify extension appears in Zed Extension Store post-merge
- [ ] **T-517** — Test installation from the store on a clean Zed instance

### EPIC-5.3: Community & Maintenance

- [ ] **T-520** — Create GitHub issue templates
  - [ ] Bug report template
  - [ ] Feature request template
  - [ ] New string operation request template
- [ ] **T-521** — Create GitHub Discussions category for community suggestions
- [ ] **T-522** — Set up GitHub Action for automated extension updates (using `zed-extension-action`)
- [ ] **T-523** — Create `SECURITY.md` with responsible disclosure policy
- [ ] **T-524** — Tag and release `v0.1.0`
- [ ] **T-525** — Announce on Zed Discord and relevant communities

### 🔒 GATE: ARI-3 Checkpoint (Pre-Publish)

- [ ] **ARI-3** — Run `ariscan` — **minimum composite score ≥ 8.5**
  - [ ] Record scores in `docs/ari/ARI-3.md`
  - [ ] All pillars individually ≥ 7.5
  - [ ] Documentation Density ≥ 8.0 (HINTS.md, README, rustdoc, CONTRIBUTING.md all current)
  - [ ] Security gate: **Pass** (cargo-audit clean, fuzz tests run, no unsafe)
  - [ ] If below 8.5: **block store submission** — final hardening sprint

### 🔍 AUDIT: Security Audit #2 (Pre-Publish)

- [ ] **A-050** — Pre-Publish Security Audit
  - [ ] `cargo audit` — zero advisories
  - [ ] `cargo deny check` — all clear
  - [ ] Full fuzz test run on all decode/parse operations (30 minutes per target)
  - [ ] Review: does the extension request any permissions it doesn't need?
  - [ ] Review: can any code action cause data loss? (e.g., decode fails but still replaces text)
  - [ ] Document in `docs/audits/SECURITY-AUDIT-2.md`

### 🔍 AUDIT: UX Audit #2 (Pre-Publish)

- [ ] **A-051** — Pre-Publish UX Audit
  - [ ] Fresh install test on macOS, Linux
  - [ ] Verify all code actions appear and work on first install (no manual config needed)
  - [ ] Time from install to first successful encode: target < 30 seconds
  - [ ] Verify error messages are helpful and non-technical
  - [ ] Verify no performance degradation on large files (10K+ line files)
  - [ ] Document in `docs/audits/UX-AUDIT-2.md`

**Phase 5 Exit Criteria:** Extension live in Zed Extension Store. Installable by any Zed user. ARI ≥ 8.5. Both security and UX audits passed. Community contribution pipeline in place.

---

### 📋 PM REVIEW: PMR-4 — Post-Launch Retrospective

> **Scheduled:** 2 weeks after Phase 5 store publication

- [ ] **PMR-4** — Conduct Post-Launch Retrospective
  - [ ] Gather download/install metrics from Zed extension store
  - [ ] Triage all GitHub issues opened since launch
  - [ ] Synthesise user feedback themes: what do people love? What's missing? What's broken?
  - [ ] Review: which operations are actually being used? (if telemetry is available via store metrics)
  - [ ] Competitive landscape: any copycats or superior alternatives launched?
  - [ ] Stack rank Phase 6 features based on real user demand (not assumptions)
  - [ ] **Kill decision:** any features from Phase 1–4 that should be removed?
  - [ ] **Promote decision:** any backlog items (B-001–B-015) that users are requesting?
  - [ ] Adjust Phase 6 scope and priority order based on evidence
  - [ ] Document decisions in `docs/pm-reviews/PMR-4.md`

---

## Phase 6 — Advanced Features (Post-Launch)

> **Goal:** Differentiate StringKnife from basic string tools with power-user features driven by community feedback and PMR-4 evidence.

### EPIC-6.1: Timestamp/Epoch Operations

- [ ] **T-600** — Implement `Unix Timestamp → ISO 8601` code action
  - [ ] Pure function in `transforms/timestamp.rs`
- [ ] **T-601** — Implement `ISO 8601 → Unix Timestamp` code action
- [ ] **T-602** — Implement `Unix Timestamp → Human Readable` code action (locale-aware)
- [ ] **T-603** — Detect epoch timestamps in selection (10-digit seconds, 13-digit milliseconds)
- [ ] **T-604** — Unit tests for timestamp operations (edge cases: negative epochs, Y2K38, milliseconds)

### EPIC-6.2: Number Base Conversions

- [ ] **T-610** — Implement `Decimal → Hex` code action
  - [ ] Pure function in `transforms/numbers.rs`
- [ ] **T-611** — Implement `Hex → Decimal` code action
- [ ] **T-612** — Implement `Decimal → Binary` code action
- [ ] **T-613** — Implement `Binary → Decimal` code action
- [ ] **T-614** — Implement `Decimal → Octal` code action
- [ ] **T-615** — Implement `Octal → Decimal` code action
- [ ] **T-616** — Auto-detect number base from prefix (`0x`, `0b`, `0o`)
- [ ] **T-617** — Unit tests for number conversions (large numbers, negative numbers, edge cases)

### EPIC-6.3: UUID & Random Generation

- [ ] **T-620** — Implement `Generate UUID v4` code action (inserts at cursor/replaces selection)
  - [ ] Pure function in `transforms/uuid.rs`
- [ ] **T-621** — Implement `Generate UUID v7` code action (time-sortable)
- [ ] **T-622** — Implement `Validate UUID` code action (shows version and validity as diagnostic)
- [ ] **T-623** — Implement `Generate Random String` code action (configurable length, charset via config)
- [ ] **T-624** — Unit tests for UUID operations

### EPIC-6.4: Regex & Pattern Operations

- [ ] **T-630** — Implement `Extract Emails` code action (finds all email addresses in selection)
  - [ ] Pure function in `transforms/extract.rs`
- [ ] **T-631** — Implement `Extract URLs` code action
- [ ] **T-632** — Implement `Extract IP Addresses` code action (v4 and v6)
- [ ] **T-633** — Implement `Mask Sensitive Data` code action (replace middle chars with `*` for emails, tokens)
- [ ] **T-634** — Unit tests for extraction operations

### EPIC-6.5: Text Diff & Comparison

- [ ] **T-640** — Implement `String Diff (Line)` code action (when two blocks separated by `---` or similar delimiter, show line diff)
  - [ ] Pure function in `transforms/diff.rs`
- [ ] **T-641** — Implement `String Diff (Character)` code action (char-level diff for short strings)
- [ ] **T-642** — Unit tests for diff operations

### EPIC-6.6: Compression

- [ ] **T-650** — Implement `Gzip Compress → Base64` code action
  - [ ] Pure function in `transforms/compress.rs`
- [ ] **T-651** — Implement `Base64 → Gzip Decompress` code action
- [ ] **T-652** — Implement `Deflate Compress → Base64` code action
- [ ] **T-653** — Implement `Base64 → Deflate Decompress` code action
- [ ] **T-654** — Unit tests for compression operations

### 🔒 GATE: ARI-4 Checkpoint (v1.0 Gate)

- [ ] **ARI-4** — Run `ariscan` — **minimum composite score ≥ 9.0**
  - [ ] Record scores in `docs/ari/ARI-4.md`
  - [ ] All pillars individually ≥ 8.0
  - [ ] Test Isolation ≥ 9.0 (gold standard for a pure-function codebase)
  - [ ] Security gate: **Pass**
  - [ ] Full ARI trajectory report: ARI-BASELINE → ARI-0 → ARI-1 → ARI-2 → ARI-3 → ARI-4
  - [ ] If below 9.0: continue as 0.x — do not stamp v1.0

### 🔍 AUDIT: Full Audit Suite (Pre v1.0)

- [ ] **A-060** — Code Quality Audit #3
  - [ ] Test coverage ≥ 90% on `transforms/`
  - [ ] Zero clippy warnings
  - [ ] No dead code
  - [ ] Consistent error handling
  - [ ] Document in `docs/audits/CODE-QUALITY-3.md`
- [ ] **A-061** — Security Audit #3
  - [ ] `cargo audit` clean
  - [ ] Extended fuzz testing (1 hour per decode target)
  - [ ] Review all error paths for information leakage
  - [ ] Document in `docs/audits/SECURITY-AUDIT-3.md`
- [ ] **A-062** — Architecture Audit #3
  - [ ] Module coherence review with 12+ transform modules
  - [ ] LSP handler still a thin dispatch layer?
  - [ ] Memory profiling under load
  - [ ] Document in `docs/audits/ARCH-AUDIT-3.md`
- [ ] **A-063** — Dependency Audit #4
  - [ ] Full dep tree review
  - [ ] License compliance
  - [ ] Supply chain assessment
  - [ ] Document in `docs/audits/DEP-AUDIT-4.md`
- [ ] **A-064** — UX Audit #3
  - [ ] Full feature walkthrough on macOS, Linux, Windows
  - [ ] Code action count sanity check (not overwhelming with 50+ actions?)
  - [ ] Performance audit with real-world file sizes
  - [ ] Document in `docs/audits/UX-AUDIT-3.md`

### 📋 PM REVIEW: PMR-5 — v1.0 Readiness Review

- [ ] **PMR-5** — Conduct v1.0 Readiness Review
  - [ ] Is the extension stable enough for a v1.0 commitment?
  - [ ] Are there any known bugs that would embarrass a 1.0 label?
  - [ ] Is the community healthy? (contributors, issue response time, discussion activity)
  - [ ] ARI ≥ 9.0 confirmed?
  - [ ] All audit findings from A-060–A-064 resolved?
  - [ ] Decision: **ship v1.0** or continue iterating as 0.x
  - [ ] If v1.0: define semantic versioning policy going forward (breaking changes = major bump)
  - [ ] Document decisions in `docs/pm-reviews/PMR-5.md`

**Phase 6 Exit Criteria:** Advanced features driven by community demand. ARI ≥ 9.0. Full audit suite passed. v1.0 decision made.

---

## Backlog & Parking Lot

> Ideas captured but not yet prioritised. Community upvotes and PMR-4 evidence drive promotion into a Phase.

- [ ] **B-001** — `ROT13` encode/decode (the gentleman's encryption)
- [ ] **B-002** — `Morse Code` encode/decode
- [ ] **B-003** — `NATO Phonetic Alphabet` conversion
- [ ] **B-004** — `Lorem Ipsum` generator (replace selection with N paragraphs)
- [ ] **B-005** — `Markdown → HTML` conversion
- [ ] **B-006** — `HTML → Markdown` conversion
- [ ] **B-007** — `CSV ↔ TSV` conversion
- [ ] **B-008** — `JSON Schema` generation from JSON sample
- [ ] **B-009** — `HMAC-SHA256` computation (requires key input — UX challenge)
- [ ] **B-010** — `QR Code` generation (output as Unicode block art)
- [ ] **B-011** — `Color Code` conversions (hex ↔ rgb ↔ hsl)
- [ ] **B-012** — `Slug` generation (URL-safe slugs from titles)
- [ ] **B-013** — `Emmet Abbreviation` expansion
- [ ] **B-014** — `SQL Formatter` (pretty print SQL)
- [ ] **B-015** — Custom user-defined transformations via config (pipe through shell command)

---

## Release Cadence

| Version | Phase | Target | Scope | Gate |
|---------|-------|--------|-------|------|
| `v0.1.0` | 0 + 1 | MVP | Bootstrap + Core encoding/decoding | ARI-0 ≥ 7.0, ARI-1 ≥ 7.5, PMR-0, PMR-1 |
| `v0.2.0` | 2 | +2 weeks | Hashing, JWT, JSON/YAML operations | Arch Audit #1, Security Audit #1, PMR-2 |
| `v0.3.0` | 3 | +2 weeks | Case conversions, text transforms | ARI-2 ≥ 8.0, Code Quality #2, UX Audit #1 |
| `v0.4.0` | 4 | +1 week | Configuration, performance, polish | Arch Audit #2, PMR-3 (scope lock) |
| `v0.5.0` | 5 | +1 week | Store publication, community setup | ARI-3 ≥ 8.5, Security #2, UX Audit #2 |
| `v1.0.0` | 6 | +4 weeks | Advanced features, stability | ARI-4 ≥ 9.0, Full audit suite, PMR-5 |

---

## Acceptance Criteria (Global)

Every ticket in this roadmap must satisfy the following before it is marked complete:

1. **Functional:** The operation produces the correct output for valid input.
2. **Error-safe:** Invalid input returns a `Result::Err` with structured `StringKnifeError`, never panics, never corrupts text.
3. **Tested:** Unit test covering happy path, edge cases, and error paths. Tests are isolated (no shared state, no I/O).
4. **Documented:** Operation listed in README feature table with description. Public function has rustdoc.
5. **Reversible:** Where applicable, encode/decode pairs roundtrip to identity.
6. **Performant:** Operation completes in <100ms for 100KB input.
7. **Multi-cursor compatible:** (Phase 4+) Works with multiple selections.
8. **ARI-compatible:** Transform is a pure function in its own module. No LSP types leak into transform logic.

---

## Technical Constraints & Decisions

Refer to the **Technical Architecture** section at the top of this document for the full architecture decision record, component design, data flow, dependency budget, performance model, and security model. The key constraints in summary:

- **LSP Protocol:** Code actions via `textDocument/codeAction` — the only path to context-menu integration in Zed's current extension API.
- **Three-layer separation:** WASM shim (Layer 1) → LSP router (Layer 2) → Transform engine (Layer 3). Arrows point downward only. `transforms/` has zero LSP dependencies.
- **Pure function supremacy:** Every transform is `fn(&str) -> Result<String, StringKnifeError>`. No I/O, no side effects, no shared state.
- **Rust everywhere:** Both the Zed WASM extension and the LSP binary are Rust. No Node.js runtime dependency.
- **Zero network, zero telemetry:** All operations are local, deterministic, and offline.
- **Cross-platform binaries:** macOS (Intel + ARM), Linux (x86_64 + ARM), Windows (x86_64). No FFI, no system library links.
- **Dependency budget:** < 150 transitive crates at v1.0. No `unsafe` in `transforms/`. All deps pass `cargo-deny`.
- **Performance contract:** < 100ms for 100KB input. > 1MB returns `InputTooLarge` error.
- **Agent-first:** Repository structure, test patterns, module boundaries, error types, and documentation designed for AI-agent consumption from commit zero. `ariscan` scores are a first-class engineering metric.

---

## Document Trail

| Directory | Contents |
|-----------|----------|
| `docs/ari/` | ARI checkpoint reports: ARI-BASELINE.md, ARI-0.md through ARI-4.md |
| `docs/pm-reviews/` | PM review decision records: PMR-0.md through PMR-5.md |
| `docs/audits/` | Audit reports: CODE-QUALITY-{N}.md, SECURITY-AUDIT-{N}.md, ARCH-AUDIT-{N}.md, DEP-AUDIT-{N}.md, UX-AUDIT-{N}.md |

---

*This document is the living source of truth for the StringKnife product. Update it as tickets are completed, PM reviews adjust priorities, and ariscan scores evolve.*
