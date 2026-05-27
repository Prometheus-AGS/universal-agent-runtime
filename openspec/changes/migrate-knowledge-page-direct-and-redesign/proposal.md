# migrate-knowledge-page-direct-and-redesign

## Why
Largest non-settings page (782 LOC), two entity types (KnowledgeBase + Document). Retire `knowledge-admin-store`; adopt terminal aesthetic.

## What changes
- Reads via `useKnowledge()` + a new `useDocuments(kbId)` if needed.
- Upload mutation does optimistic insert with `status: "uploading"`; SSE event updates `status: "ready" | "failed"`.
- Edit/delete via `optimisticUpsert/Remove` on both `KnowledgeBase` and `Document`.
- Delete `frontend/src/stores/knowledge-admin-store.ts`.
- Apply aesthetic; screenshot; audit flip (both rows).

## Impact
Document upload UX gains immediate row + status progression; no perceived latency.
