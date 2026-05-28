## 1. Reads
- [ ] 1.1 Swap `useMemoryAdmin()` → `useMemory()`
- [ ] 1.2 Hydrate via `loadMemoryIntoGraph()` on mount
- [ ] 1.3 Keep `/api/admin/memories/search` as direct call (one-shot)

## 2. Mutations
- [ ] 2.1 Wrap edits in `optimisticUpsert("Memory", id, patch, …)`
- [ ] 2.2 Wrap deletes in `optimisticRemove("Memory", id, …)`

## 3. Store retire
- [ ] 3.1 `git rm frontend/src/stores/memory-admin-store.ts`
- [ ] 3.2 `git grep useMemoryAdmin frontend/` → empty

## 4. Aesthetic
- [ ] 4.1 Apply terminal tokens; use shared empty-frame/loading-cursor/error-bar
- [ ] 4.2 Banned-font grep clean

## 5. Screenshot + audit
- [ ] 5.1 Playwright `screenshots/memory-page.png`
- [ ] 5.2 Flip `Memory` row to `direct`

## 6. Verification
- [ ] 6.1 36/36; clean build; manual search smoke
