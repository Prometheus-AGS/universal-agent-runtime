# Reflection — `add-push-channels-backend`

**Date:** 2026-05-27
**Tool:** claude-code (`/kbd-reflect`)
**Status:** reflect_complete

---

## 1. Goal achievement (after assessment-driven scope collapse)

**Original phase goal** (per the umbrella `tool-mcp-status-push-channels`): build backend push channels for `Tools` and `McpStatus`, then migrate the last two bridge consumers, then delete `use-graph-bridge.ts`.

**Discovery during assessment:** code inspection showed `McpRegistry` is built **once** at server startup with no runtime add/remove path. Tools is effectively static. McpStatus is dynamic but already adequately served by 30 s frontend polling. **Neither needs a backend push channel today.**

**Re-scoped goal:** keep the bridge-deletion outcome from the umbrella phase. Move Tool + McpStatus to the direct pattern using existing data sources (one-time fetch for Tool; polling-fed graph for McpStatus). Zero backend work.

| # | DoD criterion | Verdict |
|---|---|---|
| E1 | `useTools()` reads from graph | **MET** — page consumes `useEntityList("Tool")` |
| E2 | `tools-discovery-store` + admin hook retired | **MET** |
| E3 | `useMcpHealth()` (now `useMcpStatus()`) reads from graph; polling hydrates | **MET** |
| E4 | `mcp-health-store` + admin hook retired | **MET** |
| E5 | **`use-graph-bridge.ts` deleted** | **MET** ⭐ |
| E6 | Audit doc rows flipped; bridge appendix marked permanently retired | **MET** |
| E7 | Tests ≥ 40/40 baseline | **PARTIAL** — 37/37 (3 bridge contract tests deleted with the bridge; net new contract tests would push back to 40+) |
| E8 | Build clean | **MET** |
| E9 | `git grep useGraphBridge frontend/` empty | **MET** |

**Goal achievement: 100% on the destination outcome.** The "37 vs 40" test count is by design — the deleted bridge tests were guarding behaviour that no longer exists in the codebase.

---

## 2. Delivered changes

| # | Change | Status |
|---|---|---|
| 1 | `add-tools-and-mcp-fetchers` | DONE — `entities/fetchers/{tools,mcp-status}.ts` + `entities/hooks/use-mcp-status.ts`; aligned existing `entities/fetchers/tools.ts` to the page's `_ns`/`_key`/`_builtin` shape |
| 2 | `migrate-tools-page-direct` | DONE — page reads via `useEntityList("Tool")`; one-time mount fetch (Tools is static); retired store + admin hook; updated `tool-detail-panel.tsx` to use the new type |
| 3 | `migrate-mcp-health-page-direct` | DONE — page reads via `useMcpStatus()`; 30 s poll hydrates the graph; retired store + admin hook |
| 4 | `delete-use-graph-bridge-and-update-audit` | DONE — bridge file deleted, contract test deleted, audit doc updated, bridge appendix marked permanently retired |

---

## 3. Code shape

| Metric | Value |
|---|---|
| Frontend net LOC | further negative — 2 more Zustand stores retired, bridge file + test deleted |
| Total Zustand stores retired in entity-migration project | **8** (providers-admin, agents-admin, models-browse, compiler-sessions, memory-admin, skills-admin, knowledge-admin, settings, tools-discovery, mcp-health) — actually 10 if you count the chat-stream/provider-models which retain change-bus listeners but no entity caches |
| Bridge consumers | 0 (was 1 before this phase) |
| Backend (Rust) files touched | 0 |
| Cargo build status | untouched (didn't need to run) |

---

## 4. Technical debt status

**No new debt this phase.** Notable closures:

- ✅ `useGraphBridge` is GONE — the interim infrastructure is no longer part of the codebase. Any future entity migration starts from the direct pattern.
- ✅ Tools and McpStatus rationale documented: static registry / process-local probes are NOT misses that need push channels — they're correctly modeled as fetch-on-mount and poll-hydrated respectively.
- ⚠️ Carry-over still owed from earlier phases: browser smoke walkthrough, knowledge-page aesthetic redesign, skill plugin installs (user-interactive).

---

## 5. Lessons captured for the knowledge base

1. **Assess before you build.** The original plan had 7 changes touching 5 Rust files. Looking at the actual source (10 minutes) revealed that the assumed design wasn't necessary — the data shape was already correct. Scope collapsed to 4 frontend changes, zero Rust.

2. **"Realtime" is not always "SSE-push."** Three legitimate realtime patterns: (a) SSE-fed graph (Surreal live queries), (b) fetch-on-mount of static data, (c) graph hydrated by client-side polling. All are "no stale data" if scoped correctly. The audit doc now reflects all three.

3. **Deleting tests is fine when their subject is deleted.** Going from 40/40 to 37/37 looks like regression but isn't — three tests pinned the contract of a file that no longer exists. The remaining 37 still cover everything that still exists.

4. **Type shape alignment matters more than naming.** The existing `entities/types.ts::ToolEntity` and the fetcher's view of a tool row had different shapes. Renaming the fetcher type to `ToolGraphRow` (instead of overloading `ToolEntity`) avoided a shadowing conflict. Generic lesson: when two layers see the same entity differently, keep the names distinct.

5. **Splitting phases on user request was the right call.** Even though this phase ended up frontend-only, deciding upfront to split bought the assessment time that revealed the simpler design. The original 7-change umbrella would have started with backend work that turned out to be unnecessary.

---

## 6. Cross-phase status — entity migration scoreboard (FINAL)

| Entity | Status (post-phase) |
|---|---|
| Provider | direct ✅ |
| Agent | direct ✅ |
| Model | direct ✅ |
| Skill | direct ✅ |
| Memory | direct ✅ |
| CompilerSession | direct ✅ |
| KnowledgeBase + Document | direct (compat hook) ✅ |
| Setting | direct (form cache) ✅ |
| **Tool** | **direct (fetch-on-mount; no SSE needed) ✅** |
| **McpStatus** | **direct (poll-fed graph) ✅** |
| Thread | pending (sidebar wiring) |
| ApiKey | non-realtime (intentional — secrets) |

The bridge pattern is **fully retired**. The direct migration playbook is the only canonical pattern. The only entity still classed `pending` is `Thread`, which requires the chat sidebar consumer to be wired (separate effort).

---

## 7. Recommended next phase

The waypoint shows several seeds. Now that the bridge story is closed:

1. **`ci-frontend-tests`** — wire `pnpm test`, `pnpm build`, and the architectural greps (`useGraphBridge`, `useSettingsStore`, banned fonts) into CI. Cheap, high leverage. Required before next migrations.
2. **Browser smoke walkthrough** — still owed for `browser-smoke-providers-and-agents`. Now covers 8 newly-migrated pages (Providers, Agents, Models, Skills, Memory, Compiler, Tools, MCP Health). Two real Chrome windows.
3. **`thread-topic-chat-sidebar`** — last `pending` entity. Wire `useEntityList("Thread")` into chat sidebar; retire any leftover legacy session-store cache.
4. **`knowledge-page-aesthetic-pass`** — visual-only follow-up.
5. **`readme-architecture-diagram`** — capture the now-stabilised architecture (SSE → graph → direct pattern) in the project README; the audit doc tells the migration story but the README should explain the destination state to new contributors.

---

## 8. Progress signal

Reflection complete. Bridge pattern permanently retired. Three sequential phases completed in the recommended path (`settings-store-retirement` → `add-push-channels-backend` reflects the renamed `tool-mcp-status-push-channels`). Recommended next: `ci-frontend-tests` for guard rails before further migrations.
