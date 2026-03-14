# StringKnife

A surgical string transformation toolkit for the [Zed editor](https://zed.dev), delivered as an LSP-based extension with code-action transforms.

## Features

Select text in any file, then trigger code actions to transform it in place:

| Category | Operations |
|----------|-----------|
| Encoding | Base64, Base64URL, URL, HTML, Hex encode/decode, Unicode escape/unescape |
| Hashing | MD5, SHA-1, SHA-256, SHA-512, CRC32 |
| Case | UPPER, lower, Title, Sentence, camelCase, PascalCase, snake_case, SCREAMING_SNAKE, kebab-case, dot.case, path/case, CONSTANT_CASE, Toggle |
| JSON | Pretty print, minify, escape, unescape |
| XML | Pretty print, minify |
| CSV | CSV to JSON |
| Whitespace | Trim, collapse, remove blanks/dupes, sort/reverse/shuffle/number lines |
| Escape | Backslash, regex, SQL, shell, CSV |
| Inspect | Character count, byte length, encoding detection |
| Misc | Reverse string, JWT decode (header/payload/full) |

> **64 code actions** across 10 categories. See [CHANGELOG](CHANGELOG.md) for the full feature list and [roadmap](roadmap/roadmap.md) for the plan.

## Usage

1. **Select text** in any file
2. Press **`Cmd+.`** (macOS) or **`Ctrl+.`** (Linux) to open the code actions menu — or right-click the selection
3. Pick a **StringKnife** transform — the selected text is replaced in place

Only transforms that produce a different result are shown. For example, "Base64 Decode" won't appear if the selection isn't valid Base64.

## Installation

### From Zed Extensions

1. Open Zed
2. `Cmd+Shift+P` → "zed: extensions"
3. Search for **StringKnife** and install

## Configuration

Configure StringKnife through Zed's `settings.json`. All settings are optional — the defaults work out of the box.

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `stringknife.enabledCategories` | `string[]` | All categories | Which transform categories to show. Valid: `encoding`, `hashing`, `case`, `json`, `xml`, `csv`, `whitespace`, `escape`, `inspect`, `misc` |
| `stringknife.maxCodeActions` | `number` | `50` | Maximum number of code actions shown in the context menu |
| `stringknife.smartDetection` | `boolean` | `true` | When true, decode actions only appear if the selection looks like that encoding. When false, all decode actions are shown unconditionally |
| `stringknife.hashOutputFormat` | `string` | `"lowercase"` | Hash digest format: `"lowercase"` or `"uppercase"` |
| `stringknife.jsonIndent` | `number` | `2` | Spaces per indent level for JSON Pretty Print |
| `stringknife.base64LineBreaks` | `boolean` | `false` | Wrap Base64 output at 76 characters per line (MIME style) |

### Example `settings.json`

```json
{
  "lsp": {
    "stringknife-lsp": {
      "initialization_options": {
        "enabledCategories": ["encoding", "hashing", "case", "json"],
        "maxCodeActions": 20,
        "smartDetection": true,
        "hashOutputFormat": "uppercase",
        "jsonIndent": 4,
        "base64LineBreaks": false
      }
    }
  }
}
```

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
