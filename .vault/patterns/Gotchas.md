---
type: pattern
tags: [pattern]
---

# Gotchas

Things that look wrong but are deliberate. Read before "fixing" anything.

## Architecture

- **WASM crate is `cdylib`**: This is required by Zed's extension host. Do not
  change to `lib` or `rlib`.
- **Language server maps to 22+ file types**: Intentional — StringKnife should
  work everywhere, not just in specific languages.
- **No `language_server_initialization_options`**: Returns `None` for now. Will
  be wired to user config in Phase 2.

## Toolchain

- **Homebrew Rust vs rustup**: The machine may have both. WASM builds need the
  rustup toolchain: `RUSTUP_TOOLCHAIN=stable-aarch64-apple-darwin RUSTC=$(rustup which rustc)`.
- **`wasm32-wasip1` not `wasm32-wasi`**: Rust 1.78+ renamed the target.

## Dependencies

- **`zed_extension_api` pulls many transitive deps**: This is expected. The WASM
  crate's dep count does not count against the `stringknife-core` budget.
