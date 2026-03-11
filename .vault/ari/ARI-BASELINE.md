---
type: ari-checkpoint
checkpoint: ARI-BASELINE
composite_score: 59
gate_threshold: null
gate_passed: true
scan_tool: prontiq-ariscan v0.1.0
scan_date: 2026-03-11
tags: [ari-pillar]
---

# ARI Checkpoint: ARI-BASELINE

## Initial Scan (pre-remediation)

| # | Pillar | Score |
|---|--------|-------|
| P1 | Agent Context Quality | 40 |
| P2 | Feedback Loop Speed | 8 |
| P3 | Test Isolation | 65 |
| P4 | Dev Environment | 27 |
| P5 | Doc Machine-Readability | 25 |
| P6 | Build Determinism & Type Safety | 50 |
| P7 | Code Navigability | 70 |
| P8 | Security & Governance | 30 |
| **Composite** | | **42** |

Level: L2 — Fragile

## Post-Remediation Scan

| # | Pillar | Previous | Current | Delta |
|---|--------|----------|---------|-------|
| P1 | Agent Context Quality | 40 | 97 | +57 |
| P2 | Feedback Loop Speed | 8 | 19 | +11 |
| P3 | Test Isolation | 65 | 65 | 0 |
| P4 | Dev Environment | 27 | 80 | +53 |
| P5 | Doc Machine-Readability | 25 | 25 | 0 |
| P6 | Build Determinism & Type Safety | 50 | 50 | 0 |
| P7 | Code Navigability | 70 | 70 | 0 |
| P8 | Security & Governance | 30 | 75 | +45 |
| **Composite** | | **42** | **59** | **+17** |

Level: L3 — Capable

## Remediation Applied

- Created `.agentignore` (P1)
- Created `AGENTS.md` for vendor-neutral agent context (P1)
- Created `Makefile` with test/lint/fmt/watch/setup/doctor targets (P2, P4)
- Created `.devcontainer/devcontainer.json` (P4)
- Created `SECURITY.md` (P8)
- Created `.github/CODEOWNERS` (P8)
- Created `.gitleaks.toml` + `.github/workflows/gitleaks.yml` (P8)
- Added AI-specific review checklist to PR template (P8)

## Remaining Findings

### P2 Feedback Loop Speed (19/100)
- ariscan looks for `package.json` scripts — this is a Rust project using Makefile + cargo
- The scanner has a Node.js bias; Rust's `cargo test`/`cargo clippy` are not detected as feedback loop commands
- Potential improvement: add a thin `package.json` with scripts that proxy to Makefile targets

### P5 Doc Machine-Readability (25/100)
- No API specs (OpenAPI/GraphQL) — LSP protocol is the "API" here
- No formal error taxonomy doc — `StringKnifeError` enum serves this role in code
- No runbooks — not applicable yet (no deployment/ops)

### P6 Build Determinism & Type Safety (50/100)
- Scanner looks for tsconfig.json strict mode — not applicable to Rust
- Rust's type system + Clippy lints provide equivalent safety
- Cargo.lock present (detected as "lockfile: true")
