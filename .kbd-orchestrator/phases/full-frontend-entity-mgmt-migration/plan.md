# Plan — `full-frontend-entity-mgmt-migration`

**Date:** 2026-05-26
**Tool:** claude-code (`/kbd-plan`)
**Backend:** OpenSpec (detected at `openspec/`)
**Assessment input:** `.kbd-orchestrator/phases/full-frontend-entity-mgmt-migration/assessment.md`
**Supersedes:** the deferred changes `migrate-admin-knowledge-to-entity-mgmt` and `migrate-providers-models-agents-to-entity-mgmt` from `prometheus-package-integration`.

---

## Decisions locked

| Q | Answer (default applied) | Rationale |
|---|---|---|
| Q1 — threads + runs persistence | `threads` topic streams the existing **`sessions`** SurrealDB table (UAR's chat threads ARE sessions). **No `runs` topic** — runs live in `RunManager` in-memory; reconciliation continues to flow through the existing `/api/chat/completion` SSE stream. | Avoids adding new tables/persistence in this phase. Chat thread sidebar gets live updates; run-state stays where it is. |
| Q2 — store retirement | **Delete-on-migration.** Each PR removes the Zustand store the same time it adds the entity-graph consumer. | Cleaner, no zombie code; risk is contained by per-entity PRs. |
| Q3 — optimistic mutations | **High-frequency only** — toggles, name edits, enabled-flag flips. Heavy ops (creates, deletes) wait for SSE-confirmed state. | Cheap consistency for the gestures users repeat 50×/day; correctness for the ones they do once. |
| Q4 — `api_keys` | **No realtime.** Mutations refetch the list once on success. Never broadcast secrets. | Security boundary. |
| Q5 — chat-message + chat-stream stores | **Out of scope.** Stay as-is; they're not REST caches. | These are the SSE chat ingestion pipeline; replacing them is a different conversation. |

---

## Ordered change list (8 changes)

| # | Change ID | Title | Depends on |
|---|-----------|-------|------------|
| 1 | `bootstrap-entity-engine-and-realtime` | Wire `configureEngine` + `RealtimeManager` into `main.tsx`; gate everything else | — |
| 2 | `services-entities-scaffold` | Create `frontend/src/services/entities/*` for all 13 entity types | 1 |
| 3 | `backend-extend-realtime-topics` | Add `threads` (alias `sessions`), `memory`, `tools`, `compiler_sessions`, `mcp_status` to `EntityTopic`; smoke each | — (can run parallel to 1+2) |
| 4 | `migrate-isolated-pages` | Convert Knowledge, Memory, Auth, Compiler, Tools, MCP-Health to `useEntityList`/`useEntity`; delete their Zustand stores | 1, 2, 3 |
| 5 | `migrate-cross-cutting-pages` | Convert Agents, Providers, Models, Skills, Settings; rewrite every cross-view consumer in the same PR per entity | 4 |
| 6 | `optimistic-mutations` | `useEntityCRUD` patches with optimistic UI for toggles/edits across migrated entities | 5 |
| 7 | `migrate-chat-runtime-derived-state` | Refactor `useChatRuntime`, `useAgentConfig`, agent-selector, session-config-panel onto entity reads | 5 |
| 8 | `frontend-migration-tests-and-audit` | Vitest+RTL "two-views/one-mutation" contract test + `docs/migration-stale-data-audit.md` | 7 |

---

## Per-change synopsis

### 1. `bootstrap-entity-engine-and-realtime`
Single-file change to `frontend/src/main.tsx`:
```tsx
import "@/lib/entity-engine"; // side-effect configureEngine
import { RealtimeManager } from "@prometheus-ags/prometheus-entity-management";
import { createAllUarAdapters } from "@/lib/realtime/topics";

const realtime = new RealtimeManager();
for (const a of createAllUarAdapters()) {
  realtime.registerAdapter(a);
  realtime.subscribe(a, { replayOnConnect: false });
}
```
Acceptance: DevTools shows 7+ EventSource connections on first paint; no visual changes.

### 2. `services-entities-scaffold`
13 thin modules under `frontend/src/services/entities/`, each ~15 LOC:
- `providers.ts`, `models.ts`, `agents.ts`, `skills.ts`, `knowledge-bases.ts`, `knowledge-documents.ts`, `settings.ts`, `memory.ts`, `api-keys.ts`, `tools.ts`, `compiler-sessions.ts`, `mcp-status.ts`, `threads.ts`.

Each exports `fetchEntity(id)` + `fetchList(params?)` plus optional `mutate*` helpers. They wrap the existing `services/*-api.ts` transport modules — **no transport rewrites in this change**.

### 3. `backend-extend-realtime-topics`
Extend `src/uar/realtime/mod.rs::EntityTopic` with:
- `Threads` → table `sessions` (alias)
- `Memory` → table `memory`
- `Tools` → push-only stub (no Surreal table; emits `update` when MCP health refresh detects catalog drift)
- `CompilerSessions` → table `compiler_sessions` (if absent, supervisor parks at max backoff)
- `McpStatus` → push-only stub fed by the existing MCP health loop

Frontend `topics.ts` extends in parallel. Test: write a `sessions` row, SSE event arrives on `/api/live/threads`.

### 4. `migrate-isolated-pages`
Six pages whose entities have no cross-view consumers — minimal blast radius:
- `knowledge-page.tsx` → `useEntityList("knowledge_base")` + `useEntityList("knowledge_document", { kbId })`.
- `memory-page.tsx` → `useEntityList("memory")` (note: search remains a direct service call).
- `auth-page.tsx` → `useEntityList("api_key")` with `refetchOnMutation: true` (no realtime).
- `compiler-page.tsx` → `useEntityList("compiler_session")`.
- `tools-page.tsx` → `useEntityList("tool")`.
- `McpHealthPage.tsx` → `useEntity("mcp_status", "current")`.

Each PR **deletes** the corresponding store file in the same diff.

### 5. `migrate-cross-cutting-pages`
Five entities with cross-view consumers. One PR per entity; the PR migrates EVERY consumer of that entity in one sweep so no stale store reference lingers:
- **Agents**: `agents-page.tsx`, `agent-selector.tsx`, `useAgentConfig`, chat header label, default-agent fallback.
- **Providers**: `providers-page.tsx`, `session-config-panel.tsx`, header provider chip.
- **Models**: `models-page.tsx`, model selector in header/chat, capability-toggles model picker, `useProviderModels` derived hook.
- **Skills**: `skills-page.tsx` (already showing built-in badge from prior phase), capability toggles, agent → skills binding.
- **Settings**: `settings-page.tsx`, `SessionConfigPanel`, global feature toggles (RAG config, sycophancy, etc.).

Done criterion per entity: `git grep "use<Entity>AdminStore\|use<entity>Browse"` returns zero matches.

### 6. `optimistic-mutations`
Apply `useEntityCRUD({ optimistic: true })` to:
- skill toggle (`POST /api/skills/{id}/toggle`)
- agent enable/disable
- provider set-default (`POST /api/uar/providers/{id}/default`)
- settings field edits (`PUT /api/uar/settings/{key}`)
- KB rename

All other CRUD remains non-optimistic. Acceptance: clicking a toggle updates the UI immediately; SSE event arrives within 200 ms and confirms (or, on rare failure, rolls back to server state).

### 7. `migrate-chat-runtime-derived-state`
Refactor `useChatRuntime` and `useAgentConfig` to compose from `useEntity("agent", id)`, `useEntity("model", id)`, and `useEntity("setting", key)` reads instead of store getters. Preserve memoization via `useMemo` over the entity outputs. This is the last and most subtle migration because it touches the busiest call path in the SPA.

### 8. `frontend-migration-tests-and-audit`
- Vitest + RTL "two mounted components subscribe to `useEntity('provider', id)`; synthetic SSE update event → both re-render with new value".
- `docs/migration-stale-data-audit.md` — table of every old fetch site → new entity hook, with check status.
- Update README + `CLAUDE.md` to describe the new data-flow contract.

---

## Risk register

| Risk | Mitigation |
|------|------------|
| Bootstrap PR (#1) ships dormant code paths that don't fire because `RealtimeManager` is mis-instantiated | First PR includes a smoke-only adapter that logs every received `EntityChange` to DevTools console for the first 60 s; remove after migration #5 |
| Per-entity PR (#5) misses a cross-view consumer and leaves a stale Zustand reference | Each PR runs `git grep -E "use(Provider|Agent|Model|Skill|Setting)AdminStore"` as a pre-merge guard |
| `tools` topic has no real Surreal source; push-only stub may diverge from actual MCP state | Tools refresh continues to run via the existing health loop; SSE event is best-effort. Document as such in `docs/realtime.md` |
| Optimistic mutations diverge from server when the server rejects the change | `useEntityCRUD` rolls back on rejection; UI flashes correct value. Add toast for user-visible failures |
| `useChatRuntime` migration breaks the chat hot path | Migrate behind a feature flag (`VITE_ENTITY_MGMT_CHAT_RUNTIME=true`); rollback by env if anything regresses |
| Bundle size grows past comfort | Audit after #7; if >1.6 MB main bundle, split via `manualChunks` for `@prometheus-ags/*` and `lucide-react` |
| Backend `EntityTopic` enum changes break compile sites elsewhere | Add `#[non_exhaustive]` and a panic-safe default in the bus dispatch |

---

## Sources

- Prior phase docs: [`docs/realtime.md`](../../../docs/realtime.md), [`docs/skill-authoring.md`](../../../docs/skill-authoring.md)
- Library exports surveyed in the assessment §2.5 reference [`frontend/packages/prometheus-entity-management/src/`](../../../frontend/packages/prometheus-entity-management/src/)
- [SurrealDB Live Queries — Rust SDK](https://surrealdb.com/docs/sdk/rust/concepts/live)

---

## Progress signal

Completed kbd-plan — full-frontend-entity-mgmt-migration
