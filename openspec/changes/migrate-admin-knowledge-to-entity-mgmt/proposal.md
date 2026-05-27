## Why

The Admin Knowledge view is the lowest-risk pilot for the entity-mgmt migration: the page is small, its data is well-modeled (KBs + Documents), and it's where the doc-count bug originally manifested. Proving the realtime flow here unblocks the broader rollout.

## What Changes

- `frontend/src/admin/pages/knowledge-page.tsx` replaces its bespoke fetcher with:
  ```ts
  const { items: kbs } = useEntityList("knowledge_base");
  ```
- KB detail (drill-in) uses `useEntityList("knowledge_document", { where: { kb_id } })`.
- `document_count` display priority:
  1. Live list length (`docs.length`) once the documents list resolves.
  2. Fallback to the server-supplied `kb.document_count` from change `fix-kb-document-count` for first paint.
- Upload flow continues to POST as today; the live bus auto-emits `create` on `knowledge_documents`, which the adapter writes into the graph → both the doc list and the KB count update without explicit refetch.

## Acceptance

- Upload a document → KB doc count increments without a page refresh.
- Delete a document → count decrements without a refresh.
- Two browser tabs open to the Knowledge page reflect each other's mutations within ~200 ms.
