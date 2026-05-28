## Why

The current `EntityTopic` enum covers 7 entities (knowledge_bases, knowledge_documents, agents, providers, models, skills, settings). To realize "no stale data anywhere", we need realtime coverage for the remaining frontend-visible entity surfaces:

- **threads** — chat sidebar list + titles; aliases the existing `sessions` table.
- **memory** — admin memory deletes; multi-tab consistency for the memory page.
- **compiler_sessions** — compiler page; ephemeral but useful for parallel session views.
- **tools** — discovered MCP tool catalog; affects capability toggles.
- **mcp_status** — health badge in top nav; push-only (no Surreal table).

`api_keys` is intentionally excluded (never broadcast secrets); `runs` is excluded (already streams via `/api/chat/completion` SSE).

## What Changes

### Backend (`src/uar/realtime/`)

Extend `EntityTopic`:

```rust
pub enum EntityTopic {
    KnowledgeBases,
    KnowledgeDocuments,
    Agents,
    Providers,
    Models,
    Skills,
    Settings,
    Threads,            // NEW — aliases `sessions` table
    Memory,             // NEW
    CompilerSessions,   // NEW
    Tools,              // NEW — push-only stub
    McpStatus,          // NEW — push-only stub
}
```

- `Threads.table()` returns `"sessions"`.
- `Memory.table()` returns `"memory"`.
- `CompilerSessions.table()` returns `"compiler_sessions"` (supervisor parks at max backoff if the table doesn't yet exist).
- `Tools` and `McpStatus` don't run `.live()` against a Surreal table — they expose `push_event(...)` for the existing in-process publishers (MCP health loop, tool discovery loop) to feed.

### Frontend (`frontend/src/lib/realtime/topics.ts`)

Extend `UAR_TOPICS` with the new topics so `createAllUarAdapters()` includes them automatically.

### Push channel for Tools + McpStatus

New module `src/uar/realtime/push_channel.rs` with `PushPublisher::publish(topic, action, id, data)` so non-DB-backed event sources can feed the bus. `LiveQueryBus` exposes a constructor variant that accepts the push channel for these two topics.

## Acceptance

- `curl -N /api/live/threads` receives an `event: update` when a new session row is upserted.
- Same for `memory`, `compiler_sessions`.
- The MCP health loop publishes to `mcp_status`; a curl client sees a heartbeat every health-interval.
- Tool discovery publishes to `tools` when a new MCP server registers.
- `api_keys` and `runs` remain non-topics — requesting them returns 404.
