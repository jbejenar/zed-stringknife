# Phase 2 — Hashing, Cryptographic & Data Format Operations (Archived)

> Completed 2026-03-11. This is the archived record of Phase 2.
> Active roadmap: `roadmap/roadmap.md`

> **Goal:** Expand into hashing, JWT inspection, and data format conversions that developers reach for daily.

## EPIC-2.1: Hash Operations (One-Way)

**Status:** Done | **Dependencies:** EPIC-0.4

- [x] **T-200** — Implement `MD5 Hash` code action
- [x] **T-201** — Implement `SHA-1 Hash` code action
- [x] **T-202** — Implement `SHA-256 Hash` code action
- [x] **T-203** — Implement `SHA-512 Hash` code action
- [x] **T-204** — Implement `CRC32 Checksum` code action
- [x] **T-205** — Unit tests for all hash operations (27 tests)

### Verification

- [x] `cargo test -p transforms -- hash` passes all 27 tests
- [x] MD5/SHA outputs match NIST test vectors

## EPIC-2.2: JWT Operations (Read-Only Decode)

**Status:** Done | **Dependencies:** EPIC-0.4, EPIC-1.1

- [x] **T-210** — Implement `JWT Decode Header` code action
- [x] **T-211** — Implement `JWT Decode Payload` code action
- [x] **T-212** — Implement `JWT Decode (Full)` code action
- [x] **T-213** — Graceful handling of invalid JWT format
- [x] **T-214** — Unit tests with sample JWTs (20 tests)

### Verification

- [x] `cargo test -p transforms -- jwt` passes all 20 tests
- [x] Malformed JWT returns structured error

## EPIC-2.3: JSON Operations

**Status:** Done (T-224/T-225 deferred — need YAML parser dep)

- [x] **T-220** — Implement `JSON Pretty Print` code action
- [x] **T-221** — Implement `JSON Minify` code action
- [x] **T-222** — Implement `JSON Escape String` code action
- [x] **T-223** — Implement `JSON Unescape String` code action
- [x] **T-226** — Unit tests for JSON operations

## EPIC-2.4: XML/HTML Operations

**Status:** Done

- [x] **T-230** — Implement `XML Pretty Print` code action
- [x] **T-231** — Implement `XML Minify` code action
- [x] **T-232** — Unit tests for XML operations

## EPIC-2.5: TOML/CSV Utility Operations

**Status:** Done (T-240/T-241 deferred — need TOML parser dep)

- [x] **T-242** — Implement `CSV -> JSON Array` code action
- [x] **T-243** — Unit tests for format conversion operations

## Audits

- [x] **A-020** — Architecture Audit #1 (Done)
- [x] **A-021** — Security Audit #1 (Done — fuzz testing deferred, needs nightly)
- [x] **A-022** — Dependency Audit #2 (Done)

**Phase 2 Exit Criteria:** All hash, JWT, and format conversion operations functional. Error handling graceful. Architecture and security audits passed.
