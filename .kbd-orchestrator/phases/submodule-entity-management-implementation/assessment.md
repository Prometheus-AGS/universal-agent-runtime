# Phase Assessment — submodule-entity-management-implementation

- Generated: 2026-05-27
- Author: claude-code (kbd-assess)
- Active phase (current waypoint at run time): `submodule-skills-and-entity-devtools-expansion` *(reflect_complete)*
- Recommended phase under assessment: **`submodule-entity-management-implementation`** (per the prior reflection's "Recommended Next Phase")
- Argument framing: pick up the three PARTIAL goals from the prior phase as the scope of this one.

## 1. Phase carry-over from prior reflection

The prior phase shipped specs + skill docs for changes 9, 10, 11 but deferred the TypeScript implementation. The three PARTIAL goals being assessed here are:

1. **SurrealDB live-query realtime adapter** — `entity-realtime-surreal-live` skill ✅ shipped; TS adapter at `prometheus-entity-management/src/adapters/surreal-live.ts` ❌ not yet implemented.
2. **Entity Explorer (FAB + 5-tab panel)** — `entity-graph-optimize` SKILL.md "Dev-mode entity explorer" subsection ✅ shipped; React UI under `src/devtools/` ❌ not yet implemented.
3. **Chrome extension stretch** — manifest contract + bridge envelope spec ✅ shipped; `chrome-extension/` package ❌ not yet scaffolded.

In addition, the prior phase left **cross-repo commits pending** in `prometheus-skill-system` (10 dirty paths across changes 2–8). That work is not a goal of this phase, but it is a precondition for clean implementation tracking (otherwise change 9/10/11 commits in entity-management can't reference upstream SKILL.md changes that haven't merged yet).

## 2. Discovery — what's already in place in `prometheus-entity-management`

### 2.1 Adapter substrate (relevant to goal #1)

`src/adapters/types.ts` defines two interfaces — **and the interface used by the existing `realtime-manager.ts` differs from what change 9's design assumed**. This is the most important finding of this assessment:

| Surface | What change 9 spec assumed | What's actually in the codebase |
|---|---|---|
| Lifecycle | `start(handler) / stop()` | `subscribe(config, handler) → UnsubscribeFn` |
| Per-channel | One global adapter stream | One subscription **per `ChannelConfig`** |
| Registration | `manager.registerAdapter(name, adapter)` | `manager.register(adapter, channels[], normalize?)` |
| Status | `onStatus(cb) / onSynced(cb)` | `onStatusChange?(cb)` (optional, on `RealtimeAdapter`) |

The existing `RealtimeAdapter` shape (`name`, `subscribe`, `onStatusChange?`) is also what `electricsql.ts` returns. So `SurrealLiveAdapter` should match the same shape — change 9's design has a mismatch with reality that must be reconciled before implementation.

The good news: `RealtimeAdapter` is *cleaner* than the assumed shape — per-channel subscriptions map naturally to per-table `LIVE SELECT`s. The reconciliation is a simplification, not a complication.

Also confirmed in place:
- `realtime-manager.ts` already coalesces rapid changes in a 16 ms flush window (`pendingChanges` + `flushTimer`). Adapters don't need to debounce themselves.
- `ChangeSet.affectedListKeys` is part of the existing type contract.
- `EntityChange` already supports the four operations the SurrealDB live stream emits.

### 2.2 Devtools substrate (relevant to goal #2)

`src/devtools.ts` (95 LOC) already implements `collectGraphDevStats` returning the full payload the panel needs:

- `entityCounts: Record<EntityType, number>`
- `totalEntities`
- `listKeys[]` + `listCount`
- `patchedEntities: [{type, id}]`
- `staleEntities: [{type, id}]`
- `fetchingEntities: [{type, id}]`

It exports a React hook over `useSyncExternalStore` so panel components can subscribe without re-implementing the bridge to Zustand.

`engine.ts` already exposes the dev hooks (`subscribeSubscriberStats`, `getActiveSubscriberCount`) the Events tab needs. **`notifyDevtools` / `subscribeDevtoolsEvent` does NOT yet exist** — this is the only engine-side gap change 10 introduces.

### 2.3 Examples (relevant to all three goals)

`examples/` carries `vite-app/`, `nextjs-app/`, `supabase/` example workspaces. These are the natural mounting points for `<EntityExplorerFab>` once it lands — running the example apps with the new FAB validates the production tree-shake (built example bundles shouldn't include panel code).

### 2.4 Build + test pipeline

- Build: `tsup` (declared `build` / `build:watch` scripts in `package.json`).
- Tests: `vitest run` (declared in `scripts`).
- Typecheck: `tsc --noEmit -p tsconfig.json`.
- Dev deps: tsup ≥8, typescript ≥6, vitest ≥4 — all present.

The pre-publish chain (`prepublishOnly`) runs typecheck + build + test + verify:skills, so a regression in any of those will block release. Worth re-running this gate after change 9/10's code lands.

### 2.5 The other half of the skill set

`~/.claude/skills/prometheus-entity-skills` symlinks to `prometheus-skill-system/skills/react/prometheus-entity-skills` (not to a sub-package of entity-management — the skill-system repo owns the skill set). The skill SKILL.md changes for goals 1 + 2 already live in `prometheus-skill-system` and are part of the still-pending cross-repo commit from the prior phase.

## 3. Gap matrix

| # | Goal | Substrate present | What's missing |
|---|---|---|---|
| 1 | SurrealDB live-query adapter | `RealtimeAdapter` + `ChannelConfig` + `realtime-manager` 16ms coalesce + skill SKILL.md | `src/adapters/surreal-live.ts`, `surreal-live.test.ts`, `src/index.ts` re-export. **Plus**: spec→code reconciliation to use `RealtimeAdapter.subscribe(config, handler)` shape, not `SyncAdapter.start(handler)/stop()` |
| 2 | Entity Explorer FAB + 5-tab panel | `src/devtools.ts` data collection + `engine.ts` subscriber hooks + skill subsection | `src/devtools/` module (FAB, panel, 5 tabs, event bus, multi-store registry), `engine.ts` gains `subscribeDevtoolsEvent`, `__tests__/*`, `docs/devtools-design-notes.md` (pre-flight per UI/UX routing discipline) |
| 3 | Chrome extension stretch | OpenSpec contract + bridge envelope types | `chrome-extension/` package (manifest, devtools_page, content-script, page-hook, panel.tsx, bridge module), `docs/architecture-notes.md`, `EntityExplorerFab` gains hook installation |

## 4. Cross-cutting concerns

### 4.1 UI/UX routing discipline (from prior change 8)

This phase's goals 2 + 3 are UI work. The discipline is now live in this repo's `CLAUDE.md` / `AGENTS.md` `<!-- uiux-routing:start v1 -->` region:

1. `/kbd-memory-recall` (auto-fired on `assess:before`) → fills `prior-context.md`
2. UI/UX Pro Max analysis
3. Impeccable `/audit` + `/critique` + work-specific commands
4. Anthropic frontend-design + ux-designer
5. Vercel React skills + web search for "runtime devtools page best practices" + "Chrome MV3 devtools panel patterns"
6. Distill best practices in one paragraph
7. Then code

This phase's `kbd-plan` MUST schedule the pre-flight research BEFORE any production component code in the change tasks.

### 4.2 Spec reconciliation

Change 9's archived design has the wrong interface assumption. Options:

- **(A)** Leave the archived spec; the plan documents the reconciliation in the new change's design notes and the implementation follows reality.
- **(B)** Open a "spec correction" change in this phase that updates `openspec/specs/entity-surreal-live-adapter/spec.md` to match `RealtimeAdapter`'s actual shape before implementing.

Recommend **(B)** — spec drift is a known gotcha and the next phase is the right place to correct it cleanly. The plan should sequence: spec correction → implement → verify.

### 4.3 Test infrastructure

Existing tests use vitest with hand-rolled fakes (see `electricsql-tenant.test.ts` — uses `fakeShapeStream()` + `fakePGlite()`). The SurrealDB tests follow the same pattern (hand-rolled fake `Surreal`). The explorer tests will need React Testing Library — verify it's already a devDep before relying on it.

### 4.4 Production tree-shake validation

Change 10's spec requires that production bundles drop the entire `src/devtools/` directory. Need a post-build script (or addition to `prepublishOnly`) that runs the bundle analyser and asserts zero panel symbols in the prod artifact. New work.

### 4.5 Worktree convention application

This new phase should follow the worktree convention from the prior phase: create a new worktree under `~/.claude/worktrees/` for the entity-management work specifically, rather than nesting inside this UAR worktree. Document in the plan.

## 5. Risks & open items

1. **Spec-drift in change 9.** Documented in §4.2. Plan must address with a spec-correction change.
2. **MV3 API churn for the extension.** Chrome's `scripting.executeScript({ world: "MAIN" })` is the documented path; verify it works on the latest Chrome stable before sinking time into the bridge.
3. **Bundle size of the explorer.** Even with tree-shake, dev bundles will grow noticeably. Acceptable; explicit non-goal of optimisation in this phase.
4. **Multi-store registration API stability.** Apps need to opt in by calling `registerDevtoolsStore`. Until major Prometheus apps adopt it, the Duplicates tab is a "demo" feature. Plan should note this.
5. **Cross-repo commit ordering.** Skill-system changes from the prior phase need to land *first* so the entity-management PRs can reference shipped skill versions. Plan should make this an explicit first task.
6. **`/refine-validate` still unwired.** Continuing to substitute `/opsx:verify`. Carry forward as known limitation; not a goal here.
7. **Routing discipline enforcement.** Doc-only. Plan should require the `docs/devtools-design-notes.md` pre-flight artifact as a hard prerequisite for any panel-component PR.

## 6. Recommended scope for this phase

| Change ID | Scope | Repo |
|---|---|---|
| `seim-skill-system-pr-bundle` | Commit + PR the prior phase's dirty skill-system tree | prometheus-skill-system |
| `seim-surreal-live-spec-correction` | Spec correction: update `entity-surreal-live-adapter` to use `RealtimeAdapter.subscribe` shape | universal-agent-runtime (openspec/specs) |
| `seim-surreal-live-adapter-impl` | Implementation of `createSurrealLiveAdapter` + vitest suite + re-export | prometheus-entity-management |
| `seim-explorer-preflight-research` | Pre-flight UI/UX research per change 8 discipline; commit `docs/devtools-design-notes.md` | prometheus-entity-management |
| `seim-engine-devtools-tap` | `engine.ts` gains `subscribeDevtoolsEvent` + `notifyDevtools` calls at op sites | prometheus-entity-management |
| `seim-explorer-event-bus-registry` | `devtools-event-bus.ts` + `multi-store-registry.ts` | prometheus-entity-management |
| `seim-explorer-panel-components` | `<EntityExplorerFab>` + 5 panel tabs + tests | prometheus-entity-management |
| `seim-explorer-production-treeshake-check` | Bundle analyser script asserting prod build excludes `src/devtools/` | prometheus-entity-management |
| `seim-extension-architecture-notes` | `chrome-extension/docs/architecture-notes.md` with pre-flight web search synthesis | prometheus-entity-management |
| `seim-extension-scaffold` | `chrome-extension/` package with manifest, bridge module, content-script, page-hook, panel host | prometheus-entity-management |

(Plan phase will sequence these, decide which fold together, and assign waves.)

## 7. Open questions for the user (resolve during `/kbd-plan`)

- **Spec reconciliation policy** — modify the archived `entity-surreal-live-adapter` spec in-place (open-edit), or ship a delta `seim-surreal-live-spec-correction` change that records the correction in its own archive entry? Recommend the latter (preserves the historical record).
- **Worktree for entity-management work** — create a new persistent worktree under `~/.claude/worktrees/` for this phase, or run directly against the main `prometheus-entity-management` checkout? Convention prefers a new worktree per phase.
- **Chrome extension scope this phase** — produce the scaffold + manifest only (per the prior phase's stretch status), or push into a working v1 panel render? Recommend scaffold-only here; UI polish belongs in a follow-up phase.
- **Production tree-shake gate** — fail `prepublishOnly` when panel symbols leak into the prod bundle, or warn-only? Recommend hard fail.

## 8. Validation pointers carried forward

- The fenced regions in this repo's CLAUDE.md (`<!-- agent-rules:start v1 -->` + `<!-- uiux-routing:start v1 -->`) are read by every AI tool on session start — the Karpathy + Boris Cherny rules and the seven-step UI/UX routing discipline are baseline context for the entire phase.
- `kbd-memory-recall` auto-fires on `assess:before` (via the `auto-memory-recall` hook) — when surreal-memory is reachable, this assessment will be supplemented by prior-context.md in future runs. (No memory hits surfaced this run; the prior phase is the seed entry.)
- All KBD skills (`/kbd-plan`, `/kbd-execute`, etc.) fire `<kind>:before`/`<kind>:after` hooks. The `report-progress` reporter emits `starting/ending <kind> <name> [i/n]` on stderr by default — operators see live progress without configuring anything.
