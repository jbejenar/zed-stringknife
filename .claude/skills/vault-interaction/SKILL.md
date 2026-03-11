# Vault Interaction Skill

## Session Start Protocol

1. **Read** `.vault/sessions/NEXT-SESSION.md` — current state, last agent's work, what to do next
2. **Read** `HINTS.md` — overrides, suppressions, conventions
3. **Note** the `current_phase`, `current_ticket`, and `blocked_by` from frontmatter
4. **Read** any files listed in "Files to Read First" section

## Frontmatter as Structured Data

All vault notes use YAML frontmatter for metadata:

```yaml
---
type: session          # Note type: session, adr, pattern, ari-pillar, etc.
tags: [session]        # Obsidian tags for graph colouring
current_phase: 0       # Structured fields for querying
---
```

Frontmatter fields are the structured contract. Body text is for humans and agents.

## Wikilinks as Navigation

Use `[[Note Name]]` (Obsidian wikilinks) to cross-reference notes:

- `[[System Context]]` links to `.vault/architecture/System Context.md`
- `[[ARI Dashboard]]` links to `.vault/ari/ARI Dashboard.md`
- `[[Transform Registry]]` links to `.vault/transforms/Transform Registry.md`

## Session End Protocol

1. **Create session note** from `.vault/templates/Session Template.md`
   - Save as `.vault/sessions/Session-NNN.md`
   - Fill in all frontmatter fields and sections
2. **Update** `.vault/sessions/NEXT-SESSION.md`
   - Update `current_ticket` to next unchecked ticket
   - Update "What Last Agent Did" section
   - Update "What Next Agent Should Do" section
3. **Add row** to `.vault/sessions/Session Log.md`
4. **If transform implemented:** Update `.vault/transforms/Transform Registry.md`
5. **If gotcha found:** Add to `.vault/patterns/Gotchas.md`

## .vault/ File Structure

```
.vault/
├── .obsidian/
│   ├── app.json              # Source mode, frontmatter visible, line numbers
│   └── graph.json            # Colour groups by tag
├── architecture/
│   ├── System Context.md     # Component architecture
│   └── ADR-*.md              # Architecture Decision Records
├── ari/
│   ├── ARI Dashboard.md      # Composite scores and trajectory
│   └── P1..P8 *.md           # Per-pillar tracking
├── sessions/
│   ├── NEXT-SESSION.md       # Handoff state
│   ├── Session Log.md        # Chronological table
│   └── Session-NNN.md        # Individual session notes
├── patterns/
│   ├── Adding a New Transform.md
│   ├── Gotchas.md
│   └── Dependency Budget.md
├── transforms/
│   └── Transform Registry.md
├── pm-reviews/
│   └── PM Reviews.md
├── audits/
│   └── Audit Index.md
├── templates/
│   ├── Session Template.md
│   └── ARI Checkpoint Template.md
├── Home.md                   # Master index
└── README.md                 # How to use the vault
```
