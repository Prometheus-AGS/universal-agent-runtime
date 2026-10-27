---
type: SessionRecord
id: karpathy-session-dd4d04a1f9ef7df9
title: Karpathy session dd4d04a1f9ef
tags:
- karpathy
- session-learning
sources:
- resource: session:78ee08e5-9e3e-41a9-8cac-2b30d9ab88ae
generated:
  by: pk/1.11.0
  at: 2026-10-27T21:38:38.142611+00:00
created_at: 2026-10-27T21:38:38.142611+00:00
updated_at: 2026-10-27T21:38:38.142611+00:00
revision: 0
---

## Delta

C53 failed again, but the revised checks did their job on the writing turn. The failure came from a product bug: sending the next message erases most of the previous turn's bubbles, including its "Selected agent" notice. tier3.allow is deleted.

**Script change (as you approved)**
- **Per turn:** exactly one assistant bubble must contain the specialist's answer, and the selection notice must sit before it.
- **Timing check:** it now measures that answer bubble. Before, it took the first assistant text of the turn, which could be the policy card.
- **Final step:** each specialist's answer must appear exactly once, instead of requiring exactly 2 bubbles.

**What attempt 4 showed**
- **Writing turn passed every check:**
  - one selection event, for writing-specialist by intent, before the answer text;
  - the approval answered in the browser;
  - the writer's byline in exactly one bubble;
  - the selection notice visible 12.9 seconds before the answer text.
- **Math turn failed:**
  - It raised a `spawn_agent` approval that the test never answered.
  - About 5 minutes later the server ended the run with "Tool lease or budget reservation expired before claim". That's the correct server response to an approval nobody answered.

**Why**
- **Product bug:**
  - When the math message was sent, the writing turn kept only its user message and final answer. Its policy card, approval card and "Selected agent: Blog and Article Writer" notice disappeared.
  - When a stream ends, `finishStream` saves only the last bubble it created to the local database. The earlier bubbles of that turn exist only in memory.
  - Before each send, `startStream` reloads history from the local database, which wipes them. So every turn that pauses for approval loses its visible agent selection.
- **Knock-on effect in the script:** it finds the new approval by counting "Review approval" buttons and assumes earlier ones stay. The writing turn's button vanished, so the script waited for a second button that never appeared.

**Proposed fixes**
- **Product (`chat-message-store.ts`, about 10 lines):** track every assistant bubble a stream creates. When the stream finishes, save all of them, marked complete. Saving is idempotent: the insert overwrites a message that already exists.
- **Script:** find the new approval by its position after this turn's user message, not by counting buttons.

The receipt is [c53-browser-agent-selection-attempt-004-fail.json](.prometheus/verification/c53-browser-agent-selection-attempt-004-fail.json), and the [ledger](.prometheus/verification/macos-behavior-ledger.md) is updated. Nothing is committed.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 78ee08e5-9e3e-41a9-8cac-2b30d9ab88ae
- Captured: 2026-10-27T21:38:35.491804Z
- Project: /Users/gqadonis/Projects/prometheus/universal-agent-runtime

## Changed Paths

- .kbd-orchestrator/current-waypoint.json
- .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/progress.json
- .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/tasks.md
- .kbd-orchestrator/phases/uar-verification-simplification/plan.md
- .kbd-orchestrator/position-reminder.txt
- .kbd-orchestrator/position.json
- .prometheus/decisions.md
- .prometheus/gotchas.md
- .prometheus/session-log.md
- .prometheus/verification/behavior-status.md
- .prometheus/verification/macos-behavior-ledger.md
- .prometheus/verification/macos-context-budget.md
- .prometheus/verification/manual-runtime.json
- frontend/src/features/chat/use-chat-runtime.ts
- frontend/src/services/chat-titles-api.ts
- frontend/src/stores/chat-message-store.ts
- frontend/src/stores/chat-stream-store.ts
- openspec/changes/simplify-user-workflow-verification/specs/product-validation-evidence/spec.md
- openspec/changes/simplify-user-workflow-verification/tasks.md
- scripts/behaviors/README.md
- scripts/behaviors/steps/knowledge-restart.steps.mjs
- scripts/behaviors/steps/math-knowledge.steps.mjs
- scripts/behaviors/steps/two-kb-answer.steps.mjs
- src/llm/mod.rs
- src/server.rs
- src/server/tests.rs
- src/uar/api/discovery.rs
- src/uar/context/strategy.rs
- src/uar/persistence/providers/ingestion_records/surreal.rs
- src/uar/runtime/context/summarizer.rs
- src/uar/runtime/graph/nodes/agent_node.rs
- src/uar/runtime/graph/nodes/router_node.rs
- src/uar/runtime/manager.rs
- src/uar/runtime/presentation_history_tests.rs
- src/uar/runtime/skills/service.rs
- static/.vite/markdown-engine-graph.json
- static/index.html
- tests/test_chat_completion.rs
- .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/children/orchestrator-routing-reasoning-repair/
- .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/evidence/independent-diagnostic-sweep-001/S6/attempt-1/universal-agent-runtime
