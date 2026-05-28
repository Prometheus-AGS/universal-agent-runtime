# Implementation Tasks — seim-em-surreal-live-adapter-impl

> **Worktree convention applies.** Every code-writing command starts with
> `cd ~/.claude/worktrees/seim-entity-management` (the W2-provisioned
> worktree on branch `feat/seim-entity-management-impl`). See lesson
> `bash-cwd-tracking` in `progress.json.lessons_learned`.

## 0. Pre-flight

- [ ] 0.1 Confirm the worktree exists and is on the expected branch:
  ```sh
  cd ~/.claude/worktrees/seim-entity-management
  test "$(git branch --show-current)" = "feat/seim-entity-management-impl"
  ```
- [ ] 0.2 Confirm the worktree is clean (no leaked KBD sidecar from the W2 recovery):
  ```sh
  cd ~/.claude/worktrees/seim-entity-management
  git status --short   # expect empty
  ```
  If not empty, follow the `bash-cwd-tracking` lesson's recovery: move/remove the stray paths before proceeding.
- [ ] 0.3 Read `progress.json.worktrees_recorded_in` → load `worktrees.json` from the phase dir. Refuse to proceed if the `prometheus-entity-management` entry is absent (per W2 design D7).
- [ ] 0.4 Confirm the corrected spec is at `openspec/specs/entity-surreal-live-adapter/spec.md` in this UAR repo; cache the requirement/scenario list for the test-mapping table in §3.
- [ ] 0.5 Confirm `pnpm install --frozen-lockfile` runs clean from the worktree (re-validates the W2 verification).

## 1. Public surface types

Implement these as the first part of `src/adapters/surreal-live.ts` per design §Implementation Sketch:

- [ ] 1.1 `SurrealLike` (`query`, `live`, `kill`) — minimal client surface (D1)
- [ ] 1.2 `SurrealLiveAction<T>` — discriminated union over `action: CREATE | UPDATE | DELETE | CLOSE | string` + `result: T`
- [ ] 1.3 `SurrealCheckpointStore` — `get(key)` / `set(key, value)` async pair (D5)
- [ ] 1.4 `SurrealTableConfig` — `type: EntityType`, optional `table`, optional `idField`
- [ ] 1.5 `SurrealLiveAdapterOptions` — full surface per proposal §What Changes (D1+D4+D5+D7+D8 inputs)
- [ ] 1.6 Re-export the type-only public surface (`SurrealLike`, `SurrealLiveAdapterOptions`, `SurrealTableConfig`, `SurrealCheckpointStore`) for downstream consumers

## 2. Adapter core (`src/adapters/surreal-live.ts`)

Map 1:1 to design §D2–§D9:

- [ ] 2.1 `class SurrealLiveAdapter implements RealtimeAdapter` — owns `name`, channel `Map`, status `Map`, callback `Set`, current `_status` (D9)
- [ ] 2.2 `subscribe(config, handler) → UnsubscribeFn` — instantiate a `SurrealChannel`, call `channel.start()`, return cleanup function
- [ ] 2.3 `onStatusChange?(cb) → UnsubscribeFn` — Set semantics; return remover closure
- [ ] 2.4 `_recomputeStatus()` — worst-of aggregation across channel states; fires callbacks only on transition (D9)
- [ ] 2.5 `class SurrealChannel` — private; per-channel state machine per D2
- [ ] 2.6 `SurrealChannel.start()` — D3 ordering: open LIVE first, run SELECT, deliver seed `ChangeSet`, flush buffered notifications
- [ ] 2.7 `SurrealChannel._onAction(action)` — D6 normalization + D8 operations filter + handler dispatch
- [ ] 2.8 `SurrealChannel._scheduleReconnect(reason)` — D4 backoff + cancellation; D4b permanent-error detection
- [ ] 2.9 `SurrealChannel.stop()` — set `aborted`, clear timer, `kill` live UUID (defensive try/catch per risk §4)
- [ ] 2.10 `SurrealChannel._replayFromCheckpoint()` — D5 query with bound vars; only when `checkpointStore` is supplied
- [ ] 2.11 `SurrealChannel._setStatus(s)` — updates own state + arms `connectedSettleMs` timer on `connected` (D4c) + calls adapter `_recomputeStatus`
- [ ] 2.12 `_resolveListKeys(changes)` — D7 dedup via `Set`; called for `insert`/`delete` always when resolver present, for `update` only if the resolver opts in (the resolver is the signal)
- [ ] 2.13 `_isPermanent(reason)` — D4b regex list check against `opts.permanentErrorPatterns`
- [ ] 2.14 Export `createSurrealLiveAdapter(opts): RealtimeAdapter` factory (the only public function)

## 3. Test suite (`src/adapters/surreal-live.test.ts`)

Each `describe` block maps to one spec requirement; each `it` block maps to ≥1 spec scenario. Design §D10 provides the `fakeSurreal()` helper.

- [ ] 3.1 `describe("RealtimeAdapter contract")` — covers spec §"Adapter Factory" (4 scenarios)
  - [ ] 3.1.1 Public export resolves and returns a `RealtimeAdapter`
  - [ ] 3.1.2 Default name is `"surreal-live"`; `opts.name` overrides
  - [ ] 3.1.3 Type-only exports present (compile-time check via `vi.expectType`-style assert)
  - [ ] 3.1.4 Manager registration cycle — pass to `RealtimeManager.register`, assert one `subscribe` per `ChannelConfig`
- [ ] 3.2 `describe("per-channel subscription")` — covers spec §"Per-Channel Subscription Model" (4 scenarios)
  - [ ] 3.2.1 Two channels → two independent live UUIDs (assert `fakeSurreal._liveCount() === 2`)
  - [ ] 3.2.2 `filter` clause appears in both SELECT and LIVE SELECT queries
  - [ ] 3.2.3 `id`-scoped channel produces `<table>:<id>` query form
  - [ ] 3.2.4 `UnsubscribeFn` for one channel calls `kill` for that channel only; other channels unaffected
- [ ] 3.3 `describe("initial seed via first handler invocation")` — covers spec §"Initial Seed" (4 scenarios)
  - [ ] 3.3.1 Default seeding emits one ChangeSet of inserts before any live deltas
  - [ ] 3.3.2 Empty seed emits a ChangeSet with empty `changes` array (distinguishable from "not seeded yet")
  - [ ] 3.3.3 `initialQueryStrategy: "live-only"` skips the SELECT entirely
  - [ ] 3.3.4 **Ordering**: emit live actions during seed → assert seed delivered first, then buffered actions in arrival order
- [ ] 3.4 `describe("action payload mapping")` — covers spec §"Action Payload Mapping" (5 scenarios)
  - [ ] 3.4.1 CREATE → insert
  - [ ] 3.4.2 UPDATE → update (full row in `data`, `patch` undefined)
  - [ ] 3.4.3 DELETE → delete (id present; `data` MAY be undefined)
  - [ ] 3.4.4 CLOSE → no emission, channel enters reconnect path
  - [ ] 3.4.5 Unknown action → warn (spy on console) + skip
  - [ ] 3.4.6 `EntityType` derived from `ChannelConfig.type`, not parsed from record id
- [ ] 3.5 `describe("status surface via onStatusChange")` — covers spec §"Status Surface" (3 scenarios)
  - [ ] 3.5.1 Callback registration + unsubscribe
  - [ ] 3.5.2 All four `AdapterStatus` values observable across a happy + reconnect path
  - [ ] 3.5.3 Worst-of aggregation: 2 channels, one errors → adapter reports `error`
- [ ] 3.6 `describe("reconnection with backoff")` — covers spec §"Reconnection" (4 scenarios); uses `vi.useFakeTimers()`
  - [ ] 3.6.1 Backoff schedule assert (1st, 2nd, 3rd retry intervals within jitter range)
  - [ ] 3.6.2 Reconnect re-issues the same LIVE SELECT
  - [ ] 3.6.3 Attempt counter resets after `connectedSettleMs` connected duration
  - [ ] 3.6.4 Permanent error (regex match) → `error` status, no reconnect attempted
- [ ] 3.7 `describe("checkpoint replay on reconnect")` — covers spec §"Checkpoint Replay" (3 scenarios)
  - [ ] 3.7.1 With `checkpointStore`: replay query runs before new LIVE SELECT; missed rows delivered as a ChangeSet
  - [ ] 3.7.2 Without `checkpointStore`: no replay query; resume from reconnect moment
  - [ ] 3.7.3 Per-channel keying: two channels with same `type` but different `filter` produce distinct checkpoint keys
- [ ] 3.8 `describe("list refresh hints")` — covers spec §"List Refresh Hints" (2 scenarios)
  - [ ] 3.8.1 `listKeyResolver` returning array → populates `affectedListKeys` deduplicated
  - [ ] 3.8.2 Resolver absent → `affectedListKeys` undefined for updates
- [ ] 3.9 `describe("ChannelConfig.operations filter")` — covers D8 (2 scenarios)
  - [ ] 3.9.1 `operations: ["insert", "delete"]` → updates dropped
  - [ ] 3.9.2 `operations` absent → all four ops emit normally
- [ ] 3.10 `describe("companion-skill alignment metadata")` — covers spec §"Companion Skill" indirectly
  - [ ] 3.10.1 Note: skill alignment is verified at OpenSpec verify time, not in vitest. This describe block is a placeholder no-op `it.todo()` to keep the requirement → describe mapping symmetric
- [ ] 3.11 `describe("test infrastructure asserts")` — covers spec §"Test Coverage" (2 scenarios)
  - [ ] 3.11.1 Fake `Surreal` used — `expect(fakeSurreal.live).toHaveBeenCalled()`
  - [ ] 3.11.2 No leaked subscriptions — after `UnsubscribeFn` for every channel, `fakeSurreal._liveCount() === 0` and `fakeSurreal.kill` was called once per channel

## 4. Re-export from `src/index.ts`

- [ ] 4.1 Locate the existing "Local-first (ElectricSQL + PGlite)" export block in `src/index.ts`
- [ ] 4.2 Add a new "Surreal live (realtime)" export block immediately below:
  ```ts
  // ── Surreal live (realtime) ───────────────────────────────────────────────
  export { createSurrealLiveAdapter } from "./adapters/surreal-live";
  export type {
    SurrealLike,
    SurrealLiveAction,
    SurrealCheckpointStore,
    SurrealTableConfig,
    SurrealLiveAdapterOptions,
  } from "./adapters/surreal-live";
  ```
- [ ] 4.3 Confirm import order stays alphabetical within sibling blocks if a convention is in force; otherwise place after ElectricSQL block (the natural realtime-adapter neighbor)

## 5. Build + test verification

- [ ] 5.1 `cd ~/.claude/worktrees/seim-entity-management && pnpm typecheck` — green
- [ ] 5.2 `pnpm test src/adapters/surreal-live.test.ts` — every `describe` block passes (≥33 assertions)
- [ ] 5.3 `pnpm test` (full suite) — no regressions in existing tests (electricsql, electricsql-tenant, pglite-persistence, realtime-manager, etc.)
- [ ] 5.4 `pnpm build` — `tsup` produces dist artifacts including `createSurrealLiveAdapter` symbol; check via `grep -l createSurrealLiveAdapter dist/*.js`
- [ ] 5.5 Manual scan of dist output for unexpected dependencies (e.g. accidental `surrealdb` import — shouldn't appear since we only `import type`)

## 6. Worktree commit

- [ ] 6.1 `cd ~/.claude/worktrees/seim-entity-management`
- [ ] 6.2 `git status --short` → confirm exactly 3 paths: 2 new under `src/adapters/`, 1 modified `src/index.ts`. **If a `.kbd-orchestrator/` directory is dirty, recover per the lesson** before staging.
- [ ] 6.3 `git add src/adapters/surreal-live.ts src/adapters/surreal-live.test.ts src/index.ts`
- [ ] 6.4 `git commit -s -m` with body:
  ```
  feat(adapters): SurrealDB LIVE SELECT adapter (createSurrealLiveAdapter)

  Implements entity-surreal-live-adapter capability per the corrected
  spec at universal-agent-runtime/openspec/specs/entity-surreal-live-adapter/
  spec.md. RealtimeAdapter contract; per-channel subscriptions; seed-then-live
  ordering with buffer; reconnect with exponential backoff + jitter; optional
  checkpoint replay; aggregated onStatusChange stream; ChannelConfig.operations
  filter honored.

  See universal-agent-runtime/openspec/changes/archive/2026-MM-DD-seim-em-surreal-live-adapter-impl/
  for the full proposal / design / tasks once archived.
  ```
- [ ] 6.5 Don't push yet — wait until W3's other change (`seim-em-engine-devtools-tap`) also lands; then push both as a single branch update

## 7. Verification cross-checks

- [ ] 7.1 From the worktree, confirm the file count is exactly right:
  ```sh
  cd ~/.claude/worktrees/seim-entity-management
  git diff --cached --name-only HEAD~1 HEAD 2>/dev/null | wc -l   # expect 3
  ```
- [ ] 7.2 Confirm `dist/` is gitignored (`tsup` artifact, not committed)
- [ ] 7.3 Confirm no `.kbd-orchestrator/` paths landed in the commit (cross-worktree leakage guard)

## 8. Closeout

- [ ] 8.1 Update `.kbd-orchestrator/phases/submodule-entity-management-implementation/progress.json`:
  - [ ] 8.1.1 `changes_completed: 4`, append `seim-em-surreal-live-adapter-impl` to `completed_changes`
  - [ ] 8.1.2 `active_change` → the OTHER W3 change (`seim-em-engine-devtools-tap`) if not yet started, else W4's `seim-em-explorer-preflight-research`. State `ready_for_opsx_new` or `in_progress`.
- [ ] 8.2 `/opsx:verify seim-em-surreal-live-adapter-impl` — required (≥3 files, not doc-only)
- [ ] 8.3 `/opsx:archive seim-em-surreal-live-adapter-impl` — no spec promotion (capability spec already promoted in W1)
- [ ] 8.4 Refresh `current-waypoint.json` and emit closing Progress Signal

## Rollback (only if §5 surfaces critical failures)

- [ ] R1 In the worktree: `git reset --hard HEAD~1` (commit not pushed yet — safe)
- [ ] R2 Re-run pre-flight (§0)
- [ ] R3 Fix the failing scenario; loop back to §2 / §3
- [ ] R4 If the failure surfaces a spec ambiguity rather than an impl bug: pause, open a follow-up spec-correction change, do NOT silently amend the spec from inside this change
