# Zed StringKnife — Product Roadmap

> **A surgical string transformation toolkit for the Zed editor, delivered as a Language Server Protocol extension with context-menu code actions.**

**Product Owner:** John Bejenariu
**Repository:** `zed-stringknife`
**License:** MIT
**Extension ID:** `stringknife`
**Target Zed API:** `zed_extension_api` v0.7.x+

---

## Technical Architecture

> Full architecture reference (system context, data flow, dependency budget, performance/security models, CI gate policy, PM governance, audit schedule) archived to [`roadmap/archive/technical-architecture.md`](archive/technical-architecture.md). See also `.vault/architecture/System Context.md`.

Three-layer architecture: **WASM shim** (Layer 1) -> **LSP router** (Layer 2) -> **Transform engine** (Layer 3). Arrows point downward only. `transforms/` has zero LSP dependencies. Every transform is `fn(&str) -> Result<String, StringKnifeError>`.

**Current stats:** 326 tests (321 core + 5 LSP). 64 code actions. 65 public transform functions across 15 modules. Zero dependencies in stringknife-core. ~79 transitive crates in stringknife-lsp.

---

## Phase 0 — Project Bootstrap ✅ [ARCHIVED]
> Completed 2026-03-11. Full record: [roadmap/archive/phase-0.md](archive/phase-0.md)

## Phase 1 — Core Encoding & Decoding ✅ [ARCHIVED]
> Completed 2026-03-11. Full record: [roadmap/archive/phase-1.md](archive/phase-1.md)

## Phase 2 — Hashing, Cryptographic & Data Format Operations ✅ [ARCHIVED]
> Completed 2026-03-11. Full record: [roadmap/archive/phase-2.md](archive/phase-2.md)

## Phase 3 — Text Transformation & Case Conversion ✅ [ARCHIVED]
> Completed 2026-03-11. Full record: [roadmap/archive/phase-3.md](archive/phase-3.md)

---

## Phase 3.5 — Housekeeping

> **Goal:** Clean up roadmap debt, vault duplicates, and gate backlog before Phase 4 work begins.

### T-380: Roadmap Archival

**Status:** Done

Move completed Phases 0–3 (~445 checked items) to `roadmap/archive/phase-{N}.md`. Replace inline content with one-line stubs. Archive Technical Architecture reference material. Move unchecked items from those phases to Deferred Gates section.

- [x] Create `roadmap/archive/` directory
- [x] Archive Phase 0 to `roadmap/archive/phase-0.md`
- [x] Archive Phase 1 to `roadmap/archive/phase-1.md`
- [x] Archive Phase 2 to `roadmap/archive/phase-2.md`
- [x] Archive Phase 3 to `roadmap/archive/phase-3.md`
- [x] Archive Technical Architecture to `roadmap/archive/technical-architecture.md`
- [x] Replace phases with one-line stubs in main roadmap
- [x] Create Deferred Gates & Human Actions section for unchecked items

### T-381: Vault Duplicate Cleanup

**Status:** Done (no duplicates found)

Audit `.vault/ari/` for duplicate pillar notes and `.vault/architecture/` for duplicate ADRs. Fix broken wikilinks.

- [x] Audit `.vault/ari/` — only P1–P8 prefixed files exist (no originals to delete)
- [x] Audit `.vault/architecture/` — only `ADR-001 LSP Architecture.md` exists (no "Bet" variant)
- [x] Verify all wikilinks across vault — 19 wikilinks, 0 broken

### T-382: Gate Debt Triage

**Status:** Done

Categorise the 16 blocked items from NEXT-SESSION.md and create consolidated tracking.

- [x] Categorise each item as: (a) agent-doable, (b) human-required, (c) obsolete
- [x] Create tracking table in NEXT-SESSION.md
- [x] Create Deferred Gates & Human Actions section in roadmap
- [x] Remove obsolete item (T-667 already verified)

### T-383: Roadmap Line Count

**Status:** Done

Target: main roadmap under 800 lines after archival.

- [x] Archive Phases 0–3 (removed ~1128 lines)
- [x] Archive Technical Architecture reference (removed ~570 lines)
- [x] Verify final line count < 800

---

## Deferred Gates & Human Actions

> Items from Phases 0–3 that remain unchecked. Categorised by action type.

### Requires Human Action

| Item | Description | Phase | Blocker |
|------|-------------|-------|---------|
| **T-018/T-019/T-020** | Install dev extension in Zed, verify code actions work | 0 | Manual Zed testing |
| **ARI-0** | Run `ariscan` — minimum composite >= 70/100 | 0 exit | Human runs ariscan |
| **PMR-0** | Foundation Review — validate LSP architecture in Zed | 0 exit | PM review |
| **PMR-1** | MVP Scope Check — user-test with developers | 1 mid | PM review |
| **ARI-1** | Run `ariscan` — minimum composite >= 75/100 | 1 exit | Human runs ariscan |
| **PMR-2** | Feature Velocity Check — review Phase 2 velocity | 2 exit | PM review |
| **PMR-3** | Pre-Launch Review — full feature audit for v0.5.0 | 4 exit | PM review |
| **ARI-2** | Run `ariscan` — minimum composite >= 80/100 | 3 exit | Human runs ariscan |
| **A-031** | UX Audit #1 — install on clean Zed, test discoverability | 3 | Manual Zed testing |
| **T-224/T-225** | JSON <-> YAML conversion | 2 | YAML parser dep decision |
| **T-240/T-241** | TOML <-> JSON conversion | 2 | TOML parser dep decision |

### Deferred Low-Priority

| Item | Description | Phase | Notes |
|------|-------------|-------|-------|
| **T-038** | PR labeler workflow | 0 | Low priority, auto-label by size/scope |
| **T-039** | Merge queue configuration | 0 | Deferred until contributor count > 1 |
| **T-015** (partial) | Binary download from GitHub Releases | 0 | Deferred to release phase (Phase 5) |
| **A-010** (partial) | Test coverage measurement with `cargo-tarpaulin` | 1 | Linux-only, needs CI setup |
| **A-021** (partial) | Fuzz testing with `cargo-fuzz` | 2 | Needs nightly toolchain |

### Obsolete (Removed)

| Item | Reason |
|------|--------|
| **T-667** | Vault graph connectivity already verified programmatically (19 wikilinks, 0 broken) |

---

## Phase 4 — Configuration, Performance & Polish

> **Goal:** User-configurable behavior, performant operation on large selections, and production-quality error handling.

### EPIC-4.1: Extension Configuration

**Priority:** High | **Impact:** High | **Effort:** Medium | **Risk:** Low
**Source:** Product Roadmap v1 — Phase 4
**Status:** Not Started
**Dependencies:** EPIC-0.3 (LSP config plumbing)
**AI-first benefit:** Typed configuration schema with defaults makes agent-driven config changes safe and predictable.

Define and implement the LSP configuration schema, allowing users to customise behavior through Zed's `settings.json`. Includes category filtering, smart detection toggle, output format preferences, and live config reload.

#### Definition of Done

- [ ] **T-400** — Define LSP configuration schema (`initializationOptions`)
  - [ ] `stringknife.enabledCategories`: array of enabled categories (encoding, hashing, case, json, etc.)
  - [ ] `stringknife.maxCodeActions`: max number of code actions shown (default: 20)
  - [ ] `stringknife.smartDetection`: boolean to enable/disable smart decode suggestions (default: true)
  - [ ] `stringknife.hashOutputFormat`: `"lowercase"` | `"uppercase"` (default: lowercase)
  - [ ] `stringknife.jsonIndent`: number of spaces for pretty print (default: 2)
  - [ ] `stringknife.base64LineBreaks`: boolean for 76-char line wrapping (default: false)
- [ ] **T-401** — Read configuration from Zed settings via `initializationOptions`
- [ ] **T-402** — Handle `workspace/didChangeConfiguration` for live config updates
- [ ] **T-403** — Document all configuration options in README
- [ ] **T-404** — Add example Zed `settings.json` snippet to README

#### Verification

- [ ] Changing `settings.json` updates behavior without restarting Zed
- [ ] Disabling a category removes its code actions from the context menu
- [ ] Default values work correctly when no config is provided
- [ ] README configuration reference table matches actual behavior

### EPIC-4.2: Performance Optimization

**Priority:** High | **Impact:** High | **Effort:** Medium | **Risk:** Low
**Source:** Product Roadmap v1 — Phase 4
**Status:** Not Started
**Dependencies:** EPIC-0.4, all Phase 1–3 transforms
**AI-first benefit:** Benchmark-driven optimization with regression detection — agents can profile, identify bottlenecks, and optimize with guardrails.

Add benchmarks, input size guards, and streaming for large selections. Ensure all operations meet the performance contract (<100ms for 100KB).

#### Definition of Done

- [ ] **T-410** — Add `criterion` benchmark suite for all transform operations
- [ ] **T-411** — Add input size guard: reject inputs > 1MB with `InputTooLarge` error via `window/showMessage`
- [ ] **T-412** — Profile and optimise any transform that exceeds 50ms for 100KB input
- [ ] **T-413** — Add lazy code action resolution (`codeAction/resolve`) to avoid computing all edits upfront
- [ ] **T-414** — Benchmark and optimise smart detection for large text (>100KB)
- [ ] **T-415** — Add memory usage tracking and regression tests

#### Verification

- [ ] All transforms complete in <100ms for 100KB input (benchmark suite passes)
- [ ] 1MB+ input returns user-friendly error (not timeout or crash)
- [ ] `criterion` benchmarks run in CI and catch regressions
- [ ] Memory usage does not exceed 2x input size for any transform

### EPIC-4.3: Multi-Cursor Support

**Priority:** High | **Impact:** Very High | **Effort:** Medium | **Risk:** Medium
**Source:** Product Roadmap v1 — Phase 4
**Status:** Not Started
**Dependencies:** EPIC-0.3
**AI-first benefit:** Multi-cursor is a Zed differentiator — supporting it correctly makes StringKnife a native-feeling tool.

Ensure all code actions work correctly with Zed's multi-cursor (multiple selections). Each selection gets its own independent transform.

#### Definition of Done

- [ ] **T-420** — Handle multiple ranges in `CodeActionParams` (one transform per selection)
- [ ] **T-421** — Ensure `WorkspaceEdit` contains edits for all selections
- [ ] **T-422** — Handle mixed selections (e.g., some selections are Base64, others are not)
- [ ] **T-423** — Add integration tests for multi-cursor scenarios
- [ ] **T-424** — Document multi-cursor behavior in README

#### Verification

- [ ] Multi-cursor with 3 selections: all three get transformed independently
- [ ] Mixed selections: only valid selections get transformed, invalid ones show error notification
- [ ] Undo (Cmd+Z) restores all original selections atomically

### EPIC-4.4: Error Handling Polish

**Priority:** High | **Impact:** High | **Effort:** Small | **Risk:** Low
**Source:** Product Roadmap v1 — Phase 4
**Status:** Not Started
**Dependencies:** All Phase 1–3 transforms

Improve error messages across all transforms to be user-friendly, consistent, and actionable.

#### Definition of Done

- [ ] **T-430** — Audit all `StringKnifeError` messages for clarity
- [ ] **T-431** — Add `window/showMessage` notifications for transform errors
- [ ] **T-432** — Ensure errors include the operation name and a brief hint
- [ ] **T-433** — Add "did you mean?" suggestions (e.g., "This looks like Base64 but has invalid characters at position 42")
- [ ] **T-434** — Document all error types and messages in HINTS.md

#### Verification

- [ ] Error messages are understandable without reading source code
- [ ] Every error includes the operation name
- [ ] Suggestion hints are accurate and helpful

### 🔒 GATE: ARI-3 Checkpoint (Phase 4 Exit)

**Priority:** Critical | **Impact:** Very High | **Effort:** Small | **Risk:** Medium
**Source:** ARI Governance — Phase 4 exit gate
**Status:** Not Started
**Dependencies:** EPIC-4.1, EPIC-4.2, EPIC-4.3, EPIC-4.4

#### Definition of Done

- [ ] Run `ariscan` — **minimum composite score >= 85/100**
- [ ] Record scores in `.vault/ari/ARI-3.md`
- [ ] No pillar regression from ARI-2

### 🔍 AUDIT: Architecture Audit #2

**Priority:** High | **Impact:** High | **Effort:** Medium | **Risk:** Low
**Source:** Audit Schedule — Phase 4
**Status:** Not Started
**Dependencies:** EPIC-4.1, EPIC-4.2, EPIC-4.3, EPIC-4.4

#### Definition of Done

- [ ] **A-040** — Architecture Audit #2
  - [ ] Review config layer: is it clean or leaking into transforms?
  - [ ] Review multi-cursor implementation: any shared state?
  - [ ] Benchmark regression check
  - [ ] Document in `.vault/audits/ARCH-AUDIT-2.md`

### 📋 PM REVIEW: PMR-3 — Pre-Launch Review

**Priority:** High | **Impact:** Very High | **Effort:** Small | **Risk:** Low
**Source:** PM Governance Cadence
**Status:** Not Started
**Dependencies:** EPIC-4.1, EPIC-4.2, EPIC-4.3, EPIC-4.4

#### Definition of Done

- [ ] **PMR-3** — Conduct Pre-Launch Review
  - [ ] Full feature audit: what ships in v0.5.0?
  - [ ] Review README, demo assets, store listing
  - [ ] Decision: Go/No-Go for Phase 5 (store publication)
  - [ ] Document in `.vault/pm-reviews/PMR-3.md`

**Phase 4 Exit Criteria:** Configuration working. Performance benchmarked. Multi-cursor supported. Error messages polished. ARI >= 85/100. Architecture audit passed. PMR-3 complete.

---

## Phase 5 — Publish, Distribute & Community

> **Goal:** Extension published to the Zed Extension Store, discoverable, documented, and ready for community contributions.

### EPIC-5.1: Publication Preparation

**Status:** Not Started | **Dependencies:** PMR-3

- [ ] **T-500** — Verify extension ID `stringknife` is available
- [ ] **T-501** — Ensure `extension.toml` passes Zed validation
- [ ] **T-502** — Write comprehensive `README.md` with GIF demos
- [ ] **T-503** — Create extension icon/logo (SVG)
- [ ] **T-504** — Create demo GIFs showing key workflows
- [ ] **T-505** — Update `HINTS.md` with final architecture
- [ ] **T-506** — Add ARI badge to `README.md`
- [ ] **T-507** — Add "Built with ariscan" section to `README.md`
- [ ] **T-508** — Make `.vault/` browsable on GitHub
- [ ] **T-509** — Add GitHub topics for discoverability

### EPIC-5.2: Publish to Zed Extension Store

**Status:** Not Started | **Dependencies:** EPIC-5.1, ARI-3, A-050, A-051

- [ ] **T-510** — Fork `zed-industries/extensions`
- [ ] **T-511** — Add `stringknife` as Git submodule
- [ ] **T-512** — Add entry to `extensions.toml`
- [ ] **T-513** — Run `pnpm sort-extensions`
- [ ] **T-514** — Open PR to `zed-industries/extensions`
- [ ] **T-515** — Respond to review feedback
- [ ] **T-516** — Verify extension appears in store
- [ ] **T-517** — Test installation from store on clean Zed instance

### EPIC-5.3: Community & Maintenance

**Status:** Not Started | **Dependencies:** EPIC-5.2

- [ ] **T-520** — Create GitHub issue templates
- [ ] **T-521** — Create GitHub Discussions category
- [ ] **T-522** — Set up automated extension updates
- [ ] **T-523** — Create `SECURITY.md`
- [ ] **T-524** — Tag and release `v0.1.0`
- [ ] **T-525** — Announce on Zed Discord

### Gates & Audits (Phase 5)

- [ ] **ARI-3** — Run `ariscan` — minimum composite >= 85/100
- [ ] **A-050** — Pre-Publish Security Audit
- [ ] **A-051** — Pre-Publish UX Audit
- [ ] **PMR-4** — Post-Launch Retrospective (2 weeks after publish)

**Phase 5 Exit Criteria:** Extension live in store. ARI >= 85/100. Security + UX audits passed. Community pipeline in place.

---

## Phase 6 — Advanced Features (Post-Launch)

> **Goal:** Differentiate StringKnife from basic string tools with power-user features driven by community feedback and PMR-4 evidence.

### EPIC-6.1: Timestamp/Epoch Operations

**Status:** Not Started | **Dependencies:** EPIC-0.4, PMR-4

- [ ] **T-600** — `Unix Timestamp -> ISO 8601`
- [ ] **T-601** — `ISO 8601 -> Unix Timestamp`
- [ ] **T-602** — `Unix Timestamp -> Human Readable`
- [ ] **T-603** — Detect epoch timestamps
- [ ] **T-604** — Unit tests (edge cases: negative epochs, Y2K38, milliseconds)

### EPIC-6.2: Number Base Conversions

**Status:** Not Started | **Dependencies:** EPIC-0.4

- [ ] **T-610–T-615** — Decimal <-> Hex, Binary, Octal conversions
- [ ] **T-616** — Auto-detect base from prefix (`0x`, `0b`, `0o`)
- [ ] **T-617** — Unit tests

### EPIC-6.3: UUID & Random Generation

**Status:** Not Started | **Dependencies:** EPIC-0.4

- [ ] **T-620** — Generate UUID v4
- [ ] **T-621** — Generate UUID v7
- [ ] **T-622** — Validate UUID
- [ ] **T-623** — Generate Random String
- [ ] **T-624** — Unit tests

### EPIC-6.4: Regex & Pattern Operations

**Status:** Not Started | **Dependencies:** EPIC-0.4

- [ ] **T-630** — Extract Emails
- [ ] **T-631** — Extract URLs
- [ ] **T-632** — Extract IP Addresses (v4 + v6)
- [ ] **T-633** — Mask Sensitive Data
- [ ] **T-634** — Unit tests

### EPIC-6.5: Text Diff & Comparison

**Status:** Not Started | **Dependencies:** EPIC-0.4

- [ ] **T-640** — String Diff (Line)
- [ ] **T-641** — String Diff (Character)
- [ ] **T-642** — Unit tests

### EPIC-6.6: Compression

**Status:** Not Started | **Dependencies:** EPIC-0.4, EPIC-1.1

- [ ] **T-650–T-653** — Gzip/Deflate <-> Base64 compress/decompress
- [ ] **T-654** — Unit tests

### EPIC-6.7: ariscan Showcase Artefacts

- [ ] **T-670** — Create `docs/ariscan-case-study.md`
- [ ] **T-671** — Create ARI trajectory visualisation (SVG)
- [ ] **T-672** — Create `ARCHITECTURE.md` at repo root
- [ ] **T-673** — Outsider audit

### Gates & Audits (Phase 6)

- [ ] **ARI-4** — Run `ariscan` — minimum composite >= 90/100
- [ ] **A-060** — Code Quality Audit #3 (coverage >= 90%)
- [ ] **A-061** — Security Audit #3
- [ ] **A-062** — Architecture Audit #3
- [ ] **A-063** — Dependency Audit #4
- [ ] **A-064** — UX Audit #3
- [ ] **PMR-5** — v1.0 Readiness Review

**Phase 6 Exit Criteria:** ARI >= 90/100. Full audit suite passed. v1.0 decision made.

---

## Backlog & Parking Lot

> Ideas captured but not yet prioritised. Community upvotes and PMR-4 evidence drive promotion.

| Ticket | Description | Priority | Effort |
|--------|-------------|----------|--------|
| B-001 | ROT13 encode/decode | Low | Small |
| B-002 | Morse Code encode/decode | Low | Small |
| B-003 | NATO Phonetic Alphabet | Low | Small |
| B-004 | Lorem Ipsum generator | Low | Small |
| B-005 | Markdown -> HTML | Low | Medium |
| B-006 | HTML -> Markdown | Low | Medium |
| B-007 | CSV <-> TSV | Low | Small |
| B-008 | JSON Schema from sample | Low | Medium |
| B-009 | HMAC-SHA256 (needs key input UX) | Low | High |
| B-010 | QR Code (Unicode block art) | Low | Medium |
| B-011 | Color Code conversions (hex/rgb/hsl) | Low | Small |
| B-012 | Slug generation | Low | Small |
| B-013 | Emmet Abbreviation expansion | Low | High |
| B-014 | SQL Formatter | Low | Medium |
| B-015 | Custom user-defined transforms (pipe through shell) | Low | High |
| B-016 | ARI P2: Add `package.json` proxy scripts for ariscan | Low | Small |
| B-017 | ARI P5: Machine-readable error taxonomy doc | Low | Small |
| B-018 | ARI P6: Investigate ariscan Rust-specific detection | Low | Small |
| B-019 | Action registry pattern for `build_actions()` | Low | Medium |
| B-020 | YAML parser for JSON<->YAML (T-224/T-225) | Medium | High |
| B-021 | TOML parser for TOML<->JSON (T-240/T-241) | Medium | Medium |
| B-022 | Notification API for inspect actions | Medium | Small |
| B-023 | Fuzz testing setup with `cargo-fuzz` | Medium | Medium |

---

## Release Cadence

| Version | Phase | Scope | Gate |
|---------|-------|-------|------|
| `v0.1.0` | 0 + 1 | Bootstrap + Core encoding/decoding | ARI-0 >= 70, ARI-1 >= 75 |
| `v0.2.0` | 2 | Hashing, JWT, JSON/YAML | Arch + Security Audit |
| `v0.3.0` | 3 | Case conversions, text transforms | ARI-2 >= 80, UX Audit |
| `v0.4.0` | 4 | Configuration, performance, polish | Arch Audit #2, PMR-3 |
| `v0.5.0` | 5 | Store publication | ARI-3 >= 85, Security + UX Audit |
| `v1.0.0` | 6 | Advanced features, stability | ARI-4 >= 90, Full audit suite |

## Acceptance Criteria (Global)

1. **Functional:** Correct output for valid input.
2. **Error-safe:** Invalid input returns `Result::Err`, never panics.
3. **Tested:** Unit tests covering happy path, edge cases, error paths. Isolated.
4. **Documented:** Listed in README. Public function has rustdoc.
5. **Reversible:** Encode/decode pairs roundtrip to identity.
6. **Performant:** < 100ms for 100KB input.
7. **ARI-compatible:** Pure function in own module. No LSP type leakage.

---

*This document is the living source of truth for the StringKnife product. Update it as tickets are completed, PM reviews adjust priorities, and ariscan scores evolve.*
