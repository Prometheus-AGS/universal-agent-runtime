# Plan — orchestrator-routing-reasoning-repair (revision 2)

Stage: Plan. Date: 2026-10-27. Status: **DRAFT for operator review. Execution is not authorized.** The verification gate stays CLOSED until the operator approves the exact commands below.

Inputs: [assessment.md](assessment.md), including its "Revision after independent critique" section; OpenSpec change `openspec/changes/fix-orchestrator-routing-reasoning/`.

Revision 2 incorporates an artifact-only critique that rejected revision 1. Changes from revision 1:

- **T3 removed.** A new `uar.request/finish` custom event would break context summarization (`src/uar/runtime/context/summarizer.rs:173-185`) and LLM skill classification (`src/uar/runtime/matching/intent/mod.rs:247-253`) on every Liter call. It also could not reach the AG-UI stream that C34 reads.
- **D1(b) removed.** It had no effect: the graph driver clamps any request cap to the run's cap (`manager.rs:73-74`, `7192-7196`).
- **R1 downgraded to a hypothesis**, with a pre-build diagnostic (D0) and content-free router diagnostics (T2).
- The formatting case from G1 is now an explicit decision (D4).
- T4's output and error encoding are pinned down; scope and gate commands are made exact; the exit criterion now matches the handoff.

## Objective

Make registered-agent intent routing accept a valid choice that the model wraps in inline reasoning (and, if D4 is approved, light formatting). A delegated turn with no answer must stop being reported as a successful answer, and reasoning must stop leaking into the delegated contribution. Prove both through the real path by rerunning only C34.

## Decisions for the operator (needed before Execute)

| ID | Decision | Options | Recommendation |
|---|---|---|---|
| D0 | Observe the classifier reply before building | (a) build the fix on the hypothesis; (b) first send the router's exact system and user prompt to `MiniMax-M3` through Liter **three times** with the same 1,000-token limit, record only the reply *shape* (closed or unclosed `<think>`, wrapper characters, length, finish reason), and discard the text | **(b).** At most 3 model calls. It decides whether the cause is reasoning, formatting or truncation before the operator pays for a build and restart. |
| D2 | Routed child's output allowance (1,000 tokens by default) | (a) keep it and only fail honestly; (b) raise it, which changes the run-level output reservation in `manager.rs` (a budget-governance change, outside this phase's scope) | **(a).** State plainly: under (a), attempt 2 shows the child can use its whole allowance drafting, so **C34 FAIL with `delegated_answer_missing` is a likely outcome.** Then (b) would need its own approved change. |
| D3 | Answers that contain text but were cut off | (a) out of scope, named residual; (b) fail them | **(a).** Recording the finish reason safely needs a design that does not add stream events (see residuals). |
| D4 | Formatting around a valid ID (G1 "or formatting") | (a) reasoning only; (b) after excluding reasoning, also strip one layer of surrounding whitespace, backticks, straight or curly quotes, and one trailing period, **then require an exact ID match** | **(b).** This is normalization, not fuzzy matching. No substrings, prefixes, case folding or first-candidate fallback. |

## Tasks

One writer per file, in this checkout. Tests are **authored, not run**, until the gate opens.

| Task | Change | Files | Done when (source state) |
|---|---|---|---|
| T1 | Pure splitter `split_inline_reasoning(&str) -> InlineReasoning { answer: String, had_reasoning: bool, unclosed: bool }`. Rules: case-insensitive `<think>`/`<thinking>` opening tags, attributes allowed; closed blocks removed; an unclosed opening tag runs to the end and counts as reasoning; an orphan closing tag at the start (template-injected opening) means the text before it is reasoning. Limits stated in a doc comment: a literal tag inside a real answer is treated as reasoning; text joined across tool steps after an unclosed block is swallowed. | new `src/llm/inline_reasoning.rs`; `src/llm/mod.rs` (one `pub(crate) mod` line) | Function and `#[cfg(test)]` unit tests covering every rule above |
| T2 | RouterNode **registered-agent mode only**: run T1 on the reply, apply D4 normalization if approved, then require an exact eligible ID. Errors are content-free messages (graph errors carry an empty `code`; the message is the code, consistent with today's `graph_routing_selection_invalid`): `graph_routing_classifier_incomplete` when `unclosed` or the answer is empty; `graph_routing_selection_invalid` otherwise. Add one `tracing::warn!` on failure with **content-free** fields only: `answer_chars`, `had_reasoning`, `unclosed`, `normalized_changed`. No classifier text appears in errors or logs (`agent-thread-kernel` spec). Fixed-option mode is unchanged. | `src/uar/runtime/graph/nodes/router_node.rs` | Code plus a `#[cfg(test)]` module: closed reasoning then ID selects; unclosed → incomplete; non-ID → invalid; formatted ID behaves according to D4; fixed-option fallback unchanged |
| T4 | `AgentNode::apply_outcome` for `Completed { output }`: store **only T1's answer part** as the agent output. That stops closed reasoning reaching the `[route]\n\n{output}` delta, history and the next classification (`manager.rs:7970-7985`). If the answer part is empty: `NodeResult::Error` with message `delegated_answer_missing: reasoning_only` when `had_reasoning`, otherwise the existing "returned empty output" message (unchanged). This also applies to remote A2A results passing through `apply_outcome` (`agent_node.rs:254`). Named and accepted, not hidden. | `src/uar/runtime/graph/nodes/agent_node.rs` | Code plus a `#[cfg(test)]` module: reasoning-only → `delegated_answer_missing: reasoning_only`; closed reasoning plus answer → output equals the answer only; plain answer unchanged |
| T5 | Integration tests: the classifier mock replies with a closed `<think>` block plus the exact ID, and selects; the child mock replies with a closed block plus an answer, and the streamed text contains no `<think>`; the child mock replies with reasoning only, and the run ends in `RUN_ERROR` with message `delegated_answer_missing: reasoning_only`. | `tests/test_chat_completion.rs` (next to `orchestrator_run_routes_and_streams_delegated_answer`, line 276) | Tests authored |
| T6 | C34 oracle: classify a route that selected an agent and then failed separately ("selected then failed", reported with its message), instead of the misleading "refused route must not select". The post-removal refusal stays accepted only as `graph_routing_selection_invalid`; this is already enforced by the substring check, now made explicit as an equality check on the message. All other assertions unchanged. | `scripts/behaviors/steps/live-roster-routing.steps.mjs` | Assertions strengthened only |
| T7 | Gotcha entry and implementation-complete inventory. The OpenSpec delta was already drafted in planning. | `.prometheus/gotchas.md`, this directory | Inventory lists files changed and their source state |

Order: (D0 if approved) → T1 → T2 ∥ T4 → T5 → T6 → T7 → inventory → operator gate approval.

## Verification gate — exact commands (each needs operator approval)

Run serially from the repository root after the implementation-complete inventory.

0. *(D0, before implementation, only if approved)*. A disposable script in the session scratchpad. It sends the router's prompt, as built at `router_node.rs:117-150`, for a C34-like roster, to `http://127.0.0.1:4000/v1/chat/completions`, model `MiniMax-M3`, `max_completion_tokens` 1000, three times. It prints only the shape fields. The exact script is shown to the operator before running.
1. `CARGO_TARGET_DIR=target/uar-team-concurrency-repair cargo build --locked --no-default-features --features server-full --bin universal-agent-runtime`
2. ~~`CARGO_TARGET_DIR=target/uar-team-concurrency-repair cargo test --locked --no-default-features --features server-full --lib -- inline_reasoning router_node agent_node`~~ **Removed 2026-10-27 by operator instruction** (see note below).
3. ~~`CARGO_TARGET_DIR=target/uar-team-concurrency-repair cargo test --locked --no-default-features --features server-full --test test_chat_completion -- orchestrator`~~ **Removed 2026-10-27 by operator instruction** (see note below).
4. `node /Users/gqadonis/Projects/prometheus/prometheus-skill-system/dist/plugins/claude/prometheus-skill-pack/skills/kbd-process-orchestrator/shared/openspec/cli.mjs run --project "$PWD" -- validate fix-orchestrator-routing-reasoning`
5. Owned restart onto the new binary with the same config and store:
   `node --input-type=module -e "import { restartOwnedUar } from './scripts/behaviors/support/restart-owned-uar.mjs'; const r = {}; await restartOwnedUar(r, process.argv[1]); console.log(JSON.stringify(r));" "$(shasum -a 256 target/uar-team-concurrency-repair/debug/universal-agent-runtime | cut -d' ' -f1)"`
6. `UAR_BASE_URL=http://127.0.0.1:60675 UAR_WORKSPACE_ID=uar-user-outcomes-007-supporting-workspace UAR_MODEL=liter/MiniMax-M3 node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/live-roster-routing.cucumber.mjs`, followed by review of the raw evidence, not only the Cucumber status.

**Removal note (2026-10-27).** Steps 2 and 3 were planning additions that conflicted with the repo rule in `CLAUDE.md`: "Unit, module-local, mock-only, and filtered function tests do not count as completion evidence." Step 2 is filtered unit tests; step 3 runs against a mocked LLM. The operator removed both. Step 2 had already run (first attempt failed to compile on two stale test initializers; after an approved test-only repair it passed 20/20); that result is recorded in `implementation-inventory.md` and is not completion evidence. Step 3 never ran. The authored tests stay in source as code, not evidence.

Not run: the full test suite, Clippy, release builds, CI, or any other BDD scenario. Planning already ran `openspec validate` once as a spec-format check on the drafted change; this is disclosed, not counted as gate evidence.

## Exit criteria

- **Phase complete** only when C34 passes on Mac and its raw evidence shows genuine non-reasoning answers (matching handoff-in.md), and gate steps 1, 4 and 5 pass. *(Revised 2026-10-27: previously "gate steps 1–4", which made the removed unit and mock tests completion criteria.)*
- If C34 fails honestly, for example `delegated_answer_missing` under D2(a), the phase **stays open**. The result goes back to the operator with the backlog choice; it is not closed as done.
- On completion: handoff-out.md is written, and the waypoint returns to `uar-verification-simplification`.

## Named residuals (not fixed here)

- Inline reasoning still reaches clients as `TEXT_MESSAGE_CONTENT` on ordinary (non-delegated) MiniMax-M3 chats.
- Other consumers of `AgentThreadResult::Completed` keep the false-success behavior for reasoning-only output: team execution (`team_execution/execution.rs:39`, the same model as TEAM-01), spawn/wait tool results (`thread/service.rs:1242`), actors (`actor/system.rs:625`, `actor/agent_actor.rs:72`) and A2A thread service (`api/a2a/thread_service.rs:127,1587`).
- The provider finish reason stays unrecorded. A safe design must not add events that strict stream consumers reject.
- Splitter limits as documented in T1.
- The related unarchived change `reengineer-user-outcome-verification` also modifies `multi-agent-orchestration` ("Orchestrator intent routing uses the current registered roster"). Archive that one first, then this one.

## Risks

- If D0 is declined and the real cause is something else (for example an ID with other formatting), the build and restart are spent, and C34 fails again with the same code. T2's diagnostics would then at least show which case it was.
- The new binary is a new source cohort. Earlier receipts are not re-certified.
- The waypoint's generated `exactNextCommand` still says `/kbd-assess`, and the implementation count reads 0/0. Before Execute, the lead records the stage transition canonically; it is not hand-edited.
