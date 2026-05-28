## Why

The promoted spec at `openspec/specs/entity-surreal-live-adapter/spec.md` (landed at the end of the prior phase from change 9) declares:

> `createSurrealLiveAdapter(opts): SyncAdapter` … registered via `realtime-manager.registerAdapter`.

with lifecycle implied as a top-level `start(handler) / stop()`. The actual contract in `prometheus-entity-management/src/adapters/types.ts` is:

```ts
export interface RealtimeAdapter {
  readonly name: string;
  subscribe(config: SubscriptionConfig, handler: (changeset: ChangeSet) => void): UnsubscribeFn;
  onStatusChange?: (cb: (status: AdapterStatus) => void) => UnsubscribeFn;
}

export interface SyncAdapter extends RealtimeAdapter {
  query<T>(sql, params?): Promise<SyncQueryResult<T>>;
  execute(sql, params?): Promise<void>;
  isSynced(): boolean;
  onSyncComplete(cb: () => void): UnsubscribeFn;
}
```

with the manager API being `RealtimeManager.register(adapter, channels[], normalize?)` — one `subscribe(...)` call per `ChannelConfig`, not a global `start(handler)`. There is **no** `start`/`stop` pair on either interface; lifecycle is per-subscription via the returned `UnsubscribeFn`.

Implementing change 4 (`seim-em-surreal-live-adapter-impl`) against the spec as written would either silently diverge from the actual codebase or fail TypeScript. Both are bad outcomes. The correction is small, well-scoped, and produces a spec that an implementor can follow without surprises.

This is the W1 change of the current phase, sequenced ahead of change 4 deliberately so the implementor of the adapter reads a correct spec.

## What Changes

### Rewrite `openspec/specs/entity-surreal-live-adapter/spec.md`

Sections to rewrite:

- **Purpose**: replace "implements the existing `SyncAdapter` contract" with "implements the `RealtimeAdapter` contract (the lifecycle interface) and may optionally extend it to `SyncAdapter` only if the adapter needs to expose query/execute". Default is `RealtimeAdapter` since SurrealDB live queries don't need the `SyncAdapter` extras (the SurrealDB driver itself is the query surface, not the adapter).
- **Requirement: Adapter Factory** → returns `RealtimeAdapter`, not `SyncAdapter`. Re-export unchanged. Registration via `RealtimeManager.register(adapter, channels[], normalize?)`.
- **Initial Seed** → reframe as "first subscription handler invocation". Instead of "before opening the live subscription", "as the first `ChangeSet` emitted through the subscription handler".
- **Per-channel subscription** *(new requirement)*: each `ChannelConfig` becomes one independent SurrealDB `LIVE SELECT`. The adapter's `subscribe(config, handler)` is the per-channel hook; the manager calls it once per declared channel.
- **Status events**: replace `onStatus(cb): UnsubscribeFn` with optional `onStatusChange?(cb): UnsubscribeFn` matching the `RealtimeAdapter` definition. Status enum unchanged: `connecting | connected | disconnected | error`.
- **Sync completion**: no `onSynced(cb)` requirement — that was a `SyncAdapter`-ism. Drop the requirement entirely. (If a consumer needs sync-complete signal, they listen for the first non-empty `ChangeSet` on each channel — that's the documented contract.)
- **Reconnect**: unchanged in spirit. Per-channel `UnsubscribeFn` is the cleanup path; reconnect strategy (exponential backoff) lives inside the adapter and is invisible to the manager.
- **Replay / checkpoint**: keep, but rephrase so the checkpoint persistence is per-channel-key rather than global.
- **affectedListKeys**: keep — `ChangeSet.affectedListKeys` is part of the actual type contract.
- **Reconciliation preamble**: add a top-of-file note naming the supersession:

  ```markdown
  > **Note**: This spec replaces an earlier draft that targeted a
  > `SyncAdapter.start/stop` lifecycle. The corrected spec below matches
  > the actual `RealtimeAdapter.subscribe(config, handler) → UnsubscribeFn`
  > contract in `prometheus-entity-management/src/adapters/types.ts`. The
  > superseded text is preserved in the archived change at
  > `openspec/changes/archive/2026-05-27-ssed-entity-surreal-live-adapter/`.
  ```

### What stays the same

- **Capability ID**: `entity-surreal-live-adapter` (no rename — the capability is the same, just specified correctly).
- **Companion skill**: `entity-realtime-surreal-live/SKILL.md` in `prometheus-skill-system` (already shipped via the change 1 PR merge). Its instructions already match the corrected shape; verifying that against the spec is part of this correction's verification.
- **Intent**: A SurrealDB-backed realtime source for the entity graph.

### What this change does NOT include

- **No TypeScript implementation.** That's change 4 (`seim-em-surreal-live-adapter-impl`).
- **No edit to the archived prior spec.** History is preserved; the archived change is read-only.
- **No rename of `entity-realtime-surreal-live` skill.** It's already on origin per the change 1 merge.

## Capabilities

### New Capabilities

- None. Same capability ID; only the spec text is corrected.

### Modified Capabilities

- `entity-surreal-live-adapter`: Spec rewritten to match the actual `RealtimeAdapter.subscribe(config, handler) → UnsubscribeFn` contract in `prometheus-entity-management/src/adapters/types.ts`. Per-channel subscription model replaces the `start(handler)/stop()` lifecycle. `onStatusChange?` replaces `onStatus`. `onSynced` requirement dropped (no analog on the actual interface). Other requirements (seed, action mapping, reconnect, replay, `affectedListKeys`) preserved in spirit.

## Impact

- **Risk**: Low. Documentation-only change. The corrected spec describes a *cleaner* interface (the cleanup-via-`UnsubscribeFn` model is simpler than `start/stop`), so the downstream implementation is also smaller.
- **Affected files** (UAR repo only):
  - `openspec/specs/entity-surreal-live-adapter/spec.md` — rewritten in place
  - This change's own `openspec/changes/seim-surreal-live-spec-correction/{proposal,design,tasks,specs}` — the OpenSpec record itself
- **Cross-repo**: No.
- **Reversibility**: Trivial — `git revert` of the rewrite commit.
- **Unblocks**: Change 4 (`seim-em-surreal-live-adapter-impl`) can proceed against a correct contract. Without this correction, change 4 implements against a non-existent interface.

### Sequencing note

This change has zero new code and one modified capability. Same artifact-sequence variance as W0 — capability count == spec count (1 == 1), so the spec-driven flow runs proposal → spec → design → tasks. The single spec file IS the rewritten `entity-surreal-live-adapter/spec.md`, drafted in `openspec/changes/seim-surreal-live-spec-correction/specs/entity-surreal-live-adapter/spec.md` per OpenSpec convention; on archive it overwrites the promoted location at `openspec/specs/entity-surreal-live-adapter/spec.md`.
