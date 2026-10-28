---
type: SessionRecord
id: karpathy-session-cd49c8b176633594
title: Karpathy session cd49c8b17663
tags:
- karpathy
- session-learning
sources:
- resource: session:78ee08e5-9e3e-41a9-8cac-2b30d9ab88ae
generated:
  by: pk/1.11.0
  at: 2026-10-28T13:39:31.717285+00:00
created_at: 2026-10-28T13:39:31.717285+00:00
updated_at: 2026-10-28T13:39:31.717285+00:00
revision: 0
---

## Delta

BDD tests that prove the experience and the intended function. Fixtures are only a means: they belong where a scenario's starting state can't be reached through the product itself. Proposing fixture work as the next step was me drifting.

**Where the work so far drifted:**
- **Internal evidence.** Most of today's scenarios prove a real user outcome: a real model, the public API, and a correct answer that depends on private data. But I also asserted internal evidence where a user would only see the answer. The prompt-fragment manifests in E13/E14/E25/E27 are the clearest case. They are useful diagnostics, but they don't demonstrate an experience.
- **Plumbing for this fixture.** Several attempts went into Cedar permits, restarts and approval answering. This runtime requires those, but none of it is what a user sees. One real product finding came out of it: every catalog skill activation needs human approval, because `activate_skill` never declares its effect.

**The two remaining scenarios, by that test:**
- **E26:** the catalog definition asks for a skill universe of *exactly* {A, B}. That is a test-isolation demand, not an experience. What a user actually experiences is: with skills set to "all", the model can find and use the right skill among everything installed, and its rule drives the answer. That runs on the real runtime with its 212 skills, with no new fixture. I'd rewrite the scenario that way.
- **E03:** "my old memories expire after the retention period I chose, and opting out survives a restart" is a genuine experience. The only obstacle is time: proving expiry needs a record that really is eight days old. Writing one directly into the store is a fixture that fakes the product's own history. I'd keep the experience and choose between two honest options: run it once real aged data exists, or prove only the opt-out-survives-restart half now.

**E18** stays a real product failure either way: the LLM skill matcher couldn't select a skill, and the runtime hides why.

If you agree, I'll rewrite E26 as the user-facing outcome and run it on the main runtime, and treat E03 as "opt-out survives restart now, expiry once real aged data exists".

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 78ee08e5-9e3e-41a9-8cac-2b30d9ab88ae
- Captured: 2026-10-28T13:39:28.685923Z
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
