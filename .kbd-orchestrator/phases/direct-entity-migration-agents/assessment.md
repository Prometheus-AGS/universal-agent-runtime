# Assessment — `direct-entity-migration-agents`

**Date:** 2026-05-27
**Tool:** claude-code (`/kbd-assess`)
**Prior phase:** `direct-entity-migration-providers` (reflect_complete, 80% goal achievement)

---

## 1. Phase goal

Apply the Provider playbook to the `Agent` entity to retire `useAgentsAdmin` + `useAgentsAdminStore`. This is the **first fan-out migration** — unlike Providers, Agents have real cross-view consumers (the chat sidebar's `AgentSelector` and the derived `useAgentConfig` context). Migrating Agents exercises the playbook against drift that didn't exist for Providers and locks the pattern before Models / Skills / Settings ship.

Definition of "fully migrated":

1. `agents-page.tsx` reads via `useAgents()` (the existing entity hook) and mutates via direct service calls + optimistic graph patches.
2. `AgentSelector` swaps its bespoke `useState`-cached `fetchAgentsList()` for `useAgents()` so the chat sidebar shares the same authoritative graph slot.
3. `useAgentConfig` continues to receive its `AgentConfig` through context from `AgentSelector` — no consumer change, but the data origin shifts from a sidebar-local cache to the graph.
4. `hooks/use-agents-admin.ts` + `stores/agents-admin-store.ts` deleted.
5. `AgentEntity` reconciled with the actual stored shape (raw `UarAgent`).

---

## 2. Current state inventory

### 2.1 Direct consumers of `useAgentsAdmin` / `useAgentsAdminStore`

| Site | File | What it reads / does |
|------|------|----------------------|
| Admin page reads | `admin/pages/agents-page.tsx:201` | `{ agents, loading, error, load }` |
| Per-agent memory section | `admin/pages/agents-page.tsx:80` (in `AgentMemorySection`) | grabs `patchAgent` action from store directly |
| Admin hook itself | `hooks/use-agents-admin.ts` | wraps store; bridged via `useGraphBridge` |
| Store impl | `stores/agents-admin-store.ts` | already has the optimistic-patch implementation from the prior phase's change #6 |

### 2.2 Cross-view consumers (the fan-out)

| Site | File | How it reads agents |
|------|------|---------------------|
| Chat sidebar selector | `features/chat/agent-selector.tsx` | calls `fetchAgentsList()` directly on mount, stashes in **local `useState`**. Does NOT use the store. Has its own fetch lifecycle independent of admin freshness. |
| Chat hot path config | `features/chat/agent-config-context.ts` + `useAgentConfig()` | React context fed by `AgentSelector` via `onAgentConfigChange` prop callback. No direct entity read. |
| Enhanced thread | `components/assistant-ui/enhanced-thread.tsx:105` | `const agentConfig = useAgentConfig();` — reads the context. |
| Chat page state holder | `pages/chat-page.tsx:64-72,90-91,149-168` | holds `agentConfigState` driven by `handleAgentConfigChange`; passes through `<AgentConfigContext.Provider>`. |

**Surprise:** `AgentSelector` has been running its own private REST-fetched copy of the agents list this whole time. Even when the agent admin page is bridged, the selector has no realtime signal — its list goes stale until the user reloads. Migrating it to `useAgents()` will give it cross-tab freshness for free.

### 2.3 Existing entity scaffolds (already in tree)

- `entities/fetchers/agents.ts::loadAgentsIntoGraph()` calls `fetchAgentsList()` and `upsertEntity("Agent", a.id, a)` — **stores the raw `UarAgent` shape, typed as `AgentEntity`** (a lie that works because both are `Record<string, unknown>`).
- `entities/hooks/use-agents.ts::useAgents(searchTerm?, statusFilter?)` returns `useEntityView<AgentEntity>` with alphabetical sort + optional search/filter. **The view's items are actually `UarAgent`-shaped at runtime.**

### 2.4 The `AgentEntity` shape problem

`AgentEntity` (in `entities/types.ts:39`) declares **flat** fields: `name`, `description`, `system_prompt`, `model`, `protocol`, `skills`, `tools`, `knowledge_bases`, `status`, …

`UarAgent` (in `types/index.ts:187`) declares **nested** fields: `metadata.title`, `policy.provider.default.model`, `memory.kb.knowledge_bases`, `tools.bundles[…]`.

Today `loadAgentsIntoGraph` casts away the difference. Page render code reads through the nested shape (e.g. `agent.policy?.provider?.default?.model`). If we kept the lie, `useAgents()` would return entities typed flat but used nested.

**Resolution options:**

- **A. Re-type `AgentEntity = UarAgent`.** Smallest blast radius; matches reality; loses the `Record<string, unknown>` constraint that some `useGraphStore` paths assume but in practice both shapes satisfy it.
- **B. Normalize at upsert.** Project nested → flat in `loadAgentsIntoGraph`. Risk: every page that previously read nested fields breaks until rewritten.
- **C. Dual stores.** Keep both shapes; pick per-page. Worst-of-both.

Default to **(A)**: align `AgentEntity` to `UarAgent`. The render code never has to change.

### 2.5 Optimistic-patch incumbent

The prior phase landed an optimistic shallow-merge patch in `useAgentsAdminStore.patchAgent` (change #7). When we delete the store, this needs to migrate to the page-local `patchAgent` callback. The new pattern follows Providers: snapshot via `useGraphStore.getState().entities["Agent"][id]`, optimistically `upsertEntity` the merged value, call `patchAgentApi`, rollback on failure.

### 2.6 Realtime / SSE state

- `Agent` is one of the 10 enrolled topics. Backend live-query is open and verified.
- The graph receives `Agent` mutations today, but **only the admin page hook is bridged**. `AgentSelector` runs blind to SSE because it never reads the graph. Migration fixes this.

---

## 3. Definition of done

| # | Criterion | Verification |
|---|-----------|--------------|
| A1 | `agents-page.tsx` reads exclusively from `useAgents()`. Zero `useAgentsAdmin` / `useAgentsAdminStore` references in the page. | `git grep useAgentsAdmin frontend/src/admin/pages` empty |
| A2 | `AgentSelector` reads via `useAgents()` instead of its local `useState`-cached `fetchAgentsList()`. | code review + the selector picks up SSE-delivered changes without reload |
| A3 | `patchAgent` flow in `AgentMemorySection` calls service + optimistic graph patch directly, not the store. | code review |
| A4 | `frontend/src/hooks/use-agents-admin.ts` deleted. | file absent |
| A5 | `frontend/src/stores/agents-admin-store.ts` deleted. | file absent |
| A6 | `AgentEntity` aligned with `UarAgent` (typed truth matches stored shape). | `entities/types.ts` |
| A7 | Two-tab smoke: edit an agent's memory toggle in tab A (Admin) → selector dropdown in tab B reflects the change ≤200 ms; deleting an agent in tab A removes it from the selector in tab B without reload. | manual |
| A8 | `useAgentConfig` consumers (`enhanced-thread.tsx`, anything reading the context) untouched and unbroken. | code review + manual chat session smoke |
| A9 | Bridge entry removed for Agent — `useGraphBridge` no longer called with `["Agent"]`. | (it's gone with the hook deletion) |
| A10 | `docs/migration-stale-data-audit.md` flips `Agent` row from `bridged` → `direct`. | doc updated |
| A11 | Net frontend LOC delta negative. | git diff stat |

---

## 4. Gap analysis

| ID | Gap | Severity | Notes |
|----|-----|----------|-------|
| G1 | `AgentEntity` shape doesn't match what's stored. | **High** | Re-type to `UarAgent` (option A). |
| G2 | `AgentSelector` has a local `useState` agents cache that doesn't react to SSE. | **High** | Replace with `useAgents()` consumption. Selection state stays local; only the source-of-truth list moves. |
| G3 | `patchAgent` lives in the store today. The page's `AgentMemorySection` grabs the store action directly. | Med | Move to a small page-scope helper or inline the optimistic-patch logic in the component. |
| G4 | `fetchAgentsList()` returns a different shape than the graph stores (it returns `UarAgent[]` flattened from runtime + federated; graph stores by `id` alone — federated and runtime agents may collide if ids overlap). | Med | The current fetcher already flattens; collision risk exists today too. Document but don't fix in this phase. |
| G5 | Agent selector emits `extractAgentConfig(agent)` into a context. If the underlying agent record changes via SSE, the context value doesn't auto-update (it's set imperatively via `onAgentConfigChange`). | Med | Either (a) re-derive the context from `useAgents() + selectedId` on every render, or (b) keep the imperative push and add a `useEffect` that re-pushes when the underlying agent updates. Default = (a) — cleaner. |
| G6 | Existing optimistic patch in the store is correct but lives in the wrong place after migration. | Low | Port logic to the page; same snapshot+merge+rollback pattern from Providers. |
| G7 | `AgentSelector` POSTs `applyAgentConfig` to `/api/uar/sessions/{threadId}/agent-config` after selection. This is session-side, not agent-entity-side — no migration needed. | Low | Leave untouched. |
| G8 | If both runtime and federated agents are returned with the same id, the graph's keyed-by-id store will collapse them. | Low | Existing behavior would already collapse them; status quo. |
| G9 | Browser smoke from prior phase is still pending — we're stacking unverified migrations. | Med | Acknowledge; run a single smoke session after this phase covering BOTH Providers and Agents. |

---

## 5. Sequencing recommendation

1. **G1 — Align `AgentEntity` to `UarAgent`.** Single edit to `entities/types.ts`. Compile across the SPA must stay clean.
2. **G2 + G3 partial — Migrate `agents-page.tsx`.** Reads via `useAgents()`. Then page-local `patchAgent` replaces the store action used by `AgentMemorySection`. Optimistic patch logic ported from store.
3. **G2 full + G5 — Migrate `AgentSelector`.** Replace local fetch + `useState` with `useAgents()`. Re-derive `AgentConfig` from `useAgents().items.find(a => a.id === selectedId)` via `useEffect` → `onAgentConfigChange`. This step also fixes the cross-tab stale-data hole in the chat sidebar.
4. **Retire `useAgentsAdmin` + `agents-admin-store`.** `git grep` clean, delete, update audit doc.

Each step compiles independently.

---

## 6. Open questions for the user before planning

1. **`AgentEntity` retype** — option A (alias to `UarAgent`) confirmed? Or normalize at upsert (B)?
2. **`AgentConfig` derivation** in `AgentSelector` — derive on every render from `useAgents()` (cleaner, slightly more renders) or keep imperative push and add a `useEffect` watcher (less rendering churn)?
3. **`AgentMemorySection.patchAgent`** — keep the optimistic shallow-merge pattern from the store, or introduce a small reusable `useOptimisticPatch(type, id)` helper that captures the rollback pattern for reuse in Models/Skills/Settings?
4. **Smoke testing schedule** — keep accumulating unverified phases and run one big smoke session after Settings, or pause after this phase for a Providers+Agents browser sweep?

---

## 7. Progress signal

Completed kbd-assess — direct-entity-migration-agents
