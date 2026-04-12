# CRUD patterns — @prometheus-ags/prometheus-entity-management

## Pattern A — Single hook owns the surface

Use **`useEntityCRUD`** when one screen coordinates list + detail + forms.

**Why:** Edit buffer isolation, built-in optimistic update/rollback for saves, automatic `cascadeInvalidation` after mutations.

**Sketch:**

```ts
const crud = useEntityCRUD<Task>({
  type: "Task",
  listQueryKey: ["tasks", "list", viewSignature],
  listFetch,
  normalize: (raw) => ({ id: String(raw.id), data: raw }),
  detailFetch: fetchTaskById,
  onCreate: createTaskApi,
  onUpdate: updateTaskApi,
  onDelete: deleteTaskApi,
  initialView: { filter: initialFilter, sort: initialSort },
});
```

UI reads `crud.list`, `crud.detail`, `crud.mode` only—no store imports.

**`EntityTable` wiring:** pass **`viewResult={crud.list}`** (type **`UseEntityViewResult<T>`**). Do not pass a parallel `useState` copy of rows; the hook resolves **IDs → entities** from the graph for you.

## Pattern B — Read-mostly entity with custom toolbar

Combine **`useEntityView`** for the list and **`useEntity`** for detail when you do not need the full CRUD state machine (e.g. no create/delete in-app).

Still route fetches through hook-level callbacks, not components.

## Pattern C — Partial lists + detail hydration

**Symptom:** Row cards show title only; opening detail needs description, comments, etc.

**Fix:** Provide **`detailFetch`** in `useEntityCRUD` so `useEntity` subscribes with `enabled: !!selectedId`.

## Pattern D — Optimistic toggles

For sliders/checkboxes where waiting for server feels sluggish:

- Call **`crud.applyOptimistic()`** after buffer change to mirror selected fields into graph patches.
- Ensure server success still `replaceEntity` / `clearPatch` via library paths on save completion.

## Pattern E — Relation-aware detail panels

After `registerSchema`, use **`readRelations(type, detail)`** from `crud.relations` (already memoized in `useEntityCRUD`) to render parent/child previews.

Ensure child lists are populated by their own hooks elsewhere so `readRelations` resolves non-empty arrays.

## Anti-patterns

| Anti-pattern | Why it hurts |
|--------------|--------------|
| `useGraphStore` in page component | Breaks layering; hard to test; duplicates subscription logic |
| Duplicate `useState` entity rows | Breaks global reactivity; stale cells |
| Manual `invalidateLists` everywhere | Drifts from schema-driven cascade rules |
| Different `normalize` in list vs detail | Split-brain IDs; graph overwrites |

## Testing mindset

The library lacks a bundled test runner in-repo—use example apps (`pnpm run dev:vite`, `pnpm run dev:next`) for manual verification.
