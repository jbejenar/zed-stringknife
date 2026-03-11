---
type: adr
status: accepted
tags: [adr, architecture]
---

# System Context

## Component Architecture

StringKnife is a three-layer system:

```
Layer 1  WASM shim         src/lib.rs              (Zed API surface)
Layer 2  LSP router         stringknife-lsp/src/    (JSON-RPC dispatch)
Layer 3  Transform engine   stringknife-core/src/   (pure fn logic)
```

### Layer 1: Zed Extension (WASM)

Thin shim compiled to `wasm32-wasip1`. Manages LSP binary lifecycle via
`zed_extension_api`. Contains zero business logic.

- `language_server_command()` → path to `stringknife-lsp`
- `language_server_initialization_options()` → JSON config

### Layer 2: LSP Server (native binary)

Speaks LSP over stdio. Maintains document text state. Routes
`textDocument/codeAction` to Layer 3. Never contains transform logic.

- [[ADR-001 LSP Architecture]] — Why LSP over alternatives

### Layer 3: Transform Engine (pure library)

Every transform: `fn(&str) -> Result<String, StringKnifeError>`

- Zero LSP dependencies
- No I/O, no side effects, no shared state
- Publishable independently

## Data Flow

1. User selects text in Zed
2. Zed sends `textDocument/codeAction` to LSP (stdio)
3. LSP extracts selected text from document store
4. Smart detection identifies applicable decode operations
5. LSP returns `CodeAction[]` with `WorkspaceEdit` payloads
6. User picks action → Zed applies the `WorkspaceEdit`

## Key Constraints

- Arrows point downward only (no upward dependencies)
- `transforms/` has zero LSP dependencies
- No `unwrap()` in library code
- No `unsafe` in `transforms/`
- < 150 transitive crates at v1.0
- < 100ms for 100KB input; > 1MB returns `InputTooLarge`
