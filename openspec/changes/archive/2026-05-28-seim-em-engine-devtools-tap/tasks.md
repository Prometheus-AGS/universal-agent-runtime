# Implementation Tasks — seim-em-engine-devtools-tap

> **Worktree convention applies.** Every Bash invocation starts with
> `cd ~/.claude/worktrees/seim-entity-management`. UAR-side writes use
> absolute paths (UAR=…; ...). Carrying both lessons from
> `progress.json.lessons_learned[]`.

## 0. Pre-flight

- [ ] 0.1 `cd ~/.claude/worktrees/seim-entity-management`; confirm on `feat/seim-entity-management-impl`:
  ```sh
  test "$(git branch --show-current)" = "feat/seim-entity-management-impl"
  ```
- [ ] 0.2 Confirm previous W3 sibling commit (`9f314ba`) is present and the only unpushed commit:
  ```sh
  git log --oneline origin/main..HEAD
  ```
  Expect: one line for `9f314ba feat(adapters): SurrealDB LIVE SELECT adapter (createSurrealLiveAdapter)`.
- [ ] 0.3 Confirm worktree is clean (no uncommitted edits leaking from prior work):
  ```sh
  git status --short
  ```
  Expect: empty.
- [ ] 0.4 Read `graph-actions.ts` end-to-end. Pay attention to:
  - Exact export name (factory function or named export?)
  - Whether ops return synchronously or via a transaction promise
  - Whether a "commit" hook exists (per design §Risks #5)
- [ ] 0.5 Read `engine.ts` to confirm the insertion point next to `subscribeSubscriberStats` is the right neighborhood for the new exports.
- [ ] 0.6 Read `src/index.ts` to locate the existing devtools / subscriber export block; this is where the new exports slot in.
- [ ] 0.7 If §0.4 reveals a transaction-rollback hook, **pause** and update design.md §Risks #5 → resolution before continuing.

## 1. `src/engine.ts` — pub/sub primitive

- [ ] 1.1 Append the section per design §Implementation Sketch → "`src/engine.ts` (additions)":
  - [ ] 1.1.1 `import type { EntityType, EntityId } from "./graph";` (if not already imported)
  - [ ] 1.1.2 `DevtoolsEvent` discriminated union (5 kinds, including `unpatch` + `list` as forward-compat)
  - [ ] 1.1.3 Module-scope `devtoolsSubscribers = new Set<DevtoolsListener>()`
  - [ ] 1.1.4 `export function subscribeDevtoolsEvent(cb): () => void` — idempotent add, returns `UnsubscribeFn`
  - [ ] 1.1.5 `export function notifyDevtools(event): void` — early-return on empty (D4), snapshot iteration (D2), try/catch per subscriber (D3), `console.warn` on error
- [ ] 1.2 Order check: the early-return is the LITERAL FIRST STATEMENT of `notifyDevtools` (spec mandate)
- [ ] 1.3 `pnpm typecheck` green

## 2. `src/graph-actions.ts` — call-site instrumentation

- [ ] 2.1 Add `import { notifyDevtools } from "./engine";` near the top
- [ ] 2.2 Inside the factory / object literal that creates the actions:
  - [ ] 2.2.1 `upsertEntity`: after `useGraphStore.getState().upsertEntity(...)`, wrap a `notifyDevtools({ kind: "upsert", type, id, data, at: new Date().toISOString() })` in `if (process.env.NODE_ENV !== "production")`
  - [ ] 2.2.2 `patchEntity`: parallel addition with `kind: "patch"` and `patch:` field
  - [ ] 2.2.3 `clearPatch`: parallel addition with `kind: "clearPatch"`, no `data` / `patch` field
- [ ] 2.3 If a transaction-commit hook was discovered in §0.4 (§Risks #5), insert the notify at the commit point instead of right after the store call; document the choice in this tasks.md before committing
- [ ] 2.4 `pnpm typecheck` green

## 3. `src/index.ts` — re-export

- [ ] 3.1 Locate the existing devtools / subscriber-stats export region (around the `subscribeSubscriberStats` re-export, ~line 200)
- [ ] 3.2 Add:
  ```ts
  export { subscribeDevtoolsEvent } from "./engine";
  export type { DevtoolsEvent } from "./engine";
  ```
- [ ] 3.3 Do NOT re-export `notifyDevtools` (design D9 — internal-only)
- [ ] 3.4 `pnpm typecheck` green

## 4. `src/engine-devtools-tap.test.ts` — test suite

- [ ] 4.1 Create the new test file at `src/engine-devtools-tap.test.ts`
- [ ] 4.2 Imports: `describe`, `it`, `expect`, `vi`, `beforeEach` from vitest; `subscribeDevtoolsEvent`, `DevtoolsEvent` from `./engine`; the entity-actions factory from `./graph-actions`; the store from `./graph` (for resetting state between tests)
- [ ] 4.3 Setup: `beforeEach` resets the graph store so each `it` starts with a clean entity set
- [ ] 4.4 `describe("public observation surface")` — 4 `it`s covering spec §"Public Observation Surface" scenarios
  - [ ] 4.4.1 Public export resolves and returns an `UnsubscribeFn`
  - [ ] 4.4.2 Type re-export resolves (`type _Smoke = DevtoolsEvent;`)
  - [ ] 4.4.3 Multiple subscribers all receive the event
  - [ ] 4.4.4 Unsubscribe stops delivery; other subscribers continue
- [ ] 4.5 `describe("event payload shape")` — 4 `it`s
  - [ ] 4.5.1 upsert payload exact field set + ISO-8601 `at`
  - [ ] 4.5.2 patch payload exact field set
  - [ ] 4.5.3 clearPatch payload exact field set (no `data` / `patch`)
  - [ ] 4.5.4 `DevtoolsEvent` type union includes forward-compat `unpatch` + `list` kinds (compile-time `type` assertion via `satisfies`)
- [ ] 4.6 `describe("op-site instrumentation")` — 4 `it`s
  - [ ] 4.6.1 `upsertEntity` triggers exactly one `kind: "upsert"` event
  - [ ] 4.6.2 `patchEntity` triggers exactly one `kind: "patch"` event
  - [ ] 4.6.3 `clearPatch` triggers exactly one `kind: "clearPatch"` event
  - [ ] 4.6.4 Sequential `upsert → patch → clearPatch` produces three events in that order
- [ ] 4.7 `describe("hot-path no-op")` — 2 `it`s
  - [ ] 4.7.1 With zero subscribers, calling a graph-actions op does not throw and does not allocate (spy on `console.warn` to confirm no listener errors)
  - [ ] 4.7.2 Indirect verification: subscriber added after several ops doesn't receive backfill (events are fire-and-forget)
- [ ] 4.8 `describe("production tree-shake gate")` — 1 `it.todo()` placeholder noting that W7 (`seim-em-explorer-production-treeshake-check`) verifies elision in prod bundles
- [ ] 4.9 `describe("subscriber lifecycle")` — 3 `it`s
  - [ ] 4.9.1 Same function subscribed twice → received once per event
  - [ ] 4.9.2 `UnsubscribeFn` called twice → no error, no double-removal effect
  - [ ] 4.9.3 Subscribe / unsubscribe / re-subscribe cycle works
- [ ] 4.10 `describe("re-entrancy safety")` — 2 `it`s
  - [ ] 4.10.1 Subscriber that calls `subscribeDevtoolsEvent` during dispatch: new subscriber receives FUTURE events, not the in-flight one
  - [ ] 4.10.2 Subscriber that throws → other subscribers still receive event, `console.warn` invoked
- [ ] 4.11 At least 12 total assertions across all `it` blocks (spec §"Test Coverage" requirement; expected actual: ~19 across 6 describes excluding the todo)

## 5. Build + test verification

- [ ] 5.1 `pnpm typecheck` green
- [ ] 5.2 `pnpm test src/engine-devtools-tap.test.ts` green (target ≥12 assertions, expected ~19)
- [ ] 5.3 `pnpm test` full suite — confirms no regression to existing 104 tests; new total should be 104 + new test count
- [ ] 5.4 `pnpm build` green
- [ ] 5.5 Confirm `subscribeDevtoolsEvent` appears in `dist/index.js` (public API):
  ```sh
  grep -l subscribeDevtoolsEvent dist/*.js
  ```
- [ ] 5.6 Confirm the `notifyDevtools` call-site literals are present in **dev** dist (we don't gate the bundle output for production builds in this change — that's W7's job to verify under NODE_ENV=production):
  ```sh
  grep -c 'kind:"upsert"\|kind: "upsert"' dist/index.js
  ```
  Expect ≥ 1 since `tsup` likely builds with NODE_ENV unset → guard is true → literal preserved.

## 6. Worktree commit

- [ ] 6.1 `cd ~/.claude/worktrees/seim-entity-management`
- [ ] 6.2 `git status --short` — expect exactly 4 paths:
  - `M  src/engine.ts`
  - `M  src/graph-actions.ts`
  - `M  src/index.ts`
  - `?? src/engine-devtools-tap.test.ts`
- [ ] 6.3 `git add src/engine.ts src/graph-actions.ts src/index.ts src/engine-devtools-tap.test.ts`
- [ ] 6.4 Commit with sign-off and detailed body:
  ```
  feat(engine): devtools push stream + per-op tap

  Adds subscribeDevtoolsEvent / notifyDevtools to src/engine.ts and
  instruments the three mutating graph-actions ops (upsertEntity,
  patchEntity, clearPatch) to fire DevtoolsEvent on every mutation.

  - Hot-path no-op when no subscribers registered
  - Snapshot iteration for re-entrancy safety
  - Throwing subscriber doesn't block delivery to others (logged via
    console.warn)
  - DevtoolsEvent union forward-compatible with future "unpatch" + "list"
    kinds (W5+ event bus consumes these without further type changes)
  - Op-site calls wrapped in process.env.NODE_ENV !== "production" so
    esbuild DCE elides them from prod bundles (W7 tree-shake gate
    asserts this)

  Implements capability entity-engine-devtools-tap per
  universal-agent-runtime/openspec/changes/seim-em-engine-devtools-tap/
  specs/entity-engine-devtools-tap/spec.md.

  Tests: NEW assertions in src/engine-devtools-tap.test.ts; full suite
  green.
  ```
- [ ] 6.5 Capture the new commit's SHA

## 7. Push both W3 commits

Now that both W3 changes are committed:

- [ ] 7.1 `git push -u origin feat/seim-entity-management-impl` — first push for this branch, so `-u` for tracking
- [ ] 7.2 Capture the push output's branch URL
- [ ] 7.3 Open a PR via `gh pr create` against `prometheus-entity-management`:
  - **Title**: `feat(entity): SurrealDB live adapter + engine devtools tap (W3)`
  - **Body**: enumerate both commits + cross-reference the archived OpenSpec changes in UAR
- [ ] 7.4 Capture PR URL + number

## 8. Verification cross-checks

- [ ] 8.1 The W3 PR should show exactly 2 commits + the right file delta (typically `src/adapters/surreal-live.{ts,test.ts}` + `src/engine.ts` + `src/graph-actions.ts` + `src/engine-devtools-tap.test.ts` + `src/index.ts` modifications)
- [ ] 8.2 No `.kbd-orchestrator/` paths in the PR (cross-worktree leakage guard)
- [ ] 8.3 PR CI green (if entity-mgmt has CI configured)

## 9. Closeout

- [ ] 9.1 Update `.kbd-orchestrator/phases/submodule-entity-management-implementation/progress.json`:
  - [ ] 9.1.1 `changes_completed: 5`, append `seim-em-engine-devtools-tap` to `completed_changes`
  - [ ] 9.1.2 Add this change's commit to `entity_mgmt_worktree_local_commits[]` (or move both W3 commits to `completed_cross_repo_commits[]` after merge)
  - [ ] 9.1.3 Set `active_change: "seim-em-explorer-preflight-research"`, state `ready_for_opsx_new` (W4 — the UI/UX routing pre-flight)
- [ ] 9.2 `/opsx:verify seim-em-engine-devtools-tap` — required (4 files modified, test suite present)
- [ ] 9.3 `/opsx:archive seim-em-engine-devtools-tap` — promotes `entity-engine-devtools-tap` spec into `openspec/specs/`
- [ ] 9.4 Refresh waypoint to W4 (`seim-em-explorer-preflight-research`)
- [ ] 9.5 W3 complete → W4 can start. W4 is gating for W5/W6 (UI/UX preflight research required before any panel code per the routing discipline in CLAUDE.md/AGENTS.md)

## Rollback (only if §5 surfaces failures)

- [ ] R1 In the worktree: `git reset --hard 9f314ba` (back to the W3 sibling commit; surreal-live work preserved)
- [ ] R2 Investigate failing test, fix, re-run from §2
- [ ] R3 If failure surfaces a spec ambiguity, pause and update the spec before continuing — do NOT silently amend the spec from inside this change
- [ ] R4 If the failure is in §0.4 (graph-actions.ts shape mismatch) — pause, read the file, update design.md §D7 with the actual export shape, then resume
