## Why

Manual verification scenario C34 attempt 4 (2026-10-27) routed the request to the eligible houseplant specialist. The specialist's 252-event stream contained reasoning only, and the run ended `delegated_answer_missing: reasoning_only`. In attempt 2, the specialist's whole output was one unclosed `<think>` block of about 1,000 tokens, which fits truncation.

A local child turn is built without an output cap (`src/uar/runtime/thread/kernel.rs:593`). With no cap and no destination budget contract, `execute_request_inner` falls back to `RESPONSE_RESERVE_TOKENS` (1,000) (`src/uar/runtime/manager.rs`, `bounded_output`). For a reasoning model such as MiniMax-M3, that allowance can run out before any answer text.

The cap is the leading explanation, not a confirmed one: the finish reason is not recorded, and the accounting API had no rows for the child run.

Evidence: `.prometheus/verification/c34-live-roster-routing-attempt-00{2,4}-fail.json`. Plan: `.kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/children/orchestrator-routing-reasoning-repair/d2b/plan.md` (revision 2; operator decisions B1(a), B2 = 8,192, B3(a), B4(a), B5(a)).

## What Changes

- A captured child turn that runs on the local thread kernel, is not a team attempt, and has neither an explicit output cap nor a destination budget contract receives a delegated default output allowance of 8,192 tokens instead of 1,000.
  - This covers graph-routed children, `spawn_agent`-tool children, and follow-up turns to existing children.
  - For a known context window, the default is halved, but it never drops below the previous 1,000.
- The existing clamps still apply: explicit caller cap, destination contract, model maximum, and the context-capacity check.
- One content-free log line, `delegated output allowance bound`, records each captured child turn's `run_id`, bound `output_token_cap`, and its `source` (`caller`, `contract`, `delegated_default` or `reserve`).
- Unchanged:
  - `RESPONSE_RESERVE_TOKENS` for ordinary chats, provider probes and actors.
  - Team allocations and the team output check.
  - Accounting reservations: with no contract and no team grant, these are derived from catalog limits, not from the request cap.
- Changed side effects:
  - Context withheld from a child's input grows by the larger allowance.
  - Cost grows with the tokens actually generated, up to about 8 times per child model call.
- Archive order: three unarchived changes modify `multi-agent-orchestration`. Archive `reengineer-user-outcome-verification` first, then `fix-orchestrator-routing-reasoning`, then this change.

## Capabilities

### Modified Capabilities

- `multi-agent-orchestration`: a delegated child turn without an explicit output cap receives a delegated default allowance large enough for a reasoning model to reach its answer.

## Impact

- Code: `src/uar/runtime/manager.rs` only.
- Verification: build, `openspec validate`, owned restart, then the C34 manual BDD (`scripts/behaviors/live-roster-routing.cucumber.mjs`) rerun alone, plus a read of the new log line to confirm the child ran with 8,192. No unit or mock tests count as evidence.
- No API shape, persistence or configuration changes.
