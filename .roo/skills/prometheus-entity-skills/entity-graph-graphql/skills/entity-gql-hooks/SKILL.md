---
name: entity-gql-hooks
description: >
  Slash command /entity-gql-hooks — produce typed useGQLEntity, useGQLList, useGQLMutation wrappers
  for concrete GraphQL documents; wire queryKey, getItems, getPagination, sideDescriptors, and
  invalidateLists for @prometheus-ags/prometheus-entity-management.
---

# /entity-gql-hooks

## When invoked

User has (or will have) `GQLClient` + descriptors and needs **React hook wrappers** for specific operations.

## Steps

1. For each operation, choose hook:
   - Single record → `useGQLEntity`
   - Collection → `useGQLList`
   - Create/update/delete → `useGQLMutation`

2. **Type parameters**
   - `TData`: shape of `data` from the operation (narrow as much as possible).
   - `TEntity`: normalized entity stored in graph (must extend `Record<string, unknown>`).

3. **List wiring**
   - `queryKey`: include all filter/sort variables that should isolate list state.
   - `getItems`: defensive `?? []` for nullable connections.
   - `getPagination`: map your schema’s page info.

4. **Mutations**
   - `descriptors` when payload includes entities to merge.
   - `invalidateLists`: pass `serializeKey`-compatible keys from list hooks, or raw string keys if using graph store’s list invalidation API consistently.

5. **refs**
   - Library hooks use `useRef` for options; wrappers should pass stable `client` reference.

6. **List invalidation after mutations**
   - `invalidateLists` expects the same string key the list hook uses internally: `serializeKey(queryKey)` from `@prometheus-ags/prometheus-entity-management` (see `src/engine.ts`). Example:

```typescript
import { serializeKey } from "@prometheus-ags/prometheus-entity-management";

const LIST_KEY = ["Post", "list", "dashboard"] as const;

export function usePostsList() {
  return useGQLList<PostsData, Post>({
    /* ... */
    queryKey: [...LIST_KEY],
  });
}

export function useUpdatePost() {
  return useGQLMutation<UpdateData, Post>({
    client: gqlClient,
    document: UPDATE_POST,
    type: "Post",
    descriptors: [postDescriptor],
    invalidateLists: [serializeKey([...LIST_KEY])],
  });
}
```

7. **Subscriptions (hook path)**

```typescript
export function usePostSubscription(id: string) {
  return useGQLSubscription<SubData>({
    client: gqlClient,
    wsClient,
    document: POST_CHANGED,
    variables: { id },
    descriptors: [postDescriptor],
    enabled: !!id,
  });
}
```

## Anti-patterns

- Duplicating fetched data with `useState` alongside graph-backed hooks.
- Omitting `sideDescriptors` when the query returns multiple entity types that UI reads elsewhere.

## References

- `../prompts/execute.md` — code patterns
- Library: `src/graphql/hooks.ts`
