---
type: SessionRecord
id: karpathy-session-3d67daac0f0983cc
title: Karpathy session 3d67daac0f09
tags:
- karpathy
- session-learning
sources:
- resource: session:78ee08e5-9e3e-41a9-8cac-2b30d9ab88ae
generated:
  by: pk/1.11.0
  at: 2026-10-28T04:02:28.765266+00:00
created_at: 2026-10-28T04:02:28.765266+00:00
updated_at: 2026-10-28T04:02:28.765266+00:00
revision: 0
---

## Delta

The critic confirmed that revision 4's port matches `web_core`'s `DataModel.set` and `get` statement by statement, and all 18 fixed cases trace correctly through both appliers. But that faithfulness is what causes the two new critical defects. I confirmed both against the source. The review is in the [design](openspec/changes/a2ui-surface-state-patches/design.md) and the KBD plan, and I've stopped before implementation.

**Critical**
1. **A long data path can crash the whole server.**
   - Nothing limits path length or depth (`protocol.rs`), and requests can be up to 10 MB (`server.rs:2898`).
   - A path like `/a/a/a/…` would build about 50,000 nested levels. Cloning, dropping and serializing that value is recursive, so the process aborts with a stack overflow.
   - The reduction runs under a single mutex shared by every run (`realtime.rs:121-124`), so every tenant goes down with it.
   - Both REST callers and model tool output can send such a path. The test corpus only goes 4 levels deep, so it can't catch this.
2. **Patch size can grow without limit.**
   - Rust fills array gaps with explicit `null`s where the browser leaves free holes. About 600 bytes of path can produce roughly 1 million nulls.
   - Writing more than one index past the end of an array resends the whole array, so repeated writes grow quadratically.
   - All of it is kept in the 512-event ring and in durable history, and sent to every subscriber.

Both fixes are departures from `web_core`: server-side caps on path depth, on how much an array may grow, and on the bytes of reduced state per surface and per run.

**Warnings**
- **The continuation check depends on the model.** If the continuation re-creates the surface with the same id, the assertion and its spec scenario are false.
- **The 60-second reader cap cuts off the 180-second continuation step.**
- **The event-budget assertion can never fail.** The ring is contiguous; the right test is whether the first retained event id is 1.
- **A server-side patch failure would only show up as a timeout.** The comparisons and the first-mismatch report would never run.
- **Case 16 both changes state and is refused.** The harness needs separate "changed" and "refused" flags.
- **Values in the generated corpus share objects.** Each sent value must be cloned.
- **The version-check path is wrong.** It should read `dirname(resolved)/../../package.json`.

The critic also pointed out that no Rust unit tests are planned, so every reducer bug would cost a full C14 run with model calls to find.

Mirroring a browser library's data model inside a shared server needs explicit limits that the browser never needed. The next revision should add them as named departures from `web_core`, rather than trying to match it more closely.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 78ee08e5-9e3e-41a9-8cac-2b30d9ab88ae
- Captured: 2026-10-28T04:02:25.984251Z
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
