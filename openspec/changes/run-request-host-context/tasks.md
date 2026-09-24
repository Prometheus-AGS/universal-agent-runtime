## 1. Contract tests first (must fail before implementation, pass after)

Shared fixtures (in `tests/support/`): a deterministic **fact-echo driver** — a `wiremock` OpenAI-compatible provider whose custom `Respond` implementation scans the request's messages for lines of the form `FACT <key>=<value>` and answers `ANSWER <key>=<value>; ...` for every fact found, or `ANSWER NONE` when there are none; it also records every request body. Temporary directories `w/a` and `w/b` with distinct `secret.txt` files. Per-session principals come from `sidecar-session-principal`.

D1 falsifier map: F1 field contract rows "working directory" and "reasoning_effort" → 1.1–1.6; F2 session isolation "sidecar restart with history reload" → 1.9–1.11. D2 falsifier map: F7 restart (deterministic replacement, round-3 open risk 2) → 1.9–1.11.

- [ ] 1.1 `tests/run_host_context.rs::working_directory_is_validated` — relative path, nonexistent path, regular file, `/` → 422 `working_directory_invalid`, no run id. A symlink to `w/a` → run starts and `GET` of the run record shows the canonical target. Verify: fails today (field ignored, no 422).
- [ ] 1.2 `run_host_context.rs::terminal_defaults_to_the_working_directory` — `terminal_exec` enabled; scripted model calls `pwd` with no `working_dir`. Assert the tool result is the canonical `w/a`.
- [ ] 1.3 `run_host_context.rs::file_tools_are_confined_per_run` — sidecar-mode request (launch-token extension set), no operator roots; run A in `w/a` scripted to `file_read` `../b/secret.txt` and the absolute `w/b/secret.txt`, run B in `w/b` scripted to read `secret.txt`, concurrently. Assert both A reads return errors and B returns its own file content.
- [ ] 1.4 `run_host_context.rs::working_directory_outside_operator_roots_is_rejected` — operator roots `[w/a]`; request `w/b` → 422 `working_directory_not_allowed`.
- [ ] 1.5 `run_host_context.rs::standalone_request_does_not_gain_file_roots` — no operator roots, no launch-token extension; run in `w/a` calls `file_read`. Assert behavior equals today's (file tools refuse with the existing "no configured roots" error).
- [ ] 1.6 `run_host_context.rs::reasoning_effort_reaches_the_provider_request` — for an Anthropic-dialect model id and an OpenAI-family model id, runs with `none`, `low`, `high`, `max`. Assert on recorded request bodies: `none` has no reasoning parameter even with `thinking_budget` configured; Anthropic `budget_tokens` is non-decreasing across `low` < `high` ≤ `max`; the OpenAI request carries the confirmed reasoning parameter. `"ultra"` → 422 `reasoning_effort_invalid`.
- [ ] 1.7 `run_host_context.rs::child_runs_inherit_host_context` — parent with `w/a` and `high` spawns a child that calls `pwd`. Assert `w/a` and the child's provider request carries the `high` mapping.
- [ ] 1.8 `run_host_context.rs::host_context_is_recorded_on_the_run` — the run record shows `host_context.working_directory` (canonical) and `host_context.reasoning_effort`.
- [ ] 1.9 `tests/host_history_restart.rs::restarted_runtime_answers_from_host_history` — the D2 restart fixture. Prior turns (fixed): session S1 user `FACT codename=HELIOTROPE`, assistant `noted`, user `FACT deadline=2026-11-04`, assistant `noted`; session S2 the same shape with `codename=MARIGOLD`, `deadline=2027-01-15`. Run both conversations on RunManager #1, drop it, build RunManager #2 on fresh in-memory stores. Fixed question to S1: `What are the codename and deadline?`, with S1's history. Timeout 30 s per run. Assert the answer contains `codename=HELIOTROPE` and `deadline=2026-11-04`; fail with message `history missing` if either is absent and `cross-session history` if `MARIGOLD` or `2027-01-15` appears; assert the response reports `history: "seeded"`, `seeded_messages: 4`. Repeat for S2 in the other direction. Verify: fails today with `ANSWER NONE` (field ignored).
- [ ] 1.10 `host_history_restart.rs::restart_without_history_fails_the_fixture_explicitly` — same restart, question to S1 without history. Assert the response reports `history: "none"`, the answer is `ANSWER NONE`, and the fixture's own check reports `history missing` (proves the fixture cannot pass by accident).
- [ ] 1.11 `host_history_restart.rs::history_cannot_bind_to_another_session` — after restart: a run for S1 carrying history labeled S2 → 422 `history_session_mismatch`, and S1 and S2 sessions remain empty (a following question to each returns `ANSWER NONE`); a run with history and no `session_id` → 422.
- [ ] 1.12 `host_history_restart.rs::warm_session_keeps_its_own_history` — without restart, a second turn for S1 carries a history with an extra fabricated `FACT codename=WRONG`. Assert the response reports `ignored_warm_session`, the recorded request contains the in-memory turns once and no `WRONG`.
- [ ] 1.13 `host_history_restart.rs::malformed_history_is_rejected` — orphaned tool result, duplicate tool call id, `system` message, unknown role, over-limit count → 422 `history_invalid` / `history_too_large`, session unchanged.
- [ ] 1.14 `host_history_restart.rs::tool_call_pairs_survive_seeding` — history with assistant tool call `c1` and its result. Assert the first provider request contains both as a pair (not removed by history normalization).
- [ ] 1.15 `host_history_restart.rs::history_is_refused_on_resume_routes` — `POST /runs/{id}/resume` and `/resume/{checkpoint_id}` with `history` → 422 `history_invalid`.
- [ ] 1.16 `host_history_restart.rs::concurrent_first_turns_seed_once` — two runs for the same empty session started together with the same history. Assert exactly one reports `seeded` and the session contains the history once.
- [ ] 1.17 Record the failing output of 1.1–1.16 against the unmodified branch before implementation starts.

## 2. Working directory

- [x] 2.1 Validate and canonicalize `working_directory` in `create_run`; set it on the request; verify 1.1.
- [ ] 2.2 Pass the run working directory to `terminal_exec` as its default cwd; verify 1.2.
- [ ] 2.3 Register run-local file tools with roots from the working directory under the D2 root rules; verify 1.3–1.5 and 1.7.

## 3. Reasoning effort

- [x] 3.1 Add `ReasoningEffort`, parse it at the route, store it as `#[serde(skip)]` run metadata on `LlmConfig`; verify the 422 case of 1.6.
- [x] 3.2 Extend `DialectRequest` with `effort` and the per-dialect mapping; confirm the OpenAI-family parameter name through liter-llm before coding it; verify 1.6 and 1.7.
- [x] 3.3 Record `host_context` in `run.context`; verify 1.8.

## 4. Host history

- [x] 4.1 Add `HostHistory` types and route-level validation and limits; verify 1.11, 1.13, 1.15.
- [x] 4.2 Add `Session::seed_if_empty` and use it at the seeding site for both host and embedded seed paths; record the outcome in `run.context`; verify 1.9, 1.12, 1.14, 1.16.
- [x] 4.3 Return `history` and `seeded_messages` from `create_run`; verify 1.9 and 1.10.
- [x] 4.4 Advertise the `host_history`, `working_directory` and `reasoning_effort` flags in `GET /api/uar/capabilities` (added by `sidecar-launch-security`) once their behaviour ships; verify the endpoint lists each flag only when its contract tests pass.

## 5. Phase-boundary verification

- [ ] 5.1 Run `cargo test --test run_host_context --test host_history_restart` once after sections 2–4 are complete, then the broader integration gate once; record both outputs.
- [ ] 5.2 Run `openspec validate run-request-host-context --strict`; verify it passes.
