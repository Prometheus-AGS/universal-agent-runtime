---
type: SessionRecord
id: karpathy-session-91f606150a7a27c9
title: Karpathy session 91f606150a7a
tags:
- karpathy
- session-learning
sources:
- resource: session:78ee08e5-9e3e-41a9-8cac-2b30d9ab88ae
generated:
  by: pk/1.11.0
  at: 2026-10-28T00:44:16.090436+00:00
created_at: 2026-10-28T00:44:16.090436+00:00
updated_at: 2026-10-28T00:44:16.090436+00:00
revision: 0
---

## Delta

I wrote revision 4 and a fresh critic reviewed it, tracing real create-and-update sequences this time. It found 2 new critical defects and 10 warnings, and I confirmed the key claims against the source. I've stopped before implementation. Rather than a fifth revision, I recommend re-scoping.

**Critical defects**
1. **The plan rests on state that is deleted 10 minutes after a run ends.**
   - Each run's A2UI state history is kept in memory and evicted after `retention_after_terminal_secs`, which defaults to 600 seconds (`manager.rs:1610`, `config.rs:344`).
   - The thread snapshot, the action check and a continuation's view of earlier surfaces all depend on that history.
   - A user who returns to a surface 10 minutes later would have their action refused, and the next run's snapshot would wipe what their screen shows.
   - Every test scenario finishes within 10 minutes, so the gate would pass anyway.
2. **Forwarding every state update would leak internal IDs.** Other state updates carry the native run ID and conversation ID (`manager.rs:7858-7868`), and approval records carry owner and run identifiers. The route must forward only A2UI updates.

**Main warnings**
- **Switching `replace` to `add` breaks an existing test.** C14's step file looks for `replace` (`a2ui-response.steps.mjs:159`), so my own regression gate would fail. It also contradicts an older, unarchived spec that requires `replace`.
- **The snapshot keeps only the last update, not the merged state.** Two data updates to different paths lose the first one.
- **Action continuations would skip the route's normal admission checks**, including deployment-binding checks.
- **Validating the surface test trigger changes its contract**, which breaks two test files no work package owns.
- **A missing or mistyped seed-folder setting is silently ignored.** Because seed files are authoritative, that could quietly delete every host component.
- **I19 still bundles about 12 independent cases**, against the same catalog rule I used to split I23.

The full findings are in the [plan](.kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/children/agui-a2ui-projection-and-custom-components/plan.md).

**Why re-scope:** four reviews in a row have each closed the previous findings and then uncovered new critical ones. Critical defect 1 shows the reason: the plan builds on short-lived, per-run, in-memory state that was never designed to be a thread's source of truth. My recommendation is three changes, in order:
1. **A small A2UI update fix:** store each surface's merged state, emit `add`, and update the C14 step file and the conflicting older spec. This also repairs chat replay.
2. **A durable surface store** for each owner and conversation, independent of run retention. The dedicated route's snapshot, action checks and catalog tracking would build on it.
3. **This change, rebuilt on those two**, with I19 split per the catalog rule.

Nothing is implemented or committed.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 78ee08e5-9e3e-41a9-8cac-2b30d9ab88ae
- Captured: 2026-10-28T00:44:11.382923Z
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
- .prometheus/session-log.md
- .prometheus/verification/macos-behavior-ledger.md
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
- openspec/changes/agui-a2ui-projection-and-custom-components/
- sdks/typescript/.acceptance-artifacts/
