# Handoff out — orchestrator-routing-reasoning-repair → uar-verification-simplification

Date: 2026-10-27. Closed at operator instruction: "Close orchestrator-routing-reasoning-repair: write handoff-out.md, record the phase reflection, and return the waypoint to uar-verification-simplification. No commit."

## Result for the BDD track

- **C34 PASSED on Mac, attempt 5**, credited as **B37**. Receipt: `.prometheus/verification/c34-live-roster-routing-macos-pass.json`. Evidence: `.prometheus/verification/live-roster-routing/a6557c4a-a8fd-4736-8eae-514050b540d0.json`.
- Attempts 1–4 keep their fail receipts (`c34-live-roster-routing-attempt-00{1,2,3,4}-fail.json`).
- Catalog and status are updated: 34 outcomes credited, 68 planned, 20 common workflows remaining, 37 behavior labels.
- The next scenario is the operator's choice, from C35 on, one ID at a time.

## Runtime state the BDD track inherits

- Owned UAR runs as **PID 340** on binary `b4e68525240ce6f99028176503135ff831d022866bc2fb52ed5d737de8e688be`, with config `8726e43a…` and the store preserved. It is behind the proxy at `http://127.0.0.1:60675`. Log: `target/prompt-macos-hbitxj2c/server-conversation-restart-08b5f07a-eaab-4b08-a183-8268feef75f3.log`.
- **Behavior changes later scenarios will meet:**
  - Registered-agent routing ignores inline `<think>` reasoning before its exact ID match. It also strips one layer of backticks or quotes and a trailing period. A reply that ends inside reasoning is refused with `graph_routing_classifier_incomplete`.
  - Delegated contributions are answer text only. A reasoning-only child fails with `delegated_answer_missing: reasoning_only`.
  - Local delegated child turns without an explicit cap get an **8,192-token output allowance** instead of 1,000. That includes `spawn_agent` children and follow-up turns; team turns are excluded. The owned log records `delegated output allowance bound` for each one.
- **Test oracle:** the C34 steps treat "selected then failed" as its own disposition, and the post-deletion refusal must be exactly `graph_routing_selection_invalid`.

## Source state (uncommitted, on `codex/mac-manual-behavior-progress`)

- **Product:**
  - `src/llm/inline_reasoning.rs` (new)
  - `src/llm/mod.rs`
  - `src/uar/runtime/graph/nodes/router_node.rs`
  - `src/uar/runtime/graph/nodes/agent_node.rs`
  - `src/uar/runtime/manager.rs`
- **Tests:**
  - authored but not counted as evidence: `tests/test_chat_completion.rs`
  - stale-initializer repairs: `src/uar/runtime/presentation_history_tests.rs`, `src/server/tests.rs`
- **Script:** `scripts/behaviors/steps/live-roster-routing.steps.mjs`.
- **OpenSpec:** `fix-orchestrator-routing-reasoning`, `raise-delegated-child-output-allowance` (both validated, neither archived). Archive order: `reengineer-user-outcome-verification`, then `fix-orchestrator-routing-reasoning`, then `raise-delegated-child-output-allowance`.
- **Not run** (out of this phase's gate): the full test suite, Clippy, `cargo fmt --check`, and any other BDD scenario.

## Open residuals

See [reflection.md](reflection.md):
- inline reasoning reaching clients on non-delegated chats
- other `Completed` consumers still reporting reasoning-only output as success
- unrecorded finish reason
- the 8,192 allowance's effect on paths C34 does not exercise
