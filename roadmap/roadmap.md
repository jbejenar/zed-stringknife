# Zed StringKnife — Product Roadmap

> **A surgical string transformation toolkit for the Zed editor, delivered as a Language Server Protocol extension with context-menu code actions.**

**Product Owner:** John Bejenariu
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
├── CLAUDE.md                  # Agent entry point — read this first
├── .claude/
│   └── skills/
│       └── vault-interaction/
│           └── SKILL.md       # Vault interaction protocol for Claude Code
│
├── .vault/                    # Obsidian-compatible knowledge vault
│   ├── .obsidian/
│   │   ├── app.json           # Obsidian settings (tracked)
│   │   └── graph.json         # Graph view colour groups (tracked)
│   ├── Home.md                # Master index — all sections within 2 hops
│   ├── README.md              # How to open as Obsidian vault
│   ├── architecture/          # Architecture Decision Records
│   │   └── System Context.md
│   ├── ari/                   # ARI pillar tracking
│   │   ├── ARI Dashboard.md
│   │   └── [per-pillar notes]
│   ├── sessions/              # Agent session continuity
│   │   ├── Session Log.md
│   │   └── NEXT-SESSION.md
│   ├── patterns/              # Codebase patterns & agent guides
│   │   ├── Adding a New Transform.md
│   │   ├── Gotchas.md
│   │   └── Dependency Budget.md
│   ├── transforms/            # Transform registry
│   │   └── Transform Registry.md
│   ├── pm-reviews/            # PM Review index
│   ├── audits/                # Audit index
│   └── templates/             # Session & ARI checkpoint templates
│       ├── Session Template.md
│       └── ARI Checkpoint Template.md
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
| 5 | **Documentation Density** | ≥ 6 | ≥ 8 | `.vault/` provides graph-navigable knowledge base with structured notes covering architecture, patterns, ARI tracking, and session continuity. HINTS.md, CONTRIBUTING.md, inline rustdoc on all public APIs. LCI-compatible doc structure. |
| 6 | **Dependency Transparency** | ≥ 8 | ≥ 9 | Minimal deps. `cargo-deny` for license/advisory audit. No transitive wildcards. |
| 7 | **Error Explicitness** | ≥ 8 | ≥ 9 | All transforms return `Result<T, E>` with structured error types. No panics. No silent failures. |
| 8 | **Security (Gate)** | Pass | Pass | `cargo-audit` in CI. No `unsafe`. No network calls. No file system access in transforms. |

### `ariscan` Checkpoint Schedule

| Checkpoint | When | Minimum ARI | Action on Fail |
|-----------|------|-------------|----------------|
| **ARI-0** | End of Phase 0 | ≥ 70/100 composite | Block Phase 1 entry. Fix pillar deficiencies. |
| **ARI-1** | End of Phase 1 | ≥ 75/100 composite | Block Phase 2 entry. Remediation sprint. |
| **ARI-2** | End of Phase 3 | ≥ 80/100 composite | Block Phase 4 entry. Architectural review if below. |
| **ARI-3** | Pre-publish (Phase 5) | ≥ 85/100 composite | Block store submission. Final hardening sprint. |
| **ARI-4** | Post v1.0 (Phase 6) | ≥ 90/100 composite | Continuous. Regressions flagged in CI. |

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

Each PM Review produces a **written decision record** (committed to `.vault/pm-reviews/PMR-{N}.md`) containing:
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

## PR & CI Gate Policy

All code changes enter the repository through Pull Requests. The following gates must pass before any PR can be merged.

### Required CI Checks (GitHub Actions)

Every PR triggers the CI pipeline (`.github/workflows/ci.yml`). **All checks are required — no merge without green.**

| Check | Command | Blocking | Phase Introduced |
|-------|---------|----------|-----------------|
| **Build (WASM)** | `cargo check -p stringknife-ext --target wasm32-wasip1` | Yes | Phase 0 |
| **Build (LSP)** | `cargo check -p stringknife-lsp` | Yes | Phase 0 |
| **Unit Tests** | `cargo test --workspace` | Yes | Phase 0 |
| **Lint (Clippy)** | `cargo clippy --workspace -- -D warnings` | Yes | Phase 0 |
| **Format** | `cargo fmt --all -- --check` | Yes | Phase 0 |
| **License/Advisory** | `cargo deny check` | Yes | Phase 0 |
| **Security Audit** | `cargo audit` | Yes | Phase 0 |
| **ARI Score** | `ariscan --format pr-comment` | Advisory (Phase 0–1), Blocking (Phase 2+) | Phase 0 |
| **ARI Regression** | `ariscan --diff main` | Blocking (Phase 2+) | Phase 2 |
| **Integration Tests** | `cargo test --test integration` | Yes (Phase 1+) | Phase 1 |
| **Benchmark Regression** | `cargo bench -- --compare main` | Advisory | Phase 4 |

### Branch Protection Rules

Branch protection is configured on `main` from Phase 0 onward.

- [ ] **Require PR reviews:** Minimum 1 approving review before merge
- [ ] **Require status checks:** All CI checks listed above must pass
- [ ] **Require branch up-to-date:** PR branch must be rebased on latest `main`
- [ ] **No direct pushes to `main`:** All changes go through PRs (including maintainer)
- [ ] **Require linear history:** Squash merge or rebase merge only — no merge commits
- [ ] **Require signed commits:** All commits must be GPG or SSH signed (Phase 2+)
- [ ] **Dismiss stale reviews:** Approvals dismissed when new commits are pushed
- [ ] **Require conversation resolution:** All review comments must be resolved before merge

### PR Process Checklist

Every PR must include the following before requesting review:

- [ ] **Title** follows conventional commit format: `feat:`, `fix:`, `refactor:`, `docs:`, `test:`, `chore:`
- [ ] **Description** explains *what* changed and *why*
- [ ] **Tests** — new/modified code has corresponding unit tests
- [ ] **No `unsafe`** in `transforms/` crate (enforced by Clippy deny)
- [ ] **No new dependencies** without justification in PR description and `cargo-deny` approval
- [ ] **ARI score** does not regress below phase threshold
- [ ] **Documentation** updated if public API changed (README feature table, rustdoc)
- [ ] **Breaking changes** flagged with `BREAKING:` prefix in commit message

### PR Labels & Automation

| Label | Trigger | Action |
|-------|---------|--------|
| `ci:passed` | All CI checks green | Auto-applied by GitHub Actions |
| `ari:regression` | ARI score decreased | Blocks merge, notifies maintainer |
| `ari:improvement` | ARI score increased | Informational — shown in PR comment |
| `needs-review` | PR opened/updated | Auto-applied, removed on approval |
| `size/S`, `size/M`, `size/L`, `size/XL` | Lines changed | Auto-applied by size labeler |
| `phase-N` | Files changed in phase scope | Auto-applied by path labeler |

### CI Gate Escalation by Phase

CI strictness increases as the project matures:

| Phase | ARI Blocking | Benchmark Gate | Coverage Gate |
|-------|-------------|----------------|---------------|
| **0–1** | Advisory only | None | None |
| **2–3** | Pillar scores ≥ phase target | Advisory | ≥ 70% on `transforms/` |
| **4** | Pillar scores ≥ phase target | Warn on >10% regression | ≥ 80% on `transforms/` |
| **5–6** | Pillar scores ≥ phase target | Block on >20% regression | ≥ 85% on `transforms/` |

---

## Phase 0 — Project Bootstrap

> **Goal:** Repository scaffolded, CI green, dev extension installable in Zed with a single no-op code action proving the full pipeline works end-to-end. ARI foundations laid from first commit.

### EPIC-0.1: Repository & Toolchain Setup

**Priority:** Critical | **Impact:** Very High | **Effort:** Medium | **Risk:** Low
**Source:** Product Roadmap v1 — Phase 0
**Status:** Done
**Dependencies:** None
**AI-first benefit:** Deterministic repo structure enables agents to navigate and contribute from first clone.

Establish the foundational repository structure, Zed extension manifest, and WASM crate so that the project compiles and can be installed as a dev extension in Zed. This is the skeleton upon which all subsequent phases build.

#### Definition of Done

- [x] **T-001** — Initialise Git repository with `main` branch protection rules
- [x] **T-002** — Create `extension.toml` manifest
  - [x] Set `id = "stringknife"`, `name = "StringKnife"`, `schema_version = 1`
  - [x] Add `description`, `authors`, `repository` fields
  - [x] Register language server entry: `[language_servers.stringknife-lsp]`
  - [x] Map language server to broad file types: `["Rust", "TypeScript", "JavaScript", "Python", "Go", "Ruby", "HTML", "CSS", "JSON", "TOML", "YAML", "Markdown", "Plain Text", "C", "C++", "Java", "Kotlin", "Swift", "Shell Script", "SQL", "Elixir", "PHP"]`
- [x] **T-003** — Create `Cargo.toml` for the Zed extension WASM crate
  - [x] Set `crate-type = ["cdylib"]`
  - [x] Add `zed_extension_api = "0.7.0"` dependency
- [x] **T-004** — Create `src/lib.rs` with minimal `Extension` trait implementation
  - [x] Implement `language_server_command()` to return path to bundled LSP binary
  - [x] Implement `language_server_initialization_options()` returning empty config
  - [x] Call `register_extension!` macro
- [x] **T-005** — Add `LICENSE` (MIT) at repository root
- [x] **T-006** — Create `.gitignore` (target/, node_modules/, *.wasm, .jj/)
- [x] **T-007** — Create `README.md` with project overview, installation instructions, and feature list placeholder
- [x] **T-008** — Create `CHANGELOG.md` with `## [Unreleased]` section
- [x] **T-009** — Create `CONTRIBUTING.md` with dev setup instructions

#### Verification

- [x] `cargo check` passes on the WASM crate without errors
- [x] `extension.toml` validates against Zed's extension schema
- [x] All files listed above exist at repository root
- [x] `.gitignore` excludes `target/`, `node_modules/`, `*.wasm`, `.jj/`

### EPIC-0.1A: Codebase Intelligence Vault (Persistent Agent Memory)

**Priority:** Critical | **Impact:** Very High | **Effort:** Medium | **Risk:** Low
**Source:** Roadmap Amendment — Codebase Intelligence Vault
**Status:** Done
**Dependencies:** EPIC-0.1
**AI-first benefit:** Graph-navigable, frontmatter-queryable knowledge base compounds with every coding session. Agents read it for context, write to it for continuity, and ariscan output lands in it for longitudinal tracking.

> The `.vault/` directory is an Obsidian-compatible knowledge vault that serves as
> the persistent memory layer for AI-agent sessions. It replaces flat documentation
> with a graph-navigable, frontmatter-queryable knowledge base that compounds with
> every coding session. Agents read it for context, write to it for continuity, and
> ariscan output lands in it for longitudinal tracking.

#### Definition of Done

- [x] **T-655** — Create `CLAUDE.md` at repository root
  - [x] 30-second architecture summary (under 80 lines)
  - [x] Link to `.vault/sessions/NEXT-SESSION.md` as the agent session entry point
  - [x] File map table: path → what it is → when to read it
  - [x] ARI gate thresholds table (one row per checkpoint)
  - [x] Key constraints checklist (project-specific hard rules agents must not violate)

- [x] **T-656** — Create `.vault/` directory structure and Obsidian config
  - [x] `.vault/.obsidian/app.json` — source mode, frontmatter visible, line numbers on
  - [x] `.vault/.obsidian/graph.json` — colour groups by tag: `#ari-pillar`, `#session`,
        `#pattern`, `#adr`, `#audit`, `#transform`
  - [x] `.vault/Home.md` — master index with wikilink navigation to all vault sections
  - [x] `.vault/README.md` — how to open as Obsidian vault, agent protocol summary,
        ariscan integration notes

- [x] **T-657** — Create `.vault/architecture/` — Architecture Decision Records
  - [x] One ADR per major architectural decision, using frontmatter:
        `type: adr`, `status: accepted|proposed|deprecated`, `tags: [adr, architecture]`
  - [x] Each ADR includes: status, context, decision, alternatives rejected, consequences,
        linked notes via wikilinks
  - [x] `System Context.md` — component summary, data flow, links to ADRs
  - [x] Migrate any existing ADRs from flat `docs/` into vault format with frontmatter

- [x] **T-658** — Create `.vault/ari/` — ARI Pillar Tracking
  - [x] `ARI Dashboard.md` — composite score trajectory table (one row per checkpoint),
        per-pillar score table, remediation queue section, links to checkpoint notes
  - [x] One note per ARI pillar (8 total), each with frontmatter fields:
        `pillar_number`, `current_score`, `target_phase0`, `target_v1`, `weight`
  - [x] Each pillar note includes: definition, project-specific strategy,
        "what good looks like" checklist, current findings section, linked notes
  - [x] Weight distribution per empirical research: Test Isolation + Build Determinism +
        Type Safety at `above-equal`; Security Gate as binary `gate`; remainder at `equal`
  - [x] Migrate any existing `.vault/ari/` content into vault notes

- [x] **T-659** — Create `.vault/sessions/` — Agent Session Infrastructure
  - [x] `Session Log.md` — chronological table: session #, date, agent, focus, outcome, link
  - [x] `NEXT-SESSION.md` — frontmatter: `current_phase`, `current_ticket`, `blocked_by`;
        sections: current state, what last agent did, what next agent should do,
        files to read first, environment notes
  - [x] Convention documented: agents create a session note on end, update NEXT-SESSION.md,
        add row to Session Log

- [x] **T-660** — Create `.vault/patterns/` — Codebase Patterns & Agent Guides
  - [x] `Adding a New Transform.md` — step-by-step with code templates and anti-patterns
  - [x] `Gotchas.md` — "don't touch this, it's deliberate" annotations. Architectural
        constraints that look like bugs. Intentional trade-offs.
  - [x] `Dependency Budget.md` — hard rules on what can be added, version policies,
        approved libraries, process for adding new dependencies.
  - [x] All pattern notes tagged `#pattern` with `type: pattern` in frontmatter

- [x] **T-661** — Create `.vault/transforms/` — Transform Registry
  - [x] `Transform Registry.md` — registry tracking all transforms with columns:
        Name, Module/File, Status, Tests, Ticket
  - [x] Pre-populate from roadmap tickets where possible
  - [x] Convention: update status to ✅ on implementation, add test count and commit SHA

- [x] **T-662** — Create `.vault/pm-reviews/` and `.vault/audits/` indexes
  - [x] `PM Reviews.md` — indexed table of all PM reviews with phase gate, status, link
  - [x] `Audit Index.md` — tables for each audit series (code quality, security,
        architecture, dependency, UX)
  - [x] Migrate any existing `.vault/pm-reviews/` and `.vault/audits/` references

- [x] **T-663** — Create `.vault/templates/`
  - [x] `Session Template.md` — frontmatter: session_number, agent, phase,
        tickets_attempted/completed/blocked; sections: objective, tickets worked,
        decisions made, gotchas discovered, ARI impact, handoff to next session
  - [x] `ARI Checkpoint Template.md` — frontmatter: checkpoint, composite_score,
        gate_threshold, gate_passed; sections: per-pillar scores with delta from
        previous, remediation items

- [x] **T-664** — Create `.claude/skills/vault-interaction/SKILL.md`
  - [x] Session start protocol (what to read, in what order)
  - [x] Frontmatter as structured data (explain the YAML contract)
  - [x] Wikilinks as navigation (explain `[[Note Name]]` convention)
  - [x] Session end protocol (create note, update handoff, update registry)
  - [x] Full `.vault/` file structure reference

- [x] **T-665** — Update `.gitignore` for vault
  - [x] Track: `.vault/.obsidian/app.json`, `.vault/.obsidian/graph.json`
  - [x] Ignore: `.vault/.obsidian/workspace.json`, `workspace-mobile.json`,
        `hotkeys.json`, `community-plugins.json`, `core-plugins.json`,
        `core-plugins-migration.json`, `plugins/`

- [x] **T-666** — Update `HINTS.md` to reference vault
  - [x] Add "Vault Maintenance" section: agents must update session state (not optional)
  - [x] Add "ARI Dashboard is manually updated" note (human review required at this stage)
  - [x] Add project-specific intentional suppressions

- [x] **T-667** — Verify vault graph connectivity
  - [x] Open `.vault/` as Obsidian vault — confirm graph view renders with colour-coded nodes
  - [x] Confirm all wikilinks resolve (no broken `[[...]]` references) — verified programmatically: 19 wikilinks, 0 broken
  - [x] Confirm `Home.md` reaches every section within 2 hops — verified: all nodes reachable
  - [x] Confirm frontmatter renders correctly in Obsidian's properties view

#### Verification

- [x] `.vault/` opens as Obsidian vault with connected graph and colour-coded nodes
- [x] All wikilinks resolve (no broken `[[...]]` references) — 19 wikilinks verified
- [x] `CLAUDE.md` contains architecture summary, file map, ARI gates, and vault protocol
- [x] `HINTS.md` references vault maintenance convention
- [x] `.gitignore` correctly tracks/ignores Obsidian config files
- [x] `Home.md` reaches every vault section within 2 hops — verified programmatically
- [x] Frontmatter renders correctly in Obsidian's properties view

### EPIC-0.2: ARI Foundations (Agent-Readiness from Day One)

**Priority:** Critical | **Impact:** Very High | **Effort:** Medium | **Risk:** Low
**Source:** Product Roadmap v1 — Phase 0
**Status:** Done
**Dependencies:** EPIC-0.1, EPIC-0.1A
**AI-first benefit:** ARI-first setup ensures agents can reason about, test, and contribute to the codebase from the earliest commits.

Lay the agent-readiness infrastructure: HINTS.md for LCI-compatible context, strict Clippy lints, structured error types, the transforms module skeleton, and cargo-deny/cargo-audit integration. Establish the ARI baseline score.

#### Definition of Done

- [x] **T-025** — Create `HINTS.md` at repository root
  - [x] Document repo structure and purpose of each directory
  - [x] Document the LSP ↔ WASM extension architecture
  - [x] Document how to add a new string operation (step-by-step)
  - [x] Document test patterns and conventions
- [x] **T-026** — Create `rust-toolchain.toml` pinning stable channel (Build Determinism)
- [x] **T-027** — Commit `Cargo.lock` to version control (Build Determinism)
- [x] **T-028** — Configure strict Clippy lints in workspace `Cargo.toml` or `.clippy.toml`
  - [x] `#![deny(clippy::unwrap_used)]` in library code
  - [x] `#![deny(clippy::panic)]` in library code
  - [x] `#![warn(clippy::pedantic)]`
- [x] **T-029** — Define `StringKnifeError` enum with structured error variants (Error Explicitness)
  - [x] `InvalidInput { operation: String, reason: String }`
  - [x] `UnsupportedEncoding { encoding: String }`
  - [x] `InputTooLarge { max_bytes: usize, actual_bytes: usize }`
  - [x] Implement `Display` and `std::error::Error`
- [x] **T-030** — Create `transforms/` module directory with `mod.rs` (Modular Coherence)
  - [x] Each transform category gets its own submodule file
  - [x] All transforms are pure functions: `fn(input: &str) -> Result<String, StringKnifeError>`
  - [x] No LSP types, no I/O, no side effects in transform modules
- [x] **T-031** — Add `cargo-deny` configuration (`deny.toml`)
  - [x] License allowlist: MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause, ISC, Zlib
  - [x] Advisory database check enabled
  - [x] Duplicate crate detection enabled
- [x] **T-032** — Add `cargo-audit` to CI pipeline (Security gate)
- [x] **T-033** — Add rustdoc comments on all public types and functions (Documentation Density)
- [x] **T-034** — Install and run `ariscan` against the repo — establish **ARI-BASELINE** score
  - [x] Record baseline scores per pillar in `.vault/ari/ARI-BASELINE.md`
  - [x] Identify pillars below target and create remediation items (P2=19, P5=25, P6=50)

#### Verification

- [x] `cargo clippy -- -D warnings` passes with zero warnings
- [x] `cargo deny check` passes with zero violations
- [x] `ariscan` produces a valid ARI-BASELINE report — 59/100 (L3 Capable)
- [x] `HINTS.md` contains all four required documentation sections
- [x] `StringKnifeError` compiles with all three variants and `Display` impl

### EPIC-0.3: Language Server Skeleton

**Priority:** Critical | **Impact:** Very High | **Effort:** High | **Risk:** Medium
**Source:** Product Roadmap v1 — Phase 0
**Status:** Done
**Dependencies:** EPIC-0.1
**AI-first benefit:** Clean LSP skeleton with typed handlers enables agents to add new code actions by following established patterns.

Build the minimal LSP server binary that speaks the Language Server Protocol over stdio. This includes the initialize handshake, document sync, and an empty code action handler — proving the LSP ↔ Zed communication pipeline works.

#### Definition of Done

- [x] **T-010** — Create `lsp/` directory for the LSP binary crate *(named `stringknife-lsp/`)*
- [x] **T-011** — Create `lsp/Cargo.toml`
  - [x] Add dependencies: `tower-lsp`, `tokio`, `serde`, `serde_json`
  - [x] Set binary name: `stringknife-lsp`
- [x] **T-012** — Implement minimal LSP server in `lsp/src/main.rs`
  - [x] Implement `initialize` handler returning server capabilities
  - [x] Declare `codeActionProvider = true` in capabilities
  - [x] Declare `textDocumentSync` as `Full` (needed to access document text)
  - [x] Implement `textDocument/didOpen` handler to store document text
  - [x] Implement `textDocument/didChange` handler to update stored text
  - [x] Implement `textDocument/codeAction` handler with Reverse String action
  - [x] Implement `shutdown` handler
- [x] **T-013** — Add document text store (HashMap<Url, String>) to server state
- [x] **T-014** — Verify LSP binary compiles and runs standalone with `--stdio` flag
- [x] **T-015** — Wire extension WASM to download/locate the LSP binary
  - [x] For dev: point to local `target/release/stringknife-lsp` via `worktree.which()`
  - [ ] For published: implement binary download from GitHub Releases via `zed::download_file()` *(deferred to release phase)*

#### Verification

- [x] `cargo build -p stringknife-lsp` compiles without errors
- [x] `stringknife-lsp --stdio` starts and responds to LSP initialize request
- [x] Document store correctly tracks open/changed documents
- [ ] Extension WASM locates and launches the LSP binary in dev mode *(manual Zed verification)*

### EPIC-0.4: End-to-End Proof of Life

**Priority:** Critical | **Impact:** Very High | **Effort:** Medium | **Risk:** Medium
**Source:** Product Roadmap v1 — Phase 0
**Status:** Done (code complete — T-018/T-019/T-020 need manual Zed verification)
**Dependencies:** EPIC-0.2, EPIC-0.3
**AI-first benefit:** Proves the full pipeline end-to-end, giving agents a working reference implementation to pattern-match against.

Wire a single hardcoded code action ("Reverse String") through the entire stack: pure function in transforms → LSP code action handler → Zed context menu → text replacement. This proves the architecture works before investing in the full transform catalogue.

#### Definition of Done

- [x] **T-016** — Add a single hardcoded code action: "StringKnife: Reverse String"
  - [x] Implement as a pure function in `transforms/misc.rs`
  - [x] Wire into LSP code action handler
  - [x] Extract selected text range from `CodeActionParams`
  - [x] Return `CodeAction` with `WorkspaceEdit` replacing the selection range
- [x] **T-017** — Add unit test for reverse string transform (isolated, no LSP dependency)
- [ ] **T-018** — Install as dev extension in Zed (`zed: install dev extension`) *(manual verification)*
- [ ] **T-019** — Verify code action appears in context menu when text is selected *(manual verification)*
- [ ] **T-020** — Verify selecting the action replaces text correctly *(manual verification)*
- [x] **T-035** — Document the dev install workflow in `CONTRIBUTING.md`

#### Verification

- [x] `cargo test -p stringknife-core` passes with reverse string tests green (6 tests)
- [ ] Dev extension installed in Zed shows "StringKnife: Reverse String" in context menu *(manual)*
- [ ] Selecting text and applying the action replaces it with the reversed string *(manual)*
- [ ] Undo (Cmd+Z / Ctrl+Z) restores the original text *(manual)*

### EPIC-0.5: CI/CD Pipeline

**Priority:** Critical | **Impact:** High | **Effort:** Medium | **Risk:** Low
**Source:** Product Roadmap v1 — Phase 0
**Status:** Done (except T-023 ariscan CI, T-038 labeler, T-039 merge queue)
**Dependencies:** EPIC-0.1, EPIC-0.2, EPIC-0.3
**AI-first benefit:** Automated CI with ariscan integration gives agents immediate feedback on whether their changes maintain quality standards.

Set up GitHub Actions workflows for continuous integration (build, test, lint, audit), release automation (cross-compile for all platforms), ARI scoring on PRs, and Dependabot for dependency updates.

#### Definition of Done

- [x] **T-021** — Create `.github/workflows/ci.yml`
  - [x] Run `cargo check` on both WASM crate and LSP crate
  - [x] Run `cargo test` on LSP crate
  - [x] Run `cargo clippy` with `-D warnings`
  - [x] Run `cargo fmt --check`
  - [x] Run `cargo deny check` (license + advisory)
  - [x] Run `cargo audit` (security)
  - [x] Run `ariscan` and output ARI score summary — via separate `ariscan.yml` workflow
- [x] **T-022** — Create `.github/workflows/release.yml`
  - [x] Trigger on Git tag `v*`
  - [x] Cross-compile LSP binary for `x86_64-apple-darwin`, `aarch64-apple-darwin`, `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, `x86_64-pc-windows-msvc`
  - [x] Upload binaries as GitHub Release assets
  - [x] Generate checksums (SHA256)
- [x] **T-023** — Create `.github/workflows/ariscan.yml`
  - [x] Run `ariscan` on every PR (clones prontiq-ariscan, builds, scans)
  - [x] Post ARI score as PR comment (per-pillar breakdown with delta)
  - [ ] Fail PR if any pillar drops below its phase target threshold *(deferred — needs gate logic)*
  - [x] Cache previous score for delta comparison (scans base branch for diff)
- [x] **T-024** — Add Dependabot config for Cargo dependency updates
- [x] **T-036** — Configure branch protection rules on `main`
  - [x] Require 1 approving PR review
  - [x] Require all CI status checks to pass
  - [x] Require branch to be up-to-date before merge
  - [x] Enforce squash merge (linear history)
  - [x] Disable direct pushes to `main`
  - [x] Enable dismiss stale reviews on new pushes
  - [x] Require conversation resolution before merge
- [x] **T-037** — Create `.github/pull_request_template.md`
  - [x] Include PR checklist (tests, no unsafe, docs updated, ARI check)
  - [x] Include conventional commit format guidance
  - [x] Include breaking change flag instructions
- [ ] **T-038** — Create `.github/workflows/pr-labeler.yml` *(deferred — low priority)*
  - [ ] Auto-label PRs by size (S/M/L/XL)
  - [ ] Auto-label PRs by phase scope based on changed file paths
  - [ ] Apply `needs-review` label on PR open
- [ ] **T-039** — Add merge queue configuration (optional, enabled when contributor count > 1)

#### Verification

- [x] Push to `main` triggers CI workflow and all steps pass — confirmed via PR #3 merge
- [ ] Tagging `v0.0.1-test` triggers release workflow and produces binaries for all 5 targets *(not yet tested)*
- [x] Opening a PR triggers ariscan workflow and posts ARI score comment
- [x] Dependabot opens a PR within 7 days for any outdated dependency — confirmed (actions/checkout, upload/download-artifact PRs opened)
- [x] Direct push to `main` is rejected by branch protection — confirmed
- [x] PR without passing CI checks cannot be merged — confirmed (PR #3 required fix before merge)
- [x] PR template renders correctly when opening a new PR — confirmed
- [ ] PR labels are automatically applied based on size and scope *(T-038 deferred)*

### 🔒 GATE: ARI-0 Checkpoint

**Priority:** Critical | **Impact:** Very High | **Effort:** Small | **Risk:** Medium
**Source:** ARI Governance — Phase 0 exit gate
**Status:** Not Started
**Dependencies:** EPIC-0.1, EPIC-0.2, EPIC-0.3, EPIC-0.4, EPIC-0.5

#### Definition of Done

- [ ] **ARI-0** — Run `ariscan` — **minimum composite score ≥ 70/100**
  - [ ] Record scores in `.vault/ari/ARI-0.md`
  - [ ] All 8 pillars individually ≥ 30/100
  - [ ] Security pillar: **Pass** (no advisories, no unsafe, no panics in lib)
  - [ ] If below threshold: create remediation tickets, block Phase 1 entry
  - **Current baseline:** 59/100 (L3 Capable) — 11 points below gate threshold
  - **Weakest pillars:** P2 Feedback Loop (19), P5 Doc Machine-Readability (25), P6 Build Determinism (50)
  - **Note:** P2 and P6 are artificially low due to ariscan's Node.js bias (see B-016, B-018)

#### Verification

- [ ] `.vault/ari/ARI-0.md` committed with per-pillar scores
- [ ] Composite ARI score ≥ 70/100 confirmed
- [ ] No pillar below 30/100

### 🔍 AUDIT: Dependency Audit #1

**Priority:** High | **Impact:** High | **Effort:** Small | **Risk:** Low
**Source:** Audit Schedule — Phase 0
**Status:** Done
**Dependencies:** EPIC-0.3 (LSP crate with dependencies established)

#### Definition of Done

- [x] **A-001** — Run `cargo deny check` and review all transitive dependencies
  - [x] Document total dependency count in `.vault/audits/DEP-AUDIT-1.md`
  - [x] Flag any dependency with > 6 months since last release
  - [x] Flag any dependency with known CVE (even if not directly exploitable)
  - [x] Confirm all licenses compatible with MIT

#### Verification

- [x] `.vault/audits/DEP-AUDIT-1.md` committed with dependency count and flagged items
- [x] `cargo deny check` returns zero violations

### 📋 PM REVIEW: PMR-0 — Foundation Review

**Priority:** High | **Impact:** Very High | **Effort:** Small | **Risk:** Low
**Source:** PM Governance Cadence
**Status:** Not Started
**Dependencies:** EPIC-0.4 (proof of life complete), ARI-0 (checkpoint passed)

#### Definition of Done

- [ ] **PMR-0** — Conduct Foundation Review
  - [ ] Validate LSP code action architecture works reliably in Zed
  - [ ] Confirm no Zed API changes that invalidate the approach
  - [ ] Review Phase 1 scope: are these the right encoding operations?
  - [ ] Check Zed extension store for any competing string utility extensions
  - [ ] Decision: Go/No-Go for Phase 1
  - [ ] Document decisions in `.vault/pm-reviews/PMR-0.md`

#### Verification

- [ ] `.vault/pm-reviews/PMR-0.md` committed with Go/No-Go decision and evidence basis
- [ ] Phase 1 scope confirmed or adjusted based on review

**Phase 0 Exit Criteria:** Dev extension installs in Zed. Selecting text → right-click → "StringKnife: Reverse String" works. CI is green. Branch protection active on `main`. All PRs pass required CI checks. `.vault/` opens in Obsidian with connected graph, all wikilinks resolve, and NEXT-SESSION.md contains valid handoff state. ARI ≥ 70/100. PMR-0 complete.

---

## Phase 1 — Core Encoding & Decoding

> **Goal:** Ship the essential encoding/decoding operations that cover 90% of daily string manipulation needs.

### EPIC-1.1: Base64 Operations

**Priority:** Critical | **Impact:** Very High | **Effort:** Medium | **Risk:** Low
**Source:** Product Roadmap v1 — Phase 1
**Status:** Done
**Dependencies:** EPIC-0.4 (proof of life pattern established)
**AI-first benefit:** First real transform module establishes the pure-function pattern that agents replicate for every subsequent transform.

Implement Base64 standard and URL-safe encoding/decoding as the first production transforms. These are the highest-frequency string operations developers perform and serve as the template for all subsequent transforms.

#### Definition of Done

- [x] **T-100** — Implement `Base64 Encode` code action
  - [x] Pure function in `transforms/base64.rs`
  - [x] Standard Base64 (RFC 4648)
  - [x] Handle UTF-8 input correctly
  - [x] Preserve line selection range for replacement
- [x] **T-101** — Implement `Base64 Decode` code action
  - [x] Return `StringKnifeError::InvalidInput` for invalid Base64 (no panics, no crashes)
  - [x] Support padded and unpadded input
- [x] **T-102** — Implement `Base64URL Encode` code action (URL-safe alphabet, no padding)
- [x] **T-103** — Implement `Base64URL Decode` code action
- [x] **T-104** — Unit tests for all Base64 variants (20 tests)
  - [x] Empty string
  - [x] ASCII input
  - [x] Unicode/UTF-8 multi-byte input
  - [x] Roundtrip encode→decode identity
  - [x] Invalid input error paths (returns `Err`, never panics)

#### Verification

- [x] `cargo test -p stringknife-core -- base64` passes all 20 tests
- [x] Roundtrip identity: `decode(encode(x)) == x` for all valid inputs
- [x] Invalid Base64 input returns `Err(StringKnifeError::InvalidInput)`, never panics
- [ ] Code action appears in Zed context menu and replaces text correctly *(manual verification)*

### EPIC-1.2: URL Encoding Operations

**Priority:** Critical | **Impact:** Very High | **Effort:** Medium | **Risk:** Low
**Source:** Product Roadmap v1 — Phase 1
**Status:** Done
**Dependencies:** EPIC-0.4
**AI-first benefit:** Uniform transform signature makes URL operations immediately discoverable and testable by agents.

Implement URL percent-encoding and decoding per RFC 3986, including component encoding. These are essential for web development workflows.

#### Definition of Done

- [x] **T-110** — Implement `URL Encode` code action (percent-encoding, RFC 3986)
  - [x] Pure function in `transforms/url.rs`
- [x] **T-111** — Implement `URL Decode` code action
  - [x] Handle `+` as space (form encoding) and `%20` as space (URI encoding)
- [x] **T-112** — Implement `URL Encode (Component)` code action (encodes everything except unreserved chars)
- [x] **T-113** — Unit tests for URL encoding (19 tests)
  - [x] Reserved characters: `! # $ & ' ( ) * + , / : ; = ? @ [ ]`
  - [x] Unicode characters
  - [x] Already-encoded input (double-encoding prevention awareness — document behavior)
  - [x] Roundtrip identity

#### Verification

- [x] `cargo test -p stringknife-core -- url` passes all 19 tests
- [x] RFC 3986 reserved characters are correctly percent-encoded
- [x] `+` and `%20` both decode to space correctly
- [ ] Code action works in Zed context menu *(manual verification)*

### 📋 PM REVIEW: PMR-1 — MVP Scope Check (Mid-Phase)

**Priority:** High | **Impact:** Very High | **Effort:** Small | **Risk:** Low
**Source:** PM Governance Cadence
**Status:** Not Started
**Dependencies:** EPIC-1.1, EPIC-1.2 (first two transform EPICs shipped)

#### Definition of Done

- [ ] **PMR-1** — Conduct MVP Scope Review
  - [ ] User-test with 2–3 developers: are Base64 and URL the right first operations?
  - [ ] Review Zed extension store: any new competitors since PMR-0?
  - [ ] Assess: should HTML entities be cut in favour of something more requested?
  - [ ] Assess: is hex encoding worth its priority slot or should it move to Phase 2?
  - [ ] Reprioritise remaining Phase 1 EPICs if needed
  - [ ] Promote any backlog items that early users are requesting
  - [ ] Document decisions in `.vault/pm-reviews/PMR-1.md`

#### Verification

- [ ] `.vault/pm-reviews/PMR-1.md` committed with scope decisions and user feedback summary
- [ ] Remaining Phase 1 EPICs confirmed or reprioritised

### EPIC-1.3: HTML Entity Operations

**Priority:** High | **Impact:** High | **Effort:** Medium | **Risk:** Low
**Source:** Product Roadmap v1 — Phase 1
**Status:** Done
**Dependencies:** EPIC-0.4
**AI-first benefit:** Follows established transform pattern — agents can implement by analogy with Base64 module.

Implement HTML entity encoding and decoding, supporting named, decimal, and hex numeric entities. Essential for web developers working with HTML content.

#### Definition of Done

- [x] **T-120** — Implement `HTML Encode` code action
  - [x] Pure function in `transforms/html.rs`
  - [x] Encode `& < > " '` to named entities
  - [ ] Option: encode all non-ASCII to numeric entities *(deferred — not commonly needed)*
- [x] **T-121** — Implement `HTML Decode` code action
  - [x] Support named entities (`&amp;`, `&lt;`, `&gt;`, `&quot;`, `&apos;`, `&nbsp;`)
  - [x] Support decimal numeric entities (`&#123;`)
  - [x] Support hex numeric entities (`&#x7B;`)
- [x] **T-122** — Unit tests for HTML entities (16 tests)
  - [x] Nested/compound encoding
  - [x] Malformed entities (pass through unchanged)

#### Verification

- [x] `cargo test -p stringknife-core -- html` passes all 16 tests
- [x] All five named entities (`& < > " '`) encode and decode correctly
- [x] Malformed entities pass through unchanged without error
- [ ] Code action works in Zed context menu *(manual verification)*

### EPIC-1.4: Hex Operations

**Priority:** High | **Impact:** High | **Effort:** Small | **Risk:** Low
**Source:** Product Roadmap v1 — Phase 1
**Status:** Done
**Dependencies:** EPIC-0.4
**AI-first benefit:** Small, self-contained module — ideal for agent-driven implementation following existing patterns.

Implement hex encoding (UTF-8 bytes to hex string) and decoding (hex string to UTF-8 text), with support for `0x` prefix and space-separated byte formats.

#### Definition of Done

- [x] **T-130** — Implement `Hex Encode` code action (UTF-8 bytes → hex string)
  - [x] Pure function in `transforms/hex.rs`
- [x] **T-131** — Implement `Hex Decode` code action (hex string → UTF-8 text)
  - [x] Support with/without `0x` prefix
  - [x] Support with/without space-separated bytes
  - [x] Error on invalid hex characters
- [x] **T-132** — Unit tests for hex operations (16 tests)

#### Verification

- [x] `cargo test -p stringknife-core -- hex` passes all 16 tests
- [x] `0x` prefix handled correctly in decode
- [x] Invalid hex characters return `Err`, never panic
- [x] Roundtrip identity confirmed

### EPIC-1.5: Unicode Operations

**Priority:** High | **Impact:** High | **Effort:** Medium | **Risk:** Low
**Source:** Product Roadmap v1 — Phase 1
**Status:** Done
**Dependencies:** EPIC-0.4
**AI-first benefit:** Unicode edge cases (emoji, BMP, combining chars) provide rich test vectors for agent-generated tests.

Implement Unicode escape/unescape operations and a codepoint inspector. Supports `\uXXXX` (JavaScript/Java) and `\U{XXXXXX}` (chars above BMP) formats.

#### Definition of Done

- [x] **T-140** — Implement `Unicode Escape` code action (`Hello` → `\u0048\u0065\u006C\u006C\u006F`)
  - [x] Pure function in `transforms/unicode.rs`
  - [x] Support `\uXXXX` format (JavaScript/Java style)
  - [x] Support `\U{XXXXXX}` for chars above BMP
- [x] **T-141** — Implement `Unicode Unescape` code action
  - [x] Parse `\uXXXX` and `\U{XXXXXX}` sequences
  - [x] Leave non-escape text unchanged
- [x] **T-142** — Implement `Show Unicode Codepoints` code action (replaces with U+XXXX format)
- [x] **T-143** — Unit tests for Unicode operations (25 tests)
  - [x] Emoji (multi-codepoint sequences)
  - [x] CJK characters
  - [x] Combining characters

#### Verification

- [x] `cargo test -p stringknife-core -- unicode` passes all 25 tests
- [x] Emoji and multi-codepoint sequences roundtrip correctly
- [x] Codepoint inspector displays correct U+XXXX values
- [x] Non-escape text preserved unchanged during unescape

### EPIC-1.6: Code Action Categorisation & UX

**Priority:** Critical | **Impact:** Very High | **Effort:** Medium | **Risk:** Medium
**Source:** Product Roadmap v1 — Phase 1
**Status:** Not Started
**Dependencies:** EPIC-1.1, EPIC-1.2, EPIC-1.3, EPIC-1.4, EPIC-1.5
**AI-first benefit:** Smart detection heuristics make the extension self-organising — agents can add new detection patterns by following existing ones.

Implement the smart detection system that surfaces relevant decode actions based on the selected text pattern, group all actions under the "StringKnife:" prefix, and handle edge cases like multi-line and empty selections.

#### Definition of Done

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

#### Verification

- [ ] Selecting Base64 text surfaces "Base64 Decode" at top of context menu
- [ ] Selecting URL-encoded text surfaces "URL Decode" at top
- [ ] All encode actions always visible regardless of selection
- [ ] Empty selection returns zero code actions
- [ ] Multi-line selection produces correct WorkspaceEdit range

### 🔒 GATE: ARI-1 Checkpoint

**Priority:** Critical | **Impact:** Very High | **Effort:** Small | **Risk:** Medium
**Source:** ARI Governance — Phase 1 exit gate
**Status:** Not Started
**Dependencies:** EPIC-1.1, EPIC-1.2, EPIC-1.3, EPIC-1.4, EPIC-1.5, EPIC-1.6

#### Definition of Done

- [ ] **ARI-1** — Run `ariscan` — **minimum composite score ≥ 75/100**
  - [ ] Record scores in `.vault/ari/ARI-1.md`
  - [ ] Test Isolation pillar ≥ 80/100 (pure function transforms must be trivially testable)
  - [ ] Modular Coherence pillar ≥ 70/100 (transforms cleanly separated from LSP wiring)
  - [ ] Compare delta against ARI-0 — no pillar should have regressed
  - [ ] If below threshold: remediation sprint before Phase 2 entry

#### Verification

- [ ] `.vault/ari/ARI-1.md` committed with per-pillar scores and delta from ARI-0
- [ ] Composite ARI score ≥ 75/100 confirmed
- [ ] No pillar regression from ARI-0

### 🔍 AUDIT: Code Quality Audit #1

**Priority:** High | **Impact:** High | **Effort:** Small | **Risk:** Low
**Source:** Audit Schedule — Phase 1
**Status:** Not Started
**Dependencies:** EPIC-1.1, EPIC-1.2, EPIC-1.3, EPIC-1.4, EPIC-1.5, EPIC-1.6

#### Definition of Done

- [ ] **A-010** — Code Quality Audit
  - [ ] Run `cargo clippy` — zero warnings
  - [ ] Measure test coverage with `cargo-tarpaulin` — target ≥ 80% on `transforms/` module
  - [ ] Check for code duplication across transform modules (extract shared patterns)
  - [ ] Verify all public functions have rustdoc comments
  - [ ] Document findings in `.vault/audits/CODE-QUALITY-1.md`

#### Verification

- [ ] `.vault/audits/CODE-QUALITY-1.md` committed with coverage metrics and findings
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥ 80% on `transforms/` confirmed

**Phase 1 Exit Criteria:** All encoding/decoding actions work. Smart detection suggests relevant decode operations. Full unit test coverage. All CI checks pass on every PR. Integration tests added to CI pipeline. ARI ≥ 75/100. PMR-1 complete.

---

## Phase 2 — Hashing, Cryptographic & Data Format Operations

> **Goal:** Expand into hashing, JWT inspection, and data format conversions that developers reach for daily.

### EPIC-2.1: Hash Operations (One-Way)

**Priority:** High | **Impact:** High | **Effort:** Medium | **Risk:** Low
**Source:** Product Roadmap v1 — Phase 2
**Status:** Not Started
**Dependencies:** EPIC-0.4 (transform pattern established)
**AI-first benefit:** Hash transforms are stateless and deterministic — perfect for agent-generated test vectors from known RFCs.

Implement one-way hash operations (MD5, SHA-1, SHA-256, SHA-512, CRC32) as code actions. Each replaces the selected text with its hex digest. These are developer-essential for verifying checksums, debugging API signatures, and quick hash comparisons.

#### Definition of Done

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

#### Verification

- [ ] `cargo test -p transforms -- hash` passes all tests
- [ ] MD5/SHA outputs match NIST test vectors exactly
- [ ] Empty string produces correct hash for each algorithm
- [ ] Code actions work in Zed context menu

### EPIC-2.2: JWT Operations (Read-Only Decode)

**Priority:** High | **Impact:** High | **Effort:** Medium | **Risk:** Low
**Source:** Product Roadmap v1 — Phase 2
**Status:** Not Started
**Dependencies:** EPIC-0.4, EPIC-1.1 (Base64 decode needed internally)
**AI-first benefit:** JWT decode is a composition of existing Base64 + JSON transforms — demonstrates module composability.

Implement read-only JWT decoding (header, payload, full). No signature verification — only structural decode. This is a differentiating feature for developers debugging OAuth flows, API tokens, and authentication issues.

#### Definition of Done

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

#### Verification

- [ ] `cargo test -p transforms -- jwt` passes all tests
- [ ] Valid JWT decodes to correct header and payload JSON
- [ ] Malformed JWT returns structured error, never panics
- [ ] Timestamp fields (`exp`, `iat`, `nbf`) display as human-readable dates

### EPIC-2.3: JSON Operations

**Priority:** Critical | **Impact:** Very High | **Effort:** Medium | **Risk:** Low
**Source:** Product Roadmap v1 — Phase 2
**Status:** Not Started
**Dependencies:** EPIC-0.4
**AI-first benefit:** JSON transforms are the most frequently used data format operations — high-value, low-risk for agent contributions.

Implement JSON pretty print, minify, escape/unescape, and cross-format conversions (JSON ↔ YAML). These are the most commonly needed data format operations for developers working with APIs, configs, and data pipelines.

#### Definition of Done

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

#### Verification

- [ ] `cargo test -p transforms -- json` passes all tests
- [ ] Pretty print produces valid, readable JSON with 2-space indent
- [ ] Minify removes all unnecessary whitespace
- [ ] JSON ↔ YAML roundtrip preserves data integrity
- [ ] Invalid JSON returns structured error

### EPIC-2.4: XML/HTML Operations

**Priority:** Medium | **Impact:** Medium | **Effort:** Small | **Risk:** Low
**Source:** Product Roadmap v1 — Phase 2
**Status:** Not Started
**Dependencies:** EPIC-0.4
**AI-first benefit:** Small, isolated module with clear input/output — straightforward for agent implementation.

Implement XML pretty print and minify operations for developers working with XML configs, SOAP services, or HTML templates.

#### Definition of Done

- [ ] **T-230** — Implement `XML Pretty Print` code action
  - [ ] Pure function in `transforms/xml.rs`
- [ ] **T-231** — Implement `XML Minify` code action
- [ ] **T-232** — Unit tests for XML operations

#### Verification

- [ ] `cargo test -p transforms -- xml` passes all tests
- [ ] Nested XML elements are correctly indented
- [ ] Minified XML is valid and parseable

### EPIC-2.5: TOML/CSV Utility Operations

**Priority:** Medium | **Impact:** Medium | **Effort:** Small | **Risk:** Low
**Source:** Product Roadmap v1 — Phase 2
**Status:** Not Started
**Dependencies:** EPIC-0.4
**AI-first benefit:** Format conversion transforms are pure data-in/data-out — trivially testable by agents.

Implement cross-format conversions: TOML ↔ JSON and CSV → JSON Array. These fill common gaps when developers need to convert between configuration and data formats.

#### Definition of Done

- [ ] **T-240** — Implement `TOML → JSON` code action
- [ ] **T-241** — Implement `JSON → TOML` code action
- [ ] **T-242** — Implement `CSV → JSON Array` code action (first row as headers)
- [ ] **T-243** — Unit tests for format conversion operations

#### Verification

- [ ] `cargo test -p transforms` passes for TOML and CSV tests
- [ ] TOML ↔ JSON roundtrip preserves data types
- [ ] CSV with headers correctly maps to JSON array of objects

### 🔍 AUDIT: Architecture Audit #1

**Priority:** High | **Impact:** High | **Effort:** Medium | **Risk:** Low
**Source:** Audit Schedule — Phase 2
**Status:** Not Started
**Dependencies:** EPIC-2.1, EPIC-2.2, EPIC-2.3, EPIC-2.4, EPIC-2.5

#### Definition of Done

- [ ] **A-020** — Architecture Audit
  - [ ] Review module boundaries: are transforms fully decoupled from LSP types?
  - [ ] Review LSP handler: is it a thin dispatch layer or accumulating logic?
  - [ ] Profile code action response latency for each operation (target < 50ms for 10KB input)
  - [ ] Review dependency tree: any unnecessary transitive deps introduced by hash/JWT crates?
  - [ ] Assess: could `transforms/` be published as a standalone crate for reuse?
  - [ ] Document findings in `.vault/audits/ARCH-AUDIT-1.md`

#### Verification

- [ ] `.vault/audits/ARCH-AUDIT-1.md` committed with profiling data and boundary analysis
- [ ] All operations complete in < 50ms for 10KB input
- [ ] No LSP types found in `transforms/` crate

### 🔍 AUDIT: Security Audit #1

**Priority:** Critical | **Impact:** Very High | **Effort:** Medium | **Risk:** Low
**Source:** Audit Schedule — Phase 2
**Status:** Not Started
**Dependencies:** EPIC-2.1, EPIC-2.2, EPIC-2.3

#### Definition of Done

- [ ] **A-021** — Security Audit
  - [ ] Run `cargo audit` — zero advisories
  - [ ] Run `cargo deny check advisories`
  - [ ] Verify no `unsafe` blocks in entire codebase
  - [ ] Review hash crate dependencies for known supply chain issues
  - [ ] Fuzz test Base64 decode, URL decode, and JSON parse with `cargo-fuzz` (minimum 10 minutes per target)
  - [ ] Document findings in `.vault/audits/SECURITY-AUDIT-1.md`

#### Verification

- [ ] `.vault/audits/SECURITY-AUDIT-1.md` committed with fuzz test results
- [ ] `cargo audit` and `cargo deny check` return zero issues
- [ ] `grep -r "unsafe" transforms/` returns zero matches

### 🔍 AUDIT: Dependency Audit #2

**Priority:** High | **Impact:** High | **Effort:** Small | **Risk:** Low
**Source:** Audit Schedule — Phase 2
**Status:** Not Started
**Dependencies:** EPIC-2.1, EPIC-2.2, EPIC-2.3, EPIC-2.4, EPIC-2.5

#### Definition of Done

- [ ] **A-022** — Dependency Audit
  - [ ] Review all new dependencies added in Phase 2
  - [ ] Document total transitive dependency count delta from Phase 1
  - [ ] Verify no new license incompatibilities
  - [ ] Flag any dep with fewer than 100 downloads/week (supply chain risk)
  - [ ] Document in `.vault/audits/DEP-AUDIT-2.md`

#### Verification

- [ ] `.vault/audits/DEP-AUDIT-2.md` committed with dependency delta analysis
- [ ] No new license incompatibilities detected

### 📋 PM REVIEW: PMR-2 — Feature Velocity Check

**Priority:** High | **Impact:** High | **Effort:** Small | **Risk:** Low
**Source:** PM Governance Cadence
**Status:** Not Started
**Dependencies:** EPIC-2.1, EPIC-2.2, EPIC-2.3, EPIC-2.4, EPIC-2.5, A-020, A-021

#### Definition of Done

- [ ] **PMR-2** — Conduct Feature Velocity Check
  - [ ] Review actual velocity: how long did Phase 2 take vs. estimate?
  - [ ] Are hashing features actually useful or speculative? (check: would you use them?)
  - [ ] Is JWT decode a differentiator or bloat?
  - [ ] Review Phase 3 scope: is the full case conversion list necessary or should we ship fewer, better?
  - [ ] Re-examine backlog: anything from B-001–B-015 that should be promoted?
  - [ ] Adjust release cadence if velocity differs from plan
  - [ ] Decision: cut, defer, or accelerate Phase 3 items
  - [ ] Document decisions in `.vault/pm-reviews/PMR-2.md`

#### Verification

- [ ] `.vault/pm-reviews/PMR-2.md` committed with velocity analysis and Phase 3 scope decisions
- [ ] Backlog items reviewed and promotion decisions documented

**Phase 2 Exit Criteria:** All hash, JWT, and format conversion operations functional. Error handling is graceful across all actions. ARI score blocking enabled on PRs. Signed commits required. Architecture audit passed. Security audit passed. Test coverage ≥ 70% on `transforms/`.

---

## Phase 3 — Text Transformation & Case Conversion

> **Goal:** The string manipulation operations developers use when refactoring — case conversions, whitespace operations, text analysis.

### EPIC-3.1: Case Conversions

**Priority:** High | **Impact:** Very High | **Effort:** Medium | **Risk:** Low
**Source:** Product Roadmap v1 — Phase 3
**Status:** Not Started
**Dependencies:** EPIC-0.4
**AI-first benefit:** 13 case variants from a single module — agents can generate all variants by understanding the word-boundary splitting algorithm once.

Implement the full suite of case conversions (13 variants) that developers use when refactoring identifiers between naming conventions. Proper word boundary detection (camelCase splits, acronym handling, number boundaries) is the core challenge.

#### Definition of Done

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

#### Verification

- [ ] `cargo test -p transforms -- case` passes all tests
- [ ] Acronym handling: `HTTPSConnection` → `https_connection` → `HttpsConnection`
- [ ] Number boundary: `myVar2Name` → `my_var_2_name` → `myVar2Name`
- [ ] All 13 case variants produce correct output in Zed context menu

### EPIC-3.2: Whitespace & Line Operations

**Priority:** High | **Impact:** High | **Effort:** Medium | **Risk:** Low
**Source:** Product Roadmap v1 — Phase 3
**Status:** Not Started
**Dependencies:** EPIC-0.4
**AI-first benefit:** Line operations are pure text transforms with no encoding complexity — agents can implement and test each independently.

Implement whitespace manipulation and line-level operations: trim, collapse, sort, deduplicate, reverse, shuffle, and number lines. These are daily-use operations for cleaning up data, logs, and text.

#### Definition of Done

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

#### Verification

- [ ] `cargo test -p transforms -- whitespace` passes all tests
- [ ] Sort operations handle Unicode collation correctly
- [ ] Remove Duplicate Lines preserves first occurrence and original order
- [ ] Line operations handle trailing newline edge cases

### EPIC-3.3: String Inspection (Non-Destructive)

**Priority:** Medium | **Impact:** Medium | **Effort:** Small | **Risk:** Low
**Source:** Product Roadmap v1 — Phase 3
**Status:** Not Started
**Dependencies:** EPIC-0.4, EPIC-1.6 (detection module)
**AI-first benefit:** Inspection outputs are structured data — agents can validate them programmatically.

Implement non-destructive string inspection actions that display information (character count, byte length, encoding detection) without modifying the selected text. Results are shown via Zed notifications or diagnostics.

#### Definition of Done

- [ ] **T-340** — Implement `Count Characters` code action
  - [ ] Pure function in `transforms/inspect.rs`
  - [ ] Show total characters, bytes (UTF-8), words, lines as a Zed notification/diagnostic
  - [ ] Do NOT replace the selected text
- [ ] **T-341** — Implement `String Length (bytes)` code action (show UTF-8 byte count)
- [ ] **T-342** — Implement `Detect Encoding` code action (attempt to identify if selection is Base64, URL-encoded, hex, JWT, etc.)
- [ ] **T-343** — Unit tests for inspection operations

#### Verification

- [ ] `cargo test -p transforms -- inspect` passes all tests
- [ ] Count Characters correctly differentiates chars vs. bytes for multi-byte UTF-8
- [ ] Detect Encoding correctly identifies Base64, URL-encoded, hex, and JWT patterns
- [ ] Inspection actions do NOT modify the selected text

### EPIC-3.4: Escape/Unescape Operations

**Priority:** High | **Impact:** High | **Effort:** Medium | **Risk:** Low
**Source:** Product Roadmap v1 — Phase 3
**Status:** Not Started
**Dependencies:** EPIC-0.4
**AI-first benefit:** Escape operations have well-defined specs (regex metacharacters, SQL quoting rules) — agents can implement from specification.

Implement escape and unescape operations for common contexts: backslashes, regex metacharacters, SQL strings, shell strings, and CSV fields. Essential for developers constructing queries, patterns, and data exports.

#### Definition of Done

- [ ] **T-350** — Implement `Escape Backslashes` code action (`\` → `\\`)
  - [ ] Pure function in `transforms/escape.rs`
- [ ] **T-351** — Implement `Unescape Backslashes` code action (`\\` → `\`)
- [ ] **T-352** — Implement `Escape Regex` code action (escape regex special characters)
- [ ] **T-353** — Implement `Escape SQL String` code action (single quotes)
- [ ] **T-354** — Implement `Escape Shell String` code action
- [ ] **T-355** — Implement `Escape CSV Field` code action
- [ ] **T-356** — Unit tests for escape operations

#### Verification

- [ ] `cargo test -p transforms -- escape` passes all tests
- [ ] Regex escape handles all metacharacters: `. * + ? ^ $ { } [ ] ( ) | \`
- [ ] SQL escape doubles single quotes correctly
- [ ] Shell escape handles spaces, quotes, and special characters

### 🔒 GATE: ARI-2 Checkpoint

**Priority:** Critical | **Impact:** Very High | **Effort:** Small | **Risk:** Medium
**Source:** ARI Governance — Phase 3 exit gate
**Status:** Not Started
**Dependencies:** EPIC-3.1, EPIC-3.2, EPIC-3.3, EPIC-3.4

#### Definition of Done

- [ ] **ARI-2** — Run `ariscan` — **minimum composite score ≥ 80/100**
  - [ ] Record scores in `.vault/ari/ARI-2.md`
  - [ ] Test Isolation ≥ 85/100 (extensive pure function test suite by now)
  - [ ] Modular Coherence ≥ 80/100 (7+ transform modules, clean boundaries)
  - [ ] Documentation Density ≥ 75/100 (rustdoc on all public APIs, HINTS.md current)
  - [ ] Delta report against ARI-1 — flag any regressions
  - [ ] If below 8.0: **architectural review required** before Phase 4

#### Verification

- [ ] `.vault/ari/ARI-2.md` committed with per-pillar scores and delta from ARI-1
- [ ] Composite ARI score ≥ 80/100 confirmed
- [ ] No pillar regression from ARI-1

### 🔍 AUDIT: Code Quality Audit #2

**Priority:** High | **Impact:** High | **Effort:** Small | **Risk:** Low
**Source:** Audit Schedule — Phase 3
**Status:** Not Started
**Dependencies:** EPIC-3.1, EPIC-3.2, EPIC-3.3, EPIC-3.4

#### Definition of Done

- [ ] **A-030** — Code Quality Audit
  - [ ] Test coverage ≥ 85% on `transforms/` module
  - [ ] Zero clippy warnings
  - [ ] Review for dead code (any unused transforms? any dead feature flags?)
  - [ ] Check for consistent error handling patterns across all modules
  - [ ] Review code action naming: is the `StringKnife:` prefix consistent?
  - [ ] Document findings in `.vault/audits/CODE-QUALITY-2.md`

#### Verification

- [ ] `.vault/audits/CODE-QUALITY-2.md` committed with coverage metrics
- [ ] Test coverage ≥ 85% on `transforms/` confirmed
- [ ] `cargo clippy -- -D warnings` passes

### 🔍 AUDIT: UX Audit #1

**Priority:** High | **Impact:** Very High | **Effort:** Medium | **Risk:** Low
**Source:** Audit Schedule — Phase 3
**Status:** Not Started
**Dependencies:** EPIC-3.1, EPIC-3.2, EPIC-3.3, EPIC-3.4, EPIC-1.6

#### Definition of Done

- [ ] **A-031** — UX Audit
  - [ ] Install extension on a clean Zed instance
  - [ ] Test code action discoverability: can a new user find encode/decode in < 3 seconds?
  - [ ] Count total code actions shown when selecting arbitrary text — is it overwhelming?
  - [ ] Review smart detection: does it correctly identify Base64 vs. hex vs. URL-encoded?
  - [ ] Review error messages: are they helpful to a developer who doesn't know StringKnife internals?
  - [ ] Test with multi-line selections, single character, entire file selected
  - [ ] Document findings and recommendations in `.vault/audits/UX-AUDIT-1.md`

#### Verification

- [ ] `.vault/audits/UX-AUDIT-1.md` committed with discoverability metrics and recommendations
- [ ] Code action count documented — is it manageable or overwhelming?
- [ ] Error message clarity validated by non-expert developer

**Phase 3 Exit Criteria:** All case, whitespace, and escape operations functional. Inspection actions return info without modifying text. ARI ≥ 80/100. UX audit complete. Test coverage ≥ 70% on `transforms/`.

---

## Phase 4 — Configuration, Performance & Polish

> **Goal:** User-configurable behavior, performant operation on large selections, and production-quality error handling.

### EPIC-4.1: Extension Configuration

**Priority:** High | **Impact:** High | **Effort:** Medium | **Risk:** Low
**Source:** Product Roadmap v1 — Phase 4
**Status:** Not Started
**Dependencies:** EPIC-0.3 (LSP config plumbing)
**AI-first benefit:** Typed configuration schema with defaults makes agent-driven config changes safe and predictable.

Define and implement the LSP configuration schema, allowing users to customise behavior through Zed's `settings.json`. Includes category filtering, smart detection toggle, output format preferences, and live config reload.

#### Definition of Done

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

#### Verification

- [ ] Changing `settings.json` updates behavior without restarting Zed
- [ ] Disabling a category removes its code actions from the context menu
- [ ] Default values work correctly when no config is provided
- [ ] README configuration reference table matches actual behavior

### EPIC-4.2: Performance & Large Input Handling

**Priority:** High | **Impact:** Very High | **Effort:** Medium | **Risk:** Medium
**Source:** Product Roadmap v1 — Phase 4
**Status:** Not Started
**Dependencies:** EPIC-1.1 through EPIC-3.4 (all transforms implemented)
**AI-first benefit:** Performance benchmarks provide quantitative pass/fail criteria for agent-driven optimisations.

Benchmark all operations, enforce the 100ms/100KB performance contract, set input size limits, and optimise memory usage for sustained operation.

#### Definition of Done

- [ ] **T-410** — Benchmark code action response time for 1KB, 10KB, 100KB, 1MB selections
- [ ] **T-411** — Set maximum input size limit (default: 1MB) with clear error message
- [ ] **T-412** — Ensure document sync doesn't hold full document copies unnecessarily
- [ ] **T-413** — Profile memory usage under sustained operation
- [ ] **T-414** — Add timeout handling for code action computation (5 second max)

#### Verification

- [ ] All operations complete in < 100ms for 100KB input (benchmark proof)
- [ ] Selections > 1MB return `InputTooLarge` error via `window/showMessage`
- [ ] Memory usage remains stable after 1000 sequential code actions
- [ ] Timeout fires at 5 seconds for pathological inputs

### EPIC-4.3: Error Handling & User Feedback

**Priority:** High | **Impact:** High | **Effort:** Medium | **Risk:** Low
**Source:** Product Roadmap v1 — Phase 4
**Status:** Not Started
**Dependencies:** EPIC-0.3 (LSP error mapping)
**AI-first benefit:** Structured error responses make failure modes explicit and testable by agents.

Define the error response strategy, implement user-facing notifications for failed operations, ensure zero panics under any input, and add structured logging.

#### Definition of Done

- [ ] **T-420** — Define error response strategy: return `Diagnostic` for decode errors vs. silent skip
- [ ] **T-421** — Implement `window/showMessage` notifications for operations that fail on invalid input
- [ ] **T-422** — Ensure no panics in LSP binary under any input (fuzz test critical paths)
- [ ] **T-423** — Add structured logging to LSP (`tracing` crate, configurable log level)
- [ ] **T-424** — Log level configurable via `stringknife.logLevel` setting

#### Verification

- [ ] Invalid input shows helpful error message via `window/showMessage`
- [ ] `cargo-fuzz` runs for 30+ minutes on all decode paths with zero panics
- [ ] Log output includes structured fields (operation, input_size, duration)
- [ ] Log level changes via config without restart

### EPIC-4.4: Multi-Selection Support

**Priority:** Medium | **Impact:** High | **Effort:** Medium | **Risk:** Medium
**Source:** Product Roadmap v1 — Phase 4
**Status:** Not Started
**Dependencies:** EPIC-0.3 (LSP WorkspaceEdit handling)
**AI-first benefit:** Multi-selection support tests the LSP protocol edge cases that agents need to handle correctly.

Enable code actions to work with multiple cursor selections simultaneously, returning a `WorkspaceEdit` with multiple `TextEdit` entries. Handle overlapping ranges gracefully.

#### Definition of Done

- [ ] **T-430** — Handle multiple selection ranges in a single `codeAction` request
- [ ] **T-431** — Return `WorkspaceEdit` with multiple `TextEdit` entries (one per selection)
- [ ] **T-432** — Test multi-cursor encode/decode operations
- [ ] **T-433** — Ensure edits don't conflict when ranges overlap (reject with message)

#### Verification

- [ ] Multi-cursor Base64 encode applies to all selections simultaneously
- [ ] Overlapping ranges are rejected with a clear error message
- [ ] Undo reverts all multi-cursor edits in a single step
- [ ] Performance remains within budget with 10+ simultaneous selections

### 🔍 AUDIT: Architecture Audit #2

**Priority:** High | **Impact:** High | **Effort:** Medium | **Risk:** Low
**Source:** Audit Schedule — Phase 4
**Status:** Not Started
**Dependencies:** EPIC-4.1, EPIC-4.2, EPIC-4.3, EPIC-4.4

#### Definition of Done

- [ ] **A-040** — Architecture Audit
  - [ ] Review configuration plumbing: is it clean or spaghetti?
  - [ ] Profile memory under 1000 sequential code actions (leak test)
  - [ ] Review LSP lifecycle: clean shutdown, no orphan processes
  - [ ] Benchmark: all operations < 100ms for 100KB input (hard requirement)
  - [ ] Document findings in `.vault/audits/ARCH-AUDIT-2.md`

#### Verification

- [ ] `.vault/audits/ARCH-AUDIT-2.md` committed with benchmark data and memory profile
- [ ] No memory leaks detected under sustained operation
- [ ] Performance contract met for all operations

### 🔍 AUDIT: Dependency Audit #3

**Priority:** High | **Impact:** High | **Effort:** Small | **Risk:** Low
**Source:** Audit Schedule — Phase 4
**Status:** Not Started
**Dependencies:** EPIC-4.1, EPIC-4.3 (new deps for config/logging)

#### Definition of Done

- [ ] **A-041** — Dependency Audit
  - [ ] Full transitive dependency audit
  - [ ] Check for any new crates added for config/logging
  - [ ] Verify `tracing` dependency is justified vs. simpler logging
  - [ ] Document in `.vault/audits/DEP-AUDIT-3.md`

#### Verification

- [ ] `.vault/audits/DEP-AUDIT-3.md` committed with transitive dep count
- [ ] `tracing` justification documented
- [ ] `cargo deny check` passes

### 📋 PM REVIEW: PMR-3 — Pre-Launch Review

**Priority:** Critical | **Impact:** Very High | **Effort:** Small | **Risk:** Low
**Source:** PM Governance Cadence
**Status:** Not Started
**Dependencies:** EPIC-4.1, EPIC-4.2, EPIC-4.3, EPIC-4.4, A-040, A-041

#### Definition of Done

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
  - [ ] Document decisions in `.vault/pm-reviews/PMR-3.md`

#### Verification

- [ ] `.vault/pm-reviews/PMR-3.md` committed with v0.5.0 scope lock and kill list
- [ ] README reviewed and updated for store-readiness
- [ ] Demo assets created and reviewed

**Phase 4 Exit Criteria:** Extension is configurable, performant on large inputs, handles errors gracefully, and supports multi-cursor. Benchmark regression gate active on PRs. Test coverage ≥ 80% on `transforms/`. Architecture audit passed. Pre-launch scope locked.

---

## Phase 5 — Publish, Distribute & Community

> **Goal:** Extension published to the Zed Extension Store, discoverable, documented, and ready for community contributions.

### EPIC-5.1: Publication Preparation

**Priority:** Critical | **Impact:** Very High | **Effort:** Medium | **Risk:** Low
**Source:** Product Roadmap v1 — Phase 5
**Status:** Not Started
**Dependencies:** PMR-3 (scope locked)
**AI-first benefit:** Store-ready documentation and demo assets make the extension discoverable and usable without human onboarding.

Prepare all assets for Zed Extension Store publication: comprehensive README with demos, extension icon, GIF walkthroughs of key features, and updated HINTS.md for contributor onboarding.

#### Definition of Done

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
- [ ] **T-506** — Add ARI badge to `README.md`
  - [ ] Dynamic badge sourced from ariscan CI output: `ARI 8.7 / 10` with colour scale (red <6, yellow 6–7.9, green ≥8)
  - [ ] Position: top of README alongside license and CI badges
  - [ ] Badge links to `.vault/ari/ARI Dashboard.md` on GitHub
- [ ] **T-507** — Add "Built with ariscan" section to `README.md`
  - [ ] 2–3 sentences: factual, not promotional — "This repository uses ariscan to measure AI-agent readiness at every phase gate."
  - [ ] Link to Prontiq ariscan repo
  - [ ] ARI trajectory table pulled from existing checkpoint data (BASELINE through current)
  - [ ] "Browse the Knowledge Vault" link pointing to `.vault/Home.md`
- [ ] **T-508** — Make `.vault/` browsable on GitHub
  - [ ] Confirm wikilinks render as clickable relative links in GitHub's markdown viewer
  - [ ] If not, add fallback: `[Note Name](./path/to/Note%20Name.md)` alongside wikilinks
  - [ ] Confirm frontmatter renders as visible YAML block
- [ ] **T-509** — Add GitHub topics for discoverability
  - [ ] `ariscan`, `ai-agent-readiness`, `zed-extension`, `lsp`, `rust`, `string-transformation`
  - [ ] Update repo description: "A string transformation toolkit for Zed — built with ariscan-driven development"

#### Verification

- [ ] `extension.toml` passes Zed's validation (test with `cargo build` for WASM target)
- [ ] README renders correctly on GitHub with all GIFs loading
- [ ] Icon meets Zed store guidelines (SVG, correct dimensions)
- [ ] HINTS.md "how to add a new operation" guide is accurate and complete

### EPIC-5.2: Publish to Zed Extension Store

**Priority:** Critical | **Impact:** Very High | **Effort:** Medium | **Risk:** Medium
**Source:** Product Roadmap v1 — Phase 5
**Status:** Not Started
**Dependencies:** EPIC-5.1, ARI-3, A-050, A-051
**AI-first benefit:** Published extension validates the entire build-to-deploy pipeline end-to-end.

Submit the extension to the Zed Extension Store by forking `zed-industries/extensions`, adding StringKnife as a submodule, and opening a PR. Iterate on review feedback until merged.

#### Definition of Done

- [ ] **T-510** — Fork `zed-industries/extensions` to personal GitHub account
- [ ] **T-511** — Add `stringknife` as a Git submodule in `extensions/` directory
- [ ] **T-512** — Add entry to top-level `extensions.toml`
- [ ] **T-513** — Run `pnpm sort-extensions` to sort entries
- [ ] **T-514** — Open PR to `zed-industries/extensions`
- [ ] **T-515** — Respond to review feedback and iterate
- [ ] **T-516** — Verify extension appears in Zed Extension Store post-merge
- [ ] **T-517** — Test installation from the store on a clean Zed instance

#### Verification

- [ ] Extension appears in Zed Extension Store search results
- [ ] Fresh install from store works on macOS and Linux
- [ ] All code actions functional after store installation (no dev-only dependencies)
- [ ] Binary downloads correctly for user's platform

### EPIC-5.3: Community & Maintenance

**Priority:** High | **Impact:** High | **Effort:** Medium | **Risk:** Low
**Source:** Product Roadmap v1 — Phase 5
**Status:** Not Started
**Dependencies:** EPIC-5.2
**AI-first benefit:** Issue templates and contribution guidelines enable agents to triage issues and submit PRs following established patterns.

Set up community infrastructure: issue templates, GitHub Discussions, automated extension updates, security disclosure policy, and initial release announcement.

#### Definition of Done

- [ ] **T-520** — Create GitHub issue templates
  - [ ] Bug report template
  - [ ] Feature request template
  - [ ] New string operation request template
- [ ] **T-521** — Create GitHub Discussions category for community suggestions
- [ ] **T-522** — Set up GitHub Action for automated extension updates (using `zed-extension-action`)
- [ ] **T-523** — Create `SECURITY.md` with responsible disclosure policy
- [ ] **T-524** — Tag and release `v0.1.0`
- [ ] **T-525** — Announce on Zed Discord and relevant communities

#### Verification

- [ ] Issue templates render correctly on GitHub (test by creating draft issues)
- [ ] `v0.1.0` tag exists with GitHub Release and platform binaries
- [ ] `SECURITY.md` provides clear disclosure instructions
- [ ] Announcement posted on Zed Discord

### 🔒 GATE: ARI-3 Checkpoint (Pre-Publish)

**Priority:** Critical | **Impact:** Very High | **Effort:** Small | **Risk:** Medium
**Source:** ARI Governance — Phase 5 pre-publish gate
**Status:** Not Started
**Dependencies:** EPIC-5.1

#### Definition of Done

- [ ] **ARI-3** — Run `ariscan` — **minimum composite score ≥ 85/100**
  - [ ] Record scores in `.vault/ari/ARI-3.md`
  - [ ] All pillars individually ≥ 75/100
  - [ ] Documentation Density ≥ 80/100 (HINTS.md, README, rustdoc, CONTRIBUTING.md all current)
  - [ ] Security gate: **Pass** (cargo-audit clean, fuzz tests run, no unsafe)
  - [ ] If below 8.5: **block store submission** — final hardening sprint

#### Verification

- [ ] `.vault/ari/ARI-3.md` committed with per-pillar scores
- [ ] Composite ARI score ≥ 85/100 confirmed
- [ ] All pillars individually ≥ 75/100

### 🔍 AUDIT: Security Audit #2 (Pre-Publish)

**Priority:** Critical | **Impact:** Very High | **Effort:** Medium | **Risk:** Low
**Source:** Audit Schedule — Phase 5 pre-publish
**Status:** Not Started
**Dependencies:** EPIC-5.1

#### Definition of Done

- [ ] **A-050** — Pre-Publish Security Audit
  - [ ] `cargo audit` — zero advisories
  - [ ] `cargo deny check` — all clear
  - [ ] Full fuzz test run on all decode/parse operations (30 minutes per target)
  - [ ] Review: does the extension request any permissions it doesn't need?
  - [ ] Review: can any code action cause data loss? (e.g., decode fails but still replaces text)
  - [ ] Document in `.vault/audits/SECURITY-AUDIT-2.md`

#### Verification

- [ ] `.vault/audits/SECURITY-AUDIT-2.md` committed with fuzz results and permission review
- [ ] Zero data-loss scenarios identified
- [ ] `cargo audit` and `cargo deny check` clean

### 🔍 AUDIT: UX Audit #2 (Pre-Publish)

**Priority:** High | **Impact:** Very High | **Effort:** Medium | **Risk:** Low
**Source:** Audit Schedule — Phase 5 pre-publish
**Status:** Not Started
**Dependencies:** EPIC-5.1

#### Definition of Done

- [ ] **A-051** — Pre-Publish UX Audit
  - [ ] Fresh install test on macOS, Linux
  - [ ] Verify all code actions appear and work on first install (no manual config needed)
  - [ ] Time from install to first successful encode: target < 30 seconds
  - [ ] Verify error messages are helpful and non-technical
  - [ ] Verify no performance degradation on large files (10K+ line files)
  - [ ] Document in `.vault/audits/UX-AUDIT-2.md`

#### Verification

- [ ] `.vault/audits/UX-AUDIT-2.md` committed with install timing and platform test results
- [ ] Time-to-first-encode < 30 seconds confirmed
- [ ] No performance degradation on 10K+ line files

**Phase 5 Exit Criteria:** Extension live in Zed Extension Store. Installable by any Zed user. ARI ≥ 85/100. Benchmark regression blocks merge (>20% regression). Test coverage ≥ 85% on `transforms/`. Both security and UX audits passed. Community contribution pipeline in place.

---

### 📋 PM REVIEW: PMR-4 — Post-Launch Retrospective

**Priority:** High | **Impact:** Very High | **Effort:** Small | **Risk:** Low
**Source:** PM Governance Cadence
**Status:** Not Started
**Dependencies:** EPIC-5.2 (store publication complete)

> **Scheduled:** 2 weeks after Phase 5 store publication

#### Definition of Done

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
  - [ ] Document decisions in `.vault/pm-reviews/PMR-4.md`

#### Verification

- [ ] `.vault/pm-reviews/PMR-4.md` committed with metrics, user feedback themes, and Phase 6 stack rank
- [ ] Kill and promote decisions documented with evidence basis

---

## Phase 6 — Advanced Features (Post-Launch)

> **Goal:** Differentiate StringKnife from basic string tools with power-user features driven by community feedback and PMR-4 evidence.

### EPIC-6.1: Timestamp/Epoch Operations

**Priority:** Medium | **Impact:** Medium | **Effort:** Small | **Risk:** Low
**Source:** Product Roadmap v1 — Phase 6
**Status:** Not Started
**Dependencies:** EPIC-0.4, PMR-4 (priority confirmed by user demand)
**AI-first benefit:** Timestamp edge cases (Y2K38, negative epochs) are well-documented — agents can generate comprehensive test suites from specs.

Implement Unix timestamp ↔ ISO 8601 ↔ human-readable conversions with smart detection for epoch timestamps (10-digit seconds, 13-digit milliseconds).

#### Definition of Done

- [ ] **T-600** — Implement `Unix Timestamp → ISO 8601` code action
  - [ ] Pure function in `transforms/timestamp.rs`
- [ ] **T-601** — Implement `ISO 8601 → Unix Timestamp` code action
- [ ] **T-602** — Implement `Unix Timestamp → Human Readable` code action (locale-aware)
- [ ] **T-603** — Detect epoch timestamps in selection (10-digit seconds, 13-digit milliseconds)
- [ ] **T-604** — Unit tests for timestamp operations (edge cases: negative epochs, Y2K38, milliseconds)

#### Verification

- [ ] `cargo test -p transforms -- timestamp` passes all tests
- [ ] Known epoch values produce correct ISO 8601 output
- [ ] Y2K38 boundary (2147483647) handled correctly
- [ ] 13-digit millisecond timestamps detected and converted

### EPIC-6.2: Number Base Conversions

**Priority:** Medium | **Impact:** Medium | **Effort:** Small | **Risk:** Low
**Source:** Product Roadmap v1 — Phase 6
**Status:** Not Started
**Dependencies:** EPIC-0.4
**AI-first benefit:** Number base conversions are pure math — trivially testable with known input/output pairs.

Implement decimal ↔ hex ↔ binary ↔ octal conversions with auto-detection of number base from prefix (`0x`, `0b`, `0o`).

#### Definition of Done

- [ ] **T-610** — Implement `Decimal → Hex` code action
  - [ ] Pure function in `transforms/numbers.rs`
- [ ] **T-611** — Implement `Hex → Decimal` code action
- [ ] **T-612** — Implement `Decimal → Binary` code action
- [ ] **T-613** — Implement `Binary → Decimal` code action
- [ ] **T-614** — Implement `Decimal → Octal` code action
- [ ] **T-615** — Implement `Octal → Decimal` code action
- [ ] **T-616** — Auto-detect number base from prefix (`0x`, `0b`, `0o`)
- [ ] **T-617** — Unit tests for number conversions (large numbers, negative numbers, edge cases)

#### Verification

- [ ] `cargo test -p transforms -- numbers` passes all tests
- [ ] `0xFF` auto-detected as hex and converted to decimal 255
- [ ] Large numbers (u64 range) handled without overflow
- [ ] Negative numbers handled or rejected with clear error

### EPIC-6.3: UUID & Random Generation

**Priority:** Medium | **Impact:** High | **Effort:** Small | **Risk:** Low
**Source:** Product Roadmap v1 — Phase 6
**Status:** Not Started
**Dependencies:** EPIC-0.4
**AI-first benefit:** UUID generation and validation are self-contained — agents can implement and test without cross-module dependencies.

Implement UUID v4 (random) and v7 (time-sortable) generation, UUID validation, and configurable random string generation.

#### Definition of Done

- [ ] **T-620** — Implement `Generate UUID v4` code action (inserts at cursor/replaces selection)
  - [ ] Pure function in `transforms/uuid.rs`
- [ ] **T-621** — Implement `Generate UUID v7` code action (time-sortable)
- [ ] **T-622** — Implement `Validate UUID` code action (shows version and validity as diagnostic)
- [ ] **T-623** — Implement `Generate Random String` code action (configurable length, charset via config)
- [ ] **T-624** — Unit tests for UUID operations

#### Verification

- [ ] `cargo test -p transforms -- uuid` passes all tests
- [ ] Generated UUIDs pass RFC 4122 validation
- [ ] UUID v7 timestamps are monotonically increasing
- [ ] Validate UUID correctly identifies version for v1, v4, v7

### EPIC-6.4: Regex & Pattern Operations

**Priority:** Medium | **Impact:** High | **Effort:** Medium | **Risk:** Low
**Source:** Product Roadmap v1 — Phase 6
**Status:** Not Started
**Dependencies:** EPIC-0.4
**AI-first benefit:** Pattern extraction uses well-defined regex specs — agents can extend with new patterns by following the existing extraction template.

Implement pattern extraction (emails, URLs, IP addresses) and sensitive data masking from selected text. Useful for log analysis, data sanitisation, and security reviews.

#### Definition of Done

- [ ] **T-630** — Implement `Extract Emails` code action (finds all email addresses in selection)
  - [ ] Pure function in `transforms/extract.rs`
- [ ] **T-631** — Implement `Extract URLs` code action
- [ ] **T-632** — Implement `Extract IP Addresses` code action (v4 and v6)
- [ ] **T-633** — Implement `Mask Sensitive Data` code action (replace middle chars with `*` for emails, tokens)
- [ ] **T-634** — Unit tests for extraction operations

#### Verification

- [ ] `cargo test -p transforms -- extract` passes all tests
- [ ] Email extraction handles edge cases (subdomains, plus addressing)
- [ ] IPv6 addresses extracted correctly (full and abbreviated forms)
- [ ] Masking preserves first/last characters with `*` fill

### EPIC-6.5: Text Diff & Comparison

**Priority:** Low | **Impact:** Medium | **Effort:** Small | **Risk:** Low
**Source:** Product Roadmap v1 — Phase 6
**Status:** Not Started
**Dependencies:** EPIC-0.4
**AI-first benefit:** Diff output is structured — agents can validate correctness against known diff algorithms.

Implement line-level and character-level text diffing for selections containing two text blocks separated by a delimiter.

#### Definition of Done

- [ ] **T-640** — Implement `String Diff (Line)` code action (when two blocks separated by `---` or similar delimiter, show line diff)
  - [ ] Pure function in `transforms/diff.rs`
- [ ] **T-641** — Implement `String Diff (Character)` code action (char-level diff for short strings)
- [ ] **T-642** — Unit tests for diff operations

#### Verification

- [ ] `cargo test -p transforms -- diff` passes all tests
- [ ] Line diff correctly identifies additions, deletions, and unchanged lines
- [ ] Character diff highlights individual character changes
- [ ] Missing delimiter returns clear error

### EPIC-6.6: Compression

**Priority:** Low | **Impact:** Medium | **Effort:** Small | **Risk:** Low
**Source:** Product Roadmap v1 — Phase 6
**Status:** Not Started
**Dependencies:** EPIC-0.4, EPIC-1.1 (Base64 for output encoding)
**AI-first benefit:** Compression transforms compose with Base64 — tests validate the composition pipeline.

Implement gzip and deflate compression/decompression with Base64 encoding for text-safe transport. Useful for developers working with compressed API payloads and data URIs.

#### Definition of Done

- [ ] **T-650** — Implement `Gzip Compress → Base64` code action
  - [ ] Pure function in `transforms/compress.rs`
- [ ] **T-651** — Implement `Base64 → Gzip Decompress` code action
- [ ] **T-652** — Implement `Deflate Compress → Base64` code action
- [ ] **T-653** — Implement `Base64 → Deflate Decompress` code action
- [ ] **T-654** — Unit tests for compression operations

#### Verification

- [ ] `cargo test -p transforms -- compress` passes all tests
- [ ] Gzip roundtrip: `decompress(compress(x)) == x`
- [ ] Deflate roundtrip: `decompress(compress(x)) == x`
- [ ] Invalid compressed data returns structured error

### EPIC-6.7: ariscan Showcase Artefacts

> The ARI evidence already exists in the vault. This EPIC packages it for external consumption.

- [ ] **T-670** — Create `docs/ariscan-case-study.md`
  - [ ] Structure: Problem → Approach → Results → Methodology
  - [ ] Results pulled from existing vault checkpoint notes — no fabrication, no cherry-picking
  - [ ] Acknowledge dips with explanations (e.g., modular coherence dropped when 6 new modules landed, recovered after refactor)
  - [ ] Link to every vault checkpoint note as evidence
  - [ ] Final section: how to replicate this approach in any repo
- [ ] **T-671** — Create ARI trajectory visualisation
  - [ ] SVG chart: X = checkpoint, Y = composite score
  - [ ] Per-pillar sparklines showing individual progression
  - [ ] Data source: vault checkpoint frontmatter
  - [ ] Embed in case study and README
- [ ] **T-672** — Create `ARCHITECTURE.md` at repo root
  - [ ] Standalone architecture overview — doesn't require opening the vault
  - [ ] Three-layer diagram, crate graph, data flow, key design decisions
  - [ ] Final section: "Agent-Readiness by Design" — how each architectural choice maps to an ARI pillar
  - [ ] This is the file a developer reads when evaluating whether to structure their repo similarly
- [ ] **T-673** — Outsider audit
  - [ ] Clone the repo fresh, no context, read only what's in the repo
  - [ ] Can you understand the architecture within 5 minutes?
  - [ ] Can you find the ARI scores within 30 seconds?
  - [ ] Can you trace the journey from empty repo to v1.0 through the vault?
  - [ ] Document friction points and fix them

### 🔒 GATE: ARI-4 Checkpoint (v1.0 Gate)

**Priority:** Critical | **Impact:** Very High | **Effort:** Small | **Risk:** Medium
**Source:** ARI Governance — Phase 6 v1.0 gate
**Status:** Not Started
**Dependencies:** EPIC-6.1, EPIC-6.2, EPIC-6.3, EPIC-6.4, EPIC-6.5, EPIC-6.6, EPIC-6.7

#### Definition of Done

- [ ] **ARI-4** — Run `ariscan` — **minimum composite score ≥ 90/100**
  - [ ] Record scores in `.vault/ari/ARI-4.md`
  - [ ] All pillars individually ≥ 80/100
  - [ ] Test Isolation ≥ 90/100 (gold standard for a pure-function codebase)
  - [ ] Security gate: **Pass**
  - [ ] Full ARI trajectory report: ARI-BASELINE → ARI-0 → ARI-1 → ARI-2 → ARI-3 → ARI-4
  - [ ] If below 9.0: continue as 0.x — do not stamp v1.0

#### Verification

- [ ] `.vault/ari/ARI-4.md` committed with full trajectory report
- [ ] Composite ARI score ≥ 90/100 confirmed
- [ ] All pillars individually ≥ 80/100
- [ ] ARI trajectory shows consistent upward trend

### 🔍 AUDIT: Full Audit Suite (Pre v1.0)

**Priority:** High | **Impact:** Very High | **Effort:** High | **Risk:** Low
**Source:** Audit Schedule — Phase 6 pre-v1.0
**Status:** Not Started
**Dependencies:** EPIC-6.1, EPIC-6.2, EPIC-6.3, EPIC-6.4, EPIC-6.5, EPIC-6.6

#### Definition of Done

- [ ] **A-060** — Code Quality Audit #3
  - [ ] Test coverage ≥ 90% on `transforms/`
  - [ ] Zero clippy warnings
  - [ ] No dead code
  - [ ] Consistent error handling
  - [ ] Document in `.vault/audits/CODE-QUALITY-3.md`
- [ ] **A-061** — Security Audit #3
  - [ ] `cargo audit` clean
  - [ ] Extended fuzz testing (1 hour per decode target)
  - [ ] Review all error paths for information leakage
  - [ ] Document in `.vault/audits/SECURITY-AUDIT-3.md`
- [ ] **A-062** — Architecture Audit #3
  - [ ] Module coherence review with 12+ transform modules
  - [ ] LSP handler still a thin dispatch layer?
  - [ ] Memory profiling under load
  - [ ] Document in `.vault/audits/ARCH-AUDIT-3.md`
- [ ] **A-063** — Dependency Audit #4
  - [ ] Full dep tree review
  - [ ] License compliance
  - [ ] Supply chain assessment
  - [ ] Document in `.vault/audits/DEP-AUDIT-4.md`
- [ ] **A-064** — UX Audit #3
  - [ ] Full feature walkthrough on macOS, Linux, Windows
  - [ ] Code action count sanity check (not overwhelming with 50+ actions?)
  - [ ] Performance audit with real-world file sizes
  - [ ] Document in `.vault/audits/UX-AUDIT-3.md`

#### Verification

- [ ] All five audit reports committed to `.vault/audits/`
- [ ] Test coverage ≥ 90% confirmed
- [ ] Zero security advisories, zero clippy warnings, zero dead code
- [ ] Cross-platform walkthrough completed on macOS, Linux, Windows

### 📋 PM REVIEW: PMR-5 — v1.0 Readiness Review

**Priority:** Critical | **Impact:** Very High | **Effort:** Small | **Risk:** Low
**Source:** PM Governance Cadence
**Status:** Not Started
**Dependencies:** ARI-4, A-060, A-061, A-062, A-063, A-064

#### Definition of Done

- [ ] **PMR-5** — Conduct v1.0 Readiness Review
  - [ ] Is the extension stable enough for a v1.0 commitment?
  - [ ] Are there any known bugs that would embarrass a 1.0 label?
  - [ ] Is the community healthy? (contributors, issue response time, discussion activity)
  - [ ] ARI ≥ 90/100 confirmed?
  - [ ] All audit findings from A-060–A-064 resolved?
  - [ ] Decision: **ship v1.0** or continue iterating as 0.x
  - [ ] If v1.0: define semantic versioning policy going forward (breaking changes = major bump)
  - [ ] Document decisions in `.vault/pm-reviews/PMR-5.md`

#### Verification

- [ ] `.vault/pm-reviews/PMR-5.md` committed with v1.0 ship/no-ship decision and evidence
- [ ] Semver policy documented if shipping v1.0
- [ ] All blocking audit findings resolved

**Phase 6 Exit Criteria:** Advanced features driven by community demand. ARI ≥ 90/100. All CI gates at maximum strictness. Test coverage ≥ 85% on `transforms/`. Full audit suite passed. v1.0 decision made.

---

## Backlog & Parking Lot

> Ideas captured but not yet prioritised. Community upvotes and PMR-4 evidence drive promotion into a Phase.

- [ ] **B-001** — `ROT13` encode/decode (the gentleman's encryption)
  - **Priority:** Low | **Impact:** Low | **Effort:** Small | **Risk:** Low
  - **Status:** Not Started
  - **Dependencies:** Phase 6 complete or PMR-4 promotion

- [ ] **B-002** — `Morse Code` encode/decode
  - **Priority:** Low | **Impact:** Low | **Effort:** Small | **Risk:** Low
  - **Status:** Not Started
  - **Dependencies:** Phase 6 complete or PMR-4 promotion

- [ ] **B-003** — `NATO Phonetic Alphabet` conversion
  - **Priority:** Low | **Impact:** Low | **Effort:** Small | **Risk:** Low
  - **Status:** Not Started
  - **Dependencies:** Phase 6 complete or PMR-4 promotion

- [ ] **B-004** — `Lorem Ipsum` generator (replace selection with N paragraphs)
  - **Priority:** Low | **Impact:** Low | **Effort:** Small | **Risk:** Low
  - **Status:** Not Started
  - **Dependencies:** Phase 6 complete or PMR-4 promotion

- [ ] **B-005** — `Markdown → HTML` conversion
  - **Priority:** Low | **Impact:** Medium | **Effort:** Medium | **Risk:** Low
  - **Status:** Not Started
  - **Dependencies:** Phase 6 complete or PMR-4 promotion

- [ ] **B-006** — `HTML → Markdown` conversion
  - **Priority:** Low | **Impact:** Medium | **Effort:** Medium | **Risk:** Low
  - **Status:** Not Started
  - **Dependencies:** Phase 6 complete or PMR-4 promotion

- [ ] **B-007** — `CSV ↔ TSV` conversion
  - **Priority:** Low | **Impact:** Low | **Effort:** Small | **Risk:** Low
  - **Status:** Not Started
  - **Dependencies:** Phase 6 complete or PMR-4 promotion

- [ ] **B-008** — `JSON Schema` generation from JSON sample
  - **Priority:** Low | **Impact:** Medium | **Effort:** Medium | **Risk:** Low
  - **Status:** Not Started
  - **Dependencies:** Phase 6 complete or PMR-4 promotion

- [ ] **B-009** — `HMAC-SHA256` computation (requires key input — UX challenge)
  - **Priority:** Low | **Impact:** Medium | **Effort:** High | **Risk:** Medium
  - **Status:** Not Started
  - **Dependencies:** Phase 6 complete or PMR-4 promotion

- [ ] **B-010** — `QR Code` generation (output as Unicode block art)
  - **Priority:** Low | **Impact:** Low | **Effort:** Medium | **Risk:** Low
  - **Status:** Not Started
  - **Dependencies:** Phase 6 complete or PMR-4 promotion

- [ ] **B-011** — `Color Code` conversions (hex ↔ rgb ↔ hsl)
  - **Priority:** Low | **Impact:** Medium | **Effort:** Small | **Risk:** Low
  - **Status:** Not Started
  - **Dependencies:** Phase 6 complete or PMR-4 promotion

- [ ] **B-012** — `Slug` generation (URL-safe slugs from titles)
  - **Priority:** Low | **Impact:** Low | **Effort:** Small | **Risk:** Low
  - **Status:** Not Started
  - **Dependencies:** Phase 6 complete or PMR-4 promotion

- [ ] **B-013** — `Emmet Abbreviation` expansion
  - **Priority:** Low | **Impact:** Medium | **Effort:** High | **Risk:** Medium
  - **Status:** Not Started
  - **Dependencies:** Phase 6 complete or PMR-4 promotion

- [ ] **B-014** — `SQL Formatter` (pretty print SQL)
  - **Priority:** Low | **Impact:** Medium | **Effort:** Medium | **Risk:** Low
  - **Status:** Not Started
  - **Dependencies:** Phase 6 complete or PMR-4 promotion

- [ ] **B-015** — Custom user-defined transformations via config (pipe through shell command)
  - **Priority:** Low | **Impact:** High | **Effort:** High | **Risk:** High
  - **Status:** Not Started
  - **Dependencies:** Phase 6 complete or PMR-4 promotion

- [ ] **B-016** — ARI P2 Remediation: Add `package.json` with proxy scripts for ariscan detection
  - **Priority:** Low | **Impact:** Medium | **Effort:** Small | **Risk:** Low
  - **Status:** Not Started
  - **Notes:** ariscan v0.1.0 has Node.js bias — looks for `package.json` scripts for test/lint detection. Makefile with `make test`/`make lint` targets exists but is not detected. Adding a thin `package.json` with `"test": "make test"` etc. would boost P2 score. Alternatively, wait for ariscan to support Makefile/Cargo detection natively.

- [ ] **B-017** — ARI P5 Remediation: Add machine-readable error taxonomy documentation
  - **Priority:** Low | **Impact:** Medium | **Effort:** Small | **Risk:** Low
  - **Status:** Not Started
  - **Notes:** ariscan P5 (Doc Machine-Readability) scored 25/100. It looks for API specs (OpenAPI/GraphQL), formal error taxonomy docs, and runbooks. `StringKnifeError` enum serves as the error taxonomy in code but is not surfaced as a standalone doc. Consider generating an error catalog from rustdoc or adding an `errors.md`.

- [ ] **B-018** — ARI P6 Remediation: Investigate ariscan Rust-specific build determinism detection
  - **Priority:** Low | **Impact:** Medium | **Effort:** Small | **Risk:** Low
  - **Status:** Not Started
  - **Notes:** ariscan P6 (Build Determinism & Type Safety) scored 50/100. It checks for `tsconfig.json strict` mode (not applicable to Rust). Rust's type system + Clippy pedantic lints provide equivalent safety. This may need an ariscan enhancement for Rust ecosystem detection.

- [ ] **B-019** — Prontiq blog post: "Building a Zed Extension Agent-First" (drafted from `docs/ariscan-case-study.md`)
  - **Priority:** Low | **Impact:** Medium | **Effort:** Medium | **Risk:** Low
  - **Status:** Not Started
  - **Dependencies:** T-670 complete

- [ ] **B-020** — Conference talk abstract: "ARI Score as an Engineering Metric" using StringKnife as worked example
  - **Priority:** Low | **Impact:** Medium | **Effort:** Medium | **Risk:** Low
  - **Status:** Not Started
  - **Dependencies:** T-670 complete

---

## Release Cadence

| Version | Phase | Target | Scope | Gate |
|---------|-------|--------|-------|------|
| `v0.1.0` | 0 + 1 | MVP | Bootstrap + Core encoding/decoding | ARI-0 ≥ 70/100, ARI-1 ≥ 75/100, PMR-0, PMR-1 |
| `v0.2.0` | 2 | +2 weeks | Hashing, JWT, JSON/YAML operations | Arch Audit #1, Security Audit #1, PMR-2 |
| `v0.3.0` | 3 | +2 weeks | Case conversions, text transforms | ARI-2 ≥ 80/100, Code Quality #2, UX Audit #1 |
| `v0.4.0` | 4 | +1 week | Configuration, performance, polish | Arch Audit #2, PMR-3 (scope lock) |
| `v0.5.0` | 5 | +1 week | Store publication, community setup | ARI-3 ≥ 85/100, Security #2, UX Audit #2 |
| `v1.0.0` | 6 | +4 weeks | Advanced features, stability | ARI-4 ≥ 90/100, Full audit suite, PMR-5 |

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
- **VCS:** Git is primary. [Jujutsu (jj)](https://github.com/martinvonz/jj) is supported as an optional colocated workflow — `.jj/` is gitignored. No repo-level jj config; contributors use their own `~/.jjconfig.toml`.
- **Zero network, zero telemetry:** All operations are local, deterministic, and offline.
- **Cross-platform binaries:** macOS (Intel + ARM), Linux (x86_64 + ARM), Windows (x86_64). No FFI, no system library links.
- **Dependency budget:** < 150 transitive crates at v1.0. No `unsafe` in `transforms/`. All deps pass `cargo-deny`.
- **Performance contract:** < 100ms for 100KB input. > 1MB returns `InputTooLarge` error.
- **Agent-first:** Repository structure, test patterns, module boundaries, error types, and documentation designed for AI-agent consumption from commit zero. `ariscan` scores are a first-class engineering metric.
- **Codebase intelligence vault:** `.vault/` is an Obsidian-compatible knowledge graph providing persistent agent memory, ARI pillar tracking, architecture decision records, session handoff continuity, and codebase pattern documentation. Frontmatter is the structured API; wikilinks are the navigation graph; plain markdown is the format contract.

---

## Document Trail

| Directory | Contents | Consumer |
|-----------|----------|----------|
| `CLAUDE.md` | Agent entry point — architecture summary, vault protocol, constraints | Agents (cold start) |
| `HINTS.md` | Human overrides, suppressions, style rules, vault maintenance policy | Agents (before changes) |
| `.vault/ari/` | ARI Dashboard, per-pillar notes, checkpoint reports | Agents + ariscan |
| `.vault/architecture/` | ADRs, system context | Agents (before arch changes) |
| `.vault/sessions/` | NEXT-SESSION.md handoff, Session Log, session notes | Agents (every session) |
| `.vault/patterns/` | "How to add a transform", Gotchas, dependency constraints | Agents (before implementing) |
| `.vault/transforms/` | Registry of all transforms with status | Agents (after implementing) |
| `.vault/pm-reviews/` | PM Review index, decision records | PO + agents |
| `.vault/audits/` | Audit index — all audit series | PO + agents |
| `.vault/templates/` | Session Template, ARI Checkpoint Template | Agents (creating notes) |
| `.claude/skills/` | Vault interaction skill for Claude Code | Claude Code agents |
| `.vault/ari/` | ARI checkpoint reports: ARI-BASELINE.md, ARI-0.md through ARI-4.md | Historical reference |
| `.vault/pm-reviews/` | PM review decision records: PMR-0.md through PMR-5.md | Historical reference |
| `.vault/audits/` | Audit reports: CODE-QUALITY-{N}.md, SECURITY-AUDIT-{N}.md, ARCH-AUDIT-{N}.md, DEP-AUDIT-{N}.md, UX-AUDIT-{N}.md | Historical reference |

---

*This document is the living source of truth for the StringKnife product. Update it as tickets are completed, PM reviews adjust priorities, and ariscan scores evolve.*
