---
type: ari-pillar
pillar_number: 2
pillar_name: Build Determinism
current_score: null
target_phase0: null
target_v1: null
weight: above-equal
tags: [ari-pillar]
---

# P2: Build Determinism

## Definition

Identical source produces identical artifacts on any machine, any CI runner,
any time. No ambient state leaks into builds.

## Project Strategy

- `rust-toolchain.toml` pins stable channel
- `Cargo.lock` committed to version control
- Pinned dependency versions in `Cargo.toml` (no wildcards)
- Cross-compile targets defined in CI workflow

## What Good Looks Like

- [ ] `rust-toolchain.toml` present and pinning stable
- [ ] `Cargo.lock` committed
- [ ] All deps use exact versions (no `*`, `>=`, or `~`)
- [ ] CI builds are reproducible across runners
- [ ] No build scripts that depend on system state

## Current Findings

(Baseline not yet established)
