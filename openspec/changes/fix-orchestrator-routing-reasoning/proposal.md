## Why

Manual verification scenario C34 found two orchestrator defects with MiniMax-M3, which writes its reasoning inline as `<think>…</think>` in content:

1. The registered-agent router compares the classifier's raw text, reasoning included, to agent IDs exactly. A valid choice preceded by reasoning is refused with `graph_routing_selection_invalid` (`src/uar/runtime/graph/nodes/router_node.rs:186-217`).
2. A routed child turn that ends inside unclosed reasoning, with no answer, is accepted as non-empty output, and the run reports `success` (`src/uar/runtime/graph/nodes/agent_node.rs:156-163`, `src/uar/api/agui.rs:564-573`). Closed reasoning in a delegated reply is also streamed and recorded as the contribution (`src/uar/runtime/manager.rs:7970-7985`).

Cause 1 is source-confirmed as a mechanism, but its trigger in C34 is unconfirmed: the classifier reply was never observed. Decision D0 in the plan proposes observing it before building.

Evidence: `.prometheus/verification/c34-live-roster-routing-attempt-00{2,3}-fail.json`. Assessment: `.kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/children/orchestrator-routing-reasoning-repair/assessment.md`.

## What Changes

- Add a pure inline-reasoning splitter (closed `<think>`/`<thinking>` blocks removed; an unclosed block runs to the end and counts as reasoning).
- The registered-agent router matches the answer part, optionally after operator-approved light normalization, exactly against eligible IDs. It adds the content-free failure message `graph_routing_classifier_incomplete`, keeps `graph_routing_selection_invalid` for answers that are not eligible IDs, and logs only content-free diagnostics.
- A delegated child's contribution is its answer part only. A reasoning-only result fails with `delegated_answer_missing: reasoning_only` instead of continuing as success. This also applies to remote A2A results that pass through the same agent node.
- Unchanged: fixed-option routing, run output allowances, provider finish-reason handling, client-visible inline reasoning on non-delegated chats, and other `Completed` consumers (team execution, spawn/wait tools, actors, A2A thread service). These are named residuals.
- Related: the unarchived change `reengineer-user-outcome-verification` also modifies `multi-agent-orchestration`; archive it first.

## Capabilities

### Modified Capabilities

- `multi-agent-orchestration`: registered-agent selection tolerates inline reasoning without loosening exact ID matching, and delegated turns without an answer are not reported as successful.

## Impact

- Code: `src/llm/inline_reasoning.rs` (new), `src/llm/mod.rs`, `src/uar/runtime/graph/nodes/router_node.rs`, `src/uar/runtime/graph/nodes/agent_node.rs`; tests in those modules and `tests/test_chat_completion.rs`.
- Verification script: `scripts/behaviors/steps/live-roster-routing.steps.mjs` (oracle tightened). Notes: `.prometheus/gotchas.md`.
- Verification: the C34 manual BDD (`scripts/behaviors/live-roster-routing.cucumber.mjs`) is rerun alone after an owned restart onto the new binary.
- No API shape, persistence or configuration changes.
