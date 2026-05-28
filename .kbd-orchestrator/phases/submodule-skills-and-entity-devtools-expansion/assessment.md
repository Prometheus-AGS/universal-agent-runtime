# Phase Assessment — submodule-skills-and-entity-devtools-expansion

- Generated: 2026-05-27
- Author: claude-code (kbd-assess)
- Active KBD waypoint at time of run: `runtime-provider-protocol-hardening`
- Recommended waypoint shift: **start new phase** at the same level —
  `submodule-skills-and-entity-devtools-expansion` — and demote the
  previous in-flight phase to a sibling (or place this as a parallel
  track) once `/kbd-new-phase` is run.

## 1. Scope summary

The argument supplied to `/kbd-assess` defines a **brand new omnibus phase**
that spans three external repos plus this one. Work touches:

| Concern | Target repo | Type of change |
|---|---|---|
| Persisted worktree convention | `~/.claude/worktrees/` (filesystem) | New convention + tooling |
| `/kbd-new-phase`, `/kbd-next-phase`, `/kbd-new-child`, `/kbd-next-child` | `prometheus-skill-system` (kbd-process-orchestrator) | New skills + workflow surface |
| Memory-driven phase execution (surreal-memory + pk-recall) | `prometheus-skill-system` | New cross-cutting skill |
| Karpathy + Claude-Code-author rule injector | `prometheus-skill-system` (new skill) | New skill |
| SurrealDB live-query variations | `prometheus-skill-system` (prometheus-entity-skills) | Updates to entity-* skills |
| Entity Explorer (in-app FAB + browser extension) | `prometheus-entity-management` repo | Net-new feature + extension |
| UI/UX skill discipline | this repo's `CLAUDE.md` / `AGENTS.md` | Doc updates |

## 2. Discovery — what's already in place

### KBD orchestrator (`~/.claude/skills/kbd-process-orchestrator/`)
- `skills/` already contains: `kbd-assess`, `kbd-execute`, `kbd-init`,
  `kbd-plan`, `kbd-reflect`, `kbd-status`, `kbd-next-phase`.
- `kbd-next-phase` already implements seeding from
  `reflection.md → "Recommended Next Phase"` (matches goal #2 — partially DONE).
- `SKILL.md` references `kbd-new-phase` and `kbd-full-phase`, but **no
  `kbd-new-phase/` directory exists** under `skills/` — it's mentioned but
  not implemented (gap).
- `kbd-new-child` and `kbd-next-child` are **entirely absent**.
- No nested-phase data model exists in `current-waypoint.json` (today the
  schema is flat: `phase`, `previousPhase` — no `parentPhase` /
  `childPhases[]` / `childPointer`).

### Memory / pk integration
- `kbd-process-orchestrator/SKILL.md` already documents an optional
  `surreal-memory` integration block ("Detection: check if `create_entity`
  tool is available") and references
  `shared/references/surreal-memory-integration.md`.
- `~/.claude/skills/karpathy-tokenizer/` exists — but there is **no
  `pk-recall`, `pk-librarian`, or "memory-first phase execution" skill**.
- `surreal-memory-server` repo exists at
  `/Users/gqadonis/Projects/prometheus/surreal-memory-server` with
  `TASKSTREAMS-API.md` and MCP wiring (`mcp.json`) — substrate is ready,
  the *discipline* of logging every phase event isn't enforced anywhere.

### Rule-injector skill (karpathy + claude-code author rules)
- **Does not exist.** No skill in `~/.claude/skills/` has "rules", "agent
  rules", or "agent-instructions" in its name.
- The two source rule sets need a web-search pass during *Analyze*:
  - Karpathy's set (commonly framed as "think before you code / start
    small / iterate / read the code").
  - The Claude-Code-author's 4-rule set (must be looked up — not yet
    cached anywhere in this workspace).

### prometheus-entity-skills (SurrealDB live-query gap)
- The realtime skill family is built around Electric + PGlite:
  `entity-realtime-channel`, `entity-realtime-local-first`,
  `entity-realtime-setup`, `entity-graph-realtime`.
- A grep over realtime skills' `SKILL.md` files finds **no mention of
  SurrealDB `LIVE SELECT` / live queries**.
- Substrate in `prometheus-entity-management`:
  - `src/realtime-manager.ts` exists with `ChannelConfig` and pluggable
    adapters — the *integration point* for a `SurrealLiveAdapter` is
    already there.
  - `src/adapters/` is the natural home for a `surreal-live` adapter.
- Gap: no skill teaches the agent how to wire `LIVE SELECT`, diff
  payloads (`CREATE | UPDATE | DELETE`), reconnect/replay semantics, or
  schema-version coupling.

### Entity Explorer (devtools + browser extension)
- **Foundation already present**: `prometheus-entity-management/src/devtools.ts`
  (95 lines) exposes `collectGraphDevStats` returning entity counts,
  patches, stale/fetching sets, list keys, and subscriber stats via
  `getActiveSubscriberCount` / `subscribeSubscriberStats`.
- **Missing layers**:
  1. UI surface: no floating-action-button component, no panel, no
     tree-visualization, no "which Zustand store contains this entity"
     graph.
  2. Multi-store awareness: today `useGraphStore` is the single store;
     the user's intent ("single representation of an entity across
     stores") implies multi-store coordination + duplicate detection
     across stores — not yet modelled.
  3. Event log: no append-only event stream consumable by a panel.
  4. Browser extension: zero scaffolding — no `manifest.json`, no
     `chrome-extension/` package, no devtools panel page.
  5. Connection from page → extension: needs a `window.postMessage`
     bridge or a `__PROMETHEUS_DEVTOOLS_HOOK__` global (mirroring
     react-devtools pattern). None exists.

### Worktree persistence
- Current worktree: `.claude/worktrees/adoring-booth-312094` — **inside
  the repo tree**, conflicts with the checked-in `.claude/` config dir
  (this is the user's explicit complaint).
- `~/.claude/worktrees/` already contains two siblings:
  `confident-wilbur-c27abe`, `musing-sinoussi-09cea6` — proving the
  user's preferred path is already partly in use.
- No project-level convention, hook, or doc enforces "all UAR
  worktrees must be created under `~/.claude/worktrees/`".

### UI/UX skill discipline in CLAUDE.md / AGENTS.md
- This repo's `CLAUDE.md` has zero references to "ui ux pro max",
  "impeccable", "anthropic web design", "vercel react", or skill-routing
  for UI work.
- The impeccable-skills set name needs web-search confirmation during
  Analyze (member skill names not cached locally).

## 3. Gap matrix (against the 6 numbered goals)

| # | Goal | Status | Concrete gap |
|---|---|---|---|
| 1 | `/kbd-new-phase`, `/kbd-next-phase`, `/kbd-new-child`, `/kbd-next-child` | Partial | `kbd-next-phase` exists; `kbd-new-phase` referenced but not implemented; both child variants missing; waypoint schema has no nesting model |
| 2 | `kbd-next-phase` auto-seeds from prior reflection | **DONE** | Already implemented — verify and document |
| 3 | Memory-first phase execution (surreal-memory + pk-recall) | Substrate only | No enforcement skill; no logging hook; no pk-recall skill; surreal-memory integration is documented as optional, not mandatory |
| 4 | Auto-inject Karpathy + Claude-Code-author rules into `CLAUDE.md` / `AGENTS.md` | Missing | No skill exists; rule contents not yet sourced (needs web search) |
| 5 | SurrealDB live-query variations in `prometheus-entity-skills` | Missing | All realtime skills target Electric/PGlite; no `entity-realtime-surreal-live` skill; no `SurrealLiveAdapter` in entity-management |
| 6 | Entity Explorer + browser extension | Skeleton only | `devtools.ts` collects stats; no FAB UI, no panel, no tree viz, no multi-store / duplicate detection, no extension scaffolding, no page↔extension bridge |
| 6b | UI/UX skill enforcement in CLAUDE.md / AGENTS.md | Missing | No routing rules; skill set names need web search; "best-practice analysis before writing UI code" not encoded as a guard |
| — | Persisted worktree convention `~/.claude/worktrees/` | Missing | Current worktree is *inside* the repo (the exact case the user wants to avoid) |

## 4. Risks & dependencies

1. **Cross-repo blast radius.** Work spans
   `prometheus-skill-system`, `prometheus-entity-management`, and this
   `universal-agent-runtime` worktree. Each change must land as
   independently reviewable PRs in its respective repo. The skill repos
   live at `/Users/gqadonis/Projects/prometheus/prometheus-skill-system`
   and are surfaced through `~/.claude/skills/` symlinks (or copies) —
   confirm linkage during Plan so commits reach the right origin.
2. **Nested-phase schema migration.** Adding `parentPhase`,
   `childPhases[]`, `childPointer` to `current-waypoint.json` is a
   breaking change to any tool that reads the file (Roo, Cursor, Codex,
   OpenCode all sit here). Backward-compatible defaults are required.
3. **Web-search dependencies.** Three pieces of content are not yet
   sourced and must come from the open web during Analyze:
   - Karpathy's rule set (verbatim).
   - The Claude-Code-author's 4-rule set.
   - Full enumeration of "impeccable" skill family member names.
4. **Browser extension surface.** Chrome MV3 + a React Devtools-style
   bridge introduces a long-running maintenance surface (manifest, host
   permissions, devtools_page, content script, panel page). Decide in
   Plan whether v1 ships *in-app only* (FAB + panel) and the extension
   is a v2 stretch.
5. **Surreal `LIVE SELECT` semantics.** The adapter must handle
   reconnects, missed events, and schema drift — needs a documented
   replay strategy parallel to ElectricSQL's shape-stream behavior.
6. **Memory hygiene.** "Constant logging to surreal-memory" can balloon
   storage and noise; need a retention/relevance policy before turning
   it on as a default.
7. **Worktree relocation.** Migrating off the in-repo worktree mid-phase
   risks losing uncommitted state. The convention change should land
   *and* the next worktree should be the first one created under the
   new path — don't try to relocate this one.

## 5. Recommended next step

Run `/kbd-new-phase submodule-skills-and-entity-devtools-expansion` to
register this phase formally, then `/kbd-plan` to convert the gap matrix
in §3 into an ordered change list. Suggested ordering for the planner:

1. Worktree convention + doc update (lowest risk, unblocks the rest).
2. KBD orchestrator schema extension for nested phases (foundation for
   `/kbd-new-child` & `/kbd-next-child`).
3. `/kbd-new-phase` implementation (closes the
   referenced-but-missing gap), then `/kbd-new-child`, then
   `/kbd-next-child`.
4. Memory-first execution skill + pk-recall skill (raises every
   subsequent phase's quality, including the rest of *this* phase).
5. Karpathy + Claude-Code rule-injector skill (cheap; depends on web
   search results).
6. UI/UX routing rules added to this repo's `CLAUDE.md` / `AGENTS.md`
   (requires web-search confirmation of impeccable-set names).
7. `entity-realtime-surreal-live` skill + `SurrealLiveAdapter` in
   `prometheus-entity-management`.
8. Entity Explorer in-app (FAB + panel + tree + multi-store/duplicate
   detection) atop the existing `devtools.ts` foundation.
9. Browser extension scaffold + page-↔-extension bridge (stretch).

## 6. Open questions for the user (resolve during Plan)

- Should this new phase **replace** `runtime-provider-protocol-hardening`
  as the active waypoint, or run as a parallel track? (Today's schema
  doesn't support parallel — pick one or expand the schema.)
- Browser extension: v1 deliverable or v2 stretch?
- Multi-store entity coordination — is there a known list of Zustand
  stores the explorer must enumerate, or should it auto-discover via a
  registration API the explorer introduces?
- For the rule-injector skill: idempotent rewrite (manage a fenced
  region in `CLAUDE.md`) vs. one-shot append? Recommend fenced region.
