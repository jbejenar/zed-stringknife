---
type: ari-pillar
pillar_number: 5
pillar_name: Modular Coherence
current_score: null
target_phase0: null
target_v1: null
weight: equal
tags: [ari-pillar]
---

# P5: Modular Coherence

## Definition

Each module has a single responsibility, clear boundaries, and minimal coupling.
Dependencies flow in one direction.

## Project Strategy

- Three-layer architecture with downward-only dependencies
- One transform per file in `transforms/`
- `transforms/` crate has zero LSP dependencies
- Every transform is `fn(&str) -> Result<String, StringKnifeError>`

## What Good Looks Like

- [ ] No circular dependencies between crates
- [ ] `transforms/` compiles without `tower-lsp` or `tokio`
- [ ] Each transform file exports a focused set of related functions
- [ ] Clear module boundaries between encode/decode/hash/case/etc.

## Current Findings

(Baseline not yet established)
