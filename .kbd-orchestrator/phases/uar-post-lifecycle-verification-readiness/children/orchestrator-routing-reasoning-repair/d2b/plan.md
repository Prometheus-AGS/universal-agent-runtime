# D2(b) plan — raise the delegated child's default output allowance

Stage: Plan (revision 2). Date: 2026-10-27. Status: **DRAFT for operator review. Execution is not authorized.** The verification gate is CLOSED.

Parent: child phase `orchestrator-routing-reasoning-repair`, plan revision 2, decision D2. After C34 attempt 4 the operator chose D2(b) on 2026-10-27: "Plan D2(b) as its own narrow change: raise the routed child's output allowance in manager.rs." This is a separate OpenSpec change inside the same child phase, and its work joins the phase's implementation boundary.

## Revision 2 — changes after independent critique

Reviewed by the `artifact-critic` subagent, which saw only this artifact and the source. Verdict: ACCEPT WITH AMENDMENTS. It confirmed T8 is implementable: `inherited` is only borrowed through `execute_request_inner`, `is_team_attempt` is a bool at `manager.rs:5211`, and the graph path reaches `execute_captured_thread` with `Some(bindings)`. It also confirmed the change is not a no-op for C34, because AG-UI sets the client-tools output cap to `None` (`agui.rs:129, 434`).

Amendments applied:

1. **A single C34 pass cannot prove cause.** Gate step 5 now confirms the allowance that actually applied (see B5). Decision B4 asks how many C34 passes are required.
2. A **timeout branch** is added to the exit criteria.
3. The **accounting statement** is qualified and corrects the parent plan's wording.
4. The **blast radius** now names follow-up turns to existing children. The context-reduction claim is corrected.
5. **Live-read premises** are labelled as such.
6. **B3(a)** can no longer go below today's 1,000. B3's regression claim is narrowed.
7. **Archive order** is named.
8. Line citations are corrected.

## Problem and evidence

- **C34 attempt 2.** The routed houseplant specialist's whole output was one unclosed `<think>` block: 4,161 characters, about 1,000 tokens, ending mid-sentence. That fits truncation at a 1,000-token cap.
- **C34 attempt 4** (binary `2ca5c057…`). The router selected the specialist. The specialist's stream had 252 events (UAR log line 1413) and none of them was an answer. The run ended `RUN_ERROR delegated_answer_missing: reasoning_only`.
- **C34 attempt 1.** The specialist's reasoning closed and a signed answer arrived within 1,000. So the current cap sometimes suffices.
- **Where the 1,000 comes from** (traced in source, not executed):
  1. `src/uar/runtime/thread/kernel.rs:593` builds the child turn with `RunExecutionRequest::new(...)`, which leaves `output_token_cap: None` (`turn/request.rs:193`).
  2. `manager.rs:3909-3912` merges in a cap from client tools, but AG-UI roots carry none.
  3. In the `bounded_output` closure (`manager.rs:6501`), the match at 6507-6512 falls back to `RESPONSE_RESERVE_TOKENS` (1,000; `context/reduce.rs:254`) when there is no cap and no destination budget contract.
  4. Line 6521 clamps the result to the configured or catalog maximum (`maximum` at 6517).
- **Live-read premises,** not provable from source:
  - The live `liter/MiniMax-M3` entry is configured with `max_output_tokens` 64,000 and `context_window` 1,048,576 (`GET /api/uar/providers/liter`).
  - The smallest configured window in the live roster is 200,000.
  - No host-installed destination contract applies (`manager.rs:6467-6469`). If one did, T8 would do nothing and this diagnosis would be wrong.
- **Unverified.** We don't know that the child actually stopped at the cap. The accounting API returned `[]` for child run `683b9e0e-…`, and the finish reason is not recorded (a named residual of the parent plan).

## Decisions for the operator

| ID | Question | Options | Recommendation |
|---|---|---|---|
| B1 | Which child turns get the new default | (a) every turn that runs on the local thread kernel (`inherited.is_some()`), is not a team attempt, and has neither an explicit cap nor a destination contract. That covers graph-routed children, `spawn_agent`-tool children and **follow-up `send_message` turns to existing children** (`thread/service.rs:984-1033`, `1345`), under any root including A2A-admitted roots. One condition in `manager.rs`. (b) graph-routed children only. That needs a marker carried from `AgentNode` through `AgentSpawnRequest` and `kernel.rs` to `manager.rs`: about 4 files plus a field on a persisted request. | **(a).** It is the smallest change, and the instruction names `manager.rs`. The cost is that all local child turns change, not only the route C34 exercises; this is named here. Unaffected: actors (`inherited = None`, `manager.rs:3620-3627`), A2A through actors (`a2a/handler.rs:351`), checkpoint resumes, and team children (`manager.rs:3919-3923`). |
| B2 | New default value | 4,096 · 8,192 · 16,384 | **8,192.** No measurement shows how much the model needs, so any value is a judgement. |
| B3 | Small-context models | (a) for this case only, clamp the child default to `max(RESPONSE_RESERVE_TOKENS, capacity / 2)`, so it never drops below today's 1,000; (b) no extra clamp | **Ask.** `manager.rs:6526` requires `capacity > cap`. The regression only bites when the model's `max_output` is unknown, or at least as large as a window of 8,192 or less; otherwise the 6521 clamp already protects it. This comes from reading the code, not from an observed failure, so it is put as a question. |
| B4 | How many C34 passes close the phase | (a) one pass, plus confirmation that the child ran with 8,192 (gate step 5); (b) three consecutive planned runs, all of which must pass | **Ask.** The old 1,000 cap also passed once (attempt 1). So one pass shows the fix allows success, not that it caused success. (b) is stronger evidence, but costs three live runs and is a planned repetition, not a retry. |

| B5 | How to prove the child ran with 8,192 | (a) T8 also adds one content-free `tracing::info!` after `bounded_output` resolves: `run_id`, `output_token_cap`, and `source` (`delegated_default` / `caller` / `contract` / `reserve`), message `delegated output allowance bound`, emitted for captured child turns only. That gives a direct reading. (b) No code: infer it from `context_reduction.input_allowance` in the run record | **(a).** The run API does not expose the budget receipt, and (b) is only an inference. The log line exists to answer this exact gate question. It carries no request content. |

## What does not change, and what does

Unchanged:
- `RESPONSE_RESERVE_TOKENS` stays at 1,000 for ordinary chats, provider probes (`api/providers.rs:520`) and actors.
- Team attempts keep their own allocation and the `TEAM_MAX_OUTPUT_TOKENS` check.
- An explicit caller cap still takes precedence: OpenAI-compatible callers' `max_tokens` (`server.rs:4994`, `5374`), client tools, or a destination budget contract.
- The model maximum still clamps the result (6521).
- **Accounting reservation.** With no contract and no team grant, `cost_budget.rs` `begin_call` (1840-1843) reserves from the catalog's `context_window` and `max_output`, not from `request.output_token_cap`, so a larger cap does not raise it. If the catalog has no entry for the model string, nothing is reserved, which is also unchanged.
  - **This supersedes the parent plan's description of D2(b)** (`plan.md:24`) as a change to "the run-level output reservation … a budget-governance change". That was written before the reservation path was traced.
  - Cost grows only with tokens actually generated: up to about 8 times per child model call.

Changed:
- The run context's recorded `request_budget.reserved_output_tokens` goes from 1,000 to 8,192 (`manager.rs:6793-6794`).
- Context withheld from the child's input grows by the same amount at both reduction points: `manager.rs:6684-6685` and `response_aware_context_limit` (6832-6838), consistent with `llm/orchestrator.rs:2021`. That is negligible on a 1,048,576 window (live-read).
- Skill catalog, summarizer and auxiliary budgets are unaffected.

## Tasks (one writer, this checkout; nothing runs until the gate opens)

| ID | Change | Files |
|---|---|---|
| T8 | Add a named constant `DELEGATED_CHILD_OUTPUT_TOKENS: u32 = 8_192` with a one-line rationale. In the `match output_token_cap` at `manager.rs:6507-6512`, the `None` arm with no destination contract currently yields `RESPONSE_RESERVE_TOKENS`. Make it yield that constant when `inherited.is_some() && !is_team_attempt`. Under B3(a), wrap it as `max(RESPONSE_RESERVE_TOKENS, capacity / 2)`, capped at the constant. All existing clamps follow unchanged. Update the comment above the closure. Under B5(a), add the single content-free `tracing::info!` described in B5 for captured child turns. | `src/uar/runtime/manager.rs` |
| T9 | New OpenSpec change `raise-delegated-child-output-allowance`: `proposal.md`, `tasks.md`, and a delta to `multi-agent-orchestration` adding "A delegated child turn without an explicit output cap receives the delegated default allowance". Scenarios: the default; explicit-cap precedence; the team exclusion; follow-up turns included. The proposal names the archive order for the three open changes touching `multi-agent-orchestration`: `reengineer-user-outcome-verification`, then `fix-orchestrator-routing-reasoning`, then this change. | `openspec/changes/raise-delegated-child-output-allowance/**` |
| T10 | Decision entry for B1–B4 and the superseded parent wording, plus an inventory note | `.prometheus/decisions.md`, `implementation-inventory.md` |

No unit or mock tests are authored. Under the operator's 2026-10-27 instruction, they would not count as evidence.

## Verification gate — exact commands (each needs operator approval)

Run serially from the repository root after a renewed implementation-complete inventory:

1. `CARGO_TARGET_DIR=target/uar-team-concurrency-repair cargo build --locked --no-default-features --features server-full --bin universal-agent-runtime`
2. `node /Users/gqadonis/Projects/prometheus/prometheus-skill-system/dist/plugins/claude/prometheus-skill-pack/skills/kbd-process-orchestrator/shared/openspec/cli.mjs run --project "$PWD" -- validate raise-delegated-child-output-allowance`
3. `node --input-type=module -e "import { restartOwnedUar } from './scripts/behaviors/support/restart-owned-uar.mjs'; const r = {}; await restartOwnedUar(r, process.argv[1]); console.log(JSON.stringify(r));" "$(shasum -a 256 target/uar-team-concurrency-repair/debug/universal-agent-runtime | cut -d' ' -f1)"`
4. `UAR_BASE_URL=http://127.0.0.1:60675 UAR_WORKSPACE_ID=uar-user-outcomes-007-supporting-workspace UAR_MODEL=liter/MiniMax-M3 node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/live-roster-routing.cucumber.mjs`, followed by review of the raw evidence. Under B4(b), run this exact command three times in sequence, and stop at the first failure.
5. Confirm the allowance that actually applied, per B5. `<child_run_id>` comes from the step 4 UAR log line `liter-llm stream complete` whose span names the specialist agent.
   - **B5(a):** read the new log line `delegated output allowance bound` for `<child_run_id>` in the owned UAR log, and confirm `output_token_cap=8192`.
   - **B5(b):** `curl -s -H 'x-uar-workspace-id: uar-user-outcomes-007-supporting-workspace' http://127.0.0.1:60675/api/uar/runs/<child_run_id>`, then compare `context_reduction.input_allowance` with attempt 4's child, `683b9e0e-…`. That run recorded 1,043,836, which is 4,740 below its window: the 1,000 cap plus about 3,740 of tool and other reserves. A drop of about 7,192 suggests the 8,192 cap applied. This is an inference, not a direct reading.
   - Either way, if the evidence is absent or ambiguous, report that the allowance actually applied is unconfirmed. Do not claim it caused the result.
   - *(Revision 2 correction. The first draft of this step read `request_budget.reserved_output_tokens` from the run record. A read of attempt 4's child record on 2026-10-27 showed the API does not expose that field: the receipt lives only in the in-memory run context, `manager.rs:8180-8184`.)*

Not run: unit tests, mock tests, the full suite, Clippy, release builds, CI, or any other BDD scenario.

## Exit criteria

- **Phase complete** only when all of these hold:
  - gate steps 1–3 pass;
  - C34 passes on Mac, under B4's chosen count, with genuine non-reasoning answers in its raw evidence;
  - step 5 confirms 8,192.
- **Step 5 unconfirmed.** If C34 passes but step 5 cannot confirm 8,192, the result goes to the operator as "passed, cause unconfirmed". It is not closed automatically.
- **Reasoning-only again.** If C34 fails with `delegated_answer_missing: reasoning_only` at 8,192, the phase stays open: a larger allowance is not the remedy. The operator chooses the next one, for example lower reasoning effort for delegated children or a different specialist model.
- **Timeout.** If C34 fails by client abort or step timeout (the route POST aborts at 120 s, `live-roster-routing.steps.mjs:40`; step timeout 125 s, line 7), or with a child `timeout_seconds` error, the phase stays open. The result goes to the operator as a separate outcome. The larger allowance may have traded truncation for latency. The test timeouts are not raised without operator approval.

## The uncomfortable part

This plan raises a cap on a hypothesis we have not confirmed. The most likely good outcome, one green C34, cannot tell a fix from a lucky run, because the old cap also passed once. That is why step 5 and B4 exist. The likely bad outcome is a slower failure: a reasoning model given 8 times the room can run into the 120-second client abort.

Either way, every local child turn and every follow-up turn pays for the larger allowance, and C34 observes none of those other paths.
