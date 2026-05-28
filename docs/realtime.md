# Realtime change-notification spine

UAR delivers cross-view freshness via a database-level live-query bus on the server and a single `EventSource`-backed entity-graph adapter on the client. The contract below is what skill authors, frontend devs, and embedders need to know.

## Server side

### SurrealDB live queries

UAR opens one `db.select(table).live()` stream per supported topic at startup:

| Topic               | SurrealDB table        |
|---------------------|------------------------|
| `knowledge_bases`   | `knowledge_bases`      |
| `knowledge_documents` | `knowledge_documents`|
| `agents`            | `agents`               |
| `providers`         | `providers`            |
| `models`            | `models`               |
| `skills`            | `skills`               |
| `settings`          | `settings`             |

Each `Notification<Value>` is converted to a `LiveEvent { action, topic, id, data }` and fanned out through a `tokio::sync::broadcast` channel.

Source: [`src/uar/realtime/surreal_bus.rs`](../src/uar/realtime/surreal_bus.rs).

### SSE endpoint

`GET /api/live/{topic}` (JWT-authenticated, like the rest of `/api`).

Each connected client gets its own broadcast receiver. Events are emitted as:

```
event: create
data: { "topic": "knowledge_documents", "id": "ec8c…", "data": { … row body … } }

event: update
data: { … }

event: delete
data: { … }
```

15-second keep-alive frames are sent to defeat intermediate idle timeouts.

Source: [`src/uar/api/live.rs`](../src/uar/api/live.rs).

### Supervised reconnect

Each topic's stream task runs under a supervisor with exponential backoff (250 ms → 30 s, full jitter). When a Surreal table doesn't yet exist (normal early-life condition for empty tables), the supervisor logs at `debug` and sits at the maximum backoff until the table starts producing notifications. Any other error is logged at `warn` with the backoff window.

## Client side

### Entity engine

`frontend/src/lib/entity-engine.ts` calls `configureEngine` once at SPA bootstrap with the locked defaults:

- `defaultStaleTime: 30_000 ms`
- `defaultGcTime: 5 min`
- `gcInterval: 60_000 ms`
- `maxRetries: 3`, `retryBaseDelay: 250 ms`
- `revalidateOnFocus: true`, `revalidateOnReconnect: true`

### SSE adapter

`createUarSseAdapter({ topic, entityType })` returns a `RealtimeAdapter` that:

1. Opens `EventSource("/api/live/{topic}", { withCredentials: true })`. The proxy injects JWT; native fetch users supply credentials separately.
2. Listens for `create`, `update`, `delete` event names.
3. Translates each frame into a graph `EntityChange { op, type, id, data }` and publishes a `ChangeSet`.
4. Exponential-backoff reconnect with `attempts < maxReconnectAttempts`.

Source: [`frontend/src/lib/realtime/uar-sse-adapter.ts`](../frontend/src/lib/realtime/uar-sse-adapter.ts).

### Topic catalogue

`frontend/src/lib/realtime/topics.ts` enumerates the 7 topics and exposes `createAllUarAdapters()` for one-call registration of every UAR realtime channel.

### Wiring

A SPA bootstrap sequence looks like:

```tsx
import "@/lib/entity-engine"; // side-effect: configureEngine
import { createAllUarAdapters } from "@/lib/realtime/topics";
import { RealtimeManager } from "@prometheus-ags/prometheus-entity-management";

const manager = new RealtimeManager();
for (const adapter of createAllUarAdapters()) {
  manager.registerAdapter(adapter);
  manager.subscribe(adapter, { replayOnConnect: false });
}
```

The actual call site (planned for the entity-mgmt migration changes) lives near `main.tsx` so adapters are alive before any view renders.

## End-to-end timing

The smoke test bundled with `surreal-live-query-bus` writes a `knowledge_documents` row and observes the SSE frame arrive at a concurrent `curl -N` subscriber. Empirically on a Mac mini against Docker SurrealDB (`ws://localhost:28000`), the median publish→deliver latency is well under 200 ms. Tighter targets can be set in the integration tests when they ship.

## Anti-stale data guarantee

Because every view consumes the same entity-mgmt graph keys (`type`, `id`) for its reads, a single SSE-published mutation triggers re-render in every subscribed component — toolbars, header chips, list pages, settings forms — without any per-view fetch logic. The realtime adapter is the single point of truth for cross-view freshness.
