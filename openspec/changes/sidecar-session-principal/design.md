## Context

See proposal.md for why. Relevant facts:

- **Identity.** `resolve_user_context_with_config` returns the fixed `anonymous` context when no bearer token is present and JWT is not required (`src/uar/security/middleware.rs:13-27, 29-65`). The sidecar sets `UAR_SECURITY__JWT_REQUIRED=false` when the operator did not choose (`src/bin/uar-sidecar.rs:58-63, 100-105`). Change `sidecar-launch-security` adds a launch-token guard that runs before this middleware and strips `Authorization` after checking it.
- **Verified owners.** `RunExecutionRequest::with_user_context` keeps `verified_owner = None` for `anonymous` and builds an `ActorOwner` otherwise (`src/uar/runtime/turn/request.rs:101-117`); `ActorOwner::from_verified_context` refuses `anonymous` and requires `sub == user_id` (`src/uar/runtime/actor/messages.rs:21-37`). Owner-scoped stores already key on the owner: sessions (`src/session/thread.rs:388-432`), runs (`manager.rs:5713-5730`), KBs (`manager.rs:1393-1397, 2782-2800`), root MCP capture (`manager.rs:2223-2229`), presentations (`manager.rs:1387-1392`), stored credentials (user scope, `src/uar/security/credentials/resolver.rs:55-98`), memory tools (`src/uar/tools/memory.rs:33-47` confine to the verified owner when there is one).
- **Approvals.** The route checks `get_run_for_user(&user.user_id, run_id)` before `resolve_approval_request` (`src/uar/api/routes.rs:181-203`); the broker itself resolves by run id only (`src/uar/runtime/thread/approvals.rs:73-95`). With one shared `anonymous` identity the route check passes for every caller.
- **Retention.** `active_runs` entries (`RunStreamState`: record, dialogue copy, 512-event history, sender; `manager.rs:219-238`) are inserted at run start (`manager.rs:2586-2601`) and never removed. `InMemoryReplayBackbone` keeps every A2UI patch per run for the process lifetime (`src/uar/a2ui/realtime.rs:115-150`). `run_cancellations` entries are removed at terminal paths already.

## Goals / Non-Goals

**Goals:**
- Reuse the owner checks UAR already has by giving each the-boss session its own owner.
- Close the anonymous memory hole independent of the sidecar.
- Bound retained run state in a process that lives for days.

**Non-Goals:**
- Tenants for sidecar principals (`tenant_id` stays `None`).
- Persisting run records or sessions.
- Session eviction by default in standalone UAR (D5).

## Decisions

### D1 — The principal is trusted because of the launch token, and only then
The launch-token guard inserts a request extension (`HostAuthenticated`) after a successful check. The auth middleware, when that extension is present and `X-UAR-Principal` is set, validates the value and builds a `UserContext` with `user_id = sub = value`, `tenant_id = None`, `roles = ["host-session"]`. Without the extension the header is a 400. The header name is fixed; the value format excludes `anonymous` so the anonymous compatibility scope cannot be claimed.
- **Why:** the launch token proves the request comes from the process that started the sidecar (`sidecar-launch-security`). That host already decides which session a request belongs to; the header tells UAR.
- **Alternative rejected:** a JWT minted by the host per session. It needs a signing key shared with the sidecar and JWKS or secret configuration, and gives no more assurance than the launch token over the same loopback channel.
- **Alternative rejected:** one sidecar process per session (D1's fallback). It multiplies memory use and startup latency by the number of open sessions; per-session owners make it unnecessary if the isolation tests pass.

### D2 — No header keeps today's anonymous behavior
The UAR desktop app (`desktop-sidecar-conversion`) and other sidecar hosts do not send the header. Refusing header-less requests would break them. The-boss's driver sends the header on every request; its own contract test checks that.

### D3 — Refuse unscoped memory access at the tool boundary
`scoped_arguments` returns `memory_requires_verified_owner` for `memory_list` and every by-id tool when `context.verified_owner` is `None`, instead of returning the model's arguments unchanged (`memory.rs:42-47`). Save, search and recall keep their current behavior for anonymous runs.
- **Why at the tool:** the model chooses the arguments. A filter the model can omit is not a boundary (A-3: untrusted input at a tool-execution boundary).
- **Breaking:** anonymous standalone runs lose list and by-id memory tools. Named in the proposal.

### D4 — Eviction sweep
`RunStreamState` gains `terminal_at: Option<Instant>` and `last_detached_at`. A sweep runs every 30 s and on each terminal transition:
1. Candidates: terminal, `sender.receiver_count() == 0`, no live delegation (`delegation` weak reference cannot upgrade), and `max(terminal_at, last_detached_at) + retention` has passed.
2. If more than `runs.max_retained_terminal` terminal runs remain after step 1, evict additional terminal runs with no subscriber, oldest `terminal_at` first.
3. Eviction removes the `active_runs` entry, any `session_current_run` entry that points to it, and the run's `InMemoryReplayBackbone` history (new `remove(run_id)`).
Settings (proposed names): `runs.retention_after_terminal_secs` (default 600), `runs.max_retained_terminal` (default 1000). A gauge `uar_runs_retained` reports the count.
- **Why no acknowledgement protocol:** none exists, and an attached subscriber already signals interest. A host that needs a finished run's events after retention has its own event log (D2: the host is canonical).
- **Retention vs. resync grace:** 600 s is far above the 5 s resync grace of `agui-runs-stream-fidelity`.

### D5 — Idle-session eviction
Today the session store is never evicted. `SessionStore` holds every `(owner, session_id)` pair in one map (`src/session/thread.rs:343-350`, insert at `:390-398`). `cleanup_expired` and `cleanup_expired_with_timeout` exist but are `#[allow(dead_code)]` and nothing in `src/` or `tests/` calls them (`thread.rs:463-483`). With a principal per the-boss session, that is one full message list per conversation ever opened, for the life of the process.

The rule, run by the D4 sweep (every 30 s):
1. **Idle candidates:** sessions whose last activity is older than `sessions.idle_timeout_secs` and that have no non-terminal run. Last activity is `Session::touch` (`thread.rs:298-302`), which message, system-prompt and world-state writes call (`:175`, `:190`, `:255`, `:295`). "No non-terminal run" is read from `session_current_run` (`src/uar/runtime/manager.rs:349`, keyed by `tenant_storage_key(owner, session_id)`, set at `:2642-2646`) and the run's state in `active_runs`. A run waiting an hour on an approval writes nothing but is not terminal, so its session stays.
2. **Cap:** if more than `sessions.max_retained` sessions remain after step 1, evict further sessions with no non-terminal run, least recent activity first.
3. **Removal:** remove the entry under the store's write lock, re-checking both conditions there; drop a `session_current_run` entry for it whose run is terminal. The existing `active_sessions` gauge is updated as `cleanup_expired_with_timeout` already does (`thread.rs:478-481`).
4. **Race with a starting run:** a run resolves its session with `get_or_create_for_user` (`manager.rs:2404-2405`) before its first write. That call does not touch the session today (`thread.rs:422-433`). This change makes it touch, so a session resolved within the idle timeout is not idle, and the lock-held re-check in step 3 sees it.

`SessionStore::cleanup_expired_with_timeout` gains the live-run predicate and the cap rather than a second implementation.

Settings (proposed names; no `sessions` section exists in `src/config.rs` today): `sessions.idle_timeout_secs` and `sessions.max_retained`; `0` disables each. Defaults: **off in standalone UAR; on in sidecar mode** with 1800 s — the store's existing `DEFAULT_SESSION_TIMEOUT` (`thread.rs:15`) — and 1000. The sidecar bootstrap sets `UAR_SESSIONS__IDLE_TIMEOUT_SECS=1800` and `UAR_SESSIONS__MAX_RETAINED=1000` only when the operator set neither, the pattern it already uses for JWT (`src/bin/uar-sidecar.rs:58-63`, `:97-106`).
- **Why off in standalone:** the UAR SPA sends no host history (`run-request-host-context` proposal, Runtime UX), and nothing rehydrates an in-memory session from storage. An evicted standalone session would silently lose its context. An operator can turn it on.
- **Why the run-retention rule is always on and this one is not:** an evicted run was already unrecoverable after a restart, so eviction changes nothing a caller could rely on. An evicted session loses context the standalone UI does rely on.

**Interaction with host-supplied history (`run-request-host-context`).** Eviction makes the session empty, which is exactly the cold case host history exists for. The next run of an evicted session that carries `history` seeds the new empty session and reports `history: "seeded"`; one without `history` starts empty and reports `history: "none"`, as after a restart. No merge rule is needed: a session is either warm (in-memory wins) or evicted (host history seeds). the-boss must therefore send history on every run, not only after a restart. Principals keep sessions apart across eviction, because the key is `(owner, session_id)`.

**Interaction with run retention.** 1800 s idle exceeds 600 s run retention, so idleness never evicts a session whose latest run is still retained. The cap can; the run then stays addressable until its own retention ends, and `cancel_session_run_for_user` finds no current run for the evicted session (`manager.rs:1311-1321`), which is correct for a terminal run.

## Risks / Trade-offs

- [No cross-session UAR memory or KBs for the-boss] → Accepted; the-boss serves memory and KBs through its own MCP tools.
- [A the-boss driver bug that omits the header collapses sessions into `anonymous`] → The-boss driver contract test asserts the header on every request; UAR cannot tell a forgotten header from a deliberate anonymous host.
- [Principal values are not secret but are session identifiers] → They appear in logs as user ids today do; they carry no credential.
- [Eviction removes A2UI surfaces of old runs] → Actions on them get 404; the-boss shows such surfaces as inactive.
- [A the-boss driver that sends history only after a restart loses context after 30 idle minutes] → The host contract is "history on every run"; test 1.16 is the UAR half, the-boss's driver test is the other.
- [Compaction summaries and tool-call detail in an evicted session are gone] → Accepted. The host's projection is what the next run sees; `run-request-host-context` D5 already names the host as canonical for the cold case.
- [Idle measured by writes, and by `get_or_create_for_user` after this change] → A session read by other paths without a run (for example a session-listing API) is not kept alive by the read. Accepted.
- [JWT explicitly required on a sidecar] → The launch-token guard consumes `Authorization`, so JWT and launch token cannot both ride that header. Out of scope; the principal header does not change it.

## Migration Plan

Deploy after `sidecar-launch-security`. Standalone UAR sees only the anonymous memory refusal and run eviction. Rollback: revert; runs fall back to the shared anonymous owner.
