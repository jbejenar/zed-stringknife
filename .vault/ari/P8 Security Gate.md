---
type: ari-pillar
pillar_number: 8
pillar_name: Security Gate
current_score: null
target_phase0: null
target_v1: null
weight: gate
tags: [ari-pillar]
---

# P8: Security Gate

## Definition

Binary pass/fail. No known vulnerabilities in dependencies. No `unsafe` in
transform code. No network or filesystem access in transforms.

## Project Strategy

- `cargo-audit` in CI (fail on known advisories)
- `cargo-deny` advisories check
- `#![deny(unsafe_code)]` in transforms crate
- No network calls anywhere in codebase
- No filesystem access in transforms
- WASM sandbox for extension code

## What Good Looks Like

- [ ] `cargo audit` clean
- [ ] `cargo deny check advisories` clean
- [ ] Zero `unsafe` in `transforms/`
- [ ] No network or FS calls in transforms
- [ ] WASM extension runs sandboxed

## Current Findings

(Baseline not yet established)
