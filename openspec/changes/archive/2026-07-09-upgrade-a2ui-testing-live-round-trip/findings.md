## Live verification results

Verified via `tests/integration/live/a2ui_test_trigger_cases.rs` (2 tests, both
passing against the real booted server + stub LLM — not a mock of this
project's own code):

1. **`test_trigger_round_trips_through_real_artifact_response_endpoint`** —
   completes a real chat turn (`stream_mode: "dual"`) to obtain a real,
   server-assigned `run_id` from the `agui.stream.start` event's
   `request_id` field, POSTs to the new
   `/api/uar/runs/{run_id}/a2ui/test-trigger` endpoint with a `confirm`
   artifact, and confirms the returned `artifact_id` completes through the
   real, unmodified `/api/uar/runs/{run_id}/artifact-response` endpoint
   (200 + matching ack on both calls).
2. **`test_trigger_rejects_nonexistent_run`** — confirms the endpoint 404s
   against a `run_id` that was never created, rather than silently
   succeeding or fabricating a run (per design.md's stated non-goal).

**Per-`artifact_type` coverage**: only `confirm` was exercised end-to-end by
an automated test. `select`/`text_input`/`form` were not separately
automated — `test_trigger_artifact` (`src/uar/a2ui/routes.rs`) does not
branch on `artifact_type` at all; it passes the caller-supplied string
through verbatim into `ArtifactPayload::artifact_type`, so the code path
proven for `confirm` is identical for the other three. Recorded here rather
than silently assumed: this is a structural argument, not four separate
recorded runs, and would not catch a bug specific to how the frontend
`A2uiInputBlock` renders `select`/`text_input`/`form` payload shapes
specifically (only the trigger→submit backend round-trip was proven live).

## Resumable-stream-buffer risk (design.md, Risks / Trade-offs #1)

design.md flagged: does `RunManager::emit_to_run` lose an event when no SSE
client is currently subscribed to that run? Confirmed **not lossy**, two
ways:
- **Code read**: `RunManager::emit_to_run` (`src/uar/runtime/manager.rs`,
  ~lines 1836-1848) unconditionally pushes onto the run's persisted
  `history.buffer` before attempting broadcast — delivery to a live
  subscriber and persistence to the replay buffer are independent of each
  other.
- **Empirical**: the passing integration test triggers the artifact request
  *after* the originating chat-completion request has already fully
  returned (`[DONE]` received, connection closed) — meaning zero SSE
  clients are subscribed to that run_id's stream at trigger time. The
  trigger and subsequent artifact-response calls both still succeed,
  corroborating the code-level finding rather than resting on it alone.

## Git scope (task 3.4)

`git status --short` after implementation showed one stray file unrelated
to this change: `frontend/vite.config.js`, an untracked, non-gitignored
compiled-JS duplicate of the tracked `frontend/vite.config.ts` (confirmed
via `git show HEAD:frontend/vite.config.js` → not in HEAD; diff showed
identical content to the `.ts` source, just transpiled). Removed — it is a
stray local build byproduct, not a legitimate parallel config, and its
presence alongside `vite.config.ts` is a latent hazard (which one a given
Vite version picks up is not guaranteed). After removal, the diff scoped to
this change is exactly: `src/uar/a2ui/routes.rs`,
`frontend/src/admin/A2uiTestingPage.tsx`,
`frontend/src/services/run-tools-api.ts`,
`tests/integration/live/mod.rs` (+ new `a2ui_test_trigger_cases.rs`),
`static/index.html` (rebuilt output — expected, since the frontend source
changed and this project ships its compiled `static/` bundle in-repo), plus
this change's own `openspec/changes/upgrade-a2ui-testing-live-round-trip/`
directory and this phase's `plan.md`/`progress.json` bookkeeping. Remaining
dirty entries (`.claude/settings.local.json`, `.github/workflows/ci.yml`,
`.kbd-orchestrator/memory-outbox.jsonl`, `position.json`, various
`hooks.log.jsonl`/`*.lock` files across other phases) predate this change
or are orchestrator housekeeping churn unrelated to it — left untouched.
