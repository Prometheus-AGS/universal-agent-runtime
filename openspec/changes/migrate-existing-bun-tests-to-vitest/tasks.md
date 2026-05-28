## 1. Import swaps

- [ ] 1.1 `src/index.cursor-policy.test.ts`
- [ ] 1.2 `src/stores/chat-message-store.test.ts`
- [ ] 1.3 `src/entities/runtime-ingest.test.ts`
- [ ] 1.4 `src/features/chat/use-message-stream.test.ts`
- [ ] 1.5 `src/features/chat/use-thread-naming.test.ts`
- [ ] 1.6 `src/admin/pages/skills-page.utils.test.ts`

## 2. Audit

- [ ] 2.1 `git grep -nE "mock\\(|spyOn\\(|mock\\.module\\(" frontend/src/**/*.test.*` and remap each occurrence to `vi.*` equivalents.

## 3. Sweep

- [ ] 3.1 `git grep "bun:test" frontend/src` returns empty.

## 4. Verification

- [ ] 4.1 `pnpm --filter ./frontend test` exits 0 with all 6 tests passing.
