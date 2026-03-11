# SKILL.md — Vault Operations

> This skill teaches agents how to navigate, search, query, create, and maintain the `.vault/` knowledge base. The vault is Obsidian-compatible structured markdown with YAML frontmatter and `[[wikilinks]]`.

---

## Vault Topology

The vault is a directed graph. Some notes are hubs (many inbound links), some are leaves (few or none). Knowing the shape saves traversal time.

### Hub Notes (start here)

| Note | Role | When to read |
|------|------|-------------|
| `Home.md` | Master index — links to every section | When you need orientation |
| `sessions/NEXT-SESSION.md` | Current state, blockers, handoff | Start of every session |
| `ari/ARI Dashboard.md` | All pillar scores, trajectory, remediation queue | Before/after ariscan runs |
| `transforms/Transform Registry.md` | Every transform: status, module, tests | Before/after implementing transforms |
| `audits/Audit Index.md` | All audit series with status | When working on audit tickets |
| `pm-reviews/PM Reviews.md` | All PMR gates with status | When approaching a phase gate |

### Navigation Shortcuts

| I need to know... | Start at | Then follow |
|-------------------|----------|-------------|
| What to work on next | `sessions/NEXT-SESSION.md` | Read "What Next Agent Should Do" |
| Current ARI scores | `ari/ARI Dashboard.md` | Per-Pillar Scores table |
| A specific pillar's strategy | `ari/ARI Dashboard.md` | Click the pillar wikilink (e.g. `[[P1 Test Isolation]]`) |
| How to add a transform | `patterns/Adding a New Transform.md` | Follow step-by-step, then update `[[Transform Registry]]` |
| What not to change | `patterns/Gotchas.md` | Read all sections — they're short |
| Dependency rules | `patterns/Dependency Budget.md` | Check the 5 hard rules |
| Why LSP? | `architecture/ADR-001 LSP Architecture Bet.md` | Read Alternatives Rejected table |
| Layer boundaries | `architecture/ADR-002 Three-Layer Separation.md` | Crate graph diagram |
| What happened last session | `sessions/NEXT-SESSION.md` | "What Last Agent Did" section |
| Full session history | `sessions/Session Log.md` | Table, then follow `[[Session-NNN]]` links |
| What ariscan found | `ari/ARI-BASELINE.md` (or latest checkpoint) | Remediation Applied + Remaining Findings |

### Manual Traversal

When reading a note, follow wikilinks by opening the linked file:

```
Reading: .vault/ari/ARI Dashboard.md
See:     [[P2 Build Determinism]]
Resolve: find .vault -name 'P2 Build Determinism.md'
Open:    .vault/ari/P2 Build Determinism.md
```

**The two-hop rule:** From `Home.md`, every note in the vault should be reachable within 2 wikilink hops. If you create a note that isn't linked from any hub, it's effectively invisible. Always add at least one inbound wikilink from a hub note.

**Reading order for full context** (when you need deep understanding, not speed):
1. `Home.md` → get the lay of the land
2. `architecture/System Context.md` → understand the components
3. `architecture/ADR-001` → `ADR-002` → `ADR-003` → understand why
4. `ari/ARI Dashboard.md` → understand current health
5. `patterns/Gotchas.md` → understand constraints
6. `sessions/NEXT-SESSION.md` → understand what to do

**Reading order for speed** (when you just need to start working):
1. `sessions/NEXT-SESSION.md` → what to do
2. `patterns/Gotchas.md` → what not to do
3. Go.

---

## Frontmatter Contract

Frontmatter is YAML between `---` markers at the top of every note. It is the structured API — agents query it programmatically, Obsidian renders it as properties.

### Required fields (every note)
```yaml
---
type: session|adr|pattern|ari-pillar|ari-checkpoint|audit|registry|index|session-handoff
tags: [at-least-one-tag]
---
```

### Common fields by type

**Session notes:**
```yaml
type: session
session_number: 5
agent: Claude Opus 4.6
phase: 4
tickets_attempted: [T-400, T-401]
tickets_completed: [T-400]
tickets_blocked: [T-401]
tags: [session]
```

**ARI pillar notes:**
```yaml
type: ari-pillar
pillar_id: P1
current_score: 65
weight: 18%
tags: [ari-pillar]
```

**ARI checkpoint notes:**
```yaml
type: ari-checkpoint
checkpoint: ARI-BASELINE
composite_score: 59
gate_threshold: null
gate_passed: true
scan_tool: prontiq-ariscan v0.1.0
scan_date: 2026-03-11
tags: [ari-pillar]
```

**Audit notes:**
```yaml
type: audit
audit_id: A-030
tags: [audit, quality]
date: 2026-03-11
```

**ADR notes:**
```yaml
type: adr
status: accepted|proposed|deprecated|superseded
tags: [adr, architecture]
```

### Query frontmatter from the command line

```bash
# Extract a single field
sed -n '/^---$/,/^---$/p' '.vault/ari/ARI Dashboard.md' | grep '^composite_score:'

# Get current phase from NEXT-SESSION
sed -n '/^---$/,/^---$/p' .vault/sessions/NEXT-SESSION.md | grep -E '^(current_phase|current_ticket|blocked_by):'

# Find all pillar scores at once
for f in .vault/ari/P*.md; do
  name=$(basename "$f" .md)
  score=$(sed -n '/^---$/,/^---$/p' "$f" | grep '^current_score:' | awk '{print $2}')
  echo "$name: $score"
done

# Find notes by frontmatter type
grep -rl '^type: audit' .vault/

# Find notes with a specific tag
grep -rl 'tags:.*session' .vault/
```

---

## Search

### By content
```bash
# Full-text search across the vault
grep -ri 'search term' .vault/ --include='*.md'

# Search only note bodies (skip frontmatter)
grep -ri 'search term' .vault/ --include='*.md' | grep -v '^---$'
```

### By link (who links to this note?)
```bash
grep -rl '\[\[ARI Dashboard\]\]' .vault/
grep -rl '\[\[Transform Registry\]\]' .vault/
```

### By file type / directory
```bash
# All session notes
ls .vault/sessions/Session-*.md

# All audit reports
ls .vault/audits/*.md

# All architecture decisions
ls .vault/architecture/ADR-*.md

# All ARI pillar notes
ls .vault/ari/P*.md
```

### Find orphan notes (not linked from anywhere)
```bash
for f in $(find .vault -name '*.md' -not -path '*/.obsidian/*' -not -name 'README.md'); do
  name=$(basename "$f" .md)
  if ! grep -rl "\[\[$name\]\]" .vault/ --include='*.md' -q 2>/dev/null; then
    echo "ORPHAN: $f"
  fi
done
```

### Find broken wikilinks
```bash
grep -roh '\[\[[^]]*\]\]' .vault/ --include='*.md' | sort -u | while read link; do
  name=$(echo "$link" | sed 's/\[\[//;s/\]\]//')
  case "$name" in *"{"*) continue ;; esac  # skip template placeholders
  if ! find .vault -name "$name.md" -print -quit 2>/dev/null | grep -q .; then
    echo "BROKEN: $link"
  fi
done
```

---

## Creating Notes

### From a template
```bash
cp '.vault/templates/Session Template.md' '.vault/sessions/Session-005.md'
# Then edit frontmatter and body
```

### From scratch
Always include frontmatter. Always add at least one inbound wikilink from a hub note.

| Type | Directory | Template exists? | Hub to link from |
|------|-----------|-----------------|------------------|
| Session | `.vault/sessions/` | Yes | `Session Log.md` (add table row) |
| ARI checkpoint | `.vault/ari/` | Yes | `ARI Dashboard.md` (update trajectory table) |
| Audit | `.vault/audits/` | No | `Audit Index.md` (update status + add link) |
| ADR | `.vault/architecture/` | No | `Home.md` (add to Architecture section) |
| Pattern | `.vault/patterns/` | No | `Home.md` (add to Patterns section) |
| Gotcha | Don't create a new note | — | Append to `patterns/Gotchas.md` |

### Wikilink conventions
- Use exact filename without extension: `[[P1 Test Isolation]]`
- Obsidian resolves by filename, not path — no directory prefix needed
- Spaces in filenames are literal: the file `P1 Test Isolation.md` is linked as `[[P1 Test Isolation]]`
- After creating a note, always verify at least one other note links to it

---

## Updating Notes

### Update frontmatter field in place
```bash
sed -i 's/^current_score:.*/current_score: 85/' '.vault/ari/P1 Test Isolation.md'
sed -i 's/^current_phase:.*/current_phase: 4/' .vault/sessions/NEXT-SESSION.md
```

### Append a row to a table
```bash
echo "| 5 | $(date +%Y-%m-%d) | Claude Opus 4.6 | Phase 4 | Description | [[Session-005]] |" >> '.vault/sessions/Session Log.md'
```

### Update ARI Dashboard after a scan
1. Update the trajectory table (add row or fill in blanks)
2. Update per-pillar scores table
3. Update remediation queue (add new findings, remove resolved ones)
4. Update `last_updated` in frontmatter
5. Update each `P*.md` note's `current_score` frontmatter field

### When vault and code disagree
Code is the source of truth. If the Transform Registry says a function exists but it doesn't, update the registry. If a Gotcha references a pattern that's been refactored away, update the Gotcha. The vault reflects reality — it doesn't define it.

---

## Archival & Lifecycle

### Notes that are never archived
- Session notes (they're the audit trail)
- ARI checkpoints (the trajectory is the showcase story)
- Session Log (grows indefinitely — it's a flat index, this is fine)

### Deprecating a pattern
If a pattern in `patterns/` becomes obsolete:
1. Add `status: deprecated` to its frontmatter
2. Add a note at the top: `> ⚠️ Deprecated: {reason}. Superseded by [[New Note]].`
3. Do not delete — other notes may still link to it

### Superseding an ADR
1. Change `status: accepted` to `status: superseded`
2. Add `superseded_by: ADR-XXX` to frontmatter
3. Add a note at the top linking to the replacement
4. Do not delete

### General rule
Never delete vault notes. Deprecate, supersede, or annotate. The graph's history is part of the value.

---

## Cross-References Outside the Vault

| Vault references... | Link format | Example |
|---------------------|-------------|---------|
| Another vault note | `[[Note Name]]` wikilink | `[[P1 Test Isolation]]` |
| Roadmap | Plain path in prose | `roadmap/roadmap.md` |
| Source files | Plain path in prose | `stringknife-core/src/transforms/base64.rs` |
| Repo-root docs | Plain path in prose | `HINTS.md`, `CLAUDE.md` |

Wikilinks are vault-internal only. Never use wikilinks for files outside `.vault/`.

---

## Integrity Check (run at session end)

```bash
echo "=== Vault Stats ==="
echo "Notes: $(find .vault -name '*.md' -not -path '*/.obsidian/*' | wc -l)"
echo "Wikilinks: $(grep -roh '\[\[[^]]*\]\]' .vault/ --include='*.md' | wc -l)"
echo "Unique targets: $(grep -roh '\[\[[^]]*\]\]' .vault/ --include='*.md' | sort -u | wc -l)"

echo ""
echo "=== Broken Links ==="
grep -roh '\[\[[^]]*\]\]' .vault/ --include='*.md' | sort -u | while read link; do
  name=$(echo "$link" | sed 's/\[\[//;s/\]\]//')
  case "$name" in *"{"*) continue ;; esac
  if ! find .vault -name "$name.md" -print -quit 2>/dev/null | grep -q .; then
    echo "  $link"
  fi
done

echo ""
echo "=== Orphan Notes ==="
for f in $(find .vault -name '*.md' -not -path '*/.obsidian/*' -not -name 'README.md'); do
  name=$(basename "$f" .md)
  if ! grep -rl "\[\[$name\]\]" .vault/ --include='*.md' -q 2>/dev/null; then
    echo "  $f"
  fi
done

echo ""
echo "=== Missing Frontmatter ==="
for f in $(find .vault -name '*.md' -not -path '*/.obsidian/*' -not -name 'README.md'); do
  if ! head -1 "$f" | grep -q '^---$'; then
    echo "  $f"
  fi
done

echo ""
echo "=== Session Continuity ==="
for f in .vault/sessions/Session-*.md; do
  num=$(basename "$f" .md | sed 's/Session-//')
  if ! grep -q "Session-$num" '.vault/sessions/Session Log.md'; then
    echo "  MISSING from Session Log: $f"
  fi
done

echo ""
echo "=== Transform Registry Sync ==="
grep -rh 'pub fn ' stringknife-core/src/transforms/*.rs 2>/dev/null | sed 's/pub fn //;s/(.*//' | sort > /tmp/code_fns.txt
grep -oE '[a-z_]+' '.vault/transforms/Transform Registry.md' | sort -u > /tmp/vault_fns.txt
MISSING=$(comm -23 /tmp/code_fns.txt /tmp/vault_fns.txt | grep -v '^$')
if [ -n "$MISSING" ]; then
  echo "  In code but not in registry:"
  echo "$MISSING" | sed 's/^/    /'
fi
```

---

## Session Protocol

### On Start
1. Read `sessions/NEXT-SESSION.md` — get current state, blockers, next ticket
2. Read `patterns/Gotchas.md` — refresh constraints
3. If the task involves a specific area, follow the Navigation Shortcuts table to the relevant notes
4. Check `HINTS.md` in repo root for suppressions relevant to your ticket

### On End
1. Verify build is green (`make test && make lint`)
2. Mark completed tickets in `roadmap/roadmap.md`
3. Create session note from template in `.vault/sessions/`
4. Append row to `.vault/sessions/Session Log.md`
5. Rewrite `.vault/sessions/NEXT-SESSION.md` with fresh handoff
6. Update `.vault/transforms/Transform Registry.md` if transforms changed
7. Run the integrity check script
8. Commit

---

## Rules

1. **Every note gets frontmatter.** No exceptions. Type and tags at minimum.
2. **Wikilinks for vault-internal, paths for external.** Never mix them.
3. **Don't duplicate.** Link to the roadmap, link to the code. The vault holds context and decisions, not copies.
4. **Templates exist — use them.** Don't freestyle session notes or ARI checkpoints.
5. **Two-hop rule.** Every note reachable from `Home.md` in two clicks. If you create a note, link it from a hub.
6. **Run the integrity check.** Broken links and orphans compound. Catch them per session.
7. **Vault reflects reality.** When vault and code disagree, update the vault.
8. **Never delete, only deprecate.** The graph's history is part of the value.
