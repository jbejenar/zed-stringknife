---
type: pm-review
review: PMR-3
phase: 4
date: 2026-03-14
status: Complete
tags: [pm-review, scope-lock]
---

# PMR-3: Pre-Launch Review

**Date:** 2026-03-14
**Phase:** 4 exit gate
**Reviewer:** Product Owner (with agent facilitation)

## 1. Feature Inventory (v0.5.0)

**64 code actions** across 10 categories, backed by **67 public transform functions** in 15 modules.

| Category | Actions | Count | Status |
|----------|---------|-------|--------|
| Encoding | Base64, Base64URL, URL, URL (Component), HTML, Hex encode/decode, Unicode escape/unescape, Show Codepoints | 13 | Ship |
| Hashing | MD5, SHA-1, SHA-256, SHA-512, CRC32 | 5 | Ship |
| Case | UPPER, lower, Title, Sentence, camel, Pascal, snake, SCREAMING_SNAKE, kebab, dot, path, CONSTANT, Toggle | 13 | Ship |
| JSON | Pretty print, minify, escape, unescape | 4 | Ship |
| XML | Pretty print, minify | 2 | Ship |
| CSV | CSV to JSON array | 1 | Ship |
| Whitespace | Trim, trim leading/trailing, collapse, remove blanks, remove dupes, sort/reverse/shuffle/number lines | 10 | Ship |
| Escape | Backslash (escape/unescape), regex, SQL, shell, CSV | 6 | Ship |
| Inspect | Character count, byte length, encoding detection | 3 | Ship |
| Misc | Reverse string, JWT decode (header/payload/full) | 4 | Ship |

**Total: 64 code actions (after smart detection filtering).**

## 2. Kill List

**No features cut.** All 64 code actions are fully implemented, tested, and functional.

Deferred items (correctly NOT in build):
- JSON<->YAML (T-224/T-225) — needs YAML parser dependency decision
- TOML<->JSON (T-240/T-241) — needs TOML parser dependency decision

These remain in Phase 6 backlog and do not ship in v0.5.0.

## 3. Quality Metrics

| Metric | Value | Budget/Target |
|--------|-------|---------------|
| Tests | 371 (329 core + 14 no-panic + 28 LSP) | All passing |
| Transitive crates | 83 | < 150 |
| Clippy warnings | 0 | 0 |
| `unsafe` blocks | 0 | 0 |
| `unwrap()` in lib code | 0 | 0 |
| Performance | < 100ms for 100KB | < 100ms |
| Input size limit | 1MB | 1MB |
| Timeout | 5 seconds | 5 seconds |

## 4. README Assessment

**Status:** Functional but needs updates for store publication.

Actions taken in this review:
- Updated CHANGELOG with comprehensive Phase 1-4 history
- Bumped extension.toml version to 0.5.0
- Updated README status text (removed "coming soon")

Deferred to Phase 5 (EPIC-5.1):
- GIF/video demos (T-504)
- ARI badge (T-506)
- Contributing guidelines
- "Built with ariscan" section (T-507)

## 5. CHANGELOG Assessment

**Action:** Comprehensive CHANGELOG written for v0.5.0 covering all Phase 1-4 work.

## 6. Store Listing

**extension.toml** — validated:
- ID: `stringknife` (no "zed" prefix)
- Version: bumped to `0.5.0`
- License: MIT (file present)
- Repository: HTTPS URL
- 22 language bindings configured

## 7. Competitive Landscape

No dedicated string manipulation extensions found in the Zed extension store as of 2026-03-14. Zed's extension ecosystem is still young. StringKnife would be **first-mover** in this category.

## 8. Marketing Checklist

Recommended approach for launch:
- **Community forums:** Reddit r/zed, Zed Discord — highest ROI for niche dev tool
- **Organic discovery:** Zed extension store listing itself
- Blog post or social media optional, not required for initial traction

## 9. Scope Lock Decision

**v0.5.0 SCOPE LOCKED:**
- Ship all 64 code actions across 10 categories
- No features cut
- Version: 0.5.0
- Next: Phase 5 — Publication Preparation

## Next Review Trigger

PMR-4 (Post-Launch Retrospective) scheduled 2 weeks after Phase 5 store publication.
