# .vault/ — Codebase Intelligence Vault

This directory is an [Obsidian](https://obsidian.md)-compatible knowledge vault
that serves as the persistent memory layer for AI-agent sessions.

## Opening as an Obsidian Vault

1. Open Obsidian
2. "Open folder as vault" → select this `.vault/` directory
3. Graph view should render with colour-coded nodes by tag

## Agent Protocol

### Session Start
1. Read `.vault/sessions/NEXT-SESSION.md` for current state
2. Read `HINTS.md` (repo root) for overrides and constraints
3. Note current phase, active ticket, and blockers

### Session End
1. Create a session note from `.vault/templates/Session Template.md`
2. Update `.vault/sessions/NEXT-SESSION.md` with handoff
3. Add row to `.vault/sessions/Session Log.md`
4. If transform implemented: update `.vault/transforms/Transform Registry.md`
5. If gotcha found: add to `.vault/patterns/Gotchas.md`

## Structure

```
.vault/
├── .obsidian/          # Obsidian config (app.json, graph.json tracked)
├── architecture/       # ADRs and system context
├── ari/                # ARI pillar tracking and dashboard
├── sessions/           # Session notes and handoff state
├── patterns/           # Codebase patterns and agent guides
├── transforms/         # Transform registry
├── pm-reviews/         # PM review index
├── audits/             # Audit index
├── templates/          # Note templates
├── Home.md             # Master index
└── README.md           # This file
```

## Conventions

- All notes use YAML frontmatter for structured metadata
- Wikilinks (`[[Note Name]]`) for cross-references
- Tags: `#ari-pillar`, `#session`, `#pattern`, `#adr`, `#audit`, `#transform`
- Obsidian graph view colour-codes nodes by tag
