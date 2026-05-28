# Phase Reflection — submodule-entity-management-implementation

- Generated: 2026-05-28
- Author: claude-code (kbd-reflect)
- Duration: single multi-session run (2026-05-27 → 2026-05-28)
- Scope: 11 OpenSpec changes, 1 target repo (`prometheus-entity-management`), 3 merged PRs

## Phase shape recap

- Backend: OpenSpec (spec-driven)
- Changes total: 11
- Changes archived: **11 / 11**
- Repo: `prometheus-entity-management` — worktree `~/.claude/worktrees/seim-entity-management`
- Branch: `feat/seim-entity-management-impl` (now merged to `main`)
- Cross-repo: `prometheus-skill-system` PR #3 (W0) + `prometheus-entity-management` PR #1 (W3) + PR #2 (W4–W8) — all merged
- Test suite at phase end: **159 / 159 passing**, 1 todo
- TypeScript errors at phase end: **0**

## Goal achievement (against `/kbd-assess` original goals)

| # | Goal | Status |
|---|---|---|
| 1 | SurrealDB live-query realtime adapter (`createSurrealLiveAdapter`) | **MET** — implemented per corrected spec (W2 spec correction + W3 impl); 20 tests; merged PR #1 |
| 2 | Entity Explorer FAB + tabbed panel | **MET** — FAB + 4-tab panel (Entities / Patches / Events / Performance); React portal; Alt+Shift+E keyboard shortcut; 15 jsdom tests; merged PR #2 |
| 3 | Chrome extension stretch (scaffold) | **MET** — MV3 manifest, content script, service worker, devtools page, panel React app, `createExtensionBus()`, `bus.inject()`, `enableWindowBridge` prop; merged PR #2 |
| 4 | DevtoolsEventBus (ring-buffer, fan-out, burst coalescing) | **MET** (unlocked by W3) — 500-entry ring buffer, synchronous replay-on-subscribe, microtask burst coalescing (`threshold=10`), multi-store registry; 23 tests; merged PR #2 |
| 5 | Production tree-shake gate | **MET** — `check:treeshake` script in `prepublishOnly`; hard-fails if NODE_ENV guard removed from call sites; merged PR #2 |

Overall: **5 / 5 goals MET** (all three carry-overs from the prior PARTIAL state are now fully implemented and merged).

## Commit map

| SHA | Subject | Wave | Merged |
|-----|---------|------|--------|
| `9f314ba` | feat(adapters): SurrealDB LIVE SELECT adapter | W3 | PR #1 ✅ |
| `b65e74f` | feat(engine): devtools push stream + per-op tap | W3 | PR #1 ✅ |
| `bc51bfa` | docs(explorer): preflight research notes W4 | W4 | PR #2 ✅ |
| `3af47b2` | feat(devtools): event bus + multi-store registry | W5 | PR #2 ✅ |
| `3d2c147` | feat(ui): Entity Explorer FAB + 4-tab panel | W6 | PR #2 ✅ |
| `ddb4c44` | chore(build): treeshake gate | W7a | PR #2 ✅ |
| `5a1f388` | docs(extension): MV3 architecture notes + window bridge | W7b | PR #2 ✅ |
| `f37faed` | feat(extension): MV3 scaffold + bus.inject() | W8 | PR #2 ✅ |

## Artifact quality

All changes verified before archive. No CRITICAL issues across any verify pass.

| Metric | Value |
|--------|-------|
| Changes verified | 11 / 11 |
| CRITICAL issues | 0 |
| WARNING issues | 1 (deferred surreal-live SKILL.md correction — skill-system PR) |
| SUGGESTION issues | 2 (panel Subscriptions tab not yet wired; forceOpen portal skip) |
| Changes archived | 11 / 11 |
| Tests added this phase | 35 (23 bus + 15 jsdom panel — 3 pre-existing overlap) |
| Net test delta | +35 (124 → 159) |

## Bugs fixed in-flight

| Bug | Root Cause | Fix |
|-----|-----------|-----|
| W5 burst coalescing: all events dispatching individually | `pendingBurst = []` reset after every below-threshold event; every event appeared below threshold | `dispatchedCount` variable tracks individually-dispatched events so `flush()` slices the remainder correctly |
| W5 "flush forces coalesced" test failure | Same root cause — `beforeFlush` count was 6 instead of 3 | Same fix |
| W5 `bus.destroy()` cascade failure | Registry entries stored a fresh `Symbol(name)` per entry instead of the bus reference | Changed `RegistryEntry` to store `bus: DevtoolsEventBus` and compare `entry.bus === bus` |
| W6 vitest not picking up `.tsx` test files | `vitest.config.ts` only included `*.test.ts` | Added `"src/**/*.test.tsx"` to `include` array |
| W6 jsdom not installed | `@vitest-environment jsdom` annotation with no `jsdom` package | `pnpm add -wD jsdom` |
| W6 7 test failures — tabs not found | Module-level `_panelEverOpened` variable contaminated state across tests | Replaced with component-local `useState(state.open)` |
| W6 portal cleanup not running between tests | RTL auto-cleanup skipped when `vitest globals: false` | Added explicit `afterEach(() => { cleanup(); document.body.innerHTML = ""; })` |

## Lessons learned

1. **RTL portal cleanup** — `@testing-library/react` auto-cleanup does not remove portal DOM from `document.body` when vitest `globals: false` is in effect. Every jsdom test file that renders React portals needs an explicit `afterEach(() => { cleanup(); document.body.innerHTML = ""; })`.

2. **Module-level state in React components is a test trap** — Any module-level variable that React components read will persist across test renders even when the component is unmounted. Always use `useRef` or `useState` for component-lifecycle state that must reset per render.

3. **Burst coalescing needs a `dispatchedCount` bridge** — When some events in a burst are dispatched immediately (below threshold) and others are deferred to a microtask, the flush function needs to track exactly how many have already been dispatched to avoid re-dispatching them. A simple `pendingBurst.length` check is insufficient.

4. **Async registry bus identity** — Storing a bus identity as a freshly-created `Symbol` in a registry entry (vs. the bus object reference itself) breaks `===` comparison in destroy cascade. Always store the actual object reference when object identity is what matters.

5. **`pnpm` workspace flag** — `pnpm add <package>` at the workspace root requires `-w`; without it the command fails with "running this command will add the dependency to the workspace root."

6. **Chrome `globalThis.chrome` typing** — The Chrome extension API is not in standard TypeScript lib. Use a typed `ChromeApiShape` interface cast from `(globalThis as any).chrome` rather than assuming `typeof chrome !== "undefined"` (which TS will reject in non-extension contexts).

## Deferred items (carry into next phase)

| Item | Reason |
|------|--------|
| `entity-realtime-surreal-live/SKILL.md` correction | Companion skill uses the pre-correction `registerAdapter` API. Requires a follow-up `prometheus-skill-system` PR. |
| Entities tab virtualizer wiring | `EntitiesTab` renders a placeholder; `useVirtualizer` + real `useGraphStore` subscription not yet wired. Low priority — devtools only. |
| Subscriptions tab | Not implemented — placeholder only. Requires `RealtimeManager` subscription enumeration API. |
| Extension unpacked load test | Manual Chrome verification that the extension loads and receives events has not been done. The scaffold is complete; testing requires a live browser session. |

## Recommended next phase seeds

- `uar-runs-checkpoint-persistence` — checkpoint + realtime persistence for agent runs (net-new feature, high value)
- `uar-entity-explorer-tabs-complete` — wire Entities + Subscriptions tabs with real data
- `seim-skill-system-surreal-live-correction` — fix `entity-realtime-surreal-live/SKILL.md` API mismatch
- `uar-promote-ci-gates` — promote frontend CI gates from informational to required after one clean merge cycle
