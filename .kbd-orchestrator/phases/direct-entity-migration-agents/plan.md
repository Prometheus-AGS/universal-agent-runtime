# Plan — `direct-entity-migration-agents`

**Date:** 2026-05-27
**Tool:** claude-code (`/kbd-plan`)
**Backend:** OpenSpec (detected at `openspec/`)
**Assessment input:** `.kbd-orchestrator/phases/direct-entity-migration-agents/assessment.md`

---

## Decisions locked (defaults applied)

| Q | Answer |
|---|--------|
| Q1 — `AgentEntity` retype | **Alias to `UarAgent`** (option A). Smallest blast radius; matches what `loadAgentsIntoGraph` already stores. |
| Q2 — `AgentConfig` derivation in `AgentSelector` | **Render-derived** from `useAgents().items.find(a => a.id === selectedId)` (option a). Cleaner; SSE-driven updates flow into context automatically. |
| Q3 — `patchAgent` pattern | **Inline optimistic pattern** matching the Providers playbook (snapshot → merge → call → rollback). No `useOptimisticPatch` helper yet; extract when 3+ entities share the same shape. |
| Q4 — Smoke testing schedule | **Pause after this phase** for a combined Providers+Agents browser smoke session. Don't stack more unverified migrations. |

---

## Ordered change list (4 changes)

| # | Change ID | Title | Depends on |
|---|-----------|-------|------------|
| 1 | `agent-entity-realign` | Re-type `AgentEntity` to alias `UarAgent`; verify SPA still compiles | — |
| 2 | `agents-page-direct-reads` | `agents-page.tsx` reads from `useAgents()`; mutations still via legacy store temporarily | 1 |
| 3 | `agents-page-direct-mutations` | Move `patchAgent` + `deleteAgent` flows to direct service + optimistic graph patches; local UI state | 2 |
| 4 | `agent-selector-and-store-retire` | `AgentSelector` reads via `useAgents()` (fixes the silent staleness bug); delete `use-agents-admin.ts` + `agents-admin-store.ts`; update audit doc | 3 |

Each change compiles independently — bailing out at any step leaves the page + chat sidebar functional.

---

## Per-change synopsis

### 1. `agent-entity-realign`
- In `frontend/src/entities/types.ts`, replace the flat `AgentEntity` interface with `export type AgentEntity = UarAgent;` (importing `UarAgent` from `@/types`).
- Remove the unused `EMPTY_AGENTS` stable-empty constant if it relied on the flat shape; keep it if it still type-checks.
- `entities/fetchers/agents.ts` cast becomes a no-op since types align.
- Acceptance: `pnpm --filter ./frontend tsc --noEmit` passes; no functional change.

### 2. `agents-page-direct-reads`
- `agents-page.tsx:201` swap:
  ```ts
  // before:
  const { agents, loading, error, load } = useAgentsAdmin();
  // after:
  const agentsView = useAgents();
  const agents = agentsView.items;
  const loading = agents.length === 0;
  const [error, setError] = useState<string | null>(null);
  const load = () => loadAgentsIntoGraph();
  ```
- `AgentMemorySection` still uses `useAgentsAdminStore((s) => s.patchAgent)` — untouched in this PR.
- Acceptance: page renders pixel-equivalent; agent list reflects SSE updates without reload.

### 3. `agents-page-direct-mutations`
- Replace `AgentMemorySection.patchAgent` with a page-scope-local helper that:
  1. Captures `useGraphStore.getState().entities["Agent"][id]` as snapshot.
  2. Optimistically `upsertEntity("Agent", id, { ...snapshot, ...body })`.
  3. Calls `services/agents-api.ts::patchAgent`.
  4. On error: re-upsert snapshot + set local `error`.
- Replace `deleteAgent` flow in `handleDelete` to capture snapshot, optimistically `removeEntity("Agent", id)`, call service, rollback on error.
- Local `useState` for `saving`/`deleting`/`error` (already partially in place).
- Acceptance: edit memory toggle flips instantly; delete removes row instantly; forced rejection rolls back.

### 4. `agent-selector-and-store-retire`
This is the highest-value change because it fixes the cross-tab staleness in the chat sidebar:
- `AgentSelector`:
  - Drop local `useState<AgentWithType[]>` agents cache.
  - Drop `fetchAgentsList()` `useEffect`.
  - Add `const agentsView = useAgents(); const agents = agentsView.items;` instead.
  - Trigger `loadAgentsIntoGraph()` on mount (idempotent — already runs from the admin page; safe to run from anywhere).
  - **Render-derived `AgentConfig`**: `const currentAgent = agents.find(a => a.id === selectedId); useEffect(() => onAgentConfigChange?.(currentAgent ? extractAgentConfig(currentAgent) : null), [currentAgent, onAgentConfigChange]);` so context auto-updates when the underlying agent is mutated via SSE.
  - Federated vs runtime `_type` tag — preserve via a side-channel or drop if no consumer reads it (audit during execute).
- Delete `frontend/src/hooks/use-agents-admin.ts`.
- Delete `frontend/src/stores/agents-admin-store.ts`.
- `git grep -nE "useAgentsAdmin|agents-admin-store" frontend/src` → empty.
- Flip `Agent` row in `docs/migration-stale-data-audit.md` from `bridged` → `direct`.

---

## Risk register

| Risk | Mitigation |
|------|------------|
| `AgentEntity = UarAgent` retype breaks unrelated code that imported `AgentEntity`-flat-shape | Pre-flight grep `AgentEntity` across `frontend/src`; if any consumers exist outside the entity scaffolds, address in change 1 |
| Render-derived `AgentConfig` causes infinite re-render loop (useEffect → onAgentConfigChange → parent setState → re-render → useEffect…) | Memoize: use `useMemo` on `extractAgentConfig(currentAgent)` keyed by `currentAgent`. The `useEffect` only fires when memoized config identity changes |
| `loadAgentsIntoGraph()` running from both the admin page and the selector double-fetches at startup | Acceptable — entity-mgmt dedupes via in-flight request cache; if not, add a tiny guard |
| `AgentSelector` loses the `_type` runtime/federated tag because graph stores by raw id | Audit consumers in change 4; if `_type` matters, keep it as a derived computation (e.g. `runtime_agents` list has `_type: "runtime"`) |
| `AgentMemorySection` snapshot is missing nested-field types | The snapshot is `Record<string, unknown>`; the shallow merge `{...snapshot, ...body}` preserves nested fields untouched |
| Optimistic delete fires before user confirms — wait, that's not a risk because the confirm dialog still gates it | n/a |
| Other admin pages depend on `useAgentsAdminStore` indirectly | `git grep` pre-flight in change 4 |

---

## Acceptance gate before phase reflect

1. `pnpm --filter ./frontend build` clean.
2. `git grep -nE "useAgentsAdmin|agents-admin-store" frontend/src` → empty.
3. **Browser smoke (combined with Providers from prior phase):**
   - Edit agent memory toggle in Admin → AgentSelector dropdown in another tab reflects.
   - Delete an agent in Admin → row disappears from AgentSelector ≤200 ms in another tab.
   - Force a `patchAgent` rejection → optimistic flip rolls back.
   - Force a `deleteAgent` rejection → row reappears.
   - Configure a provider in Admin → Provider list updates in another tab.
   - Set default provider → badge flips instantly.
4. Audit doc updated.

---

## Sources

- [Assessment](.kbd-orchestrator/phases/direct-entity-migration-agents/assessment.md) — §2 inventory, §4 gap analysis.
- Provider playbook: [docs/migration-stale-data-audit.md](docs/migration-stale-data-audit.md) "Bridge pattern vs. direct migration" section.
- Existing infra: [`entities/fetchers/agents.ts`](frontend/src/entities/fetchers/agents.ts), [`entities/hooks/use-agents.ts`](frontend/src/entities/hooks/use-agents.ts).

---

## Progress signal

Completed kbd-plan — direct-entity-migration-agents
