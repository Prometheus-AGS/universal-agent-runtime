---
name: entity-gc
description: >
  Slash command /entity-gc — design application-level eviction for @prometheus-ags/prometheus-entity-management:
  removeEntity for stale entities, clearPatch for UI overlays, list refresh after bulk delete,
  route-based cleanup, and policies for long-lived sessions. Notes library has no automatic GC yet.
---

# /entity-gc

## Current library state

@prometheus-ags/prometheus-entity-management **does not** ship automatic entity garbage collection (see root `CLAUDE.md` limitations). All “GC” is **explicit application policy**.

## APIs to use

| API | Effect |
|-----|--------|
| `removeEntity(type, id)` | Removes canonical entity from `entities` |
| `clearPatch(type, id)` | Removes local patch overlay |
| `invalidateLists(key)` / store list mutations | Forces refetch; does not delete entities by itself |

## Strategies

### 1. Route-scoped ephemeral types

Use a dedicated `EntityType` (e.g. `"SearchHit"`) for disposable results. On route leave, iterate known ids and `removeEntity`.

### 2. Pagination caps

Keep list ids aligned with visible window: after refetch with new filter, old entities may remain in memory until explicitly removed — optional sweeper removes entities not referenced by any mounted list query (app-specific bookkeeping).

### 3. Logout

Clear sensitive types or full session: implement app `resetGraph()` that reinitializes store state (if you expose a reset) or removes known types. **Note:** core store may not export full reset — you may need `useGraphStore.setState` in a controlled bootstrap module (treat as advanced; coordinate with maintainers).

### 4. Stale id in lists

After delete mutation, library CRUD paths update lists; if manual, call `setListResult` without deleted id or refetch.

## Anti-patterns

- Removing entities still visible in another mounted view
- Calling `removeEntity` for every navigation without tracking references

## Verification

- Memory snapshot before/after heavy browse (Chrome heap)
- Functional: no missing rows after eviction policy runs

## References

- Library: `src/graph.ts` (`removeEntity`, `clearPatch`)
