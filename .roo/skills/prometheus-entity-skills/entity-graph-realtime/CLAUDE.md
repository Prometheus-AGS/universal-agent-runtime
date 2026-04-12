# CLAUDE.md — entity-graph-realtime

Rules for adding realtime to `@prometheus-ags/prometheus-entity-management`.

## Adapter boundary

- **Adapters implement `RealtimeAdapter`** (`subscribe`, optional `onStatusChange`).
- **Adapters must not** call `useGraphStore` directly. They only invoke the **`handler`** passed by `RealtimeManager.register`.
- **Normalization** happens either:
  - Inside the adapter (preferred for vendor SDKs), producing `ChangeSet`, or
  - Via optional `normalize` argument on **`manager.register(adapter, channels, normalize)`** mapping raw events → `EntityChange | null`.

## RealtimeManager responsibilities

- **Coalesces** rapid `EntityChange` events per `(type, id)` inside a flush window (default **16ms**) to collapse UI thrash.
- **`flushInterval: 0`** disables batching—each changeset flushes immediately (use only when ordering/low latency demands synchronous writes).
- On flush, applies:
  - `insert` / `upsert` → `upsertEntity` + `setEntityFetched`
  - `update` with `patch` → merge if entity exists; else `invalidateEntity`
  - `delete` → `removeEntity` + `removeIdFromAllLists`
  - `affectedListKeys` → `invalidateLists`

## Channel configuration

- `ChannelConfig` carries `type` (entity), optional `filter`, `id`, `operations`.
- Each channel subscription uses `adapter.subscribe({ label, replayOnConnect: true }, handler)`.

## Status and lifecycle

- Track connection status via `ManagerOptions.onStatusChange` and adapter `onStatusChange`.
- **Unregister** adapters on hot reload / user logout to avoid duplicate handlers:
  - Keep `UnsubscribeFn` from `register` and call on cleanup.
  - Use `resetRealtimeManager()` only when you intend to drop the singleton (tests, strict remount scenarios).

## Local-first (ElectricSQL)

- **`createElectricAdapter`** bridges ShapeStream messages + PGlite `LISTEN` into `ChangeSet` events.
- **`SyncAdapter`** extends realtime with `query` / `execute` / `isSynced` / `onSyncComplete`.
- Hooks like **`useLocalFirst`** may use `useGraphStore` internally—acceptable **inside library hooks**, not in app components.

## App integration pattern

1. Create adapter instance(s) with vendor credentials / endpoints in a **`realtime/`** or **`sync/`** module (not in React components).
2. On app bootstrap (provider effect), `getRealtimeManager(opts).register(...)`.
3. Return cleanup calling the stored `UnsubscribeFn`.

## Pitfalls

| Pitfall | Outcome |
|---------|---------|
| Double `register` without unregister | Duplicate writes / memory leaks |
| Normalizing to wrong `EntityType` | Cross-table corruption in graph |
| Ignoring `delete` ops | Stale rows remain in lists |
| `flushInterval: 0` with noisy source | Main-thread churn |

## Verification

- Log `onChangeReceived` in dev to inspect throughput pre-coalescing.
- Confirm single Zustand update per frame under burst updates with default flush.
