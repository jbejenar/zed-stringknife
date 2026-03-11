---
type: ari-pillar
pillar_number: 3
pillar_name: Type Safety
current_score: null
target_phase0: null
target_v1: null
weight: above-equal
tags: [ari-pillar]
---

# P3: Type Safety

## Definition

The type system encodes domain constraints. Invalid states are unrepresentable.
Errors are typed, not stringly-typed.

## Project Strategy

- `StringKnifeError` enum with structured variants
- All transforms return `Result<String, StringKnifeError>`
- No `unwrap()` in library code (`clippy::unwrap_used` denied)
- No `unsafe` in `transforms/`

## What Good Looks Like

- [ ] Zero `unwrap()` in non-test code
- [ ] Zero `unsafe` blocks in `transforms/`
- [ ] All error paths use typed variants
- [ ] `clippy::pedantic` warnings resolved
- [ ] No `as` casts without bounds checking

## Current Findings

(Baseline not yet established)
