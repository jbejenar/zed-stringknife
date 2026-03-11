---
type: ari-dashboard
tags: [ari-pillar]
last_updated: 2026-03-11
---

# ARI Dashboard

## Composite Score Trajectory

| Checkpoint | Phase | Target | Actual | Gate Passed |
|-----------|-------|--------|--------|-------------|
| ARI-BASELINE | 0 | — | TBD | N/A |
| ARI-0 | 0 exit | >= 7.0 | — | — |
| ARI-1 | 1 exit | >= 7.5 | — | — |
| ARI-2 | 2 exit | >= 8.0 | — | — |
| ARI-3 | 3 exit | >= 8.5 | — | — |
| ARI-4 | 4 exit | >= 9.0 | — | — |

## Per-Pillar Scores

| # | Pillar | Weight | Baseline | Current | Phase 0 Target |
|---|--------|--------|----------|---------|----------------|
| 1 | [[P1 Test Isolation]] | above-equal | — | — | — |
| 2 | [[P2 Build Determinism]] | above-equal | — | — | — |
| 3 | [[P3 Type Safety]] | above-equal | — | — | — |
| 4 | [[P4 Error Explicitness]] | equal | — | — | — |
| 5 | [[P5 Modular Coherence]] | equal | — | — | — |
| 6 | [[P6 Documentation Density]] | equal | — | — | — |
| 7 | [[P7 Dependency Hygiene]] | equal | — | — | — |
| 8 | [[P8 Security Gate]] | gate | — | — | — |

## Remediation Queue

(No items — baseline not yet established)

## Notes

- ARI Dashboard is **manually updated** by human review
- `ariscan` output is the source of truth
- Do not modify scores without explicit human approval
