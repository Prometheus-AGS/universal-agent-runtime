# Assessment — orchestrator-routing-reasoning-repair

Stage: Assess. Date: 2026-10-27. Method: source reading plus existing C34 evidence. No build, test, check or model call was run for this assessment.

## Observed problems (evidence)

| ID | Observation | Evidence |
|---|---|---|
| O1 | With the matching specialist eligible, the orchestrator refused the request about 3.5s after start: `RUN_STARTED` → `RUN_ERROR graph_routing_selection_invalid`, with no selection and no approval. | C34 attempt 3 receipt `.prometheus/verification/c34-live-roster-routing-attempt-003-fail.json`; UAR log line 1569 `node_id=router error=graph_routing_selection_invalid` |
| O2 | The same refusal hit attempt 2's post-removal fresh houseplant task. It is therefore suspect evidence, not proof of roster logic. | attempt-002 receipt; UAR log line 1514 |
| O3 | A routed child streamed only an unclosed `<think>` block (4,161 chars, cut mid-sentence, no answer), yet AG-UI emitted `RUN_FINISHED outcome success`. | attempt-002 receipt and evidence `live-roster-routing/6a9624bd-….json` |
| O4 | Attempt 1's routed child closed its reasoning and produced a genuine signed answer. The behavior is intermittent and depends on how much the model reasons. | attempt-001 evidence |

## Root causes (source-confirmed)

### R1 — The router compares raw classifier text, including inline reasoning, to agent IDs

- The Liter/OpenAI-compatible driver puts provider `delta.content` verbatim into `MessageDelta` (`src/llm/liter_driver.rs:790-796`). Only a separate `reasoning_content` field becomes `ReasoningDelta` (`:781-788`). MiniMax-M3 through Liter writes its reasoning *inline* as `<think>…</think>` in content.
- `RouterNode` registered-agent mode concatenates only `MessageDelta` text (`src/uar/runtime/graph/nodes/router_node.rs:186-187`). It then requires `response_text.trim() == candidate_id` (`:202-206`), otherwise it returns `graph_routing_selection_invalid` (`:217`). A valid ID preceded by a `<think>` block always fails.
- No helper in `src/` strips or splits inline `<think>` tags (none found).
- The classification request has `output_token_cap: None` (`router_node.rs:158`). `DestinationBudgetDriver` replaces that with the run cap (`manager.rs:72-74`). Without a destination contract the run cap is `RESPONSE_RESERVE_TOKENS = 1_000` (`src/uar/runtime/context/reduce.rs:254`; `manager.rs:6501-6527`). A long reasoning preamble can also exhaust this before the ID appears.
- The intermittency matches the model's choice to reason or not; this is consistent with O4.

### R2 — A routed child's turn that ends inside reasoning is reported as success

- `liter_driver.rs:826-832` uses `finish_reason` only to set `saw_terminal`/`saw_tool_finish`. `FinishReason::Length` is never distinguished or recorded, and the stream always ends with `Done` (`:903`).
- `RunCompletionCapture::record` (`src/uar/runtime/thread/execution.rs:60-94`) concatenates `ChatDelta` text into `Completed { output }` unless an `Error` event was seen.
- `AgentNode::apply_outcome` (`src/uar/runtime/graph/nodes/agent_node.rs:156-163`) rejects only output that is empty after trim. An unclosed `<think>…` block is non-empty, so it continues.
- AG-UI emits `RUN_FINISHED {"type":"success"}` for any run-done event unless the run status is Error (`src/uar/api/agui.rs:564-573`).
- The routed child gets no explicit output cap (`turn/request.rs:193`; `kernel.rs:593-605`), so it uses the same 1,000-token default (`reduce.rs:254`). 4,161 characters is about 1,000 tokens, which fits a length cutoff during reasoning. This is an inference; the finish reason itself was not observed because it is not recorded.

## Related, out of scope (named, not fixed here)

- Inline `<think>` text reaches clients as `TEXT_MESSAGE_CONTENT` (`agui.rs:638-641`). The `ag-ui-chat-conformance` spec expects separate reasoning frames (`openspec/specs/ag-ui-chat-conformance/spec.md:17-25`). This affects every MiniMax-M3 chat, not just routing. It is a separate change.
- A length-truncated *answer* (non-reasoning text cut short) is also reported as success. This phase only distinguishes "no answer at all", unless D3 below chooses otherwise.

## Existing test anchors (not run)

- `tests/test_chat_completion.rs:276` `orchestrator_run_routes_and_streams_delegated_answer` (a mock classifier returns the exact ID) — the natural place for the new cases.
- `tests/truthful_tool_outcomes.rs:945` `run_child_outcome_boundary`; `tests/agent_threads.rs:401`.
- No existing test asserts `graph_routing_selection_invalid`; the router, agent-node and delegation modules have no unit test modules.

## Specs

- `openspec/specs/multi-agent-orchestration/spec.md` — "The orchestrator agent delegates to sub-agents" (the final answer includes the sub-agent's contribution). There is no requirement on inline reasoning or truncated completions. The delta extends this capability.
- `openspec/specs/agent-thread-kernel/spec.md:61` — lifecycle events must not leak hidden reasoning. Any new diagnostic must respect this, so diagnostic codes must carry no classifier text.

## Uncomfortable points

- R2's length explanation is inferred, not observed. If the cause is different, for example a provider stream defect, the fix still makes the outcome honest but will not explain it. Recording the finish reason (plan task T3) is what would confirm it.
- After the fix, C34 can still fail honestly. If MiniMax-M3 regularly reasons past the routed child's 1,000-token allowance, the scenario will fail with an explicit code instead of a false success. Whether to change that allowance is decision D2. It is a product decision, not a test fix.

## Revision after independent critique (2026-10-27)

An artifact-only critic spot-checked every file:line citation above and found them accurate. It corrected the conclusions:

- **R1 is a hypothesis, not confirmed.** The mechanism is source-confirmed: a reasoning-wrapped ID would fail. That this is what happened in C34 is not. The classifier reply was never observed: the attempt-3 receipt says "root cause unconfirmed", and the classifier text is not logged. Attempt 3's whole scenario took 3,480 ms, registration and roster calls included. That makes the "exhausted 1,000 tokens of reasoning" sub-claim implausible for that attempt; it would need roughly 300 tokens/s. A short reply that is a *formatted* ID (backticks, quotes, a trailing period, "ID: x") fits the evidence equally well. The revised plan adds decision D0 (observe the reply shape first) and content-free router diagnostics.
- **Graph errors carry no code.** `manager.rs:7961-7968` emits `NormalizedEvent::Error { message: err, code: String::new() }`, and attempt 3 shows RUN_ERROR `code: ''` with the code text in `message`. New failures follow the same message convention.
- **Delegated output leaks reasoning downstream.** On success, `manager.rs:7970-7985` streams `[route]\n\n{output}` and records it in history, and the router feeds history back into the next classification (`router_node.rs:117-131`). Attempt 2's only text delta began `[disposable-plant-clinic-…]\n\n<think>`.
- **The same false-success pattern exists in other consumers of `Completed` output** (team execution, spawn/wait tools, actors, A2A thread service). These are named as residuals.
- **An explicit classification allowance cannot exceed the run cap** (`manager.rs:73-74`, `7192-7196`). Raising it is a run-level budget change.
- **Recording the finish reason as a new `uar.request` custom event would break strict stream consumers** (`context/summarizer.rs:173-185`, `matching/intent/mod.rs:247-253`) and would not reach the AG-UI stream C34 reads. R2's length explanation therefore stays an inference in this phase.
