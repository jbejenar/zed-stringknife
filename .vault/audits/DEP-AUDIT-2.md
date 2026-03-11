---
type: audit
tags: [audit, dependency]
date: 2026-03-11
---

# Dependency Audit #2 (A-022)

## Phase 2 Dependency Delta

**New dependencies added in Phase 2: ZERO.**

All Phase 2 features (hash, JWT, JSON, XML, CSV) were implemented from scratch
with no external dependencies.

| Metric | Phase 1 | Phase 2 | Delta |
|--------|---------|---------|-------|
| stringknife-core deps | 0 | 0 | 0 |
| stringknife-lsp direct deps | 4 | 4 | 0 |
| Total transitive crates | ~79 | ~79 | 0 |

## Budget Status

- Budget: < 150 transitive crates at v1.0
- Current: ~79 transitive crates
- Headroom: ~71 crates remaining

## Deferred Features Needing Dependencies

| Feature | Ticket | Estimated New Deps |
|---------|--------|-------------------|
| TOML ↔ JSON | T-240, T-241 | toml (~5 crates) |
| YAML ↔ JSON | T-224, T-225 | serde_yaml (~5 crates) |

If both are added: ~89 crates, still well under budget.

## License Compatibility

No new dependencies, so no new license concerns. Existing deps are all
MIT/Apache-2.0 dual-licensed (verified in Phase 1 DEP-AUDIT-1).

## Supply Chain Risk

No new deps with low download counts. stringknife-core remains zero-dep,
eliminating supply chain risk for the core transform engine entirely.

## Findings

1. **Phase 2 added zero new dependencies** — best possible outcome.
2. **Budget is healthy** — 71 crates of headroom for Phase 3+.
3. **Deferred TOML/YAML** would add ~10 crates if implemented with serde ecosystem.
