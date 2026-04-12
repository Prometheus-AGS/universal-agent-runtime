# CLAUDE.md — Entity Graph GraphQL

Rules and patterns for implementing GraphQL on **@prometheus-ags/prometheus-entity-management**.

## EntityDescriptor contract

```ts
interface EntityDescriptor<TNode, TEntity extends Record<string, unknown>> {
  type: EntityType;       // graph bucket, e.g. "Post"
  path: string;           // dot path from query root, or "." for whole data
  extractId?: (node: TNode) => EntityId;
  normalize: (node: TNode) => TEntity;
  relations?: EntityDescriptor<unknown, Record<string, unknown>>[];
}
```

- **`path`**: `normalizeGQLResponse` resolves this on the response `data` object. Use `"."` only when the entire `data` payload should be walked as a single node (rare).
- **`extractId`**: Defaults to `String(node.id)` if omitted. Override for `uuid`, `databaseId`, or composite keys.
- **`normalize`**: Must return the **entity shape stored in the graph** (plain objects). Strip `__typename` if it should not persist.
- **`relations`**: For each relation descriptor, `rel.path` is resolved **relative to the current node** (not re-rooted at `data`).

## Normalization walk semantics

1. For each top-level descriptor, resolve `path` → `subtree`.
2. If `subtree` is an array, each element is processed; else the object is processed once.
3. Each node: `extractId` → `normalize` → `upsertEntity` + `setEntityFetched`.
4. Then recurse into each `relations` entry using `resolvePath(node, rel.path)`.

**Implication**: List queries should either use a descriptor whose `path` points to the array field, or a parent node descriptor with a relation whose `path` is the edges/nodes field.

## GQLClient usage

- **`query`**: Pass `descriptors` for every entity type that should be written to the graph from that operation. Use `cacheKey` when variables are stable but default key truncation could collide.
- **`mutate`**: Provide `descriptors` when the mutation payload returns populated objects. Use `optimistic` only with awareness that rollback uses a JSON snapshot of `entities` and `patches`.
- **`subscribe`**: Requires a `wsClient` matching `{ subscribe(payload, sink) => unsubscribe }`. On each `next`, `normalizeGQLResponse` runs like a query.

## Hooks

- **`useGQLEntity`**: Registers engine subscriber `${type}:${id}`; fetches when missing/stale. Pass `sideDescriptors` for nested types that should also land in the graph.
- **`useGQLList`**: Requires `getItems(data)` to return raw nodes; IDs come from `descriptor.extractId`. Pagination via `getPagination` and `appendListResult` when `mode: "append"`.
- **`useGQLSubscription`**: Effect-driven; cleans up unsubscribe on dep change. Does not replace list invalidation — trigger `invalidateLists` from mutations when needed.

## GraphQL subscription adapter vs direct normalize

| Approach | Best for |
|----------|----------|
| `useGQLSubscription` / `GQLClient.subscribe` | UI-tied streams; immediate `upsertEntity` via descriptors |
| `createGraphQLSubscriptionAdapter` + `RealtimeManager` | Same coalescing (16ms) as other adapters; `getPayload` maps subscription payload → `EntityChange` |

Adapter `getPayload` should return `{ type: "created"|"updated"|"deleted", node?, id? }` per library conventions in `realtime-adapters.ts`.

## Security

- Never embed secrets in client bundles. Use `headers: () => ({ Authorization: ... })` from secure storage or server-proxied cookies.
- Introspection against production should be read-only and rate-limited; prefer committed SDL in CI.

## TypeScript

- Keep `TEntity extends Record<string, unknown>`.
- Avoid `any`; use `unknown` at boundaries when introspection is untyped.
