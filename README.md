# StringKnife

A surgical string transformation toolkit for the [Zed editor](https://zed.dev), delivered as an LSP-based extension with code-action transforms.

## Features

Select text in any file, then trigger code actions to transform it in place:

| Category | Operations |
|----------|-----------|
| Encoding | Base64, Base64URL, URL, HTML, Hex encode/decode |
| Unicode | Escape/unescape, show codepoints |
| Misc | Reverse string |
| *Planned* | Hashing, case conversion, JSON/XML, JWT, UUID, timestamps, and more |

> **Status:** Under active development — Phase 1 (core transforms). See [roadmap](roadmap/roadmap.md) for the full plan.

## Usage

1. **Select text** in any supported file
2. Open code actions:
   - Right-click the selection and choose from the **StringKnife** entries, or
   - Press `Cmd+.` (macOS) / `Ctrl+.` (Linux) to open the code action menu
3. Pick a transform — the selected text is replaced in place

Only transforms that produce a different result from the input are shown, so the menu stays clean (e.g., "Base64 Decode" won't appear if the selection isn't valid Base64).

## Installation

### From Zed Extensions (coming soon)

1. Open Zed
2. `Cmd+Shift+P` → "zed: extensions"
3. Search for "StringKnife" and install

## Usage

1. **Select text** in any file
2. Press **`Cmd+.`** (macOS) or **`Ctrl+.`** (Linux) to open the code actions menu — or right-click the selection
3. Pick a **StringKnife** transform — the selected text is replaced in place

Only transforms that produce a different result are shown. For example, "Base64 Decode" won't appear if the selection isn't valid Base64.

## Architecture

StringKnife is a three-layer stack — arrows point downward only:

```
WASM Extension (src/lib.rs)          → Zed API surface, starts the LSP binary
LSP Server     (stringknife-lsp/)    → JSON-RPC dispatch, code action routing
Transform Core (stringknife-core/)   → Pure functions: fn(&str) -> Result<String>
```

The transform layer has zero LSP dependencies, no I/O, and no side effects.

## License

[MIT](LICENSE)
