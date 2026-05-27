# Assessment — `full-frontend-entity-mgmt-migration`

**Date:** 2026-05-26
**Tool:** claude-code (`/kbd-assess`)
**Replaces:** the deferred changes 10 + 11 of `prometheus-package-integration`
**Project source of truth:** `.kbd-orchestrator/`
**Memory mirror:** `surreal_memory` MCP at `/mcp/memory` (non-authoritative)

---

## 1. Phase goal

Replace **every** REST-fetching surface in the React SPA with `useEntity` / `useEntityList` / `useEntityCRUD` from `@prometheus-ags/prometheus-entity-management`, route all change events through the realtime SSE adapter into a single graph store, and retire every Zustand admin store that today exists solely to cache REST data. The contract is: **no view in the SPA ever displays stale data**, full stop.

---

## 2. Current state inventory

### 2.1 Frontend surfaces with REST data fetches (11 admin pages + 6 cross-cutting consumers)

| # | Surface | File | Endpoints | Entity types | Cross-view? |
|---|---------|------|-----------|--------------|-------------|
| 1 | Providers admin | `admin/pages/providers-page.tsx` | `/api/catalog`, `/api/uar/providers` | `provider` | **yes** (header) |
| 2 | Models admin | `admin/pages/models-page.tsx` | `/api/models` | `model` | **yes** (header, session config) |
| 3 | Agents admin | `admin/pages/agents-page.tsx` | `/api/agents` | `agent` | **yes** (agent selector, chat header) |
| 4 | Skills admin | `admin/pages/skills-page.tsx` | `/api/skills` | `skill` | **yes** (capability toggles) |
| 5 | Knowledge admin | `admin/pages/knowledge-page.tsx` | `/api/knowledge`, `/api/knowledge/{id}/documents`, `/api/knowledge/{id}/search` | `knowledge_base`, `knowledge_document` | no |
| 6 | Settings admin | `admin/pages/settings-page.tsx` | `/api/uar/settings/*` | `setting` | **yes** (global toggles, RAG config) |
| 7 | Memory admin | `admin/pages/memory-page.tsx` | `/api/admin/memories*` | `memory` | no |
| 8 | Auth admin | `admin/pages/auth-page.tsx` | `/api/auth/keys` | `api_key` | no |
| 9 | Tools admin | `admin/pages/tools-page.tsx` | `/api/tools` | `tool` | **yes** (tool registry, UI visibility) |
| 10 | Compiler admin | `admin/pages/compiler-page.tsx` | `/api/compiler/sessions` | `compiler_session` | no |
| 11 | MCP health admin | `admin/McpHealthPage.tsx` | `/api/uar/mcp/health` | `mcp_status` | **yes** (top-nav badge) |
| 12 | Agent selector | `features/chat/agent-selector.tsx` | `/api/agents` | `agent` | — |
| 13 | Session config panel | `features/chat/session-config-panel.tsx` | `/api/uar/providers`, `/api/models` | `provider`, `model` | — |
| 14 | Chat runtime hook | `features/chat/use-chat-runtime.ts` | uses `useAgentConfig`, `useProviderModels` | derived | — |
| 15 | Left sidebar | `components/layout/left-sidebar.tsx` | thread mgmt | `thread` | — |
| 16 | Top nav | `components/layout/top-nav.tsx` | MCP health, settings | `mcp_status`, `setting` | — |
| 17 | Attachment manager | `features/chat/use-attachment-manager.ts` | knowledge upload | `knowledge_document` | — |

### 2.2 Zustand admin stores (13 total)

Each one wraps REST in `set/get` state. **Targeted for retirement** as the entity graph becomes the single source of truth:

- `providers-admin-store.ts` ⟶ replace with `useEntityList("provider")` + `useEntityCRUD("provider")`.
- `agents-admin-store.ts` ⟶ replace with `useEntityList("agent")` + …
- `skills-admin-store.ts` ⟶ replace with `useEntityList("skill")` + …
- `models-browse-store.ts` ⟶ replace with `useEntityList("model")`.
- `knowledge-admin-store.ts` ⟶ replace with KB + document list hooks.
- `settings-store.ts` ⟶ replace with `useEntityList("setting", { namespace })`.
- `memory-admin-store.ts` ⟶ replace with `useEntityList("memory")` (after adding realtime topic).
- `auth-keys-store.ts` ⟶ replace with `useEntityList("api_key")` (after adding topic).
- `tools-discovery-store.ts` ⟶ replace with `useEntityList("tool")` (after adding topic).
- `compiler-sessions-store.ts` ⟶ replace with `useEntityList("compiler_session")` (after adding topic).
- `mcp-health-store.ts` ⟶ replace with `useEntity("mcp_status", "current")` (after adding topic).
- `chat-message-store.ts` ⟶ **keep** (client-side message buffer; not a REST cache).
- `chat-stream-store.ts` ⟶ **keep** (SSE streaming events into message store).
- `thread-registry-store.ts` ⟶ replace with `useEntityList("thread")` (after adding topic).
- `thread-title-store.ts` ⟶ keep as a thin transient title-derivation utility.

### 2.3 Service modules (15) and their endpoints (~70 routes)

`agents-api.ts`, `providers-api.ts`, `models-api.ts`, `skills-api.ts`, `knowledge-api.ts`, `settings-api.ts`, `memory-api.ts`, `auth-api.ts`, `tools-api.ts`, `compiler-api.ts`, `mcp-api.ts`, `chat-stream-api.ts`, `chat-titles-api.ts`, `run-tools-api.ts` (+ a couple more).

These remain — they're the **transport** layer that `fetchEntity`/`fetchList` will call into. The migration **does not delete** service modules; it deletes the *store wrappers* around them.

### 2.4 Realtime spine — what already exists

Shipped in the previous phase (`prometheus-package-integration`):

- **Backend bus** (`src/uar/realtime/surreal_bus.rs`): `db.select(table).live()` per topic → `tokio::broadcast` → SSE `/api/live/{topic}`. Seven topics enrolled: `knowledge_bases`, `knowledge_documents`, `agents`, `providers`, `models`, `skills`, `settings`. **Smoke-tested end-to-end** — events arrive ≤200 ms after DB write.
- **Frontend adapter** (`frontend/src/lib/realtime/uar-sse-adapter.ts`): `EventSource` → `EntityChange` published to graph via the lib's `RealtimeAdapter` contract.
- **Engine config** (`frontend/src/lib/entity-engine.ts`): `configureEngine(...)` with locked defaults (30 s stale, 5 min GC, focus + reconnect revalidation).
- **Topic catalogue** (`frontend/src/lib/realtime/topics.ts`): the 7 topics + `createAllUarAdapters()` one-call helper.

### 2.5 Realtime spine — what's missing for "everything"

Five backend topics still need to be added to `EntityTopic::ALL` and to the SurrealDB stream supervisor:

- `threads` — chat thread list + titles (critical for multi-tab freshness).
- `runs` — run status updates, tool-approval state changes (critical for chat-side reconciliation).
- `memory` — memory admin deletes (optional — admin-only scope).
- `tools` — discovered MCP tool catalog (medium — affects capability toggles).
- `compiler_sessions` — ephemeral but useful for multi-tab.
- `api_keys` — intentionally excluded for security (never broadcast key material).
- `mcp_status` — push-only health gauge (could ride a separate non-table-backed live channel).

### 2.6 SPA bootstrap — what's NOT yet wired

- `main.tsx` does **not** import `entity-engine.ts` (side-effect `configureEngine`).
- `main.tsx` does **not** instantiate `RealtimeManager` and register the adapters returned by `createAllUarAdapters()`.

Without these two lines, the infrastructure shipped last phase is **dormant**. The first execution step of this phase is to wire them.

---

## 3. Definition of done

| # | Criterion | Verification |
|---|-----------|--------------|
| A1 | `RealtimeManager` is instantiated once in `main.tsx`; all SSE adapters subscribed before React renders. | DevTools Network panel shows 7+ `EventSource` connections on first paint |
| A2 | Every admin page in §2.1 reads via `useEntityList` / `useEntity`. | grep `frontend/src` shows zero direct `fetch(` calls outside `services/entities/*` |
| A3 | All 13 entity types listed in §2.2 have a `services/entities/<type>.ts` with `fetchEntity` + `fetchList` (+ optional `mutate*` helpers). | file exists, exported |
| A4 | Every retired Zustand store is deleted (or reduced to a single-line re-export adapter for backwards compat during a transition window — pick policy). | grep `useProvidersAdminStore` etc. shows no remaining call sites |
| A5 | Cross-view propagation verified: editing a provider in Admin updates the chat-header model badge in another tab ≤200 ms. | manual two-tab test |
| A6 | Backend `EntityTopic::ALL` extended with `threads` + `runs` + (optional) `memory`, `tools`, `compiler_sessions`. SSE smoke-tested. | curl `-N /api/live/threads` + DB write |
| A7 | Mutation endpoints invoked via `useEntityCRUD` perform optimistic updates and reconcile against the SSE-delivered authoritative state. | DevTools timeline shows write → optimistic patch → server SSE confirms |
| A8 | `chat-message-store` + `chat-stream-store` remain intact (out of scope — they're not REST caches). | code unchanged |
| A9 | Migration audit doc lists every old fetch site and what it was replaced with. | `docs/migration-stale-data-audit.md` exists |
| A10 | New phase progress reflects completion; current-waypoint advances to next phase. | files updated |

---

## 4. Gap analysis

| ID | Gap | Severity | Notes |
|----|-----|----------|-------|
| G1 | `main.tsx` doesn't import `entity-engine.ts` or wire `RealtimeManager`. | **Critical** (blocks everything) | Single-file edit; do this first. |
| G2 | No `services/entities/*` modules exist yet. The library expects host-supplied `fetch`+`normalize`. | **High** | 13 thin modules to author — one per entity type. Each is ~15 LOC. |
| G3 | Backend `EntityTopic` set lacks `threads` and `runs`. | **High** | `threads` is critical for sidebar multi-tab consistency; `runs` for chat reconciliation. |
| G4 | No backend `threads` table or `runs` table in SurrealDB — the SSE live-query depends on Surreal storing the rows. | **High** | Inspect: does UAR persist threads + runs to Surreal? If not, add persistence first OR build push-only live channels for these. |
| G5 | 13 Zustand stores duplicate REST caches with bespoke loading flags and error states. Migration order matters — touching the agent store breaks the agent selector, etc. | **High** | Sequence: stores with no cross-view consumers first (Knowledge, Memory, Auth, Compiler), then cross-cutting ones (Agents, Providers, Models, Skills, Settings, MCP health). |
| G6 | Mutation paths today do not perform optimistic updates — they POST and re-fetch the list. With realtime in place, mutate→SSE→reconcile is the new contract. | Med | `useEntityCRUD` supports optimistic patches; need a consistent pattern across services. |
| G7 | Tool-approval and run-status flows are stream-side — they belong on `runs` topic but also flow through the existing chat-stream channel. Risk of dual updates. | Med | Decide single source of truth per event class; document the boundary. |
| G8 | `auth_keys` deliberately excluded from realtime (never broadcast secrets). API needs a `mutate-then-refetch-once` fallback path for `useEntityList("api_key")`. | Low | Document the explicit non-realtime contract. |
| G9 | `tools` entity is discovery-only and refreshes via MCP health checks, not direct DB writes. Need to decide whether `/api/live/tools` should be a poll-shim or a real Surreal stream. | Low | Probably poll-shim on the server side. |
| G10 | Frontend bundle size will grow with entity-mgmt's GraphStore + per-entity normalizers; current bundle is already ~1.4 MB. | Low | Verify after migration; potentially split via `manualChunks`. |
| G11 | No tests cover the entity-mgmt migration yet. | Med | Vitest + RTL "two views, one mutation, both re-render" contract test. |
| G12 | Service modules currently bake JWT auth into headers via `fetch` directly. Need to confirm `fetchEntity`/`fetchList` honors the same auth interceptor. | Med | Trivial — services pass already-authenticated `fetch` impl. |
| G13 | `useChatRuntime` and `useAgentConfig` derive composite state from multiple entities. They need to switch from store reads to `useEntity` reads, preserving memoization semantics. | Med | Likely the most subtle migration of all. |

---

## 5. Sequencing recommendation

1. **G1 — main.tsx wiring** (single PR, single file, unblocks every other change). Smoke-test: 7 `EventSource` connections visible in DevTools.
2. **G2 — services/entities/* scaffolds**, all 13 modules at once. Pure additive — no consumer yet. Type-check passes.
3. **G3 + G4 — backend `threads` + `runs` topics**. Verify Surreal persistence exists for both; if not, persist them first.
4. **Low-cross-view stores first** (G5 batch A): Knowledge, Memory, Auth, Compiler, Tools, MCP-health pages. Easy wins, low blast radius.
5. **Cross-view stores next** (G5 batch B): Agents → Providers → Models → Skills → Settings. Each PR migrates ALL consumers of that entity in a single sweep so no stale Zustand reference lingers.
6. **G6 — optimistic mutations** rolled out per-entity alongside the store migration.
7. **G13 — `useChatRuntime` + `useAgentConfig`** refactor — last, because they sit at the busiest call site.
8. **G11 — tests + audit doc** (`docs/migration-stale-data-audit.md`).
9. **Bundle audit** (G10) — split chunks if needed.

---

## 6. Open questions for the user before planning

1. **Threads + runs persistence:** are chat threads and runs currently persisted to SurrealDB (`threads` / `runs` tables), or only held in memory? This determines whether we add live-query topics directly or need to add persistence + topics together.
2. **Zustand store retirement policy:** delete-on-migration (cleaner, larger blast radius per PR) or keep as one-line re-export shims for one release cycle (safer, more code in transit)?
3. **Optimistic mutation strategy:** for every entity, or only for high-frequency ones (toggles, name edits)? Optimistic everywhere is more consistent but more code.
4. **`auth_keys` handling:** confirm "no realtime, refetch on mutation" is acceptable — secrets never flow over SSE.
5. **Out-of-scope confirm:** `chat-message-store` + `chat-stream-store` stay as-is (they're not REST caches and serve real-time SSE chat events independently). Yes?

---

## 7. Progress signal

Completed kbd-assess — full-frontend-entity-mgmt-migration
