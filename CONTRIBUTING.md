# Contributing to StringKnife

## Prerequisites

- [Rust](https://rustup.rs/) (stable, pinned via `rust-toolchain.toml`)
- [Zed editor](https://zed.dev)
- `wasm32-wasip1` target: `rustup target add wasm32-wasip1`

## Dev Setup

```bash
git clone https://github.com/jbejenar/zed-stringknife.git
cd zed-stringknife

# Build the WASM extension
cargo build --target wasm32-wasip1

# Build the LSP server (native)
cargo build -p stringknife-lsp --release

# Run tests
cargo test -p stringknife-core
```

## Installing as Dev Extension

1. Build the project (see above)
2. In Zed: `Cmd+Shift+P` → "Extensions: Install Dev Extension"
3. Select this repository's root directory
4. The extension loads and starts the LSP automatically

## Adding a New Transform

See `.vault/patterns/Adding a New Transform.md` for the step-by-step guide.

Quick summary:

1. Add a pure function in `stringknife-core/src/transforms/<module>.rs`
2. Signature: `pub fn name(input: &str) -> Result<String, StringKnifeError>`
3. Add tests in the same file under `#[cfg(test)]`
4. Register the code action in the LSP handler
5. Update `.vault/transforms/Transform Registry.md`

## Code Style

- **Clippy:** `#![warn(clippy::all, clippy::pedantic)]` — fix all warnings
- **No `unwrap()`** in library code — use `Result<T, StringKnifeError>`
- **No `unsafe`** in `transforms/`
- **Commit messages:** Imperative mood, reference ticket (e.g., "T-042 Add base64 encode transform")

## Pull Requests

- All changes go through PRs against `main`
- CI must pass: build, test, lint, `cargo-deny`, `cargo-audit`
- Include tests for new/modified code
- No new dependencies without justification
