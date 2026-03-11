# CLAUDE.md — Agent Entry Point

> Read this file first. It gives you the 30-second architecture summary,
> tells you where everything lives, and links to the vault for deeper context.

## Architecture (30 seconds)

**StringKnife** is a Zed editor extension that provides 50+ string/text
manipulation commands (encode, decode, hash, case-convert, extract, etc.).

```
Layer 1  WASM shim         zed-extension/src/lib.rs   (Zed API surface)
Layer 2  LSP router         stringknife-lsp/src/       (JSON-RPC dispatch)
Layer 3  Transform engine   stringknife-core/src/      (pure fn logic)
```

Arrows point downward only. Layer 3 has zero LSP dependencies.
Every transform is `fn(&str) -> Result<String, StringKnifeError>` — no I/O,
no side effects, no shared state.

## Session Protocol

1. **Start here:** Read `.vault/sessions/NEXT-SESSION.md` for current state
2. **Before changes:** Read `HINTS.md` for overrides and constraints
3. **Before arch changes:** Read `.vault/architecture/System Context.md`
4. **Before implementing:** Read `.vault/patterns/Adding a New Transform.md`
5. **After implementing:** Update `.vault/transforms/Transform Registry.md`
6. **End of session:** Create session note, update `NEXT-SESSION.md`, add row to Session Log

## File Map

| Path | What it is | When to read |
|------|-----------|-------------|
| `CLAUDE.md` | This file — architecture summary, protocol | Cold start |
| `HINTS.md` | Human overrides, suppressions, style rules | Before any changes |
| `.vault/sessions/NEXT-SESSION.md` | Current state + handoff | Every session start |
| `.vault/sessions/Session Log.md` | Chronological session history | Need context |
| `.vault/architecture/` | ADRs, system context | Before arch changes |
| `.vault/ari/ARI Dashboard.md` | ARI scores + trajectory | Before/after ARI work |
| `.vault/patterns/` | Codebase patterns, gotchas, dep budget | Before implementing |
| `.vault/transforms/Transform Registry.md` | All transforms with status | After implementing |
| `.vault/templates/` | Session + ARI checkpoint templates | Creating notes |
| `.claude/skills/vault-interaction/SKILL.md` | Vault interaction protocol | First time using vault |
| `roadmap/roadmap.md` | Living product roadmap with all tickets | Planning work |

## ARI Gate Thresholds

| Checkpoint | Phase | Composite Target | Gate |
|-----------|-------|-------------------|------|
| ARI-BASELINE | 0 | Establish baseline | N/A |
| ARI-0 | 0 exit | >= 7.0 | Phase gate |
| ARI-1 | 1 exit | >= 7.5 | Phase gate |
| ARI-2 | 2 exit | >= 8.0 | Phase gate |
| ARI-3 | 3 exit | >= 8.5 | Phase gate |
| ARI-4 | 4 exit | >= 9.0 | Release gate |

## Key Constraints (hard rules)

- [ ] No `unwrap()` in library code — use `Result<T, StringKnifeError>`
- [ ] No `unsafe` in `transforms/`
- [ ] No network calls, no file system access in transforms
- [ ] < 150 transitive crates at v1.0
- [ ] < 100ms for 100KB input; > 1MB returns `InputTooLarge`
- [ ] All PRs must pass CI (build, test, lint, deny, audit)
- [ ] `transforms/` has zero LSP dependencies
- [ ] Agents must update session state at end of every session (not optional)
