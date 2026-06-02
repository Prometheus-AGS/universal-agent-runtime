# persist-builtin-agents

## Why
The two built-in agents — `default-agent` (`src/uar/defaults.rs:9`) and `orchestrator-agent` (`defaults.rs:76`) — are code-only constructors that are **never persisted** (assessment D3). `GET /api/agents` injects them via an `ensure_builtin_agent` shim (`discovery.rs:90-94`), so the endpoint returns them — but the frontend hydrates them once via REST into an entity graph that the wildcard realtime subscription (`entities/sync.ts {type:"*"}`) can **evict with a `replace`/`delete` ChangeSet and never re-emit**, because they have no backing DB row. Their visibility is therefore unreliable by construction.

Agent switching is also fragile: the selector renders only when `activeThreadId` exists (`chat-page.tsx:146`), and the chat send body (`chat-stream-store.ts:572-580`) carries **no `agent_id`/`model`** — selection depends entirely on a prior best-effort side-channel POST whose errors are swallowed (`agent-selector.tsx:121`); any race/failure silently falls back to `default-agent` (`server.rs:3687`).

## What changes
- Add `seed_builtin_agents` (alongside `ensure_default_knowledge_base` in `defaults.rs`) that **persists both builtins to the datastore at startup** (idempotent upsert by id), so they become normal persisted, realtime-backed entities and survive realtime ChangeSets.
- Keep `ensure_builtin_agent` as a defensive fallback but it should no longer be the only thing surfacing them.
- Ensure the serialized agent shape carries a top-level `name`/`description`/`status` (or map `metadata.title`→`name` in the fetcher) so list sort/search treat them as first-class (`artifact.rs:5-24`, `entities/fetchers/agents.ts`).
- Render the agent selector **unconditionally** (not gated on `activeThreadId`); allow choosing an agent before the first message.
- Include `agent_id` (and `model` when chosen) in the chat send request body (`chat-stream-store.ts:572-580`) so agent selection is authoritative on the run request, not a swallowed side-channel POST. Keep the session agent-config POST as a secondary persistence path but stop depending on it.

## Impact
- Affected: `src/uar/defaults.rs`, server startup (seed call), `src/uar/api/discovery.rs`, `frontend/src/entities/fetchers/agents.ts`, `frontend/src/features/chat/agent-selector.tsx`, `frontend/src/pages/chat-page.tsx`, `frontend/src/stores/chat-stream-store.ts`.
- Behavior: both builtins appear reliably in the admin Agents list and the chat selector; users can pick and chat with them; switching no longer silently degrades to `default-agent`.
- Risk: low-medium — touches startup seeding + chat request contract; verify no duplicate rows on repeated boots (idempotent upsert).
