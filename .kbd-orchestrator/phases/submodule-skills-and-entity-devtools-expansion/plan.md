# Phase Plan — submodule-skills-and-entity-devtools-expansion

- Generated: 2026-05-27
- Author: claude-code (kbd-plan)
- Backend: **OpenSpec** (detected `openspec/` at repo root; `project.json` sets `specSystem: openspec`)
- Source assessment: [assessment.md](./assessment.md)

## Scope adjustment vs. assessment

This plan folds in one new item raised in the `/kbd-plan` invocation:

> Hook system in `kbd-process-orchestrator` letting users insert
> actions before/after each phase, child phase, plan start/end,
> execute start/end, and every task within execute (e.g. each OpenSpec
> task). Default action: emit
> `starting/ending [phase|plan|execute|task <name>], [index] out of [total]`.
> Hooks must support both *augmentation* (add to default) and
> *override* (replace default).

This is now **change #3** below — sequenced right after the nested-phase
schema and before the new-phase / child-phase skills, so those skills are
authored *with* hook callouts baked in.

## Cross-repo target map

| Change | Target repo | Path hint |
|---|---|---|
| 1, 8 | this repo (`universal-agent-runtime`) | `CLAUDE.md`, `AGENTS.md`, `.gitignore`, `scripts/` |
| 2–7 | `prometheus-skill-system` | `skills/kbd-process-orchestrator/**` |
| 9 (skill side) | `prometheus-skill-system` | `skills/prometheus-entity-skills/**` |
| 9 (adapter side), 10, 11 | `prometheus-entity-management` | `src/adapters/`, `src/devtools*`, new `chrome-extension/` package |

All work happens in a **single persisted worktree** under
`~/.claude/worktrees/<name>/` (per change #1) so changes can be staged
and committed to each origin repo independently.

## Ordered change list

> Each row maps to one OpenSpec change. IDs use kebab-case prefixed by
> phase shorthand `ssed-` ("submodule-skills-and-entity-devtools").
> Run `/opsx:new <id>` to scaffold each. The `agent` column suggests
> the best executor at Execute time.

| # | OpenSpec change ID | Why-now | Depends on | Agent |
|---|---|---|---|---|
| 1 | `ssed-worktree-persistence-convention` | Lowest risk; unblocks every other change by getting work out of the in-repo `.claude/worktrees/`. Updates this repo's `CLAUDE.md`/`AGENTS.md`, adds `.gitignore` guard, ships a `scripts/worktree-new.sh` that always creates under `~/.claude/worktrees/`. | — | claude-code |
| 2 | `ssed-kbd-nested-phase-schema` | Foundation for `/kbd-new-child` & hook context. Extends `current-waypoint.json` with `parentPhase`, `childPhases[]`, `childPointer`; defaults preserve backward compat with Roo/Cursor/Codex/OpenCode readers. | — | claude-code |
| 3 | `ssed-kbd-process-hooks` | Hook system requested this turn. Defines hook spec, default reporter (`starting/ending <kind> <name> [i/n]`), discovery rules, augment-vs-override semantics. Lands *before* the new phase/child skills so they're authored hook-aware. | 2 | claude-code |
| 4 | `ssed-kbd-new-phase-skill` | Closes the documented-but-missing `kbd-new-phase` referenced in orchestrator `SKILL.md`. Hook-aware from day 1. | 2, 3 | claude-code |
| 5 | `ssed-kbd-child-phase-skills` | Implements `/kbd-new-child` + `/kbd-next-child`; uses the nested schema + hooks. | 2, 3, 4 | claude-code |
| 6 | `ssed-kbd-memory-first-execution` | Promotes surreal-memory from "optional" to "default-on if MCP available"; adds a `pk-recall` skill that queries prior phases' surreal-memory entries before assess/plan/execute. Cross-project learning enabled. | 3 | claude-code |
| 7 | `ssed-kbd-agent-rules-injector` | New skill `kbd-inject-agent-rules` that writes a fenced region (`<!-- agent-rules:start -->…<!-- agent-rules:end -->`) into `CLAUDE.md` and `AGENTS.md`. Pulls Karpathy's rules and the Claude-Code-author's 4-rule set via web search at install/refresh time, caches them in the skill, and re-syncs on demand. Idempotent. | — | claude-code |
| 8 | `ssed-uar-uiux-skill-routing` | This repo's `CLAUDE.md` / `AGENTS.md` gain a UI/UX routing section: must run "ui ux pro max", impeccable-set members (names confirmed via web search during execute), Anthropic web-design skill, Vercel React skill, and consult surreal-memory before writing any UI code. Also bakes in a "best-practice analysis first" guard. | 7 (uses same fenced-region machinery) | claude-code |
| 9 | `ssed-entity-surreal-live-adapter` | New skill `entity-realtime-surreal-live` in `prometheus-entity-skills` **and** new `SurrealLiveAdapter` in `prometheus-entity-management/src/adapters/`. Wires `LIVE SELECT`, diff payload mapping (CREATE/UPDATE/DELETE), reconnect + replay, schema-version coupling. | — | claude-code (skill) + entity-graph-realtime agent (adapter) |
| 10 | `ssed-entity-explorer-fab-panel` | In-app explorer atop existing `src/devtools.ts`: floating action button, tree view, per-entity inspector, event log, multi-store enumeration + duplicate detection across Zustand stores. Uses ui-ux-pro-max + impeccable + Anthropic web design (enforced by #8). | 8, 9 | frontend-design + entity-graph-optimize |
| 11 | `ssed-entity-explorer-browser-extension` | **Stretch / v2.** Chrome MV3 extension with devtools panel page + content script + `window.__PROMETHEUS_DEVTOOLS_HOOK__` bridge mirroring react-devtools. Reuses panel UI from #10. | 10 | frontend-design |

## Per-change task seeds

### 1. `ssed-worktree-persistence-convention`
- [ ] Add `scripts/worktree-new.sh <name>` that creates `~/.claude/worktrees/<name>` and links a `.claude/` settings copy into it.
- [ ] Update `CLAUDE.md` + `AGENTS.md`: "Always create worktrees under `~/.claude/worktrees/`; never inside the repo tree."
- [ ] Add `.gitignore` rule for `.claude/worktrees/` (defense in depth — this repo already has the dir).
- [ ] Migrate documentation; **do not** relocate the current worktree mid-phase.

### 2. `ssed-kbd-nested-phase-schema`
- [ ] Extend `references/schemas/current-waypoint.template.json` with `parentPhase`, `childPhases[]`, `childPointer` (default `null` / `[]`).
- [ ] Update `kbd-status` to render the chain (`root › child › grand-child`).
- [ ] Update `kbd-assess`, `kbd-plan`, `kbd-execute`, `kbd-reflect` SKILL.md sections that read the waypoint to ignore unknown fields when missing.
- [ ] Migration note in orchestrator `SKILL.md`.

### 3. `ssed-kbd-process-hooks` (new — from this turn's argument)
- [ ] Define hook events: `phase:before`, `phase:after`, `child:before`, `child:after`, `plan:before`, `plan:after`, `execute:before`, `execute:after`, `task:before`, `task:after`.
- [ ] Define hook context payload: `{ kind, name, index, total, phasePath, childPath, sourceTool, startedAt }`.
- [ ] Define discovery order: project-local (`.kbd-orchestrator/hooks/*.{sh,ts,py}`) → user (`~/.claude/skills/kbd-process-orchestrator/hooks/`) → skill default.
- [ ] Define mode: each hook declares `mode: augment | override`; default is `augment`. At most one `override` per event; if multiple, last-registered wins with a warning.
- [ ] Ship a default `report-progress` hook implementing the user's required line:
  `starting <kind> <name> [<index>/<total>]` / `ending <kind> <name> [<index>/<total>]`.
- [ ] Wire all KBD skills (`kbd-assess`, `kbd-plan`, `kbd-execute`, `kbd-reflect`, plus #4 and #5) to emit hook events at the documented boundaries.
- [ ] Within `kbd-execute`, emit `task:before` / `task:after` around every OpenSpec task (and every native-KBD task when OpenSpec isn't used).
- [ ] Persist hook output (one JSONL row per fire) to `.kbd-orchestrator/phases/<phase>/hooks.log.jsonl` for replay/debug.
- [ ] Document with two examples: an *augment* hook that posts to Slack, and an *override* hook that replaces the default banner.

### 4. `ssed-kbd-new-phase-skill`
- [ ] Author `skills/kbd-new-phase/SKILL.md` (mirror `kbd-next-phase` structure).
- [ ] Accept `<name> [goals...]`; write `phases/<name>/{goals.md, progress.json}`.
- [ ] Update `current-waypoint.json` with `phase`, clear `change`, set `stage: assessment_ready`.
- [ ] Emit `phase:before` hook around its own action.
- [ ] Update orchestrator `SKILL.md` to remove the "referenced but not implemented" gap.

### 5. `ssed-kbd-child-phase-skills`
- [ ] `skills/kbd-new-child/SKILL.md`: requires active phase, creates `phases/<parent>/children/<child-name>/`, appends to parent's `childPhases[]`, sets `childPointer` to new child.
- [ ] `skills/kbd-next-child/SKILL.md`: advances `childPointer` to next entry in `childPhases[]`, seeded from the just-finished child's reflection if present (mirrors `kbd-next-phase` logic).
- [ ] Hooks: `child:before` / `child:after` fire at boundaries.
- [ ] `kbd-status` shows current child position when in a nested context.

### 6. `ssed-kbd-memory-first-execution`
- [ ] New skill `kbd-memory-recall` (a.k.a. `pk-recall`): pre-phase step that queries surreal-memory for `phase_kind`, `tags`, related prior reflections, and pastes a digest into the active phase's `prior-context.md`.
- [ ] New skill `kbd-memory-log`: post-step writer; called by the default `report-progress` hook so every hook fire also persists to surreal-memory.
- [ ] Promote surreal-memory mention in orchestrator `SKILL.md` from optional to "default-on when MCP is reachable".
- [ ] Add a retention/relevance policy doc (`shared/references/memory-retention.md`).

### 7. `ssed-kbd-agent-rules-injector`
- [ ] Web-search step (run at Execute time): cache Karpathy's rule set + the Claude-Code-author's 4-rule set into `skills/kbd-inject-agent-rules/references/rules-cache.md` with source URLs and fetch date.
- [ ] Skill writes a fenced region into target file. Idempotent rewrite (replace fenced region in place).
- [ ] Accept `--target CLAUDE.md|AGENTS.md|both` (default: both).
- [ ] Accept `--refresh` flag to re-fetch rules.

### 8. `ssed-uar-uiux-skill-routing`
- [ ] Web-search step: confirm full member list of the "impeccable" skill set; cache to `.kbd-orchestrator/references/uiux-skill-roster.md`.
- [ ] Append a UI/UX routing block (via the fenced-region machinery from #7) to this repo's `CLAUDE.md` and `AGENTS.md`.
- [ ] Block content: "Before writing any UI/UX code: (a) consult surreal-memory for prior decisions, (b) run ui-ux-pro-max + each impeccable skill listed, (c) consult Anthropic web design + Vercel React skill, (d) summarize best practices, (e) only then write code." Also lists "best practices for runtime dev tools on a web page and as a Chrome extension" as required research before #10/#11.

### 9. `ssed-entity-surreal-live-adapter`
- [ ] In `prometheus-entity-skills`: new skill `entity-realtime-surreal-live` parallel to `entity-realtime-channel` / `entity-realtime-local-first`.
- [ ] In `prometheus-entity-management/src/adapters/`: `surreal-live.ts` implementing the `SyncAdapter` contract (existing `createElectricAdapter` is the reference).
- [ ] Cover `LIVE SELECT` subscription lifecycle, `CREATE | UPDATE | DELETE` → `ChangeSet` mapping, `affectedListKeys` derivation, reconnect replay (`SELECT … WHERE updated_at > checkpoint`), schema-version mismatch handling.
- [ ] Tests: extend `realtime-manager.test.ts` patterns.

### 10. `ssed-entity-explorer-fab-panel`
- [ ] React component `<EntityExplorerFab>` (default off in production; toggled by `NODE_ENV !== 'production'` *or* `?prometheus-devtools=1`).
- [ ] Panel sections: Tree, Inspector, Events, Stores, Duplicates.
- [ ] Tree: builds from `collectGraphDevStats` output (already in `src/devtools.ts`); per-entity row shows type, id, store-id, last-updated.
- [ ] Inspector: shows full entity, patches, subscribers, lineage.
- [ ] Events: append-only stream from a new `devtools-event-bus.ts`; subscribe to engine ops + adapter notifications.
- [ ] Stores: enumerate via a new `registerDevtoolsStore(store, label)` API so multi-store apps register on creation; render coverage of which stores hold which entity ids; flag duplicates and provide a "promote to canonical" action.
- [ ] Use ui-ux-pro-max + impeccable + Anthropic web design (gating from #8) before writing UI code.

### 11. `ssed-entity-explorer-browser-extension` (stretch — v2)
- [ ] New package `prometheus-entity-management/chrome-extension/` with MV3 `manifest.json`, `devtools_page`, content-script bridge.
- [ ] Page side: `__PROMETHEUS_DEVTOOLS_HOOK__` global; emits `postMessage` events captured by the extension.
- [ ] Extension reuses the panel UI from #10 (shared package).
- [ ] Confirm need + ship plan during Plan-of-#11 (run web search for "react-devtools-like bridge MV3 best practices" first, per #8 guard).

## Risks (revisit before Execute)

1. **Cross-tool consumers of `current-waypoint.json`** must tolerate the new fields. Add a compatibility test in #2 that loads with each tool's schema.
2. **Web-search dependence** for #7 and #8 — must be done with the tool available; cache must include source URL + fetch date for audit.
3. **Hook log volume** (#3) — JSONL log rotation policy belongs in the change.
4. **`postMessage` security** in #11 — restrict origins; the extension must not leak page state to other tabs.
5. **Browser-extension v1-vs-v2 decision** still open from assessment §6 — default here is v2/stretch; flip to v1 only on explicit user confirmation.

## OpenSpec emit list

Run from repo root:

```
/opsx:new ssed-worktree-persistence-convention
/opsx:new ssed-kbd-nested-phase-schema
/opsx:new ssed-kbd-process-hooks
/opsx:new ssed-kbd-new-phase-skill
/opsx:new ssed-kbd-child-phase-skills
/opsx:new ssed-kbd-memory-first-execution
/opsx:new ssed-kbd-agent-rules-injector
/opsx:new ssed-uar-uiux-skill-routing
/opsx:new ssed-entity-surreal-live-adapter
/opsx:new ssed-entity-explorer-fab-panel
/opsx:new ssed-entity-explorer-browser-extension      # gate behind user confirmation
```

## Evolver bridge

No `.evolver/evolutions/*/plan.json` is present for this phase — no
bridge file written.
