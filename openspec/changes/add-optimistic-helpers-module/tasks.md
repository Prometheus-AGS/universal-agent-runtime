## 1. Author module

- [ ] 1.1 Create `frontend/src/lib/realtime/optimistic.ts`.
- [ ] 1.2 Export `optimisticUpsert<T extends Record<string, unknown>>(type, id, patch, serverCall)`.
- [ ] 1.3 Export `optimisticRemove(type, id, serverCall)`.
- [ ] 1.4 Both re-throw on rejection.
- [ ] 1.5 Brief JSDoc on each export.

## 2. Switch test to module

- [ ] 2.1 `import { optimisticUpsert, optimisticRemove } from "../optimistic";` at the top of `optimistic-rollback.test.tsx`.
- [ ] 2.2 Delete the inline helper definitions in the test (they were declared but will now be unused).

## 3. Verification

- [ ] 3.1 `pnpm --filter ./frontend test` → 36/36 green.
- [ ] 3.2 `pnpm --filter ./frontend build` clean.
