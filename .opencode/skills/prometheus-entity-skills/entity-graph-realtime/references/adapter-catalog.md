# Adapter catalog — @prometheus-ags/prometheus-entity-management

Source: **`src/adapters/types.ts`**, **`src/adapters/realtime-adapters.ts`**, **`src/adapters/electricsql.ts`**, **`src/adapters/realtime-manager.ts`**. All factories are re-exported from the **package root** (`src/index.ts`).

---

## Contract: `RealtimeAdapter`

```ts
interface RealtimeAdapter {
  readonly name: string;
  subscribe(config: SubscriptionConfig, handler: (changeset: ChangeSet) => void): UnsubscribeFn;
  onStatusChange?: (cb: (status: AdapterStatus) => void) => UnsubscribeFn;
}
```

- Handlers receive **`ChangeSet`**: `{ changes: EntityChange[]; affectedListKeys?; timestamp? }`
- **`EntityChange`**: `op`, `type`, `id`, optional `data` or `patch`
- Adapters **must not** call `useGraphStore`; they only invoke the handler supplied by **`RealtimeManager`**.

---

## `createWebSocketAdapter(options: WebSocketAdapterOptions)`

| Option | Role |
|--------|------|
| **`url`** | `string` or lazy `() => string` (token refresh) |
| **`parseMessage`** | `(unknown) => EntityChange[] \| null` — if omitted, a default JSON parser may apply |
| **`protocols`** | WebSocket subprotocols |
| **`reconnectBaseDelay`**, **`maxReconnectAttempts`** | Exponential backoff (default base **1s**) |
| **`pingInterval`**, **`pingMessage`** | Keepalive (default **30s**, JSON ping) |

**Use when:** custom backend or generic JSON push over WS.

**Status:** `connecting` → `connected` → `disconnected` / `error` on failures.

---

## `createSupabaseRealtimeAdapter(options: SupabaseAdapterOptions)`

**Use when:** Postgres row changes via **Supabase Realtime** channels.

- Requires a configured **Supabase client** with Realtime enabled
- Map Postgres rows → **`EntityChange`** with correct graph **`type`** and string **`id`**
- Typically one logical channel per table or filtered topic (per your RLS and filter strategy)

See implementation in **`src/adapters/realtime-adapters.ts`** for option names (channel names, schema, event types).

---

## `createConvexAdapter(...)`

**Use when:** **Convex** reactive queries / subscriptions drive document updates.

- Translate Convex documents into **`insert` / `upsert` / `update` / `delete`** as appropriate
- Respect Convex auth identity when mapping multi-tenant data

---

## `createGraphQLSubscriptionAdapter(...)`

**Use when:** GraphQL **`subscription`** over WebSocket (Apollo, Absinthe, etc.).

- Normalize each subscription payload to a **`ChangeSet`**
- Handle connection `next` errors and reconnect policies at the adapter level

---

## `createElectricAdapter(options: ElectricAdapterOptions)` → `SyncAdapter`

**Use when:** **ElectricSQL** shapes replicate into **PGlite**; local reads + graph hydration.

| Option | Role |
|--------|------|
| **`pglite`** | Minimal interface: `query`, `exec`, `listen` |
| **`tables`** | `ElectricTableConfig[]` per replicated table |
| **`onSynced?`** | Callback when all configured tables report up-to-date |

**`ElectricTableConfig`:** `type`, `table`, optional `where`, `idColumn` (default `"id"`), optional `normalize`, **`shapeStream`**.

**`SyncAdapter`** additionally exposes **`query`**, **`execute`**, **`isSynced`**, **`onSyncComplete`** — suitable for local-first SQL and “synced” gates.

---

## React helpers (Electric module)

| Export | Role |
|--------|------|
| **`useLocalFirst`** | Query/execute against PGlite + sync awareness |
| **`usePGliteQuery`** | Re-run SQL when local DB updates |

These may use **`useGraphStore`** **inside the library** — app components still use normal entity hooks.

---

## Registration with `RealtimeManager`

```ts
import {
  getRealtimeManager,
  createWebSocketAdapter,
  type ChannelConfig,
} from "@prometheus-ags/prometheus-entity-management";

const manager = getRealtimeManager({ flushInterval: 16 });
const ws = createWebSocketAdapter({
  url: WS_URL,
  parseMessage: (msg) => { /* ... */ },
});

const channels: ChannelConfig[] = [{ type: "Task" /*, filter, id, operations */ }];
const unregister = manager.register(ws, channels /*, normalizeRaw? */);
// cleanup: unregister();
```

- **`register`** returns **`UnsubscribeFn`** — call on logout, route leave, or HMR teardown
- Optional third argument **`normalize`**: `(raw: unknown) => EntityChange | null` for per-event shaping

---

## Singleton behavior: `getRealtimeManager` / `resetRealtimeManager`

- **`getRealtimeManager(opts?)`** creates the singleton on **first** call; later calls return the same instance — **`ManagerOptions` apply only on first construction**
- Use **`resetRealtimeManager()`** when you must drop all adapters and recreate (tests, controlled full reset)

---

## `forceFlush()`

`RealtimeManager` exposes **`forceFlush()`** to clear the pending timer and apply immediately — useful before teardown or deterministic tests. See **`references/coalescing-guide.md`**.
