---
type: ari-pillar
pillar_number: 6
pillar_name: Documentation Density
current_score: null
target_phase0: null
target_v1: null
weight: equal
tags: [ari-pillar]
---

# P6: Documentation Density

## Definition

Public API surface is documented with rustdoc. Non-obvious logic has inline
comments. Architecture is documented in the vault.

## Project Strategy

- Rustdoc on all public types and functions
- `CLAUDE.md` + `.vault/` for architectural documentation
- `HINTS.md` for conventions and overrides
- `CONTRIBUTING.md` for dev setup

## What Good Looks Like

- [ ] All `pub fn` have `///` doc comments
- [ ] All `pub struct`/`pub enum` have `///` doc comments
- [ ] `#![warn(missing_docs)]` on library crates
- [ ] Architecture decisions recorded as ADRs in vault

## Current Findings

(Baseline not yet established)
