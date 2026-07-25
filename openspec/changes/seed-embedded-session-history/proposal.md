# Seed the embedded session from host-supplied conversation history

## Why

The embedded runtime's `SessionStore` is an in-process, non-durable
`HashMap<String, Session>`. Embedding hosts keep the authoritative, durable
conversation history in their own storage and drive the runtime one turn at a
time via `start_run`, passing only the current message. After a cold start the
session for an ongoing conversation is therefore empty, and the model receives
only the latest message with no prior context — the agent "forgets" earlier
turns even though the host UI still shows the full thread.

`start_run` had no seam to accept the host's history, so there was no way to
repopulate the session.

## What Changes

- Add a `SeedMessage { role, content, tool_call_id }` value type.
- `RunManager::start_run_with_history` /
  `start_run_with_policy_and_history` accept `seed_history: Vec<SeedMessage>`
  and, **only when the resolved session is empty**, replay the prior turns into
  it before appending the current user message. `user`/`assistant`/`tool` map to
  session messages; `system` is skipped (the agent artifact owns the system
  prompt).
- `Runtime::start_run_with_history` exposes this on the embedded SDK.
- Existing `start_run` / `start_run_with_policy` delegate with an empty seed —
  no behavior change for current callers.

Seeding only an empty session is idempotent: a warm session already holds its
turns, so re-passing history on the next turn is a no-op.

## Impact

- Affected specs: `embedded-admin-surface` (extended)
- Affected code: `src/uar/runtime/manager.rs`, `sdks/rust/src/runtime.rs`
- Behavior: additive. Runtime-driven conversations survive a cold start; the
  runtime stays the source of truth for live session state while the host stays
  the source of truth for durable history. No new persistence dependency.
