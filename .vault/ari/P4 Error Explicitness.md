---
type: ari-pillar
pillar_number: 4
pillar_name: Error Explicitness
current_score: null
target_phase0: null
target_v1: null
weight: equal
tags: [ari-pillar]
---

# P4: Error Explicitness

## Definition

Every error path is explicit, structured, and actionable. Errors carry enough
context for agents and humans to diagnose without guessing.

## Project Strategy

- `StringKnifeError` with `InvalidInput`, `UnsupportedEncoding`, `InputTooLarge`
- `Display` impl provides human-readable messages
- LSP maps errors to `window/showMessage` notifications
- No silent failures — every error is surfaced

## What Good Looks Like

- [ ] All error variants have descriptive `Display` output
- [ ] Error context includes operation name and reason
- [ ] No `unwrap()` or `expect()` in library code
- [ ] Error types implement `std::error::Error`

## Current Findings

(Baseline not yet established)
