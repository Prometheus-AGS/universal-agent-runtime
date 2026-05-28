## Why

The snapshot → optimistic mutation → rollback-on-failure pattern is now inlined four times across providers-page and agents-page. The contract test already defines exactly the helper shapes (`optimisticUpsert`, `optimisticRemove`) but they live as local fns inside the test file rather than in production. Extracting them to a real module makes the test the canonical regression for the helper, eliminates duplication, and unblocks the next two changes that swap call sites over.

## What Changes

- New file `frontend/src/lib/realtime/optimistic.ts` exporting:
  - `optimisticUpsert<T>(type, id, patch, serverCall)` — snapshot + shallow-merge + rollback on throw.
  - `optimisticRemove(type, id, serverCall)` — snapshot + remove + re-upsert on throw.
- Both helpers **re-throw** on rejection (callers wrap with try/catch for local UI state).
- Update `frontend/src/lib/realtime/__tests__/optimistic-rollback.test.tsx` to:
  - `import { optimisticUpsert, optimisticRemove } from "../optimistic";`
  - Delete the inline helper definitions in the test (they become unused).
- All 5 existing rollback contract tests must remain green.

## Acceptance

- New module exists; exports the two helpers with the test's existing signatures.
- Test imports from the module; inline copies removed.
- `pnpm --filter ./frontend test` → 36/36 green.
- `pnpm --filter ./frontend build` clean.
