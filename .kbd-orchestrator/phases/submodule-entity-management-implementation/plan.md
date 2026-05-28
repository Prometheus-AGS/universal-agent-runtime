# Phase Plan — submodule-entity-management-implementation

- Generated: 2026-05-27
- Author: claude-code (kbd-plan)
- Backend: **OpenSpec** (`openspec/config.yaml schema: spec-driven`)
- Source assessment: [assessment.md](./assessment.md)
- Evolver bridge: none

## Default decisions on the open questions

Until the user says otherwise, this plan adopts the assessment's recommendations:

1. **Spec reconciliation**: delta change (preserves historical record).
2. **Worktree**: new persistent worktree under `~/.claude/worktrees/seim-entity-management` for the entity-management work (created via `scripts/worktree-new.sh` from change 1).
3. **Chrome extension scope**: scaffold-only this phase; UI polish is a follow-up phase.
4. **Production tree-shake gate**: hard fail in `prepublishOnly`.

## Cross-repo target map

| Change ID prefix | Target repo |
|---|---|
| `seim-skill-system-*` | `prometheus-skill-system` (carry-over commit from prior phase) |
| `seim-spec-correction-*` | this repo (`universal-agent-runtime`, openspec/specs/) |
| `seim-uar-*` | this repo |
| `seim-em-*` | `prometheus-entity-management` |

## Ordered change list

> Change IDs prefixed `seim-` (submodule-entity-management-implementation).
> Each row maps to one OpenSpec change. Run `/opsx:new <id>` to scaffold.

| # | OpenSpec change ID | Why-now | Depends on | Agent |
|---|---|---|---|---|
| 1 | `seim-skill-system-pr-bundle` | **Must land first.** The dirty `prometheus-skill-system` tree from the prior phase (10 paths across changes 2–8) blocks: the entity-management PRs in this phase want to reference shipped skill versions and the `entity-realtime-surreal-live` skill, but those don't exist on origin yet. This change opens the topic-branch PR. | — | claude-code (with operator git authentication for push) |
| 2 | `seim-surreal-live-spec-correction` | Reconciles the archived `entity-surreal-live-adapter` spec (which assumed `SyncAdapter.start/stop`) with the codebase's actual `RealtimeAdapter.subscribe(config, handler)` contract. Delta change: ships its own `proposal.md` + a new spec at `openspec/specs/entity-surreal-live-adapter/spec.md` (replacing the archived one). | — (independent of change 1) | claude-code |
| 3 | `seim-em-worktree-setup` | Create the persistent worktree at `~/.claude/worktrees/seim-entity-management` against `prometheus-entity-management`'s main branch via `scripts/worktree-new.sh`. Document the topic-branch naming convention in the phase's `execution.md`. Tiny — could fold into change 4, kept separate for the audit trail. | 1 (so the worktree pulls a tree that's seen the upstream skill changes via re-pull) | claude-code |
| 4 | `seim-em-surreal-live-adapter-impl` | Implement `createSurrealLiveAdapter` per the corrected spec from change 2. `RealtimeAdapter.subscribe(config, handler)` shape, per-channel `LIVE SELECT`, action mapping, reconnect, optional checkpoint replay, `affectedListKeys`. Vitest suite with hand-rolled fake `Surreal` client mirroring `electricsql-tenant.test.ts`'s `fakeShapeStream`/`fakePGlite` pattern. Re-export from `src/index.ts`. | 2, 3 | claude-code (Rust auditor not applicable; TS work) |
| 5 | `seim-em-engine-devtools-tap` | Engine-side hook for the explorer's event bus. Add `subscribeDevtoolsEvent(cb)` + `notifyDevtools(event)` exports to `src/engine.ts`. Call `notifyDevtools` at every op site (upsert, patch, delete, clearPatch, list ops). Guard with `NODE_ENV !== "production"` so prod builds tree-shake the calls. | 3 | claude-code |
| 6 | `seim-em-explorer-preflight-research` | Per change 8's UI/UX routing discipline (already live in this repo's `CLAUDE.md`/`AGENTS.md`), run the seven-step pre-work BEFORE writing any panel components. Commits `docs/devtools-design-notes.md`. Hard prerequisite for changes 7–9. | 3 | claude-code (web search + summarise) |
| 7 | `seim-em-explorer-event-bus-registry` | Implement `src/devtools/devtools-event-bus.ts` (1000-entry ring buffer, `push`/`subscribe`/`getSnapshot`) and `src/devtools/multi-store-registry.ts` (opt-in store registration with production no-op stub). Wire bus subscription to the engine tap from change 5 and to the realtime-manager's `ChangeSet` notifications. | 5, 6 | claude-code |
| 8 | `seim-em-explorer-panel-components` | The 5-tab React UI: `EntityExplorerFab.tsx`, `panel/{EntityExplorerPanel,TreeTab,InspectorTab,EventsTab,StoresTab,DuplicatesTab}.tsx`. Uses existing `collectGraphDevStats` for stats, event bus for live events, registry for multi-store enumeration. Dev-mode gate + URL escape hatch. Tests via React Testing Library (verify it's a devDep first — assessment §4.3). | 6, 7 | `frontend-design` agent for component scaffolding + `claude-code` for wiring |
| 9 | `seim-em-explorer-production-treeshake-check` | Bundle-analyser script (added to `scripts/` and invoked from `prepublishOnly`) that asserts no `src/devtools/` symbols leak into the published bundle. Hard-fail on regression. | 8 | claude-code |
| 10 | `seim-em-extension-architecture-notes` | Pre-flight research for the Chrome MV3 extension. Three web searches (Chrome MV3 panel patterns, react-devtools bridge, postMessage origin validation). Commits `chrome-extension/docs/architecture-notes.md` BEFORE any extension code. Hard prerequisite for change 11. | 8 (so the panel-host data-source abstraction is settled) | claude-code |
| 11 | `seim-em-extension-scaffold` | Chrome MV3 extension scaffold: `chrome-extension/{manifest.json,package.json,tsconfig.json,src/*,public/icons/*}` + bridge module + content-script + page-hook + panel host. **Scope = scaffold only** per the default-decision policy; UI polish + Web Store publication out of scope. Plus the host-side `__PROMETHEUS_DEVTOOLS_HOOK__` installation in `EntityExplorerFab.tsx`. | 8, 10 | `frontend-design` for the panel host; `claude-code` for the bridge |

## Per-change task seeds (compact — full task lists land in `/opsx:continue` cycles)

### 1. `seim-skill-system-pr-bundle`
- [ ] Stage all 10 dirty paths in `prometheus-skill-system` on topic branch `feat/kbd-orchestrator-w1-w3` (or similar)
- [ ] Commit message references prior phase + the 8 component changes
- [ ] Push, open PR, capture URL + SHA; record back in each prior-phase change's tasks.md §9 / §10
- [ ] **Wait for merge** before changes 4 and 5 land (they re-pull origin to see shipped skill SKILL.md updates)

### 2. `seim-surreal-live-spec-correction`
- [ ] Replace `openspec/specs/entity-surreal-live-adapter/spec.md` with the corrected contract (`RealtimeAdapter.subscribe(config, handler)`, per-channel subscriptions, `ChannelConfig`-driven topology, `onStatusChange?` instead of `onStatus/onSynced`)
- [ ] Archive the correction itself under `openspec/changes/archive/2026-MM-DD-seim-surreal-live-spec-correction/` so the historical record is preserved
- [ ] Update the archived change 9's tasks.md to cross-link the correction
- [ ] Note: this is a tiny doc-only change (under the 3-file QA threshold — may run `--skip-qa`)

### 3. `seim-em-worktree-setup`
- [ ] `bash scripts/worktree-new.sh seim-entity-management` (the helper installed by change 1 of the prior phase) — wait, that helper lives in *this* UAR repo, not in entity-management. Adapt: invoke `git worktree add ~/.claude/worktrees/seim-entity-management` against the entity-management repo directly, or copy the helper script
- [ ] Document the chosen approach in this phase's `execution.md`
- [ ] Verify the new worktree's `git rev-parse --show-toplevel` resolves under `~/.claude/worktrees/`

### 4. `seim-em-surreal-live-adapter-impl`
- [ ] `src/adapters/surreal-live.ts` — implement against corrected spec (per-channel subscribe, action mapping, reconnect, replay)
- [ ] `src/adapters/surreal-live.test.ts` — fake `Surreal` client; tests for seed / CREATE / UPDATE / DELETE mapping / reconnect / replay / `affectedListKeys` / `onStatusChange`
- [ ] `src/index.ts` — re-export
- [ ] `pnpm test src/adapters/surreal-live.test.ts` green
- [ ] `pnpm typecheck` green
- [ ] `pnpm build` green

### 5. `seim-em-engine-devtools-tap`
- [ ] `src/engine.ts` — add `subscribeDevtoolsEvent(cb): () => void` and `notifyDevtools(event)`
- [ ] Insert `notifyDevtools(...)` calls at every mutating op site (upsert / patch / delete / clearPatch / list mutations)
- [ ] Tree-shake gate: wrap each call in `if (process.env.NODE_ENV !== "production")` (or use a build-time `define` flag)
- [ ] Unit test: subscribe, perform ops, assert events received

### 6. `seim-em-explorer-preflight-research`
- [ ] `/kbd-memory-recall` for this phase (auto-fired)
- [ ] UI/UX Pro Max analysis of existing dashboard pages (find the host project's stylistic baseline)
- [ ] `/impeccable audit` + `/impeccable critique` on early panel sketches
- [ ] Consult Anthropic frontend-design + ux-designer skills
- [ ] Consult Vercel React Best Practices + Composition Patterns
- [ ] Web search: "runtime devtools page best practices 2026" — capture URLs + anchor keywords
- [ ] Web search: "react-devtools bridge architecture" — capture URLs + anchor keywords
- [ ] Write `docs/devtools-design-notes.md` per the contents required by change 10's spec req "Pre-flight UI/UX Research Doc"
- [ ] **Gate**: changes 7, 8 cannot start until this file is committed

### 7. `seim-em-explorer-event-bus-registry`
- [ ] `src/devtools/devtools-event-bus.ts` — ring buffer + APIs; subscribe to engine tap + realtime-manager
- [ ] `src/devtools/multi-store-registry.ts` — `registerDevtoolsStore(config)` with replacement semantics, prod no-op
- [ ] Unit tests covering capacity / eviction order / subscribe delivery / registration replacement / prod no-op

### 8. `seim-em-explorer-panel-components`
- [ ] Verify React Testing Library is in devDeps; add if missing
- [ ] `EntityExplorerFab.tsx` — dev gate + FAB + panel toggle (a11y per change 8)
- [ ] `panel/EntityExplorerPanel.tsx` — 5-tab container with keyboard nav + focus management
- [ ] `panel/TreeTab.tsx` — entities by type, store-coverage badges, status badges
- [ ] `panel/InspectorTab.tsx` — full entity drill-down + event timeline filter
- [ ] `panel/EventsTab.tsx` — live tail with filters; virtualise rows for performance
- [ ] `panel/StoresTab.tsx` — registered store enumeration + diff-against-canonical
- [ ] `panel/DuplicatesTab.tsx` — `(type, id)` duplicates + Promote-to-canonical (engine mutation path)
- [ ] Component tests: render each tab against a fixture graph
- [ ] Mount `<EntityExplorerFab />` in `examples/vite-app/` and confirm in dev mode
- [ ] Re-run `/impeccable audit` on the rendered panel; address findings

### 9. `seim-em-explorer-production-treeshake-check`
- [ ] `scripts/check-devtools-treeshake.mjs` — runs the build, inspects the output bundle (e.g. via `source-map-explorer` or a string-grep over the bundle), asserts zero matches for `src/devtools/` symbol names
- [ ] Wire into `package.json` `prepublishOnly` chain
- [ ] Hard-fail on regression; document in CONTRIBUTING.md how to debug a tree-shake failure

### 10. `seim-em-extension-architecture-notes`
- [ ] Three web searches per change 8 routing discipline
- [ ] Commit `chrome-extension/docs/architecture-notes.md` with bridge data-flow diagram, MV3 manifest rationale, security model (discriminator + nonce + origin), pre-flight references with URLs + fetch dates
- [ ] **Gate**: change 11 cannot start until this file is committed

### 11. `seim-em-extension-scaffold`
- [ ] `chrome-extension/` package: `package.json`, `tsconfig.json`, `manifest.json` (MV3 + devtools_page + content_scripts)
- [ ] `src/devtools-page.{html,ts}` — registers the Entity Graph panel
- [ ] `src/panel.{html,tsx}` — mounts `EntityExplorerPanel` (from change 8) with the bridge data source
- [ ] `src/content-script.ts` — relays page ↔ background; injects `page-hook.ts` into MAIN world via `chrome.scripting.executeScript`
- [ ] `src/page-hook.ts` — installs `window.__PROMETHEUS_DEVTOOLS_HOOK__` (this is the page-side surface)
- [ ] `src/bridge/{envelope,page-to-extension,extension-to-page}.ts`
- [ ] Host-side hook installation in `src/devtools/EntityExplorerFab.tsx` (change 8) — guarded by `NODE_ENV !== "production"`
- [ ] `chrome-extension/README.md` documenting side-load via `chrome://extensions` Developer Mode
- [ ] Manual smoke: side-load extension, open DevTools on `examples/vite-app/`, confirm the panel renders entities

## Execution waves

| Wave | Changes | Parallelism | Rationale |
|---|---|---|---|
| W0 | 1 `skill-system-pr-bundle` | serial — blocking | Must merge before W2 starts so origin reflects shipped skill SKILL.md |
| W1 | 2 `spec-correction` | serial (1 change) | Independent of W0; can land while W0's PR is in review |
| W2 | 3 `worktree-setup` | serial | After W0 merges |
| W3 | 4 `surreal-live-impl`, 5 `engine-devtools-tap` | **parallel-2** | Independent files; no overlap |
| W4 | 6 `preflight-research` | serial — **gating** | Must produce design notes before any panel code |
| W5 | 7 `event-bus-registry` | serial | Depends on engine tap + preflight |
| W6 | 8 `panel-components` | serial | Substantial; depends on bus + registry |
| W7 | 9 `treeshake-check`, 10 `extension-architecture-notes` | **parallel-2** | Independent |
| W8 | 11 `extension-scaffold` | serial — **stretch** | Final wave; scaffold-only per policy |

Total estimated changes: **11**.

## Per-change agent assignments (recap)

| # | Change | Primary agent |
|---|---|---|
| 1 | skill-system-pr-bundle | claude-code (+ operator git auth) |
| 2 | spec-correction | claude-code |
| 3 | worktree-setup | claude-code |
| 4 | surreal-live-impl | claude-code |
| 5 | engine-devtools-tap | claude-code |
| 6 | preflight-research | claude-code (web search) |
| 7 | event-bus-registry | claude-code |
| 8 | panel-components | `frontend-design` (UI scaffolding) + claude-code (wiring) |
| 9 | treeshake-check | claude-code |
| 10 | extension-architecture-notes | claude-code (web search) |
| 11 | extension-scaffold | `frontend-design` (panel host) + claude-code (bridge) |

## QA decisions per change

| # | Files modified (est.) | Doc-only? | QA required (`/opsx:verify`) |
|---|---|---|---|
| 1 | 0 here; 10 staged upstream | n/a | **n/a** — git operation, not an artifact change |
| 2 | 1 (spec file) | yes | `--skip-qa` |
| 3 | 1 (execution.md note) | yes | `--skip-qa` |
| 4 | 3+ (adapter, test, index re-export) | no | **yes** |
| 5 | 1–2 (engine.ts + a test) | no | **yes** |
| 6 | 1 (design notes doc) | yes | `--skip-qa` |
| 7 | 2+ (bus + registry + tests) | no | **yes** |
| 8 | 10+ (FAB + panel + 5 tabs + tests) | no | **yes** |
| 9 | 2 (script + package.json wire-up) | no | **yes** |
| 10 | 1 (architecture notes doc) | yes | `--skip-qa` |
| 11 | 12+ (extension package) | no | **yes** |

## Cross-cutting reminders (carried from assessment)

- **UI/UX routing discipline is live** in this repo's CLAUDE.md / AGENTS.md `<!-- uiux-routing:start v1 -->` region. Every UI/UX-producing change (6, 8, 10, 11) MUST follow the seven-step pre-work. The discipline is enforced doc-only; agents read it on session start.
- **Karpathy + Boris Cherny rules are live** in the `<!-- agent-rules:start v1 -->` region — think before coding, simplicity, surgical changes, plan-first, verification loops.
- **Memory recall** auto-fires on `assess:before` (`auto-memory-recall` hook). When surreal-memory is reachable, every phase consumes prior context automatically.
- **Hook reporter** emits `starting/ending <kind> <name> [i/n]` on stderr per `/kbd-execute` and every KBD lifecycle skill. Operators see live progress.

## Risks (carried from assessment, re-prioritised)

1. **W0 blocking risk** — if the skill-system PR can't merge cleanly (review delay, conflicts), W2 onward stalls. Mitigation: smallest possible PR; reviewers identified before W0 starts.
2. **Spec drift on change 4** — if the W1 correction misses any contract gap, change 4 hits the same issue mid-implementation. Mitigation: change 4 reads BOTH the corrected spec and the actual `types.ts` before any code; flag any new discrepancy as a P0 issue.
3. **Tree-shake regressions in change 8/9** — the explorer module is large; preserving the prod-zero invariant requires careful import discipline. Mitigation: change 9 is the *check*; if it fails, change 8 has bugs to fix.
4. **MV3 API churn (change 11)** — verify against latest Chrome stable before implementation. Captured in change 10's pre-flight.
5. **Skill SKILL.md ↔ implementation drift** — once changes 4/5/7/8 land, the upstream skill SKILL.md (already shipped in W2 of the prior phase) may need a refresh. Filed as a follow-up; not in scope here.
6. **Worktree convention application** — change 3 creates a separate worktree for entity-management work. This UAR worktree's tooling references continue to apply, but cross-references between worktrees need explicit absolute paths.

## OpenSpec emit list

Run from repo root:

```
/opsx:new seim-skill-system-pr-bundle
/opsx:new seim-surreal-live-spec-correction
/opsx:new seim-em-worktree-setup
/opsx:new seim-em-surreal-live-adapter-impl
/opsx:new seim-em-engine-devtools-tap
/opsx:new seim-em-explorer-preflight-research
/opsx:new seim-em-explorer-event-bus-registry
/opsx:new seim-em-explorer-panel-components
/opsx:new seim-em-explorer-production-treeshake-check
/opsx:new seim-em-extension-architecture-notes
/opsx:new seim-em-extension-scaffold
```

## Evolver bridge

No `.evolver/evolutions/*/plan.json` present for this phase — no bridge file written.
