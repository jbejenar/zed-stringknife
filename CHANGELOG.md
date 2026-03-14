# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.5.0] - 2026-03-14

### Added

#### Encoding & Decoding (Phase 1)
- Base64 encode/decode (RFC 4648, standard and URL-safe alphabets)
- URL encode/decode (RFC 3986, plus component encoding)
- HTML entity encode/decode (named, decimal, and hex numeric entities)
- Hex encode/decode (with `0x` prefix and space-separated byte support)
- Unicode escape/unescape (`\uXXXX` and `\U{XXXXXX}` formats)
- Show Unicode Codepoints inspector
- Smart detection: decode actions only appear when selection matches encoding pattern

#### Hashing & Cryptographic (Phase 2)
- MD5, SHA-1, SHA-256, SHA-512 hash digests (pure Rust, zero dependencies)
- CRC32 checksum
- Configurable hash output format (lowercase/uppercase)
- JWT decode: header, payload, and full decode with human-readable timestamps

#### Data Format Operations (Phase 2)
- JSON pretty print (configurable indent) and minify
- JSON string escape/unescape
- XML pretty print and minify
- CSV to JSON array conversion

#### Case Conversions (Phase 3)
- 13 case variants: UPPER, lower, Title, Sentence, camelCase, PascalCase,
  snake_case, SCREAMING_SNAKE_CASE, kebab-case, dot.case, path/case,
  CONSTANT_CASE, Toggle Case
- Smart word boundary detection (camelCase splits, acronyms, number boundaries)

#### Whitespace & Line Operations (Phase 3)
- Trim (all, leading, trailing), collapse whitespace
- Remove blank lines, remove duplicate lines (preserves order)
- Sort lines (A-Z, Z-A, by length), reverse lines, shuffle lines
- Number lines

#### Escape Operations (Phase 3)
- Backslash escape/unescape
- Regex metacharacter escape
- SQL string escape, shell string escape, CSV field escape

#### String Inspection (Phase 3)
- Character count (chars, bytes, words, lines)
- String byte length (UTF-8)
- Encoding detection (identifies Base64, URL-encoded, hex, JWT patterns)

#### Miscellaneous
- Reverse string

#### Configuration (Phase 4)
- `stringknife.enabledCategories`: filter which transform categories appear
- `stringknife.maxCodeActions`: limit context menu items (default: 50)
- `stringknife.smartDetection`: toggle smart decode suggestions (default: true)
- `stringknife.hashOutputFormat`: lowercase or uppercase hash output
- `stringknife.jsonIndent`: configurable JSON indent (default: 2 spaces)
- `stringknife.base64LineBreaks`: MIME-style 76-char line wrapping
- `stringknife.logLevel`: configurable structured logging level
- Live configuration reload via `workspace/didChangeConfiguration`

#### Performance & Reliability (Phase 4)
- Input size limit: 1MB max with clear error message
- Computation timeout: 5 seconds for pathological inputs
- Criterion benchmark suite for regression testing
- No-panic guarantee: 14 adversarial input tests across all decode paths
- Structured logging with tracing (operation, input_size, duration fields)

### Infrastructure

- Three-layer architecture: WASM extension, LSP server, transform engine
- 371 tests (329 core + 14 no-panic + 28 LSP integration)
- 83 transitive crates (budget: 150)
- Zero `unsafe` blocks, zero `unwrap()` in library code
- CI: build, test, lint, cargo-deny, ariscan on every PR
- Cross-compilation targets: macOS (x86_64, aarch64), Linux (x86_64, aarch64), Windows (x86_64)

## [0.0.1] - 2025-12-01

### Added

- Project scaffolding: `extension.toml`, WASM crate, `.gitignore`, `LICENSE` (MIT)
