# CLAUDE.md — entity-graph-crud

Architecture rules for code generated or edited under this skill. **Do not violate these rules** when implementing CRUD for `@prometheus-ags/prometheus-entity-management`.

## Layering (strict)

```
Components → Hooks → Stores (graph) → APIs / adapters
```

1. **Components** use only:
   - Public hooks from the library (`useEntityCRUD`, `useEntity`, `useEntityView`, etc.)
   - Props/callbacks provided by parent modules or thin **custom hooks** defined in `hooks/` or `features/<entity>/useXxx.ts`
2. **Hooks** orchestrate graph reads/writes by calling:
   - `useGraphStore.getState()` **only inside hooks** when the public API is insufficient (same as library internals—not for components)
   - Mutation callbacks (`onCreate`, `onUpdate`, `onDelete`) that are implemented in a **store module** or **API client module** imported by the hook layer
3. **Components must not**:
   - Call `useGraphStore` directly
   - Call `fetch`, GraphQL clients, or Supabase from event handlers

**Practical split for app code**

- **`api/<entity>.ts`** (or `server/` handlers): `fetch`, serialization, error mapping.
- **`hooks/useThingCRUD.ts`**: wraps `useEntityCRUD` with `listFetch`, `detailFetch`, `onCreate`, etc., delegating to `api/`.
- **`pages/ThingPage.tsx`**: renders `EntityTable`, sheets, buttons; receives `crud` from `useThingCRUD`.

## Entity graph rules

- **Lists store IDs only.** Table rows join `list.ids` → `readEntity(type, id)` at render time (handled inside library hooks).
- **Normalize every server payload** with a single `normalize(raw) => { id, data }` consistent across list and detail.
- **Mutations:** after success, rely on `cascadeInvalidation` (from `useEntityCRUD`) plus `registerSchema` metadata—avoid hand-rolled invalidation of unrelated keys.

## CRUD-specific rules

- **Edit buffer** lives in React state via `useEntityCRUD`—do not mirror the full buffer into graph `patches` except via **`applyOptimistic()`** for intentional UI feedback.
- **Optimistic create** patterns: follow library behavior (temporary IDs, rollback on failure).
- **Detail vs list:** prefer `detailFetch` when list rows are partial; otherwise ensure list fetch returns enough fields for the UI or accept extra `useEntity` subscriptions.

## Schema registration

- Call **`registerSchema`** once at app bootstrap (e.g. `main.tsx`, `app/providers.tsx`) per `EntityType`.
- **`listKeyPrefix`** functions must return the same key shape used by `useEntityView` / `useEntityList` for that relation-driven list.
- Use **`globalListKeys`** for coarse invalidation when a mutation should refresh all lists of a type (sparingly).

## UI components

- Use **`FieldDescriptor`** for sheets—keep labels, types, enum options, and `editControl` customizations declarative.
- Use **`meta.entityMeta`** on columns for filter toolbar behavior (`filterType`, `enumOptions`, `relationEntityType`).
- Prefer library **`SortHeader`** with TanStack Table sorting aligned to `useEntityView` remote sort when in remote/hybrid mode.

## TypeScript

- Strict typing; avoid `any` except at HTTP boundaries with a comment.
- Use `useRef` for callbacks passed into effects inside custom hooks mirroring library patterns.

## Verification

- Run `pnpm run typecheck` for the library consumer.
- Manually exercise list → detail → edit → save → delete in the example app or target app.
