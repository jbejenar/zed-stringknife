# Phase 1 — Core Encoding & Decoding (Archived)

> Completed 2026-03-11. This is the archived record of Phase 1.
> Active roadmap: `roadmap/roadmap.md`

> **Goal:** Ship the essential encoding/decoding operations that cover 90% of daily string manipulation needs.

## EPIC-1.1: Base64 Operations

**Status:** Done | **Dependencies:** EPIC-0.4

- [x] **T-100** — Implement `Base64 Encode` code action
- [x] **T-101** — Implement `Base64 Decode` code action
- [x] **T-102** — Implement `Base64URL Encode` code action
- [x] **T-103** — Implement `Base64URL Decode` code action
- [x] **T-104** — Unit tests for all Base64 variants (20 tests)

### Verification

- [x] `cargo test -p stringknife-core -- base64` passes all 20 tests
- [x] Roundtrip identity confirmed
- [x] Invalid Base64 returns `Err(StringKnifeError::InvalidInput)`

## EPIC-1.2: URL Encoding Operations

**Status:** Done | **Dependencies:** EPIC-0.4

- [x] **T-110** — Implement `URL Encode` code action
- [x] **T-111** — Implement `URL Decode` code action
- [x] **T-112** — Implement `URL Encode (Component)` code action
- [x] **T-113** — Unit tests for URL encoding (19 tests)

### Verification

- [x] `cargo test -p stringknife-core -- url` passes all 19 tests
- [x] RFC 3986 reserved characters correctly percent-encoded

## EPIC-1.3: HTML Entity Operations

**Status:** Done | **Dependencies:** EPIC-0.4

- [x] **T-120** — Implement `HTML Encode` code action
- [x] **T-121** — Implement `HTML Decode` code action
- [x] **T-122** — Unit tests for HTML entities (16 tests)

### Verification

- [x] `cargo test -p stringknife-core -- html` passes all 16 tests
- [x] Malformed entities pass through unchanged without error

## EPIC-1.4: Hex Operations

**Status:** Done | **Dependencies:** EPIC-0.4

- [x] **T-130** — Implement `Hex Encode` code action
- [x] **T-131** — Implement `Hex Decode` code action
- [x] **T-132** — Unit tests for hex operations (16 tests)

### Verification

- [x] `cargo test -p stringknife-core -- hex` passes all 16 tests
- [x] Roundtrip identity confirmed

## EPIC-1.5: Unicode Operations

**Status:** Done | **Dependencies:** EPIC-0.4

- [x] **T-140** — Implement `Unicode Escape` code action
- [x] **T-141** — Implement `Unicode Unescape` code action
- [x] **T-142** — Implement `Show Unicode Codepoints` code action
- [x] **T-143** — Unit tests for Unicode operations (25 tests)

### Verification

- [x] `cargo test -p stringknife-core -- unicode` passes all 25 tests
- [x] Emoji and multi-codepoint sequences roundtrip correctly

## EPIC-1.6: Code Action Categorisation & UX

**Status:** Done | **Dependencies:** EPIC-1.1, EPIC-1.2, EPIC-1.3, EPIC-1.4, EPIC-1.5

- [x] **T-150** — Group code actions under `"StringKnife"` category
- [x] **T-151** — Smart detection for relevant decode actions
- [x] **T-152** — Order code actions by relevance
- [x] **T-153** — Handle multi-line selections correctly
- [x] **T-154** — Handle empty selection (no code actions returned)

### Verification

- [x] Smart detection surfaces relevant decode actions at top of context menu
- [x] Empty selection returns zero code actions

## Code Quality Audit #1

**Status:** Partial (tarpaulin deferred — Linux-only, needs CI)

- [x] **A-010** — Code Quality Audit (clippy, duplication, rustdoc — all pass)
- [ ] Test coverage measurement with `cargo-tarpaulin` (deferred to CI setup)

**Phase 1 Exit Criteria:** All encoding/decoding actions work. Smart detection suggests relevant decode operations. Full unit test coverage. All CI checks pass.
