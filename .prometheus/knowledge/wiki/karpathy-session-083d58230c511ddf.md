---
type: SessionRecord
id: karpathy-session-083d58230c511ddf
title: Karpathy session 083d58230c51
tags:
- karpathy
- session-learning
sources:
- resource: session:78ee08e5-9e3e-41a9-8cac-2b30d9ab88ae
generated:
  by: pk/1.11.0
  at: 2026-10-28T08:53:50.619158+00:00
created_at: 2026-10-28T08:53:50.619158+00:00
updated_at: 2026-10-28T08:53:50.619158+00:00
revision: 0
---

## Delta

## What change 1 fixes

When an agent changes an A2UI surface, UAR sends a state patch that describes the *message* rather than the *state it produces*. Three things break as a result:

- **The first update after a surface is created fails to apply.** It is a `replace` at a key that doesn't exist yet.
- **Each update overwrites everything before it**, instead of updating one component or one data path.
- **Actions fail to apply**, because the starting state has no `actions` object.

After any of these, the run stream's state snapshot gives up and drops all state. You asked for AG-UI/A2UI next, including I19 and custom registered components, and that work depends on this state being correct. So we split it into three changes:

1. Correct patches (this change).
2. A durable surface store scoped to a thread.
3. The projection and custom components, rebuilt on changes 1 and 2.

## Why there are revisions

Your instruction for each round was: write the plan, have a fresh critic review it, then stop before implementation. The verification gate is closed, so nothing can be built or run to try ideas out. The critic is the only check before code is written. Every review so far has found at least one critical defect, and each time you approved another revision.

| Revision | Critical | Warnings | What the critical defects were |
|---|---|---|---|
| 1 | 1 | 10 | The reducer's rules didn't match the real A2UI library (`@a2ui/web_core`) |
| 2 | 1 | 12 | It still didn't match, and the trigger request body was wrong |
| 3 | 1 | 6 | Three more mismatches with web_core's `DataModel.set` |
| 4 | 2 | 7 | The match was now exact, and that exposed two ways to crash the server or bloat it |

## What revision 5 was going to do

Revision 4 copied web_core's browser behavior exactly. On a shared server that is unsafe:

- **A crafted data path can crash the server.** A path thousands of levels deep, sent through the REST route or model output, overflows the stack and takes every tenant down.
- **A short path can create about a million `null`s.** Repeated writes past the end of an array also grow quadratically.

Revision 5 would add caps on:

- path depth;
- how far a write may grow an array;
- the bytes of state per surface and per run.

It would also fix the seven test-harness warnings and add Rust unit tests for the reducer, to run at the gate.

## Why we're not finished, and my part in it

Each revision fixed the previous critical defects, but the critic kept finding new ones a level deeper. The plan isn't converging cleanly, and the cause is a design choice I made: copying web_core's full `DataModel.set` behavior onto the server, JavaScript quirks included. That is a large surface, every review finds another edge, and the caps in revision 5 add more of it. Four revisions in, no product code exists for change 1, and changes 2 and 3 haven't been planned.

The actual bug only needs patches that apply. A smaller design would get there:

- **Send the whole surface each time.** After each message, send one `add /a2ui/surfaces/<id>` patch carrying the whole reduced surface. That removes the hardest part of the plan, the rules for choosing which patch to send.
- **Accept only a safe subset of data paths.** Reject anything deeper than a fixed limit, writes that leave gaps in an array, and the JavaScript-quirk paths.
- **Keep the byte caps.** They also bound the size of each patch.

The trade-off is bigger patches, up to the per-surface cap, and the server would refuse some writes the browser would accept. I recommend this over revision 5. Revision 5 is still available if you'd rather keep exact web_core behavior.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 78ee08e5-9e3e-41a9-8cac-2b30d9ab88ae
- Captured: 2026-10-28T08:53:47.447923Z
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
