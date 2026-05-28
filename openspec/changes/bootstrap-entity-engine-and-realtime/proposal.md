## Why

The previous phase (`prometheus-package-integration`) shipped the entity-graph engine config (`frontend/src/lib/entity-engine.ts`), the SSE realtime adapter (`uar-sse-adapter.ts`), and the topic map (`topics.ts`). None of this code currently runs in the SPA because `main.tsx` doesn't import the engine module or instantiate `RealtimeManager`. Until that single bootstrap edit lands, every downstream migration is dormant.

## What Changes

Modify `frontend/src/main.tsx` to:

1. Side-effect-import `@/lib/entity-engine` **before** React renders so `configureEngine` runs once at module init.
2. Instantiate a single `RealtimeManager` (from `@prometheus-ags/prometheus-entity-management`).
3. Register every SSE adapter returned by `createAllUarAdapters()` and call `manager.subscribe(...)` so the EventSource connections open at startup.
4. Add a 60-second diagnostic that logs each received `EntityChange` to the console — removed after the first cross-cutting migration ships.

## Acceptance

- DevTools Network panel shows 7+ `EventSource` connections open within 1 s of first paint.
- Console logs `[entity-mgmt] subscribed to <topic>` for each topic.
- Writing a row directly into SurrealDB causes a `[entity-mgmt] received` console log within 200 ms.
- No visible UI changes — purely infrastructure.
