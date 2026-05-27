## Why

Two upstream pieces (the entity-mgmt library and the SurrealDB live-query bus) only deliver value once they are wired together. This change is the **single point** in the system that enforces "no stale data anywhere": one SPA bootstrap configures the entity engine, and one `UarRealtimeAdapter` listens to the bus and applies Create/Update/Delete to the graph. Every view that uses `useEntity`/`useEntityList` automatically re-renders on remote mutations.

## What Changes

### SPA bootstrap

- New `frontend/src/lib/entity-engine.ts`:
  ```ts
  import { configureEngine } from "@prometheus-ags/prometheus-entity-management";

  configureEngine({
    defaultStaleTime: 30_000,
    defaultGcTime: 5 * 60_000,
    gcInterval: 60_000,
    maxRetries: 3,
    retryBaseDelay: 250,
    revalidateOnFocus: true,
    revalidateOnReconnect: true,
  });
  ```
- Imported once at top of `main.tsx` before React renders.

### Realtime adapter

- New `frontend/src/lib/realtime/uar-realtime-adapter.ts`.
- Implements the entity-mgmt library's `SyncAdapter` interface.
- On `register(topic)`:
  - Opens an `EventSource("/api/live/" + topic, { withCredentials: true })`.
  - The proxy auto-injects JWT; native fetch path: prepend a one-shot signed query token (out of scope, document as caveat).
  - On `event: create|update|delete`, deserializes `data: { id, ... }` and calls `graph.upsertEntity` or `graph.removeEntity`.
- Registered for every topic the SPA tracks: `knowledge_base`, `knowledge_document`, `agent`, `provider`, `model`, `skill`, `setting`.

### Fetch wiring

- New per-entity fetchers in `frontend/src/services/entities/`:
  - `fetchEntity("provider", id)` → `GET /api/uar/providers/{id}`
  - `fetchList("provider")` → `GET /api/uar/providers`
- Wire into `useEntity` calls via the `fetch` + `normalize` options.

## Acceptance

- After SPA bootstrap, opening DevTools → Network shows one `EventSource` per registered topic.
- Mutating a provider via Admin causes the chat-header model badge (which reads the same provider entity) to update **without** any explicit refresh logic.
- Disconnecting and reconnecting the EventSource transparently re-syncs (validated by killing the proxy mid-session).
- No view in Admin re-fetches the same entity if another view already has it in cache within `staleTime`.
