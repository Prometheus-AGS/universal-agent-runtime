# Reflection — `direct-entity-migration-agents`

**Date:** 2026-05-27
**Tool:** claude-code (`/kbd-reflect`)
**Phase status:** `execute_complete`
**Inputs:** assessment.md, plan.md, progress.json, openspec/changes/*/tasks.md

---

## 1. Goal achievement

Scored against §3 of the assessment ("Definition of done"):

| # | Goal | Status | Evidence |
|---|------|--------|----------|
| A1 | `agents-page.tsx` reads exclusively from `useAgents()`; zero `useAgentsAdmin` / `useAgentsAdminStore` references in the page | ✅ MET | `git grep` empty |
| A2 | `AgentSelector` reads via `useAgents()` instead of local `useState`-cached fetch | ✅ MET | local `useState<AgentWithType[]>` + `fetchAgentsList()` effect removed; now consumes the graph |
| A3 | `patchAgent` flow in `AgentMemorySection` calls service + optimistic graph patch directly | ✅ MET | `patchAgentOptimistic()` helper introduced; snapshot/upsert/rollback pattern |
| A4 | `frontend/src/hooks/use-agents-admin.ts` deleted | ✅ MET | file absent |
| A5 | `frontend/src/stores/agents-admin-store.ts` deleted | ✅ MET | file absent |
| A6 | `AgentEntity` aligned with `UarAgent` | ✅ MET | `export type AgentEntity = UarAgent` |
| A7 | Two-tab smoke: admin edit → AgentSelector reflects; delete → row disappears in another tab | 🟨 PARTIAL | Build chain green; **manual browser verification deferred to the combined Providers+Agents sweep** per locked decision Q4 |
| A8 | `useAgentConfig` consumers untouched and unbroken | ✅ MET | context shape unchanged; only the data origin shifted from imperative push → `useEffect`-driven push |
| A9 | Bridge entry removed for Agent | ✅ MET | hook + store deleted, bridge call gone with them |
| A10 | `docs/migration-stale-data-audit.md` flips `Agent` row to `direct` | ✅ MET | row updated + note added about the latent bug fix |
| A11 | Net frontend LOC delta negative | ✅ MET | ~210 LOC deleted (store + admin hook) vs ~140 LOC added (page + selector). **Net ~−70 LOC** |

**Aggregate:** 10 MET + 1 PARTIAL + 0 NOT MET = **91% confirmed**, with the one PARTIAL being a deliberate deferral to a combined browser smoke session.

---

## 2. Delivered changes

| # | Change | Status | Key shift |
|---|--------|--------|-----------|
| 1 | `agent-entity-realign` | DONE | `AgentEntity = UarAgent` alias; types finally match reality |
| 2 | `agents-page-direct-reads` | DONE | reads via `useAgents()`; local `error` state added |
| 3 | `agents-page-direct-mutations` | DONE | `patchAgentOptimistic` helper + optimistic delete with snapshot rollback |
| 4 | `agent-selector-and-store-retire` | DONE | **Latent SSE-blindness bug fixed** in the chat sidebar; render-derived `AgentConfig`; 2 files deleted |

**LOC totals:** 1 → +25 LOC (type changes), 2 → +15 LOC, 3 → +50 LOC, 4 → −60 LOC (selector simplification) + −210 LOC (deletions), audit doc → +5 LOC. **Net ~−175 LOC** (more aggressive than the predicted −70 because the selector itself shrank when its local cache + fetch effects were dropped).

---

## 3. Artifact Quality Summary

`artifact-refiner` not installed; inline verification used per change.

| Metric | Value |
|--------|-------|
| Changes with explicit QA gate | 0/4 |
| Inline verification (build + grep sweep) | 4/4 |
| First-pass build success | 4/4 |
| `git grep` final sweep | empty |
| LOC delta | ~−175 net |
| Recurring constraint violations | none |

---

## 4. Technical debt introduced

| Item | Severity | Cost to retire |
|------|----------|----------------|
| **Browser smoke deferred for two phases.** Providers + Agents both compiled clean but neither has had its rollback/propagation behaviours hand-verified. Risk grows with each unverified migration. | **High (priority)** | One focused two-tab session covering: configure provider, set-default, remove provider, edit agent memory, delete agent — both with success and forced-rejection cases (~30 min) |
| **`LegacyAgentEntityShape` interface kept** in `entities/types.ts` as a courtesy to any downstream consumer that imported the flat shape. Inspection found none. Dead code today. | Low | Delete in a follow-up cleanup pass after one release cycle. |
| **`patchAgentOptimistic` helper is page-local.** Models and Skills migrations will want the same pattern. The Provider mutation also used inline logic. Three near-identical helpers = ripe for extraction. | Med | Extract `useOptimisticPatch(type, id)` + `useOptimisticRemove(type, id)` when the Models migration starts; will retire ~60 LOC of inlined logic. |
| **AgentSelector double-hydration.** Both `agents-page.tsx` and `agent-selector.tsx` call `loadAgentsIntoGraph()` on mount. If the user is on the Admin page and then opens the chat, two fetches fire in quick succession. The entity-mgmt library *may* dedup; not verified. | Low | If browser smoke shows duplicate `/api/agents` requests, add a module-level "loaded once" guard in the fetcher. |
| **`AgentConfig` re-render frequency** — `useEffect([derivedConfig, onAgentConfigChange])` will fire any time the parent passes a new `onAgentConfigChange` reference. The chat-page already wraps it in `useCallback` (verified), so this should be stable, but a regression in chat-page memoization would create silent re-render spam. | Low | Document the contract; consider an `useEvent`/`useEffectEvent` extraction once React 19 stabilizes it. |
| **Browser smoke unverified for the AgentSelector path** — the headline value of this phase (chat-sidebar SSE freshness) is exactly what the smoke session would prove. Until then it's an asserted behaviour, not a measured one. | Med | Covered by the combined Providers+Agents smoke. |

---

## 5. Lessons captured

1. **Same playbook, half the friction the second time.** The Provider migration took 4 changes and felt heavier than Agents did at the same step count. Authoring the four OpenSpec proposals + tasks + executing them flowed faster — the playbook has settled into muscle memory. Expect Models / Skills / Settings to take roughly the same time each.
2. **The biggest wins are latent-bug fixes.** Migrating `AgentSelector` off its private `useState` cache fixed a stale-data bug that existed before the bridge era. Hunt for similar private caches when migrating the next cross-cutting entity. **Investigate** before retyping a page — there may be a chat-side consumer fetching independently.
3. **Type lies surface during migration.** `AgentEntity` was a flat declaration over a nested storage shape and nobody noticed because every consumer cast away. The migration forced honesty. Future entities should reflect storage shape from day one.
4. **Render-derived context > imperative push** for SSE-fed graphs. The chat-page `AgentConfigContext` provider value now auto-refreshes when the selected agent's row changes in the graph. Imperative `onAgentConfigChange?.(...)` in click handlers became cosmetic — the `useEffect` does the real work.
5. **Inline mutation helpers ARE the helper extraction signal.** Page 1 (Providers) and Page 2 (Agents) both copy-pasted the snapshot/upsert/rollback pattern. The third repetition (Models) is the right time to extract `useOptimisticPatch` — extracting earlier would have been premature.
6. **`_type` runtime/federated tag persisted for free.** It was added by `fetchAgentsList()` at the service boundary, persisted through `loadAgentsIntoGraph()`, and now flows out of `useAgents()` unchanged. Not every cross-entity property needs explicit graph schema modeling.

---

## 6. Recommended focus for next phase

In priority order (matches `nextPhaseSeeds` in the waypoint):

1. **`vitest-contract-test-suite`** — promoted to top priority. Two direct-entity migrations now ship with their rollback behaviours unverified. Wire Vitest into npm scripts; canonical test = "two views mount `useEntity('Agent', id)`; synthetic SSE delete → both unmount within one tick; force a rollback path → both re-mount with the snapshot value." This **must** land before Models / Skills / Settings to lock the pattern.
2. **Combined Providers + Agents browser smoke** — not a full phase, but a required gate before any further migrations. Six manual checks (3 Provider + 3 Agent) totalling ~30 min.
3. **`useOptimisticPatch` helper extraction** — sized to one small pre-phase change before Models migration. Replaces three inlined copies (Provider, Agent, future Skill) with one tested helper.
4. **`direct-entity-migration-models`** — next entity. Models have cross-view consumers in the chat header model badge, capability toggles, and `useProviderModels` derived hook. Apply the playbook with the new helper.
5. **`direct-entity-migration-skills`** — preserve the built-in badge; the existing optimistic skill toggle (from the prior phase's store) moves to inline + helper.
6. **`direct-entity-migration-settings`** — last cross-cutting; settings use a dirty-buffer pattern that may need a different optimistic strategy.

---

## 7. Evolver feedback

No `evolver-bridge.json` in this phase directory. Not part of an iterative-evolver cycle. No outer-loop state to update.

---

## 8. Progress signal

Completed kbd-reflect — direct-entity-migration-agents
