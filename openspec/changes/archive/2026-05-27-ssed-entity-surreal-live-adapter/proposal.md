## Why

The user's phase request: "update the skills in the prometheus entity manager to include specific variations for the surrealdb live query mechanisms to support real time updating of the entity graph."

Today `prometheus-entity-management` ships realtime adapters for ElectricSQL + PGlite (`src/adapters/electricsql.ts`, `electricsql-tenant.ts`) — but no SurrealDB live-query adapter. The `SyncAdapter` contract in `src/adapters/types.ts` is general; what's missing is a concrete adapter that wires SurrealDB's `LIVE SELECT` statements into the same `ChangeSet → graph` pipeline. The companion skill set (`prometheus-entity-skills`) has `entity-realtime-channel`, `entity-realtime-local-first`, `entity-realtime-setup` — but none of them teach the agent how to wire `LIVE SELECT`.

This change lands two things in lockstep:

1. **`prometheus-entity-management`** — a `SurrealLiveAdapter` (`src/adapters/surreal-live.ts`) implementing the `SyncAdapter` contract, plus a `realtime-manager` integration so `useEntity` and `useEntityList` consumers get realtime updates without per-component wiring.
2. **`prometheus-entity-skills`** — a new `entity-realtime-surreal-live` skill (mirroring `entity-realtime-channel`'s shape) that teaches the agent to register `SurrealLiveAdapter` against the engine.

## What Changes

### `prometheus-entity-management/src/adapters/surreal-live.ts` (new)

Factory function `createSurrealLiveAdapter(opts)` returning a `SyncAdapter`. Inputs:

```ts
export interface SurrealTableConfig<T extends object> {
  type: EntityType;
  table: string;                  // e.g. "client"
  where?: string;                 // optional WHERE clause appended to LIVE SELECT
  idColumn?: string;              // default "id"
  normalize?: (row: T) => Record<string, unknown>;
}

export interface SurrealLiveAdapterOptions {
  db: Surreal;                    // surrealdb.js / surrealdb.wasm client instance
  tables: SurrealTableConfig<…>[];
  onSynced?: () => void;
  initialQueryStrategy?: "select-then-live" | "live-only";
  checkpointResume?: { columnName: string; loadCheckpoint: () => Promise<string | null>; saveCheckpoint: (offset: string) => Promise<void>; };
}
```

The adapter:

- For each `SurrealTableConfig`, executes `LIVE SELECT * FROM <table>[ WHERE <where>]`. SurrealDB returns a live-query UUID; subsequent CREATE/UPDATE/DELETE notifications stream through the client's `.live()` callback.
- Default `initialQueryStrategy: "select-then-live"` — performs one bulk `SELECT` to seed state before opening the live query, so the graph is populated immediately. `"live-only"` skips the seed (for cases where another path will hydrate).
- Maps SurrealDB action payloads to `EntityChange`:
  - `CREATE { result: row }` → `{ op: "insert", type, id, data }`
  - `UPDATE { result: row }` → `{ op: "upsert", type, id, data }` (SurrealDB doesn't distinguish from full updates — upsert is safe)
  - `DELETE { result: { id } }` → `{ op: "delete", type, id }`
- Calls the supplied `normalize` per row before assembling the `ChangeSet`.
- Emits `onSynced` once the initial seed completes for all configured tables.
- Reconnection: on websocket close, the adapter awaits a 1s/3s/9s exponential-backoff reconnect; on reconnect, re-issues each `LIVE SELECT` and (if `checkpointResume` is configured) replays missed changes via `SELECT … WHERE updated_at > <checkpoint>` before re-attaching the live stream.

### `realtime-manager` integration

No new code path required — `getRealtimeManager()` already accepts any `SyncAdapter`. The new adapter slots in via `registerAdapter("surreal-live", createSurrealLiveAdapter({…}))` at app startup.

### `prometheus-entity-management/src/adapters/surreal-live.test.ts` (new)

vitest suite covering:

- Initial `SELECT` seed (`select-then-live` mode) writes correct `insert` `ChangeSet`.
- Subsequent CREATE/UPDATE/DELETE actions map to the right `EntityChange` shape.
- `normalize` applied to every row.
- Reconnection: simulated WS close → exponential backoff → `LIVE SELECT` re-issued.
- Checkpoint replay: when `checkpointResume` is configured, missed updates between the last checkpoint and reconnect are replayed.
- `affectedListKeys` derivation: a CREATE on `client` triggers refresh of any list keyed on `client` (matches the ElectricSQL adapter's behavior).

### `prometheus-entity-skills/entity-realtime-surreal-live/` (new skill)

Mirroring the structure of `entity-realtime-channel`. Sections:

- "When to use" — SurrealDB-backed Prometheus apps wanting realtime graph updates.
- "What you get" — `LIVE SELECT` per registered table, automatic graph hydration, reconnect/replay.
- "Setup" — `import { createSurrealLiveAdapter } from "@prometheus-ags/prometheus-entity-management"`; `registerAdapter("surreal-live", …)`; per-table config examples.
- "Patterns" — `where` clauses for tenant scoping, `normalize` for schema mapping, `checkpointResume` for offline-tolerant apps.
- "Gotchas" — SurrealDB's lack of insert-vs-update distinction (always `upsert`), connection management, websocket auth refresh.
- "Tests" — pointer to the adapter's vitest suite as reference.

### Public re-export

`prometheus-entity-management/src/index.ts` adds:

```ts
export {
  createSurrealLiveAdapter,
  type SurrealTableConfig,
  type SurrealLiveAdapterOptions,
} from "./adapters/surreal-live";
```

### Non-changes

- **No bundled SurrealDB driver dependency.** The adapter accepts a `Surreal` client instance via options; consumers depend on `surrealdb.js` / `surrealdb` directly. This mirrors how the Electric adapter accepts a `PGlite` instance.
- **No change to the ElectricSQL adapter** or any other existing adapter.
- **No realtime-manager.ts modification.** The new adapter conforms to the existing `SyncAdapter` shape.

## Capabilities

### New Capabilities

- `entity-surreal-live-adapter`: A `SurrealLiveAdapter` implementing the `SyncAdapter` contract using SurrealDB `LIVE SELECT`, with `select-then-live` seeding, action-payload mapping (CREATE/UPDATE/DELETE → EntityChange), reconnect with exponential backoff, optional checkpoint-based replay on reconnect, and a companion `entity-realtime-surreal-live` skill in `prometheus-entity-skills`.

### Modified Capabilities

- None as separate spec entries. The `SyncAdapter` contract is unchanged; the realtime-manager is unchanged.

## Impact

- **Risk**: Medium. SurrealDB's live-query protocol is subscription-based over websocket; reconnection + replay are the tricky parts. The contract is well-isolated, and tests cover the failure modes.
- **Affected files**:
  - `prometheus-entity-management`: `src/adapters/surreal-live.ts` (new), `src/adapters/surreal-live.test.ts` (new), `src/index.ts` (re-export added). All new code; no edits to existing adapters.
  - `prometheus-skill-system`: `skills/prometheus-entity-skills/entity-realtime-surreal-live/SKILL.md` (new).
- **Cross-repo**: Yes — two repos, two coordinated PRs.
- **Reversibility**: Trivial — drop the new files; no consumers in this UAR repo yet.
- **Unblocks**: Any Prometheus app using SurrealDB as its primary store gets realtime entity graph updates without rolling its own adapter.
