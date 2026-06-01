# Assessment — `thread-topic-chat-sidebar`

**Date:** 2026-05-27
**Tool:** claude-code (`/kbd-assess`)
**Prior phase:** `add-push-channels-backend` (100%) — bridge pattern fully retired
**Goal:** wire the last `pending` entity row from the audit doc — the `Thread` SSE topic into the chat sidebar so server-side session events update local thread state automatically.

---

## 1. Phase goal

The `Thread` topic (alias of the SurrealDB `sessions` table) is **enrolled in the realtime bus** (`src/uar/realtime/mod.rs::EntityTopic::Threads`) and broadcasts live events, but **no frontend consumer subscribes**. The audit doc marks it `pending`.

`Thread` is structurally unlike the other migrated entities:

- **Threads are client-first.** They're created in the SPA, persisted to PGlite via `lib/db.ts`, and tracked in `stores/thread-registry-store.ts` (Zustand). They survive offline.
- **No `/api/sessions` REST endpoint.** The legacy route is disabled (`server.rs:584`). There is no list/get/refetch path; the only way to learn about server sessions is via the SSE Thread topic.
- **Backend writes the `sessions` row** when a chat session is persisted server-side (after the first message lands, the orchestrator writes the SurrealDB row). The frontend's locally-created Thread already exists at that point; the server event is a metadata update rather than a discovery.

So the wiring is **not** "replace the local store with graph reads" (the previous playbook). It's "subscribe to the Thread topic and reconcile incoming server events into the local store" — a thin one-directional bridge from graph → registry.

### Out of scope

- Adding a REST endpoint for sessions. Future-phase concern; not needed today because client-first creation is the authoritative path.
- Cross-device sync. Today threads are local-to-this-browser. Multi-device threads would need a server-side listing + sync flow.
- Server-side thread deletion. If we want server deletes to also wipe the local PGlite copy, we need a `delete` handler in the new subscription — included as DoD criterion below.

---

## 2. Current state inventory

| Component | Path | State |
|---|---|---|
| Thread topic enrollment | `src/uar/realtime/mod.rs` | ✅ enrolled; alias `sessions` |
| SSE adapter for `threads` | `frontend/src/lib/realtime/topics.ts` | ✅ already in `UAR_TOPICS` |
| Backend SSE endpoint | `src/uar/api/live.rs` | ✅ `/api/live/threads` |
| `thread-registry-store` | `frontend/src/stores/thread-registry-store.ts` | client-first; PGlite-backed; **no SSE listener** |
| `left-sidebar` | `frontend/src/components/layout/left-sidebar.tsx` | reads from registry; no graph subscription |
| Entity graph receives `Thread` events | (via `createAllUarAdapters`) | ✅ events arrive but nobody consumes them |

The entity graph already receives events; we just need to bridge from `useGraphStore["Thread"]` mutations into the registry store. This is essentially a one-line subscription wired in the registry hook itself or in `App.tsx` at mount.

---

## 3. Definition of done

| # | Criterion | Verification |
|---|---|---|
| F1 | A `useThreadGraphSync()` hook (or equivalent) is mounted in `App.tsx` (or `chat-page.tsx`) and reconciles `Thread` graph events into the registry store | grep |
| F2 | Server insert/update of a `Thread` row that matches a known local thread → registry `setTitle` / `touch` applied (preserves local `isEphemeral` flag) | unit test |
| F3 | Server insert of a `Thread` row that does NOT exist locally → registry creates a new persisted (non-ephemeral) entry | unit test |
| F4 | Server delete event for a `Thread` row → registry removes the local entry; PGlite row deleted | unit test |
| F5 | Local-only ephemeral threads (PGlite-only, never persisted server-side) are NOT touched by SSE absence-events | manual review |
| F6 | Audit doc: `Thread` row flipped from `pending` to `direct (SSE-driven sync)` | file diff |
| F7 | `pnpm --filter ./frontend test` ≥ 37/37 (preferably +3 for new unit tests = 40/40) | output |
| F8 | `pnpm --filter ./frontend build` clean | output |

---

## 4. Gap analysis

### 4.1 Reconciliation strategy

The hook needs to:

1. Subscribe to `useGraphStore` selecting `entities["Thread"]`.
2. On each mutation, walk the dirty key set and apply per-key reconciliation:
   - If a key is **new** (no local thread): create a new persisted local thread mirroring the server fields.
   - If a key is **known and dirty**: merge server fields into local; preserve `isEphemeral` if the local copy is still ephemeral (i.e. the user has another tab editing it).
   - If a key was **removed** from the graph: delete from registry + PGlite.

### 4.2 Snapshot diffing

The graph subscription fires whenever any `Thread` row changes. To compute "new" vs "updated" vs "removed", the hook keeps a ref of the prior snapshot keyset and diffs each tick. Cheap; same pattern as the now-deleted `useGraphBridge` but adapted to per-key actions instead of bulk refetch.

### 4.3 Server row shape

The SurrealDB `sessions` table presumably has `id`, `title`, `agent_id`, `created_at`, `updated_at` (and likely more). The local `LocalThread` shape has `id`, `title`, `isEphemeral`, `createdAt`, `updatedAt`. The reconciliation maps:

| Server (Thread graph) | Local (LocalThread) | Notes |
|---|---|---|
| `id` | `id` | identity |
| `title` (if set) | `title` | server wins; client title is best-effort |
| (presence of row) | `isEphemeral=false` | a server row means "persisted" |
| `created_at` | `createdAt` | preserve if local already has it |
| `updated_at` | `updatedAt` | server wins |

### 4.4 Tests

Three unit tests in `frontend/src/stores/__tests__/thread-registry-sync.test.tsx`:
- new server thread → registry creates entry with `isEphemeral=false`
- known thread with server title update → `setTitle` called
- server delete event → `removeThread` called

These pin the contract so future graph-shape changes don't silently regress sidebar behaviour.

### 4.5 Risk areas

- **Race between local create and server insert**: user creates ephemeral thread locally, then the first message persists it server-side. The server event arrives ~50–200 ms after creation. The reconciler must NOT recreate the row — it should hit the "known, mark persisted" branch.
- **SSE drop**: if the SSE connection drops mid-session, server events for that period are lost. Since there's no REST refetch, the registry would be stale until next session-create event. **Acceptable** for v1 — threads are client-authoritative.
- **PGlite-only ephemeral threads** should NOT be touched if the server-side graph slice is empty. The diff-by-keyset approach above handles this naturally.

---

## 5. Sequencing recommendation

2 changes:

1. **`add-thread-graph-sync-hook`** — author `frontend/src/stores/use-thread-graph-sync.ts` containing the subscription + reconciliation logic. Mount in `App.tsx`. Author 3 unit tests.
2. **`flip-thread-audit-row-to-direct`** — update audit doc; note the SSE-driven reconciliation pattern in the playbook.

Tiny phase. Defaults locked: no open questions.

---

## 6. Decisions (defaults)

| Decision | Choice | Rationale |
|---|---|---|
| Mount point | `App.tsx` (runs everywhere) | Sidebar can mount/unmount; we want the sync alive whenever the SPA is loaded |
| Hook returns | `void` (effects-only) | No render-time data needed; consumers continue using the registry store |
| Local thread protection | Keep `isEphemeral=true` until server confirms the row | Matches existing `registerThread` → `markPersisted` lifecycle |
| Backfill on first mount | None (no REST endpoint) | Live-only is acceptable |

---

## 7. Progress signal

Assessment complete. Defaults locked. Next: `/kbd-plan thread-topic-chat-sidebar` (or proceed straight to execute — plan is small).
