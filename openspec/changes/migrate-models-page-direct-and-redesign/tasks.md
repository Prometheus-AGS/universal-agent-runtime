## 1. Reads
- [ ] 1.1 Swap `useModelsBrowse()` → `useModels()` in `models-page.tsx`
- [ ] 1.2 `useEffect` hydrates via `loadModelsIntoGraph()` on mount

## 2. Mutations
- [ ] 2.1 Wrap refresh/favorite mutations in `optimisticUpsert("Model", id, patch, () => …)`
- [ ] 2.2 Remove path uses `optimisticRemove("Model", id, …)` if applicable

## 3. Store retire
- [ ] 3.1 `git rm frontend/src/stores/models-browse-store.ts`
- [ ] 3.2 `git grep useModelsBrowse frontend/` → empty

## 4. Aesthetic
- [ ] 4.1 Apply terminal tokens (--terminal-bg, --phosphor, --amber)
- [ ] 4.2 Use flicker-cursor loading + ASCII empty + mono-error patterns
- [ ] 4.3 Focus ring 2px phosphor-green on all interactive
- [ ] 4.4 Banned-font grep clean in newly written CSS

## 5. Screenshot
- [ ] 5.1 `pnpm --filter ./frontend exec playwright test --grep @models-page-visual`
- [ ] 5.2 Commit `.kbd-orchestrator/phases/direct-entity-migration-models/screenshots/models-page.png`

## 6. Audit doc
- [ ] 6.1 Flip Model row in `docs/migration-stale-data-audit.md` to `direct`

## 7. Verification
- [ ] 7.1 `pnpm --filter ./frontend test` → 36/36
- [ ] 7.2 `pnpm --filter ./frontend build` clean
- [ ] 7.3 `git grep "useGraphStore.getState" frontend/src/admin/pages/models-page.tsx` empty
