## 1. Contract tests first (must fail before implementation, pass after)

Shared fixtures: a test router built with the sidecar launch-token guard from `sidecar-launch-security` (fixed test token) and a standalone router with JWT off; principals `boss-session-1` and `boss-session-2`; a scripted `MockLlmDriver` that calls named tools; embedded persistence in a temp directory with memory enabled; retention settings overridable per test.

D1 falsifier map: F2 (session isolation in the shared process) → 1.4–1.9 here, with the restart and history-reload part in `run-request-host-context` 1.9–1.11; round-3 open risk 3 (shared mutable store inventory) → 1.4–1.9 cover credentials store, MCP bindings, approvals, memory, KBs, A2UI and sessions; logs and traces are covered by the byte searches in `run-scoped-credentials-and-mcp-servers` 1.8 and 1.16.

- [ ] 1.1 `tests/sidecar_session_principal.rs::principal_header_is_accepted_only_after_the_launch_token` — sidecar router: token + `X-UAR-Principal: boss-session-1` creates a run whose record shows owner `boss-session-1`. Standalone router: the header with no JWT and with a valid JWT → 400 `principal_header_not_allowed`. Sidecar router: `anonymous`, empty, 129 characters, a space → 400 `principal_invalid`; the body does not echo the value. Verify: fails today (header ignored, owner `anonymous`).
- [ ] 1.2 `sidecar_session_principal.rs::sidecar_without_header_stays_anonymous` — token, no header → run owner `anonymous`, as today.
- [ ] 1.3 `sidecar_session_principal.rs::run_routes_refuse_another_principal` — run owned by `boss-session-1`; as `boss-session-2`: stream, cancel, checkpoints, resume, A2UI action, surface replay → 404, and the run keeps running. As `boss-session-1` → each succeeds.
- [ ] 1.4 `sidecar_session_principal.rs::approval_from_another_principal_is_refused` — run of `boss-session-1` pauses on approval; `boss-session-2` posts an approval → 404 and the approval is still pending (a second request from `boss-session-1` resolves it, and the tool runs once).
- [ ] 1.5 `sidecar_session_principal.rs::memory_is_scoped_to_the_principal` — `boss-session-1` saves memory `P1-SECRET`; a `boss-session-2` run calls `memory_list`, `memory_search` for `P1-SECRET`, and `memory_update_by_id`, `memory_delete_by_id`, `memory_history` with `boss-session-1`'s memory id, including model-supplied `user_id: "boss-session-1"`. Assert no result contains `P1-SECRET` and the memory is unchanged afterwards.
- [ ] 1.6 `sidecar_session_principal.rs::anonymous_runs_refuse_unscoped_memory_tools` — anonymous run: `memory_list` with no filter and the three by-id tools → tool error `memory_requires_verified_owner`; an existing memory is still present afterwards. Verify: fails today (list returns all memories).
- [ ] 1.7 `sidecar_session_principal.rs::knowledge_bases_resolve_only_for_their_principal` — KB `handbook` with a distinctive chunk created as `boss-session-1`; a `boss-session-2` run with policy knowledge base `handbook` gets no chunks and no citation event; a `boss-session-1` run gets the chunk.
- [ ] 1.8 `sidecar_session_principal.rs::same_session_id_under_two_principals_is_two_sessions` — both principals use session id `s1` with different private facts; each follow-up model request contains only its own facts.
- [ ] 1.9 `sidecar_session_principal.rs::two_sessions_share_nothing_in_one_process` — D1 F2 in-process part. Concurrent runs for both principals with private facts; force a compaction (small context budget), a memory write and an A2UI surface in each. Assert: neither run's SSE body, `uar.context.updated` summary, memory records, A2UI surface replay or recorded model requests contains the other principal's facts; each principal's global-MCP tool call opens its own binding (the test MCP server observes two sessions).
- [ ] 1.10 `tests/run_retention.rs::terminal_run_is_evicted_after_retention` — retention 1 s: after the run finishes and the subscriber detaches, within two sweep intervals every run route for its id returns 404, `GET .../a2ui/surface-replay` returns 404, and `uar_runs_retained` drops by one. The session's next run still sees prior turns. Verify: fails today (run retained forever).
- [ ] 1.11 `run_retention.rs::attached_subscriber_and_live_runs_are_not_evicted` — a terminal run with an attached subscriber past retention stays; a running run older than retention stays; a parent whose child is still running stays.
- [ ] 1.12 `run_retention.rs::cap_evicts_oldest_terminal_runs_first` — cap 3, five finished runs → the first two are 404, the last three are retained.
- [ ] 1.13 `run_retention.rs::retained_state_is_bounded_under_sustained_load` — cap 50, retention 0: 500 sequential runs each rendering one A2UI surface. Assert retained runs ≤ 50 and the A2UI replay store holds histories for ≤ 50 runs.
- [ ] 1.14 `run_retention.rs::lagged_host_can_resync_a_just_finished_run` — with default retention, a lag-ended stream (from `agui-runs-stream-fidelity` 1.4) reconnects after the run finished and receives the remainder.
- [ ] 1.15 Record the failing output of 1.1–1.14 and 1.16–1.19 against the unmodified branch before implementation starts.
- [ ] 1.16 `run_retention.rs::idle_session_is_evicted_and_reseeded_from_host_history` — sidecar router, `sessions.idle_timeout_secs` 1, principal `boss-session-1`, session `s1`: a run records `FACT codename=HELIOTROPE`. After two sweep intervals, assert the session store no longer holds `(boss-session-1, s1)` (the `active_sessions` gauge dropped by one). Next run on `s1` without `history`: the model request contains no `HELIOTROPE` and the response reports `history: "none"`. Next run on `s1` with `history` holding the earlier turns: the response reports `history: "seeded"` and the model request contains `HELIOTROPE`. A session of `boss-session-2` idle for the same time is evicted independently and never receives `boss-session-1`'s history. The `history` half needs `run-request-host-context`; until it lands, record that half as blocked, not passed. Verify: fails today (the session is never evicted).
- [ ] 1.17 `run_retention.rs::session_with_a_live_run_is_not_evicted` — idle timeout 1 s: a run paused on a tool approval for 5 s keeps its session; after approval and completion, the session is evicted once idle. A run started on a session within the idle window (resolve, then first write delayed by a test hook) does not lose its session.
- [ ] 1.18 `run_retention.rs::session_cap_evicts_least_recently_active_first` — cap 2, timeout 0 (off): three sessions used in order A, B, C, none running → A is evicted, B and C remain; a fourth session with a running run is never chosen.
- [ ] 1.19 `run_retention.rs::standalone_default_keeps_idle_sessions` — standalone router with no session settings: the effective `sessions.idle_timeout_secs` and `sessions.max_retained` are 0, and a session created before three sweep intervals is still present afterwards. Sidecar bootstrap with neither variable set: the effective values are 1800 and 1000; with `UAR_SESSIONS__IDLE_TIMEOUT_SECS=0` set by the operator, the value stays 0. *Regression guard for the defaults.*
- [ ] 1.20 `sidecar_session_principal.rs::capabilities_list_session_principal` — `GET /api/uar/capabilities` (from `sidecar-launch-security`) lists `session_principal`. Verify: fails before this change lands.

## 2. Principal

- [ ] 2.1 Insert the `HostAuthenticated` extension in the launch-token guard (coordinate with `sidecar-launch-security`); verify 1.1 standalone case.
- [ ] 2.2 Accept and validate `X-UAR-Principal` in the auth middleware only with that extension; verify 1.1, 1.2.
- [ ] 2.3 Confirm routes use the request principal for every owner check (no code change expected); verify 1.3, 1.4, 1.7, 1.8, 1.9.

## 3. Memory

- [ ] 3.1 Refuse `memory_list` and by-id tools without a verified owner in `scoped_arguments`; verify 1.5, 1.6.

## 4. Retention

- [ ] 4.1 Add `terminal_at`, `last_detached_at`, the sweep, the cap and the settings; verify 1.10–1.12.
- [ ] 4.2 Add `InMemoryReplayBackbone::remove` and clear `session_current_run`; verify 1.10, 1.13.
- [ ] 4.3 Add the `uar_runs_retained` gauge; verify 1.10.
- [ ] 4.4 Session eviction (design D5): extend `cleanup_expired_with_timeout` with the live-run predicate, the cap and the lock-held re-check; make `get_or_create_for_user` touch; call it from the D4 sweep; add `sessions.idle_timeout_secs` and `sessions.max_retained` (0 = off, standalone default off); set the sidecar defaults in `prepare_sidecar_process` only when the operator set neither. Verify 1.16–1.19.
- [ ] 4.5 Add `session_principal` to the capabilities list (`sidecar-launch-security` design Decision 12) in the commit that completes sections 2–4. Verify 1.20.

## 5. Phase-boundary verification

- [ ] 5.1 Run `cargo test --test sidecar_session_principal --test run_retention` once after sections 2–4 are complete, then the broader integration gate once; record outputs.
- [ ] 5.2 Run `openspec validate sidecar-session-principal --strict`; verify it passes.
