## 1. Imports

- [ ] 1.1 `import { optimisticUpsert, optimisticRemove } from "@/lib/realtime/optimistic";`
- [ ] 1.2 Drop `useGraphStore` import if no remaining references.
- [ ] 1.3 Drop `ProviderEntity` import if no remaining references.

## 2. setDefault

- [ ] 2.1 Replace inline snapshot/upsert/rollback with `optimisticUpsert` call.
- [ ] 2.2 Wrap with try/catch for `setError(...)` local state.

## 3. removeProvider

- [ ] 3.1 Replace inline snapshot/remove/re-upsert with `optimisticRemove` call.
- [ ] 3.2 Preserve `setRemoving(id)` / `setRemoving(null)` lifecycle.
- [ ] 3.3 Wrap with try/catch for `setError(...)`.

## 4. Verification

- [ ] 4.1 `pnpm --filter ./frontend test` → 36/36 green.
- [ ] 4.2 `pnpm --filter ./frontend build` clean.
- [ ] 4.3 `git grep -nE "useGraphStore.getState" frontend/src/admin/pages/providers-page.tsx` empty.
