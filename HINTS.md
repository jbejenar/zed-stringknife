# HINTS.md — Agent Hints, Overrides & Conventions

> This file contains human-authored overrides, intentional suppressions,
> style rules, and vault maintenance policy. Read before making changes.

## Project Overview

StringKnife is a Zed editor extension for string/text manipulation.
Three-layer architecture: WASM shim -> LSP router -> Transform engine.
See `CLAUDE.md` for the full architecture summary.

## Style & Conventions

- **Rust edition:** 2021
- **Clippy:** `#![warn(clippy::all, clippy::pedantic)]` — no suppressions without comment
- **Error handling:** All transforms return `Result<T, StringKnifeError>`. No panics. No `unwrap()` in library code.
- **Module structure:** One transform per file in `stringknife-core/src/transforms/`. Each file exports a single public function.
- **Test pattern:** Each transform file has a corresponding `#[cfg(test)]` module. Tests are pure — no I/O, no shared state.
- **Commit messages:** Imperative mood, reference ticket number (e.g., "T-042 Add base64 encode transform")

## Intentional Suppressions

These look wrong but are deliberate — do not "fix" them:

- `transforms/hash.rs` — `#![allow(clippy::many_single_char_names, clippy::unreadable_literal, clippy::needless_range_loop)]`: Hash algorithms (MD5, SHA-1, SHA-256, SHA-512) use single-char working variables (a-h), RFC-specified hex constants, and indexed array mutation that matches the reference specifications from FIPS 180-4 and RFC 1321.

## Dependency Policy

- **Budget:** < 150 transitive crates at v1.0
- **Approval:** New dependencies require justification in PR description
- **Audit:** All deps must pass `cargo-deny` (license + advisory)
- **No wildcards:** Pin all dependency versions in `Cargo.toml`

## Vault Maintenance (mandatory)

Agents interacting with this repository **must** follow the vault session protocol.
This is not optional — it is a project requirement tracked by EPIC-0.1A.

### Session Start
1. Read `.vault/sessions/NEXT-SESSION.md` for current state and handoff notes
2. Read this file (`HINTS.md`) for any new overrides or suppressions
3. Note the current phase, active ticket, and any blockers

### Session End
1. Create a session note from `.vault/templates/Session Template.md`
2. Update `.vault/sessions/NEXT-SESSION.md` with what you did and what comes next
3. Add a row to `.vault/sessions/Session Log.md`
4. If you implemented a transform, update `.vault/transforms/Transform Registry.md`
5. If you discovered a gotcha, add it to `.vault/patterns/Gotchas.md`

### ARI Dashboard
- The ARI Dashboard (`.vault/ari/ARI Dashboard.md`) is **manually updated** by human review
- Do not modify ARI scores without explicit human approval
- `ariscan` output is the source of truth — dashboard reflects it, not the other way around

## Version Control

- **Git** is the primary VCS
- **[Jujutsu (jj)](https://github.com/martinvonz/jj)** is supported as an optional colocated workflow
  - Run `jj git init --colocate` in the repo root to enable
  - `.jj/` is gitignored — no jj artifacts leak into git history
  - No repo-level `.jjconfig.toml` — use your personal `~/.jjconfig.toml`
  - jj's immutable commit model and automatic rebasing work well with agent workflows
  - PRs still go through GitHub — use `jj git push` to push branches

## Branch Protection

- `main` is protected — all changes via PR
- Required CI checks: build, test, lint, deny, audit
- No force pushes to `main`

## How to Add a New Operation

See `.vault/patterns/Adding a New Transform.md` for the full guide. Quick summary:

1. Add a pure function in `stringknife-core/src/transforms/<module>.rs`
   - Signature: `pub fn name(input: &str) -> Result<String, StringKnifeError>`
   - Check `MAX_INPUT_SIZE` at entry
   - Add `#[cfg(test)]` module with tests
2. Export from `stringknife-core/src/transforms/mod.rs` (if new module)
3. Register as code action in `stringknife-lsp/src/main.rs` `build_actions()` function
4. Update `.vault/transforms/Transform Registry.md`
5. Run `make test && make lint` to verify

## File System Rules

- No file system access in transforms (Layer 3)
- No network calls anywhere in the codebase
- WASM shim (Layer 1) may only use Zed extension API
- LSP router (Layer 2) may use stdio for JSON-RPC only
