---
type: ari-dashboard
tags: [ari-pillar]
last_updated: 2026-03-11
scan_tool: prontiq-ariscan v0.1.0
---

# ARI Dashboard

## Composite Score Trajectory

| Checkpoint | Phase | Target | Actual | Level | Gate Passed |
|-----------|-------|--------|--------|-------|-------------|
| ARI-BASELINE | 0 | — | 59 | L3 Capable | N/A |
| ARI-0 | 0 exit | >= 70 | — | — | — |
| ARI-1 | 1 exit | >= 75 | — | — | — |
| ARI-2 | 2 exit | >= 80 | — | — | — |
| ARI-3 | 3 exit | >= 85 | — | — | — |
| ARI-4 | 4 exit | >= 90 | — | — | — |

## Per-Pillar Scores (ariscan rubric v1)

| # | Pillar | Weight | Baseline | Current |
|---|--------|--------|----------|---------|
| P1 | Agent Context Quality | 15% | 97 | 97 |
| P2 | Feedback Loop Speed | 15% | 19 | 19 |
| P3 | Test Isolation | 18% | 65 | 65 |
| P4 | Dev Environment | 10% | 80 | 80 |
| P5 | Doc Machine-Readability | 10% | 25 | 25 |
| P6 | Build Determinism & Type Safety | 15% | 50 | 50 |
| P7 | Code Navigability | 12% | 70 | 70 |
| P8 | Security & Governance | 5% | 75 | 75 |

## Remediation Queue

| Priority | Finding | Pillar | Expected Impact |
|----------|---------|--------|----------------|
| High | ariscan expects package.json scripts (Node.js bias) — Makefile not detected | P2 | +15-20 if scanner fixed |
| Medium | No API spec / error taxonomy doc | P5 | +10-15 |
| Medium | No tsconfig.json (Rust, not TS) — false negative | P6 | +10-15 if scanner fixed |
| Low | Naming inconsistency (camelCase/kebab/snake mix) | P7 | +5 |

## Notes

- Scan tool: `prontiq-ariscan` v0.1.0 (https://github.com/jbejenar/prontiq-ariscan)
- Baseline recorded: 2026-03-11
- P2 and P6 scores are artificially low due to Node.js assumptions in scanner
- See [[ARI-BASELINE]] for full scan details and remediation history
- ARI Dashboard is **manually updated** by human review
- `ariscan` output is the source of truth
