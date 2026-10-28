---
type: SessionRecord
id: karpathy-session-d2afab3aedc25fcd
title: Karpathy session d2afab3aedc2
tags:
- karpathy
- session-learning
sources:
- resource: session:78ee08e5-9e3e-41a9-8cac-2b30d9ab88ae
generated:
  by: pk/1.11.0
  at: 2026-10-28T08:57:15.397047+00:00
created_at: 2026-10-28T08:57:15.397047+00:00
updated_at: 2026-10-28T08:57:15.397047+00:00
revision: 0
---

## Delta

## The theory

**A2UI** lets an agent describe a user interface as data rather than code. The agent sends four kinds of message:

- `createSurface` opens a UI region.
- `updateComponents` adds or replaces components by id: buttons, text, columns.
- `updateDataModel` sets values at paths in a JSON data model that components bind to.
- `deleteSurface` closes the region.

A renderer turns that stream into a live UI, and user clicks come back to the agent as actions. On the web, the renderer is `@a2ui/web_core`'s `MessageProcessor`. A2UI is built as a message stream, and the renderer applies those messages to build the state.

**AG-UI** is the transport between an agent run and a client. It carries:

- run lifecycle, text and tool-call events;
- a shared JSON state, sent as a full `STATE_SNAPSHOT` or as `STATE_DELTA` patches (RFC 6902).

**How UAR combines them.** It puts A2UI surfaces into AG-UI's shared state under `/a2ui/surfaces/...` and sends each A2UI message as a `STATE_DELTA`.

The same surface can be represented in two ways:

| Representation | What it is | Who can apply it |
|---|---|---|
| **Message log** | An ordered list of A2UI commands | Any A2UI renderer |
| **Materialized state** | The surface after those commands: components plus data model | Any JSON patch applier |

UAR mixes them up. It sends a *message* through a channel that expects a *state change*. That is the original bug.

## The high-level objective

Agent-generated UI should behave like a durable, shared, interactive object in a conversation:

1. **Live.** The user sees the surface appear and change as the agent works.
2. **Recoverable.** A reload, a reconnect, a second device or the runtime console shows the *current* surface, not an empty one.
3. **Interactive across turns.** A click starts a new run, and that run can update the same surface. Example: submit a form, then see the confirmation in place.
4. **Extensible.** Apps register custom components. The server checks them and every client renders them.
5. **Safe.** Agent output is untrusted. It cannot crash the server, inflate storage, or inject code into clients.

## What this means for the application today

- **Live chat works.** The chat renders each artifact on its own with `web_core`, so it never reads the broken state.
- **Recovery is broken.** The snapshot drops all A2UI state after the first update or action.
- **The runtime console can't replay a run that has actions.** It throws on action entries.
- **Interactivity across turns is broken.** An action's continuation is a new run, and surface state is tied to one run. The new run can't see or update the surface the user clicked.
- **Custom components** need one catalog definition that both the server and the client use. Today the server's admission rules and the client's validator already disagree.

So the visible gap is not "patches are wrong". UAR has no coherent model of who owns a surface's state, or how long that state lives.

## Why the revision loop didn't converge

Every revision of change 1 had the server compute the materialized state and the diffs from it. That forces a **second reducer**: a Rust copy of `web_core`'s logic, which must match the browser's copy exactly, on untrusted input.

Those two goals conflict:

- **Matching the browser** means copying its JavaScript quirks.
- **Being safe on a server** means departing from the browser wherever those quirks are dangerous.

Each review found the next place where the two goals collided. That was a sign the architecture was wrong, not that the plan needed more care.

There is a second structural problem. Surface state lives in the **run's event stream**, which is in memory, limited to 512 events, and scoped to one run. Surfaces outlive runs. That causes the ring limit, the continuation gap and the ordering limits.

## Possible architectural solutions

**A. Server reducer with diff patches (the current path).**
- **How:** Rust ports `web_core`'s reducer, computes the minimal dif

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 78ee08e5-9e3e-41a9-8cac-2b30d9ab88ae
- Captured: 2026-10-28T08:57:12.435609Z
- Project: /Users/gqadonis/Projects/prometheus/universal-agent-runtime

## Changed Paths

- .kbd-orchestrator/current-waypoint.json
- .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/children/uar-verification-simplification/progress.json
- .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/children/uar-verification-simplification/tasks.md
- .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/hooks-status.json
- .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/hooks.log.jsonl
- .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/progress.json
- .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/tasks.md
- .kbd-orchestrator/position-reminder.txt
- .kbd-orchestrator/position.json
- .prometheus/decisions.md
- .prometheus/session-log.md
- .prometheus/verification/macos-behavior-ledger.md
- openspec/changes/agui-runs-stream-fidelity/proposal.md
- .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/children/agui-a2ui-projection-and-custom-components/
- .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/children/uar-verification-simplification/children/agui-a2ui-projection-and-custom-components/
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
- openspec/changes/a2ui-surface-state-patches/
- openspec/changes/agui-a2ui-projection-and-custom-components/
- sdks/typescript/.acceptance-artifacts/
