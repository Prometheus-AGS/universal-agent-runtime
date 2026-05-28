# Assessment — `add-push-channels-backend`

**Date:** 2026-05-27
**Tool:** claude-code (`/kbd-assess`)
**Prior phase:** `settings-store-retirement` (100%)
**Supersedes:** part of the original umbrella `tool-mcp-status-push-channels`

---

## 1. Phase goal & scope discovery

After inspecting `src/mcp/registry.rs` and `src/server.rs`:

- **Tool list is static after startup.** `McpRegistry` is built once via `McpRegistry::load_from_file("mcp.json")` at server boot (`server.rs:247`). The only post-startup mutation path is a config reload in `runtime/manager.rs:605` which rebuilds the whole registry — there is no per-tool add/remove API. **Conclusion:** Tools does NOT need an SSE push channel. A page-mount fetch is the correct UX.
- **McpStatus is dynamic** but already adequately served by 30 s frontend polling. The existing UX is acceptable. Adding push would be a polish item, not a correctness fix.

**Re-scoped phase:** instead of building new backend push channels, this phase becomes a frontend-only cleanup:

1. Migrate Tool to direct pattern — one-time fetch on mount; no SSE subscription needed (Tools isn't enrolled as a topic and shouldn't be).
2. Migrate McpStatus to direct pattern via the entity graph — keep the existing 30 s polling shim as the hydration source (the graph's `MetricsStatus` rows are refreshed by the poller).
3. **Delete `use-graph-bridge.ts`** — the original goal of the umbrella phase. Both consumers gone.

Backend work in this phase: **zero**. Cargo build untouched. The "push channels" name is preserved for orchestrator continuity but the actual work is renamed/repointed.

---

## 2. Current state

| Component | State | Action this phase |
|---|---|---|
| `EntityTopic::Tools` | not enrolled | leave un-enrolled (static data) |
| `EntityTopic::McpStatus` | not enrolled | leave un-enrolled (poll-driven) |
| `frontend/src/hooks/use-tools-discovery.ts` | bridged via `useGraphBridge(["Tool"], load)` | drop the bridge import; reads via `useEntityList("Tool")`; one-time fetch on mount |
| `frontend/src/stores/tools-discovery-store.ts` | Zustand cache | retire |
| `frontend/src/hooks/use-mcp-health.ts` | 30 s polling | reads via graph; polling still hydrates the graph |
| `frontend/src/stores/mcp-health-store.ts` | Zustand cache | retire |
| `frontend/src/lib/realtime/use-graph-bridge.ts` | `@deprecated` with one consumer | **DELETE** |

---

## 3. Definition of done

| # | Criterion | Verification |
|---|---|---|
| E1 | `useTools()` reads from graph; `loadToolsIntoGraph()` fetcher present | grep |
| E2 | `tools-discovery-store.ts` deleted; `use-tools-discovery.ts` deleted | grep |
| E3 | `useMcpHealth()` reads from graph; polling preserved as hydration source | grep |
| E4 | `mcp-health-store.ts` deleted | grep |
| E5 | **`frontend/src/lib/realtime/use-graph-bridge.ts` deleted** | grep |
| E6 | Audit doc: Tool + McpStatus rows updated; bridge appendix marked "Permanently retired" | file diff |
| E7 | `pnpm --filter ./frontend test ≥ 40/40` | output |
| E8 | `pnpm --filter ./frontend build` clean | output |
| E9 | `git grep useGraphBridge frontend/` empty | grep |

---

## 4. Sequencing

4 changes:

1. **`add-tools-and-mcp-fetchers`** — `entities/fetchers/{tools,mcp-status}.ts` + `entities/hooks/{use-tools,use-mcp-status}.ts`. Net-additive.
2. **`migrate-tools-page-direct`** — page reads via `useTools`; retire store + admin hook.
3. **`migrate-mcp-health-page-direct`** — page reads via `useMcpHealth` (graph-backed); retain 30 s poll for hydration; retire store + admin hook.
4. **`delete-use-graph-bridge-and-update-audit`** — delete the bridge file; mark audit appendix permanently retired; document why Tools/McpStatus are non-SSE-realtime (static / poll-fed).

Each change runs the test + build gate.

---

## 5. Decisions (defaults locked — no questions needed)

| Decision | Choice | Rationale |
|---|---|---|
| Push channels for Tools/McpStatus | **Skip** | Tools static; McpStatus poll already works |
| McpStatus poll interval | **Keep at 30 s** | Existing UX is fine |
| Graph hydration for McpStatus | **Poll-fed** | Health probes are server-process-local; no DB-backed event source |
| Bridge file disposition | **Delete** | Both consumers migrate this phase |

---

## 6. Progress signal

Assessment complete. Decisions locked. Next: `/kbd-plan add-push-channels-backend` (or just proceed to execute — the plan is small enough to inline).
