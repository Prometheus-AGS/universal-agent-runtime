## Context

`prometheus-entity-management/src/adapters/` already has a working ElectricSQL adapter and a `realtime-manager` that accepts any `SyncAdapter`. The contract is well-defined: emit `ChangeSet`s on a subscribed handler, expose status callbacks, support clean teardown.

SurrealDB's `LIVE SELECT` returns a live-query UUID. The client's `live(uuid, callback)` (or `surrealdb.js`'s equivalent) fires on every CREATE / UPDATE / DELETE matching the query. Reconnection invalidates the UUID; the adapter must re-issue.

## Goals / Non-Goals

**Goals**
- Mirror the ergonomics of `createElectricAdapter` — same call shape, same surface.
- Conform exactly to the existing `SyncAdapter` contract — no new types in `realtime-manager`.
- Test with mocked SurrealDB client so vitest doesn't need a live database.
- Provide a documented `checkpointResume` story so apps with offline-tolerance can recover missed updates.

**Non-Goals**
- No SurrealDB bundled. Consumer brings their own client (same as PGlite for the Electric adapter).
- No writes through the adapter. Writes go through the engine's existing mutation path; this adapter is read-only realtime hydration.
- No multi-tenant variant in this change. A `surreal-live-tenant.ts` can come later mirroring `electricsql-tenant.ts`.

## Decisions

### D1. Consumer supplies the `Surreal` client

`opts.db: Surreal` is whatever the consumer's bootstrap created. The adapter doesn't connect; it assumes the client is connected (or will be connected by the time `start()` is called). This matches the ElectricSQL adapter's pattern.

### D2. `select-then-live` is the default

The first thing every Prometheus app does after wiring realtime is fetch the current state. `select-then-live` collapses that into the adapter so consumers don't need a parallel hydration path. `live-only` exists for apps that prefer a custom hydration step.

### D3. Reconnect with capped exponential backoff

1s → 3s → 9s → 30s (cap). Total wait before the cap is reached: 13s. That's the right balance between "instant retry to recover from a flicker" and "don't hammer the server during sustained outage".

### D4. Replay uses the same `columnName` for read and write

`checkpointResume.columnName` (default `updated_at`) names BOTH the column the replay query reads from AND the row property the adapter pulls the next checkpoint value from. This avoids a "fetch by `updated_at` but save based on `last_modified`" mismatch.

### D5. `affectedListKeys` derivation mirrors ElectricSQL

The ElectricSQL adapter computes `affectedListKeys` from the changed entity types. Same pattern here, same helper if it's exposed (or copy the small loop).

### D6. SurrealDB UPDATE → `op: "upsert"`

SurrealDB doesn't surface "this is a partial update of an existing row" vs "this is the full new row" — both fire `UPDATE`. Treating them all as `upsert` is safe: if the row already exists, the engine merges; if not, it inserts.

### D7. Test mocks the `Surreal` client at the methods we call

`query()`, `live()`, `kill()`, and a websocket-state callback. No need to spin up a SurrealDB instance. The tests assert the *behavior of our adapter* given the contract — not SurrealDB's correctness.

### D8. Cross-repo work split

- `prometheus-entity-management`: implementation + tests + re-export.
- `prometheus-skill-system`: skill SKILL.md doc only (mirrors `entity-realtime-channel`).

Each repo lands its own PR. The skill repo is a separate worktree from this UAR session; the skill SKILL.md is small enough to ship in this same change.

## Implementation Sketch

### `src/adapters/surreal-live.ts` (outline)

```ts
import { useGraphStore } from "../graph";
import { getRealtimeManager } from "./realtime-manager";
import type { SyncAdapter, ChangeSet, EntityChange, AdapterStatus, UnsubscribeFn } from "./types";
import type { EntityType, EntityId } from "../graph";

interface Surreal {                       // minimal surface
  query<T = unknown>(sql: string, vars?: Record<string, unknown>): Promise<T>;
  live<T = unknown>(query: string, callback: (action: SurrealAction<T>) => void): Promise<string>;
  kill(uuid: string): Promise<void>;
  // Some clients expose connection state via events; we accept an optional onClose hook.
  onClose?(handler: () => void): UnsubscribeFn;
}

interface SurrealAction<T> {
  action: "CREATE" | "UPDATE" | "DELETE";
  result: T | { id: string };
}

export interface SurrealTableConfig<T extends object> {
  type: EntityType;
  table: string;
  where?: string;
  idColumn?: string;
  normalize?: (row: T) => Record<string, unknown>;
}

export interface SurrealLiveAdapterOptions {
  db: Surreal;
  tables: SurrealTableConfig<Record<string, unknown>>[];
  onSynced?: () => void;
  initialQueryStrategy?: "select-then-live" | "live-only";
  checkpointResume?: {
    columnName: string;
    loadCheckpoint: () => Promise<string | null>;
    saveCheckpoint: (offset: string) => Promise<void>;
  };
}

export function createSurrealLiveAdapter(opts: SurrealLiveAdapterOptions): SyncAdapter {
  const { db, tables, onSynced, initialQueryStrategy = "select-then-live", checkpointResume } = opts;
  const statusCbs = new Set<(s: AdapterStatus) => void>();
  const syncedCbs = new Set<() => void>();
  let globalHandler: ((cs: ChangeSet) => void) | null = null;
  let liveIds: Record<string, string> = {};   // table → live UUID
  let reconnectAttempt = 0;

  function emit(cs: ChangeSet) { if (globalHandler) globalHandler(cs); }

  function affectedListKeys(changes: EntityChange[]): string[] {
    const types = new Set(changes.map((c) => c.type));
    // Mirror the same lookup used by createElectricAdapter; helper to be
    // factored out of that file in a later refactor.
    return Array.from(types).flatMap((t) => useGraphStore.getState().listKeysForType(t) ?? []);
  }

  function toChange<T extends object>(tc: SurrealTableConfig<T>, action: SurrealAction<T>): EntityChange | null {
    const idColumn = tc.idColumn ?? "id";
    if (action.action === "DELETE") {
      const id = String((action.result as { id?: string }).id ?? "");
      return id ? { op: "delete", type: tc.type, id } : null;
    }
    const row = action.result as Record<string, unknown>;
    const id = String(row[idColumn] ?? "");
    if (!id) return null;
    const data = tc.normalize ? tc.normalize(row as T) : row;
    return { op: action.action === "CREATE" ? "insert" : "upsert", type: tc.type, id, data };
  }

  async function seed() {
    if (initialQueryStrategy === "live-only") return;
    const seedChanges: EntityChange[] = [];
    for (const tc of tables) {
      const where = tc.where ? ` WHERE ${tc.where}` : "";
      const rows = await db.query<{ result?: Array<Record<string, unknown>> }[]>(`SELECT * FROM ${tc.table}${where}`);
      // SurrealDB returns a [{result: [...]}] envelope; normalise to rows.
      const flat = Array.isArray(rows) ? (rows[0]?.result ?? []) : (rows as unknown[]);
      for (const row of flat as Array<Record<string, unknown>>) {
        const change = toChange(tc as SurrealTableConfig<Record<string, unknown>>,
          { action: "CREATE", result: row as Record<string, unknown> });
        if (change) seedChanges.push(change);
      }
    }
    if (seedChanges.length > 0) {
      emit({ changes: seedChanges, affectedListKeys: affectedListKeys(seedChanges), timestamp: new Date().toISOString() });
    }
  }

  async function openLive() {
    liveIds = {};
    for (const tc of tables) {
      const where = tc.where ? ` WHERE ${tc.where}` : "";
      const uuid = await db.live(`LIVE SELECT * FROM ${tc.table}${where}`, (action) => {
        const change = toChange(tc as SurrealTableConfig<Record<string, unknown>>, action as SurrealAction<Record<string, unknown>>);
        if (!change) return;
        emit({ changes: [change], affectedListKeys: affectedListKeys([change]), timestamp: new Date().toISOString() });
        if (checkpointResume) {
          const row = (action.result as Record<string, unknown>) ?? {};
          const offset = row[checkpointResume.columnName];
          if (typeof offset === "string") void checkpointResume.saveCheckpoint(offset);
        }
      });
      liveIds[tc.table] = uuid;
    }
  }

  async function replay() {
    if (!checkpointResume) return;
    const last = await checkpointResume.loadCheckpoint();
    if (!last) return;
    for (const tc of tables) {
      const where = `${tc.where ? `(${tc.where}) AND ` : ""}${checkpointResume.columnName} > "${last}"`;
      const rows = await db.query<{ result?: Array<Record<string, unknown>> }[]>(`SELECT * FROM ${tc.table} WHERE ${where}`);
      const flat = Array.isArray(rows) ? (rows[0]?.result ?? []) : (rows as unknown[]);
      const changes: EntityChange[] = [];
      for (const row of flat as Array<Record<string, unknown>>) {
        const change = toChange(tc as SurrealTableConfig<Record<string, unknown>>, { action: "UPDATE", result: row });
        if (change) changes.push(change);
      }
      if (changes.length > 0) {
        emit({ changes, affectedListKeys: affectedListKeys(changes), timestamp: new Date().toISOString() });
      }
    }
  }

  async function reconnectLoop() {
    for (const cb of statusCbs) cb("reconnecting");
    while (true) {
      const sleep = [1000, 3000, 9000][reconnectAttempt] ?? 30_000;
      await new Promise((r) => setTimeout(r, sleep));
      reconnectAttempt++;
      try {
        await replay();
        await openLive();
        reconnectAttempt = 0;
        for (const cb of statusCbs) cb("online");
        return;
      } catch (e) {
        // keep trying
      }
    }
  }

  let closeUnsub: UnsubscribeFn | null = null;
  return {
    async start(handler: (cs: ChangeSet) => void) {
      globalHandler = handler;
      for (const cb of statusCbs) cb("connecting");
      await seed();
      await openLive();
      for (const cb of statusCbs) cb("online");
      if (db.onClose) {
        closeUnsub = db.onClose(() => { void reconnectLoop(); });
      }
      for (const cb of syncedCbs) cb();
      onSynced?.();
    },
    async stop() {
      if (closeUnsub) closeUnsub();
      for (const tc of tables) {
        const uuid = liveIds[tc.table];
        if (uuid) await db.kill(uuid).catch(() => {});
      }
      liveIds = {};
      globalHandler = null;
    },
    onStatus(cb) { statusCbs.add(cb); return () => statusCbs.delete(cb); },
    onSynced(cb) { syncedCbs.add(cb); return () => syncedCbs.delete(cb); },
  };
}
```

### `src/adapters/surreal-live.test.ts` (outline)

Vitest with a hand-rolled fake `Surreal`:
- Records `query` calls and returns canned rows.
- Records `live` calls and exposes a helper to push synthetic actions into the registered callback.
- Records `kill` calls.
- Tracks a `triggerClose()` to simulate disconnect.

Cases: see spec req "Test Coverage".

### `prometheus-entity-skills/entity-realtime-surreal-live/SKILL.md` (outline)

Front matter, `# When to use`, `# What you get`, `# Setup` with `import { createSurrealLiveAdapter, registerAdapter } from "@prometheus-ags/prometheus-entity-management"`, `# Patterns`, `# Gotchas`, `# Tests` (pointer to surreal-live.test.ts).

## Risks

1. **SurrealDB client API drift.** `surrealdb.js` is still evolving; the live-query callback shape has changed between versions. The minimal surface in `interface Surreal` insulates the adapter — consumers can pass any client that conforms. Document the tested version range.
2. **Reconnect storms.** Capped at 30s. If the server is hard-down, the adapter loops forever — that's correct behavior (consumers want eventual recovery), but document it so operators know.
3. **`affectedListKeys` lookup performance.** The implementation reads from the graph store on every change. Mirrors ElectricSQL's behavior; cost is amortised. Profile if it shows up in flamegraphs later.
4. **DELETE payload variance.** SurrealDB DELETE notifications sometimes carry only `{id}`, sometimes the full prior row. The implementation pulls `id` defensively and ignores other fields for DELETE.

## Alternatives Considered

- **Bundle a SurrealDB client.** Rejected — couples adapter to a specific driver version; consumers may use the WASM build, Rust build via tauri-bridge, etc.
- **Use `surrealql` subscriptions instead of `LIVE SELECT`.** Rejected — `LIVE SELECT` is the canonical realtime API; subscriptions in surreal land as `LIVE` under the hood anyway.
- **Skip the seed and rely on a separate query.** Rejected per D2 — single adapter call is the better DX.
