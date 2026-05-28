## 1. KB list

- [ ] 1.1 Replace ad-hoc fetch in `knowledge-page.tsx` with `useEntityList("knowledge_base")`. — **pending**, needs RealtimeManager wiring in `main.tsx` first.
- [ ] 1.2 Implement `fetchList("knowledge_base")` calling `GET /api/knowledge`.
- [ ] 1.3 Normalize response → `KnowledgeBase` entity shape.

## 2. Document list

- [ ] 2.1 Implement `fetchList("knowledge_document", { kbId })` calling `GET /api/knowledge/{kbId}/documents`.
- [ ] 2.2 Hook into the KB detail view.

## 3. Count display

- [ ] 3.1 Prefer `docs.length`; fall back to server `document_count` (already wired from change 1).

## 4. Upload flow

- [ ] 4.1 Verify SSE bus drives upsert (already smoke-tested in change 2 — same flow).

## 5. Tests / Verification

- [ ] 5.1 Manual two-tab propagation test.
- [ ] 5.2 Manual upload-without-refresh test.
- [ ] 5.3 Screenshot diff.

---

## Status — 2026-05-26

**Infrastructure complete, consumer wiring deferred.**

All upstream prerequisites have shipped in this session:

- Live-query bus + SSE `/api/live/knowledge_documents` (change 2 — smoke-verified).
- Doc-count backend field (change 1 — live-verified).
- Entity engine bootstrap at `frontend/src/lib/entity-engine.ts` (change 9).
- SSE adapter at `frontend/src/lib/realtime/uar-sse-adapter.ts` + topic map (change 9).

The remaining work is mechanical but invasive: wire `RealtimeManager` into `main.tsx`, add fetchers under `frontend/src/services/entities/`, and rewrite `knowledge-page.tsx` to consume hooks. This belongs in a fresh session focused on the frontend so each rewrite can be verified in the browser before moving to the next page.
