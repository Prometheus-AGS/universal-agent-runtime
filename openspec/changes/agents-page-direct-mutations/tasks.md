## 1. patchAgent migration

- [ ] 1.1 Drop `const patchAgent = useAgentsAdminStore((s) => s.patchAgent);` in `AgentMemorySection`.
- [ ] 1.2 Inline (or page-scope helper) optimistic patch: snapshot → upsert → service → rollback on error.

## 2. deleteAgent optimistic

- [ ] 2.1 Capture snapshot in `handleDelete` before the service call.
- [ ] 2.2 `removeEntity("Agent", id)` immediately; service call runs after.
- [ ] 2.3 Re-upsert snapshot on rejection + set local `deleteError`.

## 3. Drop legacy hook reference

- [ ] 3.1 Remove `useAgentsAdminStore` import from `agents-page.tsx`.

## 4. Verification

- [ ] 4.1 `pnpm --filter ./frontend build` clean.
- [ ] 4.2 Manual smoke for both mutations — pending.
