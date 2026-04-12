# Plan — entity-graph-realtime

## 1. Adapter selection

Document decision using **`agents/adapter-selector.md`** checklist.

Include **rejected alternatives** with one-line reasons (maintainability).

## 2. Topology diagram

Sketch:

- Client app
- Adapter instance(s)
- `RealtimeManager` singleton
- Entity graph (Zustand)

Note **how many** `register` calls (usually one per adapter instance).

## 3. Channel map

Table:

| ChannelConfig | EntityType | filter/id | Notes |
|---------------|------------|-----------|-------|
| … | … | … | replay on connect? |

## 4. Normalization

- Field → `EntityChange.op` mapping
- ID extraction rule
- Patch merge strategy for partial updates

## 5. Coalescing policy

- Default **16ms** vs `0`
- Whether to expose `forceFlush()` for testing

## 6. Integration file layout

Example:

- `src/realtime/manager.ts` — `getRealtimeManager` options
- `src/realtime/supabase.ts` — adapter factory
- `src/realtime/RealtimeProvider.tsx` — effect bootstrap (hook/component **only** orchestrates register/unregister; adapter creation stays in module)

## 7. Observability

- Logging flags
- `onChangeReceived` usage in dev

## 8. Risk register

- Duplicate subscriptions
- Schema drift between server payload and `EntityType`
- Large binary frames (WebSocket) — parsing strategy

## Output

Ordered implementation checklist + file list.
