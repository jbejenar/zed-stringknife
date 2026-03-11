# AGENTS.md — Vendor-Neutral Agent Context

> For Claude-specific instructions, see `CLAUDE.md`.
> This file provides context for any AI coding agent.

## Project

**StringKnife** — A Zed editor extension providing 50+ string/text manipulation
commands via LSP code actions.

## Architecture

```
Layer 1  WASM shim         src/lib.rs                    (Zed extension API)
Layer 2  LSP router         stringknife-lsp/src/main.rs   (JSON-RPC dispatch)
Layer 3  Transform engine   stringknife-core/src/          (pure functions)
```

Dependencies flow downward only. Layer 3 has zero LSP dependencies.

## Key Commands

```bash
# Build
cargo check --manifest-path stringknife-core/Cargo.toml
cargo build --manifest-path stringknife-lsp/Cargo.toml

# Test
cargo test --manifest-path stringknife-core/Cargo.toml
cargo test --manifest-path stringknife-lsp/Cargo.toml

# Lint
cargo clippy --manifest-path stringknife-core/Cargo.toml -- -D warnings
cargo clippy --manifest-path stringknife-lsp/Cargo.toml -- -D warnings

# Format
cargo fmt --manifest-path stringknife-core/Cargo.toml --check
cargo fmt --manifest-path stringknife-lsp/Cargo.toml --check

# WASM build (extension shim)
RUSTUP_TOOLCHAIN=stable-aarch64-apple-darwin RUSTC=$(rustup which rustc) cargo check --target wasm32-wasip1
```

## Hard Rules

- No `unwrap()` in library code — use `Result<T, StringKnifeError>`
- No `unsafe` in `transforms/`
- No network calls, no file system access in transforms
- All transforms: `fn(&str) -> Result<String, StringKnifeError>`
- < 150 transitive crates at v1.0

## File Map

| Path | Purpose |
|------|---------|
| `extension.toml` | Zed extension manifest |
| `Cargo.toml` + `src/lib.rs` | WASM extension shim |
| `stringknife-core/` | Transform engine (pure library) |
| `stringknife-lsp/` | LSP server binary |
| `roadmap/roadmap.md` | Living product roadmap |
| `.vault/` | Codebase intelligence vault (Obsidian-compatible) |
