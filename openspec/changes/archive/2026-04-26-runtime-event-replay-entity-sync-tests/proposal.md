## Why

The runtime console now has visual coverage, but it does not yet have a deterministic validation path proving replayed runtime events normalize into the Prometheus entity graph and update the operator UI without a refresh. This is needed before deeper protocol and provider hardening because live runs, tool calls, approvals, artifacts, memory activity, provider health, routing decisions, AG-UI, and A2UI are the runtime's primary observability contract.

## What Changes

- Add deterministic runtime event replay fixtures for run lifecycle, run step, tool call, approval, artifact, memory, AG-UI, A2UI surface, model route decision, and provider health events.
- Add frontend tests that ingest replayed events through the runtime normalization entrypoints and assert the resulting entity graph state.
- Add UI-level replay coverage proving runtime console screens update visible run, tool, approval, artifact, provider, routing, AG-UI, and A2UI state without manual refresh.
- Add replay coverage for provider compatibility surfaces so provider health and model routing decisions remain inspectable as live state.
- Add replay coverage for realtime state deltas, including chunked AG-UI/A2UI-style updates where supported by existing frontend adapters.
- Update KBD workflow state with assessment, progress, blockers, and verification evidence for this validation hardening change.
- No breaking API, protocol, or persistence changes are intended.

## Capabilities

### New Capabilities

- `runtime-event-replay-entity-sync`: Deterministic replay fixtures normalize runtime events into the entity graph and drive runtime console UI updates without refresh.

### Modified Capabilities

- `frontend-validation-gate`: Require targeted runtime event replay/entity-sync checks as part of frontend validation for runtime console changes.
- `runtime-console-visual-verification`: Extend static console visual checks with replay-driven visible state checks for live runtime surfaces.
- `tool-approval-workflow`: Require replayed approval events to appear in the runtime console with their current state and action context.
- `agent-status-ui`: Require replayed run and step state transitions to update agent/runtime status displays.
- `a2ui-testing-ui`: Require replayed A2UI surface events and chunk-style updates to be visible in the protocol/testing UI.

## Impact

- Affected frontend code includes `frontend/src/entities/runtime-ingest.ts`, `frontend/src/entities/schemas.ts`, `frontend/src/entities/types.ts`, `frontend/src/entities/sync.ts`, `frontend/src/admin/pages/runtime-console-page.tsx`, and targeted frontend test files.
- Runtime UX impact: operators get validated live feedback for runs, steps, tools, approvals, artifacts, memory events, provider health, routing decisions, AG-UI events, and A2UI surfaces without manual refresh.
- Provider compatibility impact: provider health and model route decisions become replay-testable, which supports safer liter-llm provider UX and protocol compatibility work.
- Realtime state impact: replay fixtures exercise the same normalization and graph update paths used by live SSE/AG-UI/A2UI event streams, reducing regressions in live console behavior.
- Workflow impact: `.kbd-orchestrator/` state must be updated for this change, with Surreal Memory remaining a secondary mirror when available.
