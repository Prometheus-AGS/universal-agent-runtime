# CLAUDE.md — Entity graph optimization

## Non-negotiable architecture

1. **Components** never import `useGraphStore` or call `useGraphStore.getState()` for data reads.
2. **Hooks** never perform naked `fetch` / GraphQL HTTP; they call **store/engine APIs** (`fetchEntity`, `GQLClient`, etc.) through the patterns the library exposes.
3. **Stores** (Zustand graph + engine) own synchronization with the network.

Violations create silos and break cross-view reactivity.

## Zustand selectors

- Prefer **narrow** selectors in `useStore(useGraphStore, (s) => ...)` so unrelated graph updates do not re-render the component.
- Avoid returning fresh object literals from selectors when the identity is not needed — memoize in the selector or split subscriptions.

## Subscriber tokens

- Engine uses `registerSubscriber` / `unregisterSubscriber` per entity key (`${type}:${id}`).
- Leaks occur if effects omit cleanup or if keys churn without unregister (rare in library hooks if used correctly).
- Audit custom hooks that wrap engine APIs.

## List keys

- `serializeKey(queryKey)` must include every dimension that changes list membership/order.
- Over-broad keys → duplicate fetches; under-broad keys → wrong cache reuse.

## RealtimeManager

- Default `flushInterval: 16` coalesces bursts; set `0` only when debugging or when ordering requires immediate writes.
- Too many adapters → consolidate channels where possible.

## GraphQL dedupe

- `GQLClient.query` uses string cache keys — avoid collisions from truncated documents in default keying; pass explicit `cacheKey`.

## Eviction (manual)

- `removeEntity(type, id)` drops canonical entity data; ensure lists no longer reference removed ids or run `setListResult` refresh.
- `clearPatch` for UI-only overlays.
- Long session + huge datasets: implement route-change eviction for unmounted feature areas (app-specific).

## Public hook JSDoc

Library public hooks require JSDoc; app wrappers should document side effects (`invalidateLists`, etc.).
