# Phase 3 — Text Transformation & Case Conversion (Archived)

> Completed 2026-03-11. This is the archived record of Phase 3.
> Active roadmap: `roadmap/roadmap.md`

> **Goal:** The string manipulation operations developers use when refactoring — case conversions, whitespace operations, text analysis.

## EPIC-3.1: Case Conversions

**Status:** Done | **Dependencies:** EPIC-0.4

- [x] **T-300** — Implement `To UPPERCASE` code action
- [x] **T-301** — Implement `To lowercase` code action
- [x] **T-302** — Implement `To Title Case` code action
- [x] **T-303** — Implement `To Sentence Case` code action
- [x] **T-304** — Implement `To camelCase` code action
- [x] **T-305** — Implement `To PascalCase` code action
- [x] **T-306** — Implement `To snake_case` code action
- [x] **T-307** — Implement `To SCREAMING_SNAKE_CASE` code action
- [x] **T-308** — Implement `To kebab-case` code action
- [x] **T-309** — Implement `To dot.case` code action
- [x] **T-310** — Implement `To path/case` code action
- [x] **T-311** — Implement `To CONSTANT_CASE` code action
- [x] **T-312** — Implement `Toggle Case` code action
- [x] **T-313** — Unit tests for all case conversions (35 tests)

### Verification

- [x] `cargo test -p transforms -- case` passes all tests
- [x] Acronym handling and number boundaries correct

## EPIC-3.2: Whitespace & Line Operations

**Status:** Done | **Dependencies:** EPIC-0.4

- [x] **T-320** — Implement `Trim Whitespace` code action
- [x] **T-321** — Implement `Trim Leading Whitespace` code action
- [x] **T-322** — Implement `Trim Trailing Whitespace` code action
- [x] **T-323** — Implement `Collapse Whitespace` code action
- [x] **T-324** — Implement `Remove Blank Lines` code action
- [x] **T-325** — Implement `Remove Duplicate Lines` code action
- [x] **T-326** — Implement `Sort Lines (A->Z)` code action
- [x] **T-327** — Implement `Sort Lines (Z->A)` code action
- [x] **T-328** — Implement `Sort Lines (by length)` code action
- [x] **T-329** — Implement `Reverse Lines` code action
- [x] **T-330** — Implement `Shuffle Lines` code action
- [x] **T-331** — Implement `Number Lines` code action
- [x] **T-332** — Unit tests for whitespace and line operations (20 tests)

### Verification

- [x] `cargo test -p transforms -- whitespace` passes all tests

## EPIC-3.3: String Inspection (Non-Destructive)

**Status:** Done | **Dependencies:** EPIC-0.4, EPIC-1.6

- [x] **T-340** — Implement `Count Characters` code action
- [x] **T-341** — Implement `String Length (bytes)` code action
- [x] **T-342** — Implement `Detect Encoding` code action
- [x] **T-343** — Unit tests for inspection operations (8 tests)

### Verification

- [x] `cargo test -p transforms -- inspect` passes all tests

## EPIC-3.4: Escape/Unescape Operations

**Status:** Done | **Dependencies:** EPIC-0.4

- [x] **T-350** — Implement `Escape Backslashes` code action
- [x] **T-351** — Implement `Unescape Backslashes` code action
- [x] **T-352** — Implement `Escape Regex` code action
- [x] **T-353** — Implement `Escape SQL String` code action
- [x] **T-354** — Implement `Escape Shell String` code action
- [x] **T-355** — Implement `Escape CSV Field` code action
- [x] **T-356** — Unit tests for escape operations (17 tests)

### Verification

- [x] `cargo test -p transforms -- escape` passes all tests

## Audits

- [x] **A-030** — Code Quality Audit #2 (Done — estimated >90% coverage)

**Phase 3 Exit Criteria:** All case, whitespace, and escape operations functional. Inspection actions return info. Test coverage >= 70% on `transforms/`.
