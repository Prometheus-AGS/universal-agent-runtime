## Why

`providers-page.tsx` inlines the snapshot/upsert/rollback pattern in two places (`setDefault`, `removeProvider`). With `add-optimistic-helpers-module` shipped, replace both with one-line calls to `optimisticUpsert` / `optimisticRemove`. Net effect: ~25 LOC of duplication removed, error handling centralized in the helper, behaviour unchanged.

## What Changes

- `setDefault(id)` — replace the inline `useGraphStore.getState()` + manual rollback with:
  ```ts
  await optimisticUpsert("ProviderMeta", "current",
    { id: "current", default_id: id },
    () => setDefaultProviderApi(id));
  ```
  Wrap with `try/catch` for the local `setError(...)` surface.
- `removeProvider(id)` — replace with `optimisticRemove("Provider", id, () => deleteProviderApi(id))` wrapped in try/catch.
- Drop the `useGraphStore` import if no other code in the file references it. Drop `ProviderEntity` import if unused after the change.

## Acceptance

- Page renders identically.
- `pnpm --filter ./frontend test` → 36/36 green.
- `pnpm --filter ./frontend build` clean.
- `git grep -nE "useGraphStore.getState" frontend/src/admin/pages/providers-page.tsx` → empty (no remaining direct graph mutations in this file).
