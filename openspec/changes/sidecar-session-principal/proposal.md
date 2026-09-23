## Why

In the sidecar every request runs as the same identity. JWT is off by default (`src/bin/uar-sidecar.rs:58-63, 100-105`), so the auth middleware gives every request the fixed `anonymous` context (`src/uar/security/middleware.rs:13-27, 36-43`). Every owner check then compares `anonymous` with `anonymous`: sessions (`src/session/thread.rs:388-432`), run lookup and approval (`src/uar/runtime/manager.rs:5713-5718`, `src/uar/api/routes.rs:181-203`), knowledge bases (`manager.rs:1393-1397, 2782-2800`) and memory. Anonymous runs also skip the owner-scoped MCP capture (`manager.rs:2223-2229`, because `ActorOwner::from_verified_context` refuses `anonymous`, `src/uar/runtime/actor/messages.rs:21-37`). Memory tools in a run without a verified owner pass the model's arguments straight through (`src/uar/tools/memory.rs:33-47`), so `memory_list` without a user filter, and update, delete and history by id, reach every session's memories. D1 (revision 3, open risk 3) requires the shared process to keep the-boss sessions apart, or fall back to one process per session.

Separately, a long-lived sidecar grows without bound: run records, their dialogue copies and 512-event histories are never removed from `active_runs` (no removal in `manager.rs`; `manager.rs:46, 219-238, 2572-2602`), and the A2UI replay store keeps every patch of every run for the process lifetime (`src/uar/a2ui/realtime.rs:115-150`). The session store is never evicted either: `SessionStore::cleanup_expired` and `cleanup_expired_with_timeout` exist but are `#[allow(dead_code)]` and have no caller in `src/` or `tests/` (`src/session/thread.rs:463-484`), so every session ever used, with its full message list, stays in memory until the process exits. With one principal per the-boss session, that is one retained session per conversation the user has ever opened.

## What Changes

- On a request that passed the sidecar launch-token guard (change `sidecar-launch-security`), UAR accepts an `X-UAR-Principal` header and uses it as the verified owner for that request. Anywhere else the header is rejected with 400. Without the header the sidecar keeps today's anonymous behavior.
- With a principal, the existing owner-scoped stores isolate per the-boss session: sessions, runs, approvals, cancellation, A2UI surfaces, memory, knowledge bases, stored credentials, presentations and the MCP binding cache.
- An approval, cancel, resume, stream or A2UI action for a run owned by another principal is 404 and changes nothing.
- **BREAKING (anonymous runs).** In a run with no verified owner, `memory_list` and every by-id memory tool (`memory_update_by_id`, `memory_delete_by_id`, `memory_history`) return a tool error instead of reading or changing unscoped records.
- Terminal runs are evicted: after a run is terminal, has no subscriber, has no live descendant, and a retention period has passed, its record, dialogue copy, event history and A2UI replay history are removed. A cap on retained terminal runs evicts the oldest first. An evicted run id answers 404, as it does after a restart.
- Idle sessions are evicted: a session with no non-terminal run whose last activity is older than an idle timeout is removed from the session store, and a cap on retained sessions evicts the least recently active first. Both are configurable. They default on in sidecar mode (30 minutes, 1,000 sessions) and off in standalone UAR, whose clients send no host history. An evicted session is simply re-seeded by the host's history on its next run (`run-request-host-context`); without history, the next run starts with an empty session, exactly as after a restart.

## Capabilities

### New Capabilities
- `sidecar-session-principal`: the host-asserted per-request principal on the sidecar, where it is accepted and rejected, and what it scopes.
- `run-retention`: when terminal runs and their event and A2UI histories are evicted, when idle sessions are evicted, and what callers observe afterwards.

### Modified Capabilities
- `multi-tenant-isolation`: "Per-user data is isolated by authenticated identity" accepts the host-asserted sidecar principal as an authenticated identity, and gains scenarios for approvals and anonymous memory access.

## Impact

- **Code:** `src/uar/security/middleware.rs` (principal header on host-authenticated requests), the sidecar guard module from `sidecar-launch-security` (request extension marking a launch-token-authenticated request), `src/uar/tools/memory.rs` (`scoped_arguments` refusal without a verified owner), `src/uar/runtime/manager.rs` (terminal timestamp, eviction sweep for runs and sessions, `session_current_run` cleanup), `src/session/thread.rs` (session eviction with a live-run predicate and a cap; `get_or_create_for_user` refreshes activity), `src/uar/a2ui/realtime.rs` (`InMemoryReplayBackbone` removal), `src/config.rs` (run and session retention settings), `src/bin/uar-sidecar.rs` (sidecar defaults for session eviction).
- **APIs:** new request header `X-UAR-Principal` (sidecar only); new 400 codes `principal_header_not_allowed`, `principal_invalid`. Evicted run ids return 404 on every run route.
- **Trade-off (named):** with one principal per the-boss session there is no cross-session UAR memory or knowledge base. For memory that is accepted: the-boss provides memory through its own MCP tools (D1 field table). *Operator decision 2026-09-23:* UAR agents in the-boss use UAR knowledge bases owned by the host-asserted principal, not the host's own knowledge-base tools; knowledge-base API scoping by principal and credentialed ingestion are specified in `agentic-chunking` (design D8, D9). A knowledge base is visible only to the principal that created it; which the-boss scope a principal names is the host's choice.
- **Runtime UX:** the UAR SPA and standalone deployments are unchanged except for the anonymous memory refusal above. A UAR console left open on an old run sees 404 after retention.
- **Realtime state:** the retention period must exceed the resync grace of `agui-runs-stream-fidelity`, so a lagged host can still resync a just-finished run. The session idle timeout (30 minutes in sidecar mode) exceeds the run retention period (10 minutes), so a session is not evicted by idleness while its latest run is still retained.
- **Host contract:** after an idle eviction, the next run of that session gets only what the host sends as `history`. the-boss must send history on every run, not only after a sidecar restart.
- **Provider compatibility:** unaffected.
- **Depends on:** `sidecar-launch-security` (the launch token is what makes a host assertion trustworthy).
- **KBD workflow state:** yes. This change belongs to the-boss child phase `the-boss-universal-agent-runtime` (step 2); that phase's `progress.json` in `prometheus-skills-mini` must list it. This repository's `.kbd-orchestrator/` is not updated by this authoring pass.
