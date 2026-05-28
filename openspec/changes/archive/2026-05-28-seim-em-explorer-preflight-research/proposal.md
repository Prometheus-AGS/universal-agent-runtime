## Why

The Entity Explorer (5-tab FAB panel + Chrome MV3 extension scaffold) requires UI code that
is gated by the `<!-- uiux-routing:start v1 -->` discipline wired into this repo's
`CLAUDE.md` / `AGENTS.md`. That discipline mandates seven steps be completed and distilled
**before** any production component code is written:

1. `/kbd-memory-recall` — populate `prior-context.md` from surreal-memory (auto-fired on `assess:before`).
2. **UI/UX Pro Max analysis** (`nextlevelbuilder/ui-ux-pro-max-skill`) — palette, font, spacing, a11y recommendations.
3. **Impeccable** (`pbakaus/impeccable`) — `/impeccable audit` + `/impeccable critique` + `/impeccable distill` (devtools panel ≈ complexity reduction).
4. **Anthropic skills** — `frontend-design` (intentional design) + `ux-designer` (UX-engineer review).
5. **Vercel skills** — React Best Practices + Composition Patterns for the panel component tree.
6. **Web search** — "runtime devtools page best practices 2026" AND "Chrome MV3 devtools panel patterns" / "react-devtools bridge architecture" (URLs + anchor keywords + fetch dates captured per cache discipline).
7. **Distillation** — one paragraph (3-5 sentences) synthesising all of the above for this specific surface.

Changes W5 (`seim-em-explorer-event-bus-registry`) and W6 (`seim-em-explorer-panel-components`) are
explicitly blocked on this change committing `docs/devtools-design-notes.md`. No production code lands
in this change — the deliverable is documentation only.

The UI/UX skill roster is cached at
`.kbd-orchestrator/references/uiux-skill-roster.md` (last fetched 2026-05-27) and already covers
Tier 1 (UI/UX Pro Max, Impeccable) and Tier 2 (Vercel, Anthropic) entries referenced above.

## What Changes

**Single new file** committed to `feat/seim-entity-management-impl` in the
`prometheus-entity-management` worktree (`~/.claude/worktrees/seim-entity-management`):

```
docs/devtools-design-notes.md
```

Sections in that file:

| § | Content |
|---|---------|
| 1 | **Memory recall digest** — prior UI/UX decisions from `prior-context.md` relevant to a devtools panel surface |
| 2 | **UI/UX Pro Max summary** — palette + font + spacing + a11y recommendations for a dark devtools panel |
| 3 | **Impeccable audit + critique findings** — audit result + critique result + distill recommendations |
| 4 | **Anthropic `frontend-design` + `ux-designer` notes** — intentional design + UX-engineer review |
| 5 | **Vercel React Best Practices + Composition Patterns summary** |
| 6 | **Web-search results: "runtime devtools page best practices 2026"** (URL + anchor keyword + fetch date per source) |
| 7 | **Web-search results: "Chrome MV3 devtools panel patterns"** (same citation format) |
| 8 | **Distilled one-paragraph summary** (3-5 sentences; the prompt context for W6 code) |
| 9 | **Implications for W5** — event-bus / multi-store registry design hints |
| 10 | **Implications for W6** — 5-tab panel component architecture hints |
| 11 | **Implications for W8** — Chrome extension scaffold (bonus) |

No `src/` files, no `package.json`, no test files are modified.

## Capabilities

### New Capabilities

- **`entity-explorer-preflight-research`**: A committed `docs/devtools-design-notes.md` capturing
  the UI/UX routing discipline's seven-step pre-work for the Entity Explorer surface, including
  memory recall, design-system analysis, Impeccable audit/critique/distill, Anthropic skill notes,
  Vercel composition patterns, web-search citations, and a one-paragraph distillation. This file
  gates downstream changes W5 (event bus) and W6 (panel components).

### Modified Capabilities

None.
