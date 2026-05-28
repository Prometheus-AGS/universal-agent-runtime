## Status — 2026-05-27

**DEFERRED to per-page sessions.** All required infrastructure is shipped and live:

- Engine bootstrap (change 1) — running.
- Per-topic SSE bus (prior phase + change 3) — 10 topics enrolled, `/api/live/{topic}` 200-verified.
- `frontend/src/entities/{fetchers,hooks}/` — 5 entity scaffolds already exist (knowledge, providers, agents, skills, tools).

Each page below is one focused PR: rewrite the page hook, delete the Zustand store, verify cross-tab propagation in the browser. They should land one at a time with a screenshot/test before merge.

## 1. Knowledge

- [ ] 1.1 Rewrite `frontend/src/hooks/use-knowledge-admin.ts` over `useEntityList("knowledge_base")` + per-KB `useEntityList("knowledge_document", { kbId })` from `frontend/src/entities/hooks/use-knowledge.ts`.
- [ ] 1.2 Delete `frontend/src/stores/knowledge-admin-store.ts`.
- [ ] 1.3 Two-tab smoke: upload doc in tab A, list refreshes in tab B.

## 2. Memory

- [ ] 2.1 Add `entities/fetchers/memory.ts` + `entities/hooks/use-memory.ts`.
- [ ] 2.2 Migrate `memory-page.tsx`; delete `memory-admin-store.ts`.

## 3. Auth

- [ ] 3.1 Add `entities/fetchers/api-keys.ts` + `entities/hooks/use-api-keys.ts` (non-realtime).
- [ ] 3.2 Migrate `auth-page.tsx`; delete `auth-keys-store.ts`.

## 4. Compiler

- [ ] 4.1 Add `entities/fetchers/compiler-sessions.ts` + hook.
- [ ] 4.2 Migrate `compiler-page.tsx`; delete `compiler-sessions-store.ts`.

## 5. Tools

- [ ] 5.1 Migrate `tools-page.tsx` to existing `entities/hooks/use-tools.ts`; delete `tools-discovery-store.ts`.

## 6. MCP Health

- [ ] 6.1 Add `entities/fetchers/mcp-status.ts` + hook.
- [ ] 6.2 Migrate `McpHealthPage.tsx`; delete `mcp-health-store.ts`.
