# Current Waypoint

- Phase: `full-frontend-entity-mgmt-migration` — **reflect_complete, 70% goal achievement**
- Previous phase: `prometheus-package-integration` (10/14 changes shipped; remaining 2 superseded by this phase)
- Source of truth: `.kbd-orchestrator/`
- Secondary mirror: `surreal_memory` MCP at `/mcp/memory`
- Status: `reflect_complete`
- Reflection: [.kbd-orchestrator/phases/full-frontend-entity-mgmt-migration/reflection.md](phases/full-frontend-entity-mgmt-migration/reflection.md)
- Exact next command: `/kbd-new-phase direct-entity-migration-providers`
- Updated at: 2026-05-27T04:10:00-05:00

## Phase outcome — what shipped

- **Realtime spine live**: 10 SurrealDB-backed live topics streaming to JWT-gated `/api/live/{topic}` SSE endpoints.
- **Bridge migration**: 8 admin hooks bridged via `useGraphBridge` — Knowledge, Providers, Agents, Skills, Models, Settings, Memory, Tools, Compiler-Sessions. Every Zustand admin store auto-refreshes on SSE delivery.
- **Optimistic mutations** on 3 high-frequency paths: skill toggle, agent patch, provider set-default.
- **Docs**: [docs/migration-stale-data-audit.md](../docs/migration-stale-data-audit.md) + [AGENTS.md](../AGENTS.md) "Realtime freshness contract" section.

## Phase outcome — what deferred (becomes next-phase seeds)

1. **Direct `useEntity` migration per entity** — retire the bridge + Zustand stores per cross-cutting entity (Providers pilot recommended).
2. **Vitest contract test** — two-views/one-SSE-event regression guard.
3. **Push channels for `Tool` + `McpStatus`** — full realtime parity for non-DB-backed entities.
4. **`runs` topic reconsideration** — only if a non-chat consumer needs run state.
5. **README architecture diagram** — visual companion to the AGENTS.md section.

## Locked decisions (carried forward)

1. `threads` topic aliases the `sessions` table; no `runs` topic.
2. Bridge pattern is interim. Direct `useEntity` is the destination.
3. Optimistic mutations: high-frequency only.
4. `api_keys`: non-realtime; never broadcast secrets.
5. `chat-message-store` + `chat-stream-store` stay out of scope.

All tools should read this waypoint before planning or execution and update the
phase progress when work is completed.
