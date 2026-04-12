---
name: entity-realtime-local-first
description: >
  Wire ElectricSQL shape streams and PGlite with createElectricAdapter (SyncAdapter): ElectricTableConfig,
  shapeStream subscription, PGlite listen path into ChangeSet, useLocalFirst and usePGliteQuery for reads,
  and sync-complete UX.
---

# `/entity-realtime-local-first` — ElectricSQL + PGlite

## When to use

- Offline-capable or ultra-low-latency reads from embedded Postgres
- Server authority remains Postgres; **Electric** replicates allowed shapes into local **PGlite**
- You want graph updates driven from local DB notifications + shape inserts

## Building blocks (`src/adapters/electricsql.ts`)

- **`createElectricAdapter({ pglite, tables, onSynced? })`** → **`SyncAdapter`**
- **`ElectricTableConfig`**: `type`, `table`, optional `where`, `idColumn` (default `"id"`), optional `normalize(row)`, **`shapeStream`**
- **`ShapeStream`**: `subscribe(msgs => void, onErr?)`, `isUpToDate`, `lastOffset`
- Adapter maps **`ShapeMessage`** → **`EntityChange`** (`insert`/`upsert`/`delete`)

## `SyncAdapter` extras

- **`query` / `execute`** — SQL against PGlite
- **`isSynced()`**, **`onSyncComplete(cb)`** — UX gates (“synced” badge, enable editing)

## React helpers

- **`useLocalFirst`** — exposes query/execute and sync state (uses graph internally where appropriate)
- **`usePGliteQuery`** — re-run SQL when local DB updates

## Integration pattern

1. Initialize **PGlite** (IDB or worker) per vendor docs.
2. Open **Electric** shape streams per table; pass into **`ElectricTableConfig`**.
3. **`getRealtimeManager().register(electricAdapter, [{ type: "RowType" }])`** — channels may be minimal if adapter multiplexes tables internally (follow library’s **`register`** contract: one subscribe per channel entry).
4. Prefer **reads** from PGlite for instant UI; **writes** follow your product’s online/offline strategy (library comments describe optimistic local writes + background sync).

## Pitfalls

| Issue | Mitigation |
|-------|------------|
| Shape without PGlite | Adapter needs both; no half-wired demo configs |
| Wrong `idColumn` | Deletes/upserts target wrong graph id |
| Missing `normalize` | Column name mismatch vs graph entity shape |

## Parent skill

**`prometheus-entity-skills/entity-graph-realtime/SKILL.md`**, **`references/adapter-catalog.md`**
