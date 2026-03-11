# StringKnife

A surgical string transformation toolkit for the [Zed editor](https://zed.dev), delivered as an LSP-based extension with context-menu code actions.

## Features

Select text in any file and right-click to access 50+ string transformations:

| Category | Operations |
|----------|-----------|
| Encoding | Base64, URL, HTML, Hex encode/decode |
| Hashing | MD5, SHA-1, SHA-256, SHA-512, CRC32 |
| Case | camelCase, PascalCase, snake_case, kebab-case, SCREAMING_SNAKE, and more |
| JSON/XML | Pretty-print, minify, escape/unescape |
| JWT | Decode header, payload, or full token |
| Numbers | Decimal ↔ hex ↔ binary ↔ octal |
| UUID | Generate v4/v7, validate |
| Timestamps | Epoch ↔ ISO 8601 ↔ human-readable |
| Extract | Emails, URLs, IPs from text |
| Whitespace | Trim, collapse, sort lines, remove duplicates |
| Escape | Backslash, regex, SQL, shell, CSV |
| Compress | Gzip/deflate ↔ Base64 |

> **Status:** Under development. See [roadmap](roadmap/roadmap.md) for progress.

## Installation

### From Zed Extensions (coming soon)

1. Open Zed
2. `Cmd+Shift+P` → "Extensions: Install Extension"
3. Search for "StringKnife"

### Dev Install

```bash
# Clone and build
git clone https://github.com/jbejenar/zed-stringknife.git
cd zed-stringknife
cargo build --release

# Install as dev extension in Zed
# Cmd+Shift+P → "Extensions: Install Dev Extension"
# Select this repository's root directory
```

## How It Works

StringKnife uses a custom Language Server that provides code actions for text transformations. When you select text, the LSP offers relevant transformations in Zed's context menu. The architecture:

1. **WASM Extension** — Thin shim that manages the LSP binary lifecycle
2. **LSP Server** — Routes `textDocument/codeAction` requests to transform functions
3. **Transform Engine** — Pure functions: `fn(&str) -> Result<String, Error>`

## License

[MIT](LICENSE)
