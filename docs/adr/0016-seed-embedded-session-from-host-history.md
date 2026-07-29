# 16. Seed the embedded session from host-supplied conversation history

Date: 2026-07-25

## Status

Accepted

## Context

The embedded runtime keeps conversation turns in an in-process
`SessionStore` — a `HashMap<String, Session>` keyed by conversation id. A run
resolves its session via `get_or_create(session_id)`, appends the user message,
and after generation appends the assistant reply, so within one process lifetime
the session accumulates the thread and each turn is sent the prior turns.

That store is **not durable**. Embedding hosts (the mobile shell, the in-process
desktop shell) keep the authoritative, durable conversation history in their own
storage and drive the runtime one turn at a time via `start_run`, passing only
the current message. Consequently, after a cold start — or any time the runtime
is rebuilt — the session for an ongoing conversation is empty, and the model
receives only the latest message with no prior context. The user sees the full
thread in the host UI but the agent "forgets" earlier turns. (The host's own
direct-inference lane already sends full history and is unaffected; only the
runtime-driven lane had this gap.)

`start_run` had no way to accept the host's history, so there was no seam to
repopulate the session.

## Decision

Add a seed-history seam to the embedded run entry points:

- A `SeedMessage { role, content, tool_call_id }` value type describing a prior
  turn.
- `RunManager::start_run_with_history` (and the underlying
  `start_run_with_policy_and_history`) accept `seed_history: Vec<SeedMessage>`.
  After resolving the session and **only when it is empty**
  (`session.message_count() == 0`), the prior turns are replayed into it —
  `user`/`assistant`/`tool` map to the corresponding session messages; `system`
  is skipped (the agent artifact owns the system prompt). The current user
  message is then appended as before.
- `Runtime::start_run_with_history` exposes this on the embedded SDK.

Seeding only an empty session makes the call idempotent: a warm session already
holds its turns, so re-passing the same history on the next turn is a no-op and
never duplicates context.

## Consequences

- Runtime-driven conversations survive a cold start: the model receives the
  host's full prior history on the first turn after restart, then the warm
  in-process session carries it for subsequent turns.
- The runtime remains the source of truth for *live* session state; the host
  remains the source of truth for *durable* history. The seed is the one-way
  bridge from host durability into an empty in-process session — no new
  persistence dependency is added to the runtime.
- Existing `start_run`/`start_run_with_policy` callers are unchanged (they now
  delegate with an empty seed).
- Verified by `cold_started_session_is_seeded_from_supplied_history`: an empty
  session's run sees the seeded turns plus the current input, and a second turn
  on the warm session is not re-seeded (no duplicate history).
