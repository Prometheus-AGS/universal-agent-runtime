## 1. Domain extension

- [x] 1.1 `EntityTopic` enum extended with `Threads`, `Memory`, `CompilerSessions`.
- [x] 1.2 `EntityTopic::ALL`, `table()`, `as_str()`, `FromStr` updated.

## 2. SurrealDB-backed topics

- [x] 2.1 `Threads.table()` → `"sessions"` (alias).
- [x] 2.2 `Memory.table()` → `"memory"` (lights up immediately — table exists).
- [x] 2.3 `CompilerSessions.table()` → `"compiler_sessions"` (parks at max backoff until table exists).

## 3. Push-only topics — DEFERRED

- [ ] 3.1 `Tools` push channel — deferred. Current MCP tool discovery refreshes via the existing health loop; adding a real push channel is its own change once a non-DB publisher pattern lands.
- [ ] 3.2 `McpStatus` push channel — deferred for the same reason.

Both are still poll-friendly via REST; clients re-query on demand or rely on the existing health loop. No user-visible regression from omitting them in this change.

## 4. Frontend topic map

- [x] 4.1 `UAR_TOPICS` extended with the three new entries.

## 5. Verification

- [x] 5.1 `curl /api/live/threads` returns 200 (verified).
- [x] 5.2 `curl /api/live/memory` returns 200 (verified).
- [x] 5.3 Startup log shows `live stream opened` for `memory`; `threads` + `compiler_sessions` park at max backoff with the expected "table does not exist" debug entries.
