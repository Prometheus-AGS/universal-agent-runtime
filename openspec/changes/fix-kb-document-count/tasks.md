## 1. Backend

- [x] 1.1 Add `document_count: usize` to `KnowledgeBaseResponse` in `src/uar/api/knowledge.rs:62`.
- [x] 1.2 Extend `PersistenceLayer::count_documents(kb_id)` with a default impl in `src/uar/persistence/mod.rs`.
- [x] 1.3 Surreal provider: implement count via `SELECT count() AS c FROM knowledge_documents WHERE kb_id = $kb_id GROUP ALL`.
- [x] 1.4 Postgres provider: implement count via `SELECT COUNT(*)::bigint`.
- [x] 1.5 Update `kb_to_response(kb, count)` signature; all 4 call sites pass actual counts.
- [x] 1.6 Verified same field populated in single-KB endpoints (`GET /api/knowledge/{id}` and the settings-mirror variant via `kb_to_response`).

## 2. Tests

- [ ] 2.1 Integration test — deferred to `integration-tests-and-docs` change.
- [ ] 2.2 Cover `N=0`, `N=1`, `N=5` cases — deferred.

## 3. Verification

- [x] 3.1 Manual: `curl /api/knowledge` against running UAR returns `document_count: 1` for the KB with one indexed README.md.
