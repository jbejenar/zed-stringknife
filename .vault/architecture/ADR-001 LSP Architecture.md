---
type: adr
status: accepted
tags: [adr, architecture]
date: 2026-03-11
---

# ADR-001: LSP-Based Architecture

## Status

Accepted

## Context

Zed's extension API (`zed_extension_api` v0.7.x) does not expose direct editor
text manipulation. There is no `editor.replaceSelection()`, no code action
registration, no context menu hook in the WASM API.

The only surface in Zed that provides right-click context menu integration with
text replacement is the Language Server Protocol via `textDocument/codeAction`.

## Decision

Ship as a Zed extension (Rust WASM) that bundles a custom Language Server binary.
The LSP registers against broad file types so code actions are available in any file.

## Alternatives Rejected

| Approach | Why Rejected |
|----------|-------------|
| Slash Commands | Only in Assistant panel. Cannot modify editor text. |
| MCP Server | For AI context injection, not text manipulation. |
| Tasks + CLI | No context menu. Poor discoverability. |
| Wait for Editor API | Indefinite timeline. |
| Fork Zed | Disproportionate effort. |

## Consequences

- Requires maintaining a native binary for each platform
- LSP startup adds small latency on first use
- Proven pattern: Zed already uses LSP for all language intelligence
- Architecture cleanly separates concerns into three layers

## Linked Notes

- [[System Context]]
