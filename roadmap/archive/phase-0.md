# Phase 0 — Project Bootstrap (Archived)

> Completed 2026-03-11. This is the archived record of Phase 0.
> Active roadmap: `roadmap/roadmap.md`

> **Goal:** Repository scaffolded, CI green, dev extension installable in Zed with a single no-op code action proving the full pipeline works end-to-end. ARI foundations laid from first commit.

## EPIC-0.1: Repository & Toolchain Setup

**Priority:** Critical | **Impact:** Very High | **Effort:** Medium | **Risk:** Low
**Source:** Product Roadmap v1 — Phase 0
**Status:** Done
**Dependencies:** None
**AI-first benefit:** Deterministic repo structure enables agents to navigate and contribute from first clone.

Establish the foundational repository structure, Zed extension manifest, and WASM crate so that the project compiles and can be installed as a dev extension in Zed. This is the skeleton upon which all subsequent phases build.

### Definition of Done

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

### Verification

- [x] `cargo check` passes on the WASM crate without errors
- [x] `extension.toml` validates against Zed's extension schema
- [x] All files listed above exist at repository root
- [x] `.gitignore` excludes `target/`, `node_modules/`, `*.wasm`, `.jj/`

## EPIC-0.1A: Codebase Intelligence Vault (Persistent Agent Memory)

**Priority:** Critical | **Impact:** Very High | **Effort:** Medium | **Risk:** Low
**Source:** Roadmap Amendment — Codebase Intelligence Vault
**Status:** Done
**Dependencies:** EPIC-0.1

### Definition of Done

- [x] **T-655** — Create `CLAUDE.md` at repository root
- [x] **T-656** — Create `.vault/` directory structure and Obsidian config
- [x] **T-657** — Create `.vault/architecture/` — Architecture Decision Records
- [x] **T-658** — Create `.vault/ari/` — ARI Pillar Tracking
- [x] **T-659** — Create `.vault/sessions/` — Agent Session Infrastructure
- [x] **T-660** — Create `.vault/patterns/` — Codebase Patterns & Agent Guides
- [x] **T-661** — Create `.vault/transforms/` — Transform Registry
- [x] **T-662** — Create `.vault/pm-reviews/` and `.vault/audits/` indexes
- [x] **T-663** — Create `.vault/templates/`
- [x] **T-664** — Create `.claude/skills/vault-interaction/SKILL.md`
- [x] **T-665** — Update `.gitignore` for vault
- [x] **T-666** — Update `HINTS.md` to reference vault
- [x] **T-667** — Verify vault graph connectivity (verified programmatically)

### Verification

- [x] `.vault/` opens as Obsidian vault with connected graph
- [x] All wikilinks resolve — 19 wikilinks verified
- [x] `Home.md` reaches every vault section within 2 hops

## EPIC-0.2: ARI Foundations (Agent-Readiness from Day One)

**Priority:** Critical | **Impact:** Very High | **Effort:** Medium | **Risk:** Low
**Source:** Product Roadmap v1 — Phase 0
**Status:** Done
**Dependencies:** EPIC-0.1, EPIC-0.1A

### Definition of Done

- [x] **T-025** — Create `HINTS.md` at repository root
- [x] **T-026** — Create `rust-toolchain.toml` pinning stable channel
- [x] **T-027** — Commit `Cargo.lock` to version control
- [x] **T-028** — Configure strict Clippy lints in workspace `Cargo.toml`
- [x] **T-029** — Define `StringKnifeError` enum with structured error variants
- [x] **T-030** — Create `transforms/` module directory with `mod.rs`
- [x] **T-031** — Add `cargo-deny` configuration (`deny.toml`)
- [x] **T-032** — Add `cargo-audit` to CI pipeline
- [x] **T-033** — Add rustdoc comments on all public types and functions
- [x] **T-034** — Install and run `ariscan` — establish ARI-BASELINE (59/100)

### Verification

- [x] `cargo clippy -- -D warnings` passes with zero warnings
- [x] `cargo deny check` passes with zero violations
- [x] `ariscan` produces ARI-BASELINE report — 59/100 (L3 Capable)

## EPIC-0.3: Language Server Skeleton

**Priority:** Critical | **Impact:** Very High | **Effort:** High | **Risk:** Medium
**Source:** Product Roadmap v1 — Phase 0
**Status:** Done
**Dependencies:** EPIC-0.1

### Definition of Done

- [x] **T-010** — Create `lsp/` directory for the LSP binary crate
- [x] **T-011** — Create `lsp/Cargo.toml`
- [x] **T-012** — Implement minimal LSP server in `lsp/src/main.rs`
- [x] **T-013** — Add document text store to server state
- [x] **T-014** — Verify LSP binary compiles and runs standalone
- [x] **T-015** — Wire extension WASM to download/locate the LSP binary (partial — download deferred to release phase)

### Verification

- [x] `cargo build -p stringknife-lsp` compiles without errors
- [x] `stringknife-lsp --stdio` starts and responds to LSP initialize request
- [x] Document store correctly tracks open/changed documents

## EPIC-0.4: End-to-End Proof of Life

**Priority:** Critical | **Impact:** Very High | **Effort:** Medium | **Risk:** Medium
**Source:** Product Roadmap v1 — Phase 0
**Status:** Done (code complete — T-018/T-019/T-020 need manual Zed verification)
**Dependencies:** EPIC-0.2, EPIC-0.3

### Definition of Done

- [x] **T-016** — Add "StringKnife: Reverse String" hardcoded code action
- [x] **T-017** — Add unit test for reverse string transform
- [x] **T-035** — Document the dev install workflow in `CONTRIBUTING.md`

### Verification

- [x] `cargo test -p stringknife-core` passes with reverse string tests green (6 tests)

## EPIC-0.5: CI/CD Pipeline

**Priority:** Critical | **Impact:** High | **Effort:** Medium | **Risk:** Low
**Source:** Product Roadmap v1 — Phase 0
**Status:** Done (except T-038 labeler, T-039 merge queue — deferred)
**Dependencies:** EPIC-0.1, EPIC-0.2, EPIC-0.3

### Definition of Done

- [x] **T-021** — Create `.github/workflows/ci.yml`
- [x] **T-022** — Create `.github/workflows/release.yml`
- [x] **T-023** — Create `.github/workflows/ariscan.yml`
- [x] **T-024** — Add Dependabot config
- [x] **T-036** — Configure branch protection rules on `main`
- [x] **T-037** — Create `.github/pull_request_template.md`

### Verification

- [x] Push to `main` triggers CI workflow and all steps pass
- [x] Opening a PR triggers ariscan workflow and posts ARI score comment
- [x] Direct push to `main` is rejected by branch protection

## Dependency Audit #1

**Status:** Done

- [x] **A-001** — Run `cargo deny check` and review all transitive dependencies

**Phase 0 Exit Criteria:** Dev extension installs in Zed. Selecting text -> right-click -> "StringKnife: Reverse String" works. CI is green. Branch protection active on `main`. `.vault/` opens in Obsidian with connected graph. ARI >= 70/100. PMR-0 complete.
