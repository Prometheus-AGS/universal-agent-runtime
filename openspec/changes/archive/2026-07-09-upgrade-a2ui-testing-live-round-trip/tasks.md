## 1. Backend: test-trigger endpoint

- [x] 1.1 Add `POST /api/uar/runs/{run_id}/a2ui/test-trigger` handler in `src/uar/a2ui/routes.rs`: verify the run exists via `run_manager.get_run`, construct a real `NormalizedEvent::ArtifactInputRequest` with a fresh UUID `artifact_id` from the request body's `{ artifact_type, title, content, metadata? }`, emit via `run_manager.emit_to_run`, return `{ run_id, artifact_id, status: "triggered" }`.
- [x] 1.2 Register the new route on the existing A2UI router (mounted alongside `submit_artifact_response`, same `/api/uar/runs` prefix).
- [x] 1.3 Reject with a clear 404 when the run doesn't exist or isn't active, matching `submit_artifact_response`'s existing not-found handling.

## 2. Frontend: rework A2uiTestingPage

- [x] 2.1 Add an active-runs picker to `A2uiTestingPage`, reusing `useEntities<RuntimeRunEntity>("RuntimeRun")` filtered to `status === "running" || status === "waiting"`, showing each run's title/thread_id.
- [x] 2.2 Add a content/metadata JSON editor pre-filled with a sensible example per builtin `artifact_type` (confirm/select/text_input/form) when a schema is selected.
- [x] 2.3 Add a "Trigger Test Artifact" button calling the new `test-trigger` endpoint against the selected run; surface real errors (not swallowed).
- [x] 2.4 On success, add a "Go to thread" action that sets the target run's `thread_id` active in the thread registry and navigates to `/threads`.
- [x] 2.5 Add an honest empty state when no runs are currently active ("start a conversation first").

## 3. Verify

- [x] 3.1 `cargo check` clean for the new backend route.
- [x] 3.2 `pnpm run build` clean for the frontend rework.
- [x] 3.3 Manual live verification: start a real chat conversation, trigger a test artifact of each builtin type against its run, navigate to the thread, confirm the real `A2uiInputBlock` renders and a real submission completes (backend logs/response confirm the round-trip, not just UI appearance). Automated as `tests/integration/live/a2ui_test_trigger_cases.rs` instead of a manual pass (real server + stub LLM, both tests passing); see findings.md for per-artifact-type coverage caveats.
- [x] 3.4 Confirm `git status --short` shows only the expected backend + frontend files changed.

## 4. Findings

- [x] 4.1 Write `findings.md` documenting the live verification results per artifact type, and confirming the resumable-stream-buffer risk from design.md (event delivery when the client isn't yet connected) behaves as expected.
