## 1. Reads
- [ ] 1.1 Swap `useKnowledgeAdmin()` → `useKnowledge()` + `useEntityList("Document")` (filter by kbId)
- [ ] 1.2 Hydrate via existing `loadKnowledgeIntoGraph()` on mount

## 2. Mutations — KnowledgeBase
- [ ] 2.1 Create: non-optimistic; refetch after success
- [ ] 2.2 Edit: `optimisticUpsert("KnowledgeBase", id, patch, …)`
- [ ] 2.3 Delete: `optimisticRemove("KnowledgeBase", id, …)`

## 3. Mutations — Document
- [ ] 3.1 Upload: `optimisticUpsert("Document", tempId, { status: "uploading", ... }, () => uploadApi(...))` — actual id arrives via SSE
- [ ] 3.2 Edit metadata: `optimisticUpsert("Document", id, patch, …)`
- [ ] 3.3 Delete: `optimisticRemove("Document", id, …)`

## 4. Store retire
- [ ] 4.1 `git rm frontend/src/stores/knowledge-admin-store.ts`
- [ ] 4.2 `git grep useKnowledgeAdmin frontend/` → empty

## 5. Aesthetic
- [ ] 5.1 Apply terminal tokens
- [ ] 5.2 Upload state uses loading-cursor inline
- [ ] 5.3 Failed-upload state uses error-bar

## 6. Screenshot + audit
- [ ] 6.1 `screenshots/knowledge-page.png`
- [ ] 6.2 Flip both `KnowledgeBase` and `Document` rows to `direct`

## 7. Verification
- [ ] 7.1 36/36; clean build; manual upload smoke (ready transition observed)
