## Why

`openspec/specs/entity-surreal-live-adapter/spec.md` was corrected in W1 to match the actual `RealtimeAdapter` contract in `prometheus-entity-management/src/adapters/types.ts`. The spec is now implementable verbatim — but no implementation exists yet. Until it lands, the capability is "documented but not delivered", and any consumer attempting to use `import { createSurrealLiveAdapter } from "@prometheus-ags/prometheus-entity-management"` gets an undefined symbol.

This change ships the TypeScript implementation, satisfying every requirement and scenario in the corrected spec. It is the first code commit on the W2-provisioned worktree (`~/.claude/worktrees/seim-entity-management` @ `feat/seim-entity-management-impl`) and validates the worktree convention end-to-end.

It is the **first TS change in this phase**. The other parallel W3 change (`seim-em-engine-devtools-tap`) edits a different file (`src/engine.ts`) and can land independently; merge order doesn't matter.

## What Changes

### New files in the worktree

- **`src/adapters/surreal-live.ts`** — the implementation. Exports `createSurrealLiveAdapter(opts): RealtimeAdapter` plus types `SurrealLiveAdapterOptions`, `SurrealTableConfig`, `SurrealCheckpointStore`. Per-channel subscription, seed-then-live with buffer, action mapping, reconnect with backoff + cancellation, optional checkpoint replay, single aggregated `onStatusChange` stream, `affectedListKeys` via `listKeyResolver`.
- **`src/adapters/surreal-live.test.ts`** — vitest suite. Hand-rolled fake `Surreal` client (no real network), mirroring the `fakeShapeStream` / `fakePGlite` pattern from `electricsql-tenant.test.ts`. Covers every spec scenario systematically.

### Modified files

- **`src/index.ts`** — add a new "Surreal live (realtime)" export section alongside the existing "Local-first (ElectricSQL + PGlite)" block, re-exporting the factory + type-only exports.

### Implementation contract (binding to the corrected spec)

The implementation MUST satisfy every requirement in `openspec/specs/entity-surreal-live-adapter/spec.md`. The highlights, with the eight design questions stubbed in §Decisions to be resolved in `design.md`:

1. **Factory returns `RealtimeAdapter`**, not `SyncAdapter`. No `query` / `execute` / `isSynced` / `onSyncComplete`.
2. **Per-channel `subscribe(config, handler) → UnsubscribeFn`**. Each call opens one SurrealDB `LIVE SELECT`.
3. **Seed-then-live ordering**: a `SELECT * FROM <table>[ WHERE …]` runs before live notifications reach the handler; live notifications that arrive during the seed are buffered (per-channel) and flushed after the seed `ChangeSet`.
4. **Action mapping**: `CREATE → insert`, `UPDATE → update`, `DELETE → delete`, `CLOSE → reconnect`, unknown → warn + skip.
5. **`onStatusChange` aggregation**: per-channel state machines aggregate into a single status stream via "worst-of" (`error > disconnected > connecting > connected`).
6. **Reconnect**: exponential backoff with ±25% jitter, `initialDelayMs` default 500, `maxDelayMs` default 30000, attempt-counter reset after `connectedSettleMs` (default 30000) connected duration. Permanent errors (auth, schema) emit `error` and stop reconnecting.
7. **Checkpoint replay** (opt-in): when `opts.checkpointStore` is supplied, persist a per-channel checkpoint on each delivered change, replay via `SELECT … WHERE updated_at > <stored>` on reconnect.
8. **`affectedListKeys`** via `opts.listKeyResolver?(change): string[] | undefined`. Default `undefined` for updates.
9. **`ChannelConfig.operations` filter** — flagged as a W1 risk; this change resolves: **honor the filter** by dropping notifications whose mapped op isn't in `channel.operations` BEFORE invoking the handler. Default (filter absent) emits all four ops.

### Verification

- `pnpm test src/adapters/surreal-live.test.ts` green.
- `pnpm typecheck` green.
- `pnpm build` green.
- All 10 requirements / 33 scenarios in the spec have at least one test asserting their behavior.
- `entity-realtime-surreal-live` skill on disk still shows the **drift** noted in W1 §2.4 (it describes `registerAdapter` instead of `manager.register`). That drift is NOT fixed here — captured separately as the pending `seim-skill-followup-surreal-live-skill-correction` follow-up.

### What this change does NOT include

- **No companion-skill edit.** Deferred.
- **No new capability declaration.** The capability `entity-surreal-live-adapter` already exists in `openspec/specs/`; this change ships its implementation, not its spec.
- **No `SyncAdapter` extension.** Per spec D1.
- **No reactive React hooks** (e.g. `useSurrealLiveAdapter`). Out of scope — the adapter is consumed by passing it to `RealtimeManager.register(adapter, channels[], normalize?)`, which is how every other adapter is wired today.
- **No surrealdb.js peer dependency.** The adapter takes a `SurrealLike` interface (the minimal surface the adapter needs) so it works with `surrealdb` 1.x and 2.x without locking the consumer.

## Capabilities

### New Capabilities

- None. The capability `entity-surreal-live-adapter` was declared in the prior phase (change 9) and corrected in this phase's W1.

### Modified Capabilities

- `entity-surreal-live-adapter`: TypeScript implementation now ships matching the corrected spec. No spec edits; behavioural delivery only. The `openspec/specs/entity-surreal-live-adapter/spec.md` file is unchanged by this change.

## Impact

- **Risk**: Medium. The implementation surface is moderate (~250–350 LOC) and the per-channel state machine is non-trivial; bugs are easier to introduce than in pure doc changes. Mitigated by direct conformance to the corrected spec and a comprehensive vitest suite that walks every scenario.
- **Affected files**:
  - **`prometheus-entity-management`** (via worktree): `src/adapters/surreal-live.ts` (new), `src/adapters/surreal-live.test.ts` (new), `src/index.ts` (modified — single export block added).
  - **This UAR repo**: only `progress.json` + `current-waypoint.json` updates at closeout; no openspec/specs/ edits (the spec is already promoted).
- **Cross-repo**: Yes — the implementation lands in `prometheus-entity-management`. No commit to `prometheus-skill-system`.
- **Reversibility**: `git revert` of the three-file commit in the entity-mgmt worktree.
- **Unblocks**: change 10 (`seim-em-explorer-fab-panel`) — the panel's "Stores" tab can showcase a SurrealDB-backed entity graph as a demo target. Change 9 (`seim-em-explorer-production-treeshake-check`) — once the adapter is real code in the bundle, the tree-shake gate has a concrete subject to verify against.
- **Worktree discipline carry-forward**: per W2's `bash-cwd-tracking` lesson logged in `progress.json.lessons_learned`, every Bash invocation during `/opsx:apply` MUST start with an explicit `cd <abs-path>`. The lesson recurs to the agent via memory-recall on `assess:before`; this change adopts it preemptively.

### Sequencing note

Same artifact-sequence variance as W0/W1/W2: zero NEW capabilities, one MODIFIED capability whose spec was promoted in W1. No new `specs/<capability>/spec.md` file under this change directory — the binding spec is the already-promoted `openspec/specs/entity-surreal-live-adapter/spec.md`. The artifact sequence collapses to `proposal → design → tasks`.
