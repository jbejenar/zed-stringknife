---
type: ari-pillar
pillar_number: 7
pillar_name: Dependency Hygiene
current_score: null
target_phase0: null
target_v1: null
weight: equal
tags: [ari-pillar]
---

# P7: Dependency Hygiene

## Definition

Dependencies are minimal, audited, and pinned. No unnecessary transitive deps.
License compliance enforced.

## Project Strategy

- Budget: < 150 transitive crates at v1.0
- `cargo-deny` for license + advisory checks
- `cargo-audit` for security vulnerabilities
- Pinned versions in `Cargo.toml` (no wildcards)
- New deps require justification in PR description

## What Good Looks Like

- [ ] `cargo deny check` passes with zero violations
- [ ] `cargo audit` reports zero known vulnerabilities
- [ ] Total transitive crates under budget
- [ ] No duplicate crate versions
- [ ] License allowlist: MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause, ISC, Zlib

## Current Findings

(Baseline not yet established)
