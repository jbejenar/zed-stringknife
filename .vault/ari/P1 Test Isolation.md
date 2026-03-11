---
type: ari-pillar
pillar_number: 1
pillar_name: Test Isolation
current_score: null
target_phase0: null
target_v1: null
weight: above-equal
tags: [ari-pillar]
---

# P1: Test Isolation

## Definition

Tests run in isolation with no shared state, no I/O, and no ordering dependencies.
Each test is a pure function that verifies a single transform behaviour.

## Project Strategy

- Every transform has a `#[cfg(test)]` module in its own file
- Tests are pure: no network, no filesystem, no shared state
- No test fixtures — inline expected values
- `cargo test` runs all tests in any order

## What Good Looks Like

- [ ] Every public transform has >= 3 test cases (happy, edge, error)
- [ ] Zero `#[ignore]` tests without justification
- [ ] No `static mut` or `lazy_static` in test modules
- [ ] Tests pass when run in parallel (`cargo test`)
- [ ] Tests pass when run individually

## Current Findings

(Baseline not yet established)
