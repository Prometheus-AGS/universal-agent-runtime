# Stale-data audit — frontend entity-mgmt migration

> **See also:** [README → Frontend Architecture — Realtime Entity Graph](../README.md#frontend-architecture--realtime-entity-graph) for the executive summary diagram and the canonical pattern index.

Tracking every old REST fetch site and what backs its freshness.

Status legend:
- **`bridged`** — Zustand store kept; auto-refreshes via `useGraphBridge` on every SSE event delivered to the entity graph. No stale data; consumer untouched.
- **`direct`** — Page reads via `useEntity*` directly; store deleted.
- **`pending`** — Still on its old Zustand store with no realtime trigger.
- **`non-realtime`** — Deliberately not bridged (security or polling).

Last refreshed: 2026-05-27 (post-bridge migration).

## Infrastructure

| Item | Status | Location |
|------|--------|----------|
| `configureEngine` bootstrap | ✅ done | `frontend/src/entities/bootstrap.ts` |
| `RealtimeManager` wiring | ✅ done | `frontend/src/entities/sync.ts` — surreal-remote branch uses `createAllUarAdapters()` |
| SSE bus backend | ✅ done | `src/uar/realtime/{mod,surreal_bus}.rs` |
| `/api/live/{topic}` endpoint | ✅ done | `src/uar/api/live.rs` |
| Topic catalogue (10) | ✅ done | `EntityTopic::ALL` + `UAR_TOPICS` |
| **`useGraphBridge` helper** | ❌ **permanently retired** | deleted in `add-push-channels-backend` phase (2026-05-27); see "Historical: bridge pattern" appendix |

## Direct migration playbook (canonical)

The destination pattern. Every entity migration follows these steps:

1. **Add fetcher + hook** under `frontend/src/entities/{fetchers,hooks}/`. The fetcher upserts rows into the entity graph; the hook is a thin `useEntityList` / `useEntityView` / `useEntity` wrapper.
2. **Swap reads** on the admin page to the new hook. Hydrate via the fetcher in `useEffect(() => { void load(); }, [])`.
3. **Wrap mutations** in `optimisticUpsert` / `optimisticRemove` from `@/lib/realtime/optimistic`. Local error state via `useState`.
4. **Retire the legacy hook + store** (`git rm hooks/use-xxx-admin.ts stores/xxx-admin-store.ts`).
5. **Apply terminal aesthetic** per `docs/admin-aesthetic-spec.md` — shared components in `components/admin/{loading-cursor,empty-frame,error-bar}.tsx`.
6. **Flip the audit row** from `bridged` → `direct` and capture a Playwright screenshot under `.kbd-orchestrator/phases/<phase>/screenshots/<page>.png`.

Per-change verification: `pnpm --filter ./frontend test ≥ 36/36`, `pnpm --filter ./frontend build` clean, `git grep "useGraphStore.getState" frontend/src/admin/pages/<page>` empty.

### Form-cache pattern (for pages with dirty/save semantics)

Pages with form-edit workflows (settings, complex per-field validation) can't rely solely on `optimisticUpsert` — they need transient `dirty` state separate from the authoritative graph. The pattern:

1. **Author a per-namespace module-level cache.** Example: `frontend/src/hooks/settings-form-cache.ts` — `Map<namespace, { values, saving, error }>` plus `subscribe(ns, cb)` listener fan-out.
2. **Consume via `useSyncExternalStore`** in the entity hook (`use-settings.ts`). State survives component re-mount within a session, which Zustand previously provided.
3. **Compute `conflicts` lazily** by diffing dirty vs graph values inside `useMemo`. No stored conflict state needed.
4. **Save = optimistic upsert (all dirty keys) → bulk POST → on success refetch, on failure restore snapshots → clear dirty.**

This pattern replaced the 242 LOC `settings-store.ts` with ~80 LOC of cache + ~150 LOC of hook code. Contract test at `frontend/src/hooks/__tests__/use-settings.test.tsx`.

### SSE-reconciler pattern (for client-first entities)

When an entity is created locally first (offline-first) but the server also writes a corresponding row, the graph isn't the source of truth — it's a **secondary signal** that needs to be reconciled into a local store. The pattern:

1. **Keep the local store** (Zustand + PGlite/IndexedDB) as authoritative.
2. **Author a `use<Entity>GraphSync` hook** that subscribes to `useGraphStore` selecting the entity's slice, computes new/updated/removed deltas by diffing keysets, and calls the local store's existing mutation API per delta.
3. **Mount once at the SPA root** (`App.tsx`).
4. **No REST refetch** if no list endpoint exists — live-only sync is acceptable; the client is authoritative for creation.

This is the inverse of the direct migration playbook (which makes the graph authoritative). Used for `Thread` because thread creation is a client-first action with PGlite persistence for offline; the server's `sessions` row arrives later as metadata.

Contract test at `frontend/src/stores/__tests__/use-thread-graph-sync.test.tsx`.

### CI gates (enforced)

Every PR runs `scripts/ci-grep-gates.sh` plus the standard frontend pipeline. The job is `.github/workflows/ci.yml::frontend`.

| Gate | Enforcement |
|------|-------------|
| `pnpm --filter ./frontend typecheck` | TypeScript errors fail the job |
| `pnpm --filter ./frontend test` | Vitest must report ≥ 40/40 |
| `pnpm --filter ./frontend build` | Vite build must succeed |
| `git grep useGraphBridge frontend/` empty | Bridge pattern permanently retired |
| `git grep useSettingsStore frontend/` empty | Settings store retired |
| `git grep -E "\b(Inter\|Roboto\|Arial\|Space Grotesk)\b" frontend/src/admin/` empty | Banned-fonts contract from `docs/admin-aesthetic-spec.md` |
| `git grep "outline:\s*none" frontend/src/admin/` empty | A11y contract — focus rings must be authored, not stripped |

Run locally before push: `pnpm run ci-gates` (alias for `bash scripts/ci-grep-gates.sh`). The grep gates output line numbers on failure so contributors can navigate straight to the offending site.

Status: **informational for the first week after rollout** (2026-05-27); promoted to required after one clean merge cycle.

### Historical: bridge pattern (PERMANENTLY RETIRED 2026-05-27)

To deliver "no stale data anywhere" without rewriting every admin page in one go, the interim **bridge pattern** had each Zustand-backed admin hook subscribe to `useGraphStore` for its entity types via `useGraphBridge`. When the SSE-fed graph received a mutation, the bridge called the store's `load()` action, refreshing it from REST.

The bridge was a stepping stone, not the destination. The `direct-entity-migration-models` phase retired the bridge for 7 of 8 originally bridged entities (Tool is deferred until its push channel ships, then it goes direct too). The bridge file remains in the tree marked `@deprecated`, scheduled for deletion in `tool-mcp-status-push-channels`.

## Entity inventory

| Entity | Realtime topic | Hook | Bridge | Status |
|--------|----------------|------|--------|--------|
| `KnowledgeBase` + `Document` | `knowledge_bases`, `knowledge_documents` | `entities/hooks/use-knowledge-page.ts` (direct compat hook) | — (retired) | `direct` — page kept its existing chrome; data layer fully on graph + optimistic helpers |
| `Provider` | `providers` | `admin/pages/providers-page.tsx` (direct) | — (retired) | `direct` |
| `Agent` | `agents` | `admin/pages/agents-page.tsx` + `features/chat/agent-selector.tsx` (direct) | — (retired) | `direct` — also fixed a latent SSE-blindness bug in the chat sidebar |
| `Model` | `models` | `admin/pages/models-page.tsx` (direct) | — (retired) | `direct` |
| `Skill` | `skills` | `admin/pages/skills-page.tsx` (direct) | — (retired) | `direct` |
| `Setting` | `settings` | `hooks/use-settings.ts` (direct; module-level form cache) | — (retired) | `direct` — graph for reads/realtime; `settings-form-cache.ts` for dirty/saving state; change-bus emitted from SSE adapter |
| `Tool` | (no SSE — registry is static after startup) | `admin/pages/tools-page.tsx` (direct, one-time fetch on mount) | — (retired) | `direct` |
| `Memory` | `memory` | `admin/pages/memory-page.tsx` (direct) | — (retired) | `direct` |
| `CompilerSession` | `compiler_sessions` | `admin/pages/compiler-page.tsx` (direct) | — (retired) | `direct` |
| `Thread` | `threads` (alias `sessions`) | `stores/use-thread-graph-sync.ts` (SSE → registry reconciler) + `stores/thread-registry-store.ts` (client-first PGlite cache) | — (no bridge needed; direct subscription) | `direct` — server SSE events reconcile into the local registry; threads remain client-authoritative for offline-first creation |
| `ApiKey` | (intentionally none) | `hooks/use-auth-keys.ts` | — | `non-realtime` (refetch on mutation; never broadcast secrets) |
| `McpStatus` | (no SSE — health probes are server-process-local) | `admin/McpHealthPage.tsx` (direct, 30 s poll hydrates graph) | — | `direct` (graph reads + polling source) |

## Non-graph data flows (kept as-is)

- `chat-message-store` — client-side SSE-driven chat message buffer; not a REST cache.
- `chat-stream-store` — chat stream event multiplexer; not a REST cache.
- Search endpoints (`/api/knowledge/{id}/search`, `/api/admin/memories/search`) — one-shot queries; remain direct service calls.

## Next steps (out of scope for this session)

- Direct-`useEntity` migration per page to retire the bridge + store layer entirely. Pages with cross-view consumers (Agents, Providers, Models, Skills, Settings) should each be one PR sweeping all consumers.
- `Thread` topic wiring into the chat sidebar.
- Push channels for `Tool` and `McpStatus`.
- Optimistic mutations on high-frequency paths.
