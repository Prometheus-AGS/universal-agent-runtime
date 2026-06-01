## Why

The Admin → Knowledge view displays `0 documents` for every knowledge base even when documents are indexed. Root cause: `KnowledgeBaseResponse` (`src/uar/api/knowledge.rs:62`) lacks a `document_count` field, but the frontend (`frontend/src/admin/pages/knowledge-page.tsx:303`) reads `kb.document_count ?? 0`, so the count silently falls back to zero. This is the smallest user-visible bug in the phase and is independent of the broader refactor.

## What Changes

- Add `document_count: usize` to `KnowledgeBaseResponse`.
- Surreal provider: aggregate via `SELECT *, array::len((SELECT id FROM knowledge_documents WHERE kb_id = $parent.id)) AS document_count FROM knowledge_bases` (or N+1 with caching — implementer's call).
- Postgres provider: same field via `LEFT JOIN ... GROUP BY` or subquery.
- Backfill `kb_to_response()` in `src/uar/api/knowledge.rs` to populate the field.
- No frontend change required (already consumes the field).

## Acceptance

- `curl /api/knowledge` returns `document_count: N` matching the row count in `knowledge_documents WHERE kb_id = …`.
- Admin → Knowledge shows the correct number.
- Unit test against an in-memory Surreal seeded with 0, 1, 5 documents.
