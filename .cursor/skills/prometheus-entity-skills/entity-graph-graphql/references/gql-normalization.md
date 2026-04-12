# GraphQL normalization into the entity graph

This document describes runtime behavior of `normalizeGQLResponse` in **@prometheus-ags/prometheus-entity-management** (`src/graphql/client.ts`).

## Entry

```ts
normalizeGQLResponse(data, descriptors);
```

- `data` is typically `response.data` from a GraphQL execution result (object or null).
- Each **top-level** descriptor resolves its `path` against `data`.

## Path resolution

- `path` uses dot notation: `viewer.organization.members`.
- `path === "."` uses `data` itself as the subtree root.
- Missing intermediate objects yield `undefined` → walk becomes a no-op for that branch.

## Node processing

For subtree `S` and descriptor `D`:

1. If `S` is an array, each element is processed.
2. If `S` is a non-null object:
   - `id = D.extractId?.(S) ?? String(S.id)`
   - `normalized = D.normalize(S)`
   - `upsertEntity(D.type, id, normalized)` and `setEntityFetched(D.type, id)`
3. For each `rel in D.relations`:
   - `next = resolvePath(S, rel.path)`
   - `walk(next, rel)`

## Side effects

- All writes go through the graph store (Zustand + Immer).
- **Patches** are untouched — optimistic UI may still use `patchEntity`.

## Lists vs descriptors

`useGQLList` **also** relies on descriptors to normalize each item in the response; separately it computes `ids` via `getItems` + `extractId` and calls `setListResult` / `appendListResult`. Ensure:

- Descriptors actually **upsert** the same entities those IDs refer to.
- `getItems` returns the same node objects the descriptor’s `normalize` expects (or adjust paths so walk reaches them).

## Dedupe

`GQLClient.query` wraps execution in `dedupe(cacheKey, fn)`. Normalization runs inside that fn after a successful response — so duplicate concurrent queries share one network call and one normalize pass.

## Typename

GraphQL often returns `__typename`. The graph does not strip it automatically in `normalize`; strip in your `normalize` function if you want clean domain objects.
