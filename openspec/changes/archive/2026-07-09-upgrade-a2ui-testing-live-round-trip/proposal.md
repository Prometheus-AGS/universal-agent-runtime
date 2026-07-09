## Why

`A2uiTestingPage` was assessed (`uar-production-ready-uiux-2026-07`) as developer-only tooling that never exercised a real round-trip — it lists real schemas but its "submit" action just echoes input locally. On review with the user, removing it was rejected: the capability it's reaching for is genuinely needed. Investigation found the real A2UI round-trip **already exists and works end-to-end in production chat** — when an agent requests input via A2UI, `A2uiInputBlock` renders a real form and submitting it calls the real `POST /api/uar/runs/{run_id}/artifact-response` endpoint, resuming the paused agent run. The gap is narrower than first assumed: there's no way to *trigger* that real flow on demand for testing/validation without waiting for an agent to naturally request one mid-conversation.

## What Changes

- **New backend endpoint** `POST /api/uar/runs/{run_id}/a2ui/test-trigger`: given `{ artifact_type, title, content, metadata? }`, verifies the run exists and is active, constructs a real `NormalizedEvent::ArtifactInputRequest` with a fresh `artifact_id`, and emits it into the run's real SSE stream via the existing `RunManager::emit_to_run` — the exact same mechanism a live agent tool-call uses. This is additive; it does not change `submit_artifact_response` or any existing route.
- **Rework `A2uiTestingPage`**: keep the existing real schema browser; add an "active runs" picker (reusing the same `useEntities<RuntimeRunEntity>("RuntimeRun")` real entity-graph data the Runtime Console's Runs page already uses, filtered to `running`/`waiting` status), a content/metadata editor pre-filled with a sensible example per artifact type, and a "Trigger Test Artifact" button that calls the new endpoint. On success, a "Go to thread" action navigates to `/threads` with that run's thread activated — the operator then sees the **real** `A2uiInputBlock` render and can submit it for real, through the exact same code path production chat uses. No new rendering component is built; this reuses the real chat components entirely.
- Honest empty state when no runs are currently active: testing requires a live conversation in flight (or very recently active, within `RunManager`'s tracking window) — this is disclosed, not hidden.

## Capabilities

### New Capabilities

- `a2ui-live-testing`: covers the ability to trigger a real A2UI artifact-input-request against an active run for testing/validation, and observe/complete the real round-trip through production chat components.

### Modified Capabilities

(none — this is additive to the existing, already-correct A2UI round-trip behavior)

## Impact

- Backend: `src/uar/a2ui/routes.rs` (new handler + route registration), no changes to existing handlers.
- Frontend: `frontend/src/admin/A2uiTestingPage.tsx` reworked (not deleted); no changes to `A2uiInputBlock`/`A2uiDisplayBlock` or the chat SSE pipeline — this change is additive scaffolding around already-correct, already-real components.
- No changes to `resolve-runtime-protocols-page-facade` (Round 2) — that change is about the Runtime Console Protocols page's dead AG-UI/model-route/A2UI-surface display panels, a separate, still-unresolved facade. This change does not require or imply resolving that one; noted here only because both touch A2UI-adjacent code, to avoid the two changes being conflated.
