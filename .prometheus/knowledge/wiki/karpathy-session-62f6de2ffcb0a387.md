---
type: SessionRecord
id: karpathy-session-62f6de2ffcb0a387
title: Karpathy session 62f6de2ffcb0
tags:
- karpathy
- session-learning
sources:
- resource: session:78ee08e5-9e3e-41a9-8cac-2b30d9ab88ae
generated:
  by: pk/1.11.0
  at: 2026-10-28T15:03:57.823833+00:00
created_at: 2026-10-28T15:03:57.823833+00:00
updated_at: 2026-10-28T15:03:57.823833+00:00
revision: 0
---

## Delta

I've written the plan for the minimal A2UI slice and had a fresh critic review it. As written, the slice would let a user click a surface only once, so it isn't ready to build. Nothing is implemented. PEM `@prometheus-ags/a2ui-react` 4.3.0 is now confirmed on npm.

**What the plan does:**
- **The agent's UI reaches the app.** UAR's dedicated AG-UI route sends each published surface as an `ACTIVITY_SNAPSHOT` with `activityType: "a2ui-surface"`, carrying the surface's full set of operations so far. That is the shape PEM 4.3.0 and upstream AG-UI clients consume.
- **The user's click reaches the agent.** UAR accepts `forwardedProps.a2uiAction` as a new run on the same thread. It checks the action strictly: a Button with that component id and that action name must exist on the surface.
- **The acceptance test is I19,** rewritten as the user outcome and driven by a real PEM 4.3.0 headless client.

**What the critic found:** 2 critical defects and 10 warnings. I confirmed the key ones in the source.

- **A surface stops accepting clicks after the first one.** The plan accepts actions only for surfaces from the thread's most recent run. The run that answers a click becomes the most recent run, so a second click on the same card is refused with `422`. I19 clicks only once, so it would have passed without noticing.
- **A busy run can lose its surface.** While a run is live, its history is a 512-event ring (`manager.rs:46`). A chatty agent can push the surface out, and then a valid first click is refused and a reconnect fails. I had blamed restarts for this risk; a restart is actually where the durable history may keep the surface.
- **Among the warnings:**
  - the action run's input gets corrupted when it reuses the thread's messages;
  - a `profile` field that UAR accepts makes strict clients reject the whole surface;
  - PEM's `actionId` retry key is ignored;
  - the test supplies its own action name rather than reading it from the rendered button;
  - the regression list misses five scenarios that use this route.
- **One claim was simply wrong:** the tool example already includes a Button, so the S3 change isn't needed.

**One finding from PEM's source isn't fixable in this slice.** PEM 4.3.0's default catalog id doesn't match the ids UAR publishes. An app that installs PEM with default settings sees a "Catalog not found" refusal and no surface. The test passes only by registering UAR's catalog id on the client. The lasting fix is for UAR and PEM to agree on one catalog id, which belongs to the deferred D11/D15 decisions.

**Both critical defects have the same root.** The plan treats a surface as belonging to the run that published it, but a user experiences it as belonging to the conversation. Fixing that means keeping a thread-scoped record of published surfaces, read both by the action check and when restating surfaces. That pulls part of the deferred D2/D4 durable-surface work into this slice.

The full review and my assessment are appended to `plan.md` and recorded in `.prometheus/session-log.md`.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 78ee08e5-9e3e-41a9-8cac-2b30d9ab88ae
- Captured: 2026-10-28T15:03:54.828873Z
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
- .prometheus/gotchas.md
- .prometheus/session-log.md
- .prometheus/verification/bdd-validation-catalog.md
- .prometheus/verification/macos-behavior-ledger.md
- .prometheus/verification/manual-runtime.json
- openspec/changes/agui-runs-stream-fidelity/proposal.md
- scripts/behaviors/support/record-mcp-peer.mjs
- .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/children/agui-a2ui-projection-and-custom-components/
- .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/children/uar-verification-simplification/children/agui-a2ui-projection-and-custom-components/
- .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/children/uar-verification-simplification/children/identity-pass-through/
- .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/evidence/independent-diagnostic-sweep-001/S6/attempt-1/universal-agent-runtime
- .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/evidence/independent-diagnostic-sweep-001/S7/attempt-1/libuniversal_agent_runtime.rlib
- .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/evidence/independent-diagnostic-sweep-001/S8/attempt-2/libuniversal_agent_runtime.rlib
- .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/evidence/main-boundary-v1.3.0-004/artifacts/
- .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/evidence/main-boundary-v1.3.0-005/artifacts/
- .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/evidence/main-boundary-v1.3.0-006/artifacts/
- .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/identity-pass-through-evaluation.md
- .prometheus/verification/2026-10-28T11-59-16.191Z-agent-revision-restart-macos.json
- .prometheus/verification/2026-10-28T12-00-53.097Z-kb-delete-restart-macos.json
- .prometheus/verification/2026-10-28T12-02-16.915Z-kb-delete-restart-macos.json
- .prometheus/verification/2026-10-28T12-04-21.629Z-skill-threshold-tie-macos.json
- .prometheus/verification/2026-10-28T12-07-37.057Z-skill-threshold-tie-macos.json
- .prometheus/verification/2026-10-28T12-09-10.845Z-skill-threshold-tie-macos.json
- .prometheus/verification/2026-10-28T12-11-14.696Z-skill-classifier-tfidf-macos.json
- .prometheus/verification/2026-10-28T12-12-40.175Z-skill-lexical-semantic-macos.json
- .prometheus/verification/2026-10-28T12-13-56.329Z-skill-lexical-semantic-macos.json
- .prometheus/verification/2026-10-28T12-15-17.939Z-skill-lexical-semantic-macos.json
- .prometheus/verification/2026-10-28T12-16-13.410Z-skill-lexical-semantic-macos.json
- .prometheus/verification/2026-10-28T12-17-32.601Z-skill-lexical-semantic-macos.json
- .prometheus/verification/2026-10-28T12-18-55.225Z-skill-lexical-semantic-macos.json
