# Performance patterns

## Prefer library hooks

`useEntity`, `useEntityList`, `useGQLEntity`, `useGQLList`, `useEntityView`, `useEntityCRUD` bundle subscriber registration, stale logic, and dedupe keys. Reimplementing them in app hooks often drops dedupe or leaks subscribers.

## Memoize query keys and view descriptors

Unstable `queryKey` references cause redundant list fetches and empty flash states.

## Split hot and cold data

- Hot: current page rows + selected detail
- Cold: prefetched ids without full entity payload — avoid upserting huge blobs until needed

## Virtualize long lists

Use row virtualization so each scroll frame does not render hundreds of `useEntity`-backed cells synchronously.

## Debounce filter typing

Wire filter toolbar to `useEntityView` with debounced `setFilter` so remote mode does not spam network.

## Realtime coalescing

Keep `flushInterval: 16` unless you measure a correctness need for immediate flush.

## Avoid inline selectors

```typescript
// Bad: new function identity each render if not useCallback-wrapped inside custom hook
useStore(store, (s) => compute(s, props.id));
```

Wrap in `useCallback` with deps or use library hooks.

## Measure before `useMemo` everywhere

Memoization has a cost; Profiler should justify it.

## GraphQL document size

Request only fields rendered; large nested trees increase normalize time and memory.

## GraphQL `cacheKey` collisions

`GQLClient.query` defaults to a key derived from the first 60 characters of the document string plus stringified variables. Two different operations with identical prefixes and variables can dedupe incorrectly — pass an explicit `cacheKey` per operation (the library hooks often build keys from `type`, `id`, and a document slice).

## Zustand shallow compares

When you must select multiple fields as an object, use `useShallow` from `zustand/react/shallow` (or equivalent) so referential equality matches when values are unchanged:

```typescript
import { useShallow } from "zustand/react/shallow";
import { useGraphStore } from "@prometheus-ags/prometheus-entity-management";

// Advanced / non-hook access — prefer library hooks first
const { ids, isFetching } = useGraphStore(
  useShallow((s) => ({
    ids: s.lists[key]?.ids ?? [],
    isFetching: s.lists[key]?.isFetching ?? false,
  }))
);
```

**Note:** Repo `CLAUDE.md` still forbids **components** from using `useGraphStore` directly; this pattern belongs in **hooks** or infrastructure modules that feed components.

## Next.js

- Server Components should not import client graph store.
- Hydration: follow `GraphHydrationProvider` pattern for seeding graph from SSR props.
