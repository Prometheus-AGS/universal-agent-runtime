---
type: SessionRecord
id: karpathy-session-1a7004fa2a264b02
title: Karpathy session 1a7004fa2a26
tags:
- karpathy
- session-learning
sources:
- resource: session:78ee08e5-9e3e-41a9-8cac-2b30d9ab88ae
generated:
  by: pk/1.11.0
  at: 2026-10-27T23:15:09.048822+00:00
created_at: 2026-10-27T23:15:09.048822+00:00
updated_at: 2026-10-27T23:15:09.048822+00:00
revision: 0
---

## Delta

I found the I19 problem from the source alone, without compass. There are two gaps on the dedicated AG-UI route, and a third finding: custom registered components don't exist yet. I haven't changed anything.

**1. The dedicated AG-UI route drops A2UI events (outgoing).**
- The route `/api/uar/ag-ui/agents/{id}` passes events through `Projector::project` (`src/uar/api/agui.rs:635–652`). It handles only text, tool calls, approvals, client tools, skill activation and agent selection. Everything else falls into `_ => Vec::new()` and is silently dropped.
- The chat route's adapter does send the A2UI-related events (`src/uar/api/adapters.rs:205–229` and `:332–337`). The dedicated route drops all of them:

| Event | What the chat route sends |
|---|---|
| A2UI surface (`Artifact`, `ArtifactDisplay`) | `CUSTOM uar.artifact.available` |
| A2UI input request (`ArtifactInputRequest`) | `CUSTOM uar.artifact.input_required` |
| State change (`StatePatch`) | `STATE_DELTA` |

- **Fix:** add those match arms to the projector, using the same names and payloads as the chat adapter but this stream's thread and run IDs, as the skill and agent-selection arms already do. That's roughly 15 lines.

**2. An A2UI action can't come back through the dedicated route (incoming).**
- Resuming a run on the dedicated route accepts only tool-approval answers and MCP elicitation answers (`agui.rs:347–367`).
- A button press or form submit in the UI goes to `POST /api/uar/runs/{native_run_id}/a2ui/actions` (`src/uar/a2ui/routes.rs:1428`). The UI then follows the continuation on the chat or native stream.
- A client of the dedicated route never sees the native run ID, so it has no way to send the action.
- **Proper fix:** accept the action as a new AG-UI run on the same thread, carried in `forwardedProps.uar`. The route would hand it to the existing action handler and stream the continuation under the new run ID. This is a design change, and larger than fix 1.

**3. Custom registered components don't exist yet.**
- The A2UI component set is fixed at 9 types: Text, Button, TextField, CheckBox, ChoicePicker, Row, Column, Card and Divider. That holds on the server (`src/uar/a2ui/protocol.rs:267–279`) and in the client's schema (`frontend/src/platform/a2ui/protocol.ts`).
- Only two catalogs are accepted: UAR's own and the standard A2UI basic catalog (`protocol.rs:264`). Anything else is rejected as "unapproved A2UI catalog" or an unknown component.
- A component registry appears only as a proposal in `docs/full-implementation/A2UI + AG-UI.md`.
- Supporting custom components is a new feature, not a test to run. It needs a server-side registry of allowed components with schema validation, a matching client renderer registry, and a decision on how components get registered. It should go through a planned OpenSpec change.

**Ready to test now without product changes:** I01–I04, the remaining chat and native AG-UI streaming modes. Also likely due is a rerun of the earlier browser A2UI scenarios C14/C15, since the chat-store code they depend on changed this session.

The diagnosis is in the [ledger](.prometheus/verification/macos-behavior-ledger.md).

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 78ee08e5-9e3e-41a9-8cac-2b30d9ab88ae
- Captured: 2026-10-27T23:14:57.285174Z
- Project: /Users/gqadonis/Projects/prometheus/universal-agent-runtime

## Changed Paths

- .prometheus/verification/macos-behavior-ledger.md
- .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/evidence/independent-diagnostic-sweep-001/S6/attempt-1/universal-agent-runtime
- .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/evidence/independent-diagnostic-sweep-001/S7/attempt-1/libuniversal_agent_runtime.rlib
- .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/evidence/independent-diagnostic-sweep-001/S8/attempt-2/libuniversal_agent_runtime.rlib
- .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/evidence/main-boundary-v1.3.0-004/artifacts/
- .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/evidence/main-boundary-v1.3.0-005/artifacts/
- .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/evidence/main-boundary-v1.3.0-006/artifacts/
- .prometheus/verification/browser-agent-selection/c53-attempt-001-artifacts/artifacts/features-browser-agent-sel-941f6-h-selected-specialist-first/trace.zip
- .prometheus/verification/browser-agent-selection/c53-attempt-002-artifacts/artifacts/features-browser-agent-sel-941f6-h-selected-specialist-first/trace.zip
- .prometheus/verification/browser-agent-selection/c53-attempt-003-artifacts/artifacts/features-browser-agent-sel-941f6-h-selected-specialist-first/trace.zip
- .prometheus/verification/browser-agent-selection/c53-attempt-004-artifacts/artifacts/features-browser-agent-sel-941f6-h-selected-specialist-first/trace.zip
- .prometheus/verification/browser-agent-selection/c53-attempt-005-artifacts/artifacts/features-browser-agent-sel-941f6-h-selected-specialist-first/trace.zip
- .prometheus/verification/browser-history-reload/c35-attempt-001-artifacts/artifacts/features-browser-history-r-5dad2-tinue-from-its-earlier-fact/trace.zip
- .prometheus/verification/browser-history-reload/c35-attempt-002-artifacts/artifacts/features-browser-history-r-5dad2-tinue-from-its-earlier-fact/trace.zip
- .prometheus/verification/browser-history-reload/c35-attempt-003-artifacts/artifacts/features-browser-history-r-5dad2-tinue-from-its-earlier-fact/trace.zip
- .prometheus/verification/browser-history-reload/c35-attempt-004-artifacts/artifacts/features-browser-history-r-5dad2-tinue-from-its-earlier-fact/trace.zip
- .prometheus/verification/browser-history-reload/c35-attempt-005-artifacts/artifacts/features-browser-history-r-5dad2-tinue-from-its-earlier-fact/trace.zip
- .prometheus/verification/browser-history-reload/c35-attempt-006-artifacts/artifacts/features-browser-history-r-5dad2-tinue-from-its-earlier-fact/trace.zip
- docs/validation/
- history.txt
- sdks/typescript/.acceptance-artifacts/
