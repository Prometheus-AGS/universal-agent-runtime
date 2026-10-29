# Implementation-complete inventory — orchestrator-routing-reasoning-repair

Date: 2026-10-27. Plan: [plan.md](plan.md) revision 2, approved by the operator with D0(b), D2(a), D3(a), D4(b).
**Verification gate: CLOSED.** No build, test, check or C34 run was executed for the implementation. The only execution in this stage was the operator-approved D0 probe ([d0/README.md](d0/README.md)).

Base commit: `6659caf35f6d39f41db2daa355e48a2612329a16` on `codex/mac-manual-behavior-progress`. The working tree also contains earlier uncommitted session work, which is not part of this inventory.

## Planned items → source state

| Item | Status | Files (SHA-256 at inventory time) |
|---|---|---|
| D0 classifier-shape probe | Done (approved run, 3 calls) | Result in `d0/README.md`; script kept in the session scratchpad only (it reads the gateway key) |
| T1 inline-reasoning splitter + unit tests | Implemented | `src/llm/inline_reasoning.rs` (new) `785658ba207ac0d76d08a1ca77ed685cf397350ffba5183648ecb09f50429105`; `src/llm/mod.rs` (+1 line `pub(crate) mod inline_reasoning;`) `24c6e00d64568bb1224771ab1699e924a73e7ba14dc0c91e35a0090ae8d81832` |
| T2 router: answer-part exact match, D4 normalization, `graph_routing_classifier_incomplete`, content-free `warn!` diagnostics, unit tests; fixed-option path unchanged | Implemented | `src/uar/runtime/graph/nodes/router_node.rs` `c9ca48d15a5c5858119cf2a7b2a15c5758d2da71e5600cbea26191a526d9ee32` |
| T4 agent node: contribution is answer text only; `delegated_answer_missing: reasoning_only`; plain output kept as written; unit tests; module doc updated | Implemented | `src/uar/runtime/graph/nodes/agent_node.rs` `cbbb706987d5681fd47d9c308b1bf838315efb3f249c083d85b190b5db98ccca` (includes the module-doc edit) |
| T5 integration tests (think-wrapped classifier ID; reasoning-only child; unclosed classifier) | Authored | `tests/test_chat_completion.rs` `ae62128b32a37212eff8b3aaa8f360587783bc2a0241900a73452a5e654b30ce` (appended; existing tests unchanged) |
| T6 C34 oracle: "selected then failed" reported separately; post-removal refusal must equal exactly `graph_routing_selection_invalid` | Implemented | `scripts/behaviors/steps/live-roster-routing.steps.mjs` `eb219ab30d2771448323c06116a82564a31b82ce741e02d5e8fed75caa486df8` |
| T7 gotcha entry; OpenSpec tasks ticked | Done | `.prometheus/gotchas.md`; `openspec/changes/fix-orchestrator-routing-reasoning/tasks.md` |

Removed by plan revision 2 (not implemented): T3 finish-reason event, D1(b) classifier allowance. Deferred: D2(b), D3(b).

## Unit tests authored (names for the gate filter)

- `llm::inline_reasoning::tests::*` (10 tests)
- `uar::runtime::graph::nodes::router_node::tests::*` (6 tests)
- `uar::runtime::graph::nodes::agent_node::tests::*` (4 tests)
- `tests/test_chat_completion.rs`: `orchestrator_selects_id_after_inline_reasoning_and_streams_answer_only`, `orchestrator_fails_when_delegated_child_returns_only_reasoning`, `orchestrator_refuses_classifier_reply_that_ends_inside_reasoning`. The filter `orchestrator` also selects the existing `orchestrator_run_routes_and_streams_delegated_answer`.

## Known unverified points (source review only)

- Nothing has been compiled. Compile-sensitive constructs were checked by reading: let-chains with `if let … &&` (edition 2024, already used in the repo); `Option::is_none_or` (already used in `src/llm/health.rs` and `src/config.rs`); closure argument types made explicit (`split.answer.as_str()`).
- T5 assumes the router classification happens before the spawn approval interrupt. C34 attempt 3 showed exactly that (`RUN_STARTED` → `RUN_ERROR` with no interrupt).
- `_agent_result_{node}` graph state still stores the child's full typed outcome, including any reasoning. It is internal state, not streamed; only `_agent_output_*` (now answer-only) is streamed and recorded in history.

## Proposed gate commands (exact; each requires explicit operator approval)

Unchanged from plan revision 2, steps 1–6, with this exact filter for step 2:

`CARGO_TARGET_DIR=target/uar-team-concurrency-repair cargo test --locked --no-default-features --features server-full --lib -- inline_reasoning router_node agent_node`

Step 5 needs the built binary's SHA-256, which comes from step 1. The restart command computes it inline with `shasum -a 256`.

## Gate run 1 — 2026-10-27 (operator-approved steps 1–6, serial, stop at first failure)

| Step | Command (as listed above) | Result |
|---|---|---|
| 1 | `cargo build … --bin universal-agent-runtime` | PASS, exit 0, 7m 26s. Binary SHA-256 `2ca5c057817b1c5cf07b01b9617c8fb748ae327199e8db1e8b299b3eef28e817`. Lib warnings are only in untouched files (`manager.rs`, `skills/service.rs`, `rag/ingestion_admission.rs`). |
| 2 | `cargo test … --lib -- inline_reasoning router_node agent_node` | **FAIL, exit 101: the lib test target does not compile.** No test ran. Two E0063 errors, both in files this change did not touch (clean in `git status`): `src/uar/runtime/presentation_history_tests.rs:58` (`RunStreamState` missing `caller_handoff_confirmation`; file last changed `dd896d5a`, 2026-10-22) and `src/server/tests.rs:499` (`AnthropicMessagesRequest` missing `max_tokens`, `parallel_tool_calls`, `tool_choice`; file last changed `ad557b0d`, 2026-10-20). No error or warning names `inline_reasoning`, `router_node` or `agent_node`. |
| 3–6 | not run | Stopped at the first failure, as instructed. |

The failure predates this change: the lib unit-test target was already stale against production struct fields. The new code compiled in the step 1 build (non-test cfg); its `#[cfg(test)]` modules are still uncompiled and unrun.

### Operator-approved repair and step 2 rerun — 2026-10-27

- Repair (operator approval: "add only the missing fields to the two stale test initializers … then rerun gate step 2 once"): `src/uar/runtime/presentation_history_tests.rs` +1 line `caller_handoff_confirmation: Arc::new(Mutex::new(())),` (SHA-256 `dd73bfdbc7ca2d73e5f85051653a38c9103c6fdc4c27bf67f8ae80523456c9e9`); `src/server/tests.rs` +3 lines `max_tokens: None, tool_choice: None, parallel_tool_calls: None,` (SHA-256 `7028f4f2d4110284204c0f35d8a0abb938ddb956d5609b6a4018edb938e89777`). Test-only; no production source changed. The step 1 binary is unaffected.
- Step 2 rerun (same exact command): **PASS**, exit 0. `test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 820 filtered out` — inline_reasoning 10, router_node 6, agent_node 4. The one remaining warning in the touched files is the pre-existing unused import `futures::FutureExt` at `src/server/tests.rs:3`.
- Steps 3–6: not run. This repair was an edit, which closes the gate under the operator lock; they await a renewed instruction.

### Gate steps 2 and 3 removed — 2026-10-27 (operator instruction)

Operator: "remove gate steps 2 and 3 from the plan and inventory (record why), then run steps 4, 5 and 6 exactly as listed." Why: both conflict with the `CLAUDE.md` rule that unit, module-local, mock-only and filtered function tests do not count as completion evidence. Step 2 is filtered unit tests; step 3 runs against a mocked LLM. The step 2 result above is kept as history and is **not** completion evidence. Step 3 never ran. Remaining gate: step 1 (passed), 4, 5, 6. Exit criterion in `plan.md` revised accordingly. The two test-only initializer repairs remain in source.

### Gate steps 4–6 — 2026-10-27 (operator-approved, serial)

| Step | Result |
|---|---|
| 4 `openspec validate fix-orchestrator-routing-reasoning` | PASS, exit 0. "Change … is valid"; one WARNING: requirement "Registered-agent selection tolerates inline model reasoning" text exceeds 500 characters. |
| 5 owned restart | PASS, exit 0. PID 69961 → 24733, binary `2ca5c057817b1c5cf07b01b9617c8fb748ae327199e8db1e8b299b3eef28e817` (replaced `f84baeaf…`), config `8726e43a…`, store preserved, new process ready. |
| 6 C34 | **FAIL**, exit 1. 5 passed / 7 skipped / 1 failed, 32.3s. Router selected the eligible specialist; the child produced reasoning only and the run ended `RUN_ERROR delegated_answer_missing: reasoning_only`. This is the D2(a) outcome the plan named. Receipt `.prometheus/verification/c34-live-roster-routing-attempt-004-fail.json`. |

Phase stays open per the exit criteria. No fix attempted.

## Renewed implementation-complete inventory — D2(b), 2026-10-27

Plan: [d2b/plan.md](d2b/plan.md) revision 2, approved with B1(a), B2 = 8,192, B3(a), B4(a), B5(a). The operator instructed: "Implement T8–T10 and stop at the renewed implementation-complete inventory before any gate command." **Verification gate: CLOSED.** No build, check, test, validate or C34 run was executed for this work.

| Item | Status | Files (SHA-256 at inventory time) |
|---|---|---|
| T8 delegated default and log line | Implemented | `src/uar/runtime/manager.rs` `a20755b5c7421505ea7c17c2eb6fb92a90c9cacf55e92549677952e088cb84ab`. Diff vs HEAD is +28 / −1, and these are the only changes in the file: constant `DELEGATED_CHILD_OUTPUT_TOKENS = 8_192`; helper `delegated_child_output_default(capacity)` = `min(8192, max(1000, capacity/2))`; `delegated_default = inherited.is_some() && !is_team_attempt`; `output_cap_source` label; a new `None if delegated_default` arm in the no-cap/no-contract match; `tracing::info!(run_id, output_token_cap, source, "delegated output allowance bound")` for captured child turns |
| T9 OpenSpec change `raise-delegated-child-output-allowance` | Written | `proposal.md` `4c44b700…`, `tasks.md` `05108ac1…`, `specs/multi-agent-orchestration/spec.md` `e09218df…`, `.openspec.yaml` |
| T10 decision and inventory | Done | `.prometheus/decisions.md` (appended), this section. Also recorded: `fix-orchestrator-routing-reasoning/tasks.md` verification items updated to reflect the removed steps and attempt 4 |

The earlier T1–T7 sources are unchanged since gate run 1, apart from the two operator-approved test-only initializer repairs recorded above. Build at gate run 1: `2ca5c057…`. That binary does **not** contain T8.

Known unverified points (source review only):
- Not compiled. Constructs checked by reading:
  - a guarded `None if delegated_default` arm inside the nested match;
  - the closure captures `delegated_default: bool` by reference, and `capacity` is local to the closure;
  - `output_token_cap: Option<u32>` is `Copy`, so it can be read before the closure;
  - `is_team_attempt` is in scope (`manager.rs` ~5211);
  - `inherited` is only borrowed in this function, as the artifact-critic confirmed;
  - `tracing` field names `source` and `output_token_cap` follow the existing `run_id = %run_id` style.
- Premises from live reads remain: no destination contract for `liter/MiniMax-M3`; maximum output 64,000; window 1,048,576.

## Proposed gate commands (exact; each requires explicit operator approval)

From d2b plan revision 2, serially from the repository root:

1. `CARGO_TARGET_DIR=target/uar-team-concurrency-repair cargo build --locked --no-default-features --features server-full --bin universal-agent-runtime`
2. `node /Users/gqadonis/Projects/prometheus/prometheus-skill-system/dist/plugins/claude/prometheus-skill-pack/skills/kbd-process-orchestrator/shared/openspec/cli.mjs run --project "$PWD" -- validate raise-delegated-child-output-allowance`
3. `node --input-type=module -e "import { restartOwnedUar } from './scripts/behaviors/support/restart-owned-uar.mjs'; const r = {}; await restartOwnedUar(r, process.argv[1]); console.log(JSON.stringify(r));" "$(shasum -a 256 target/uar-team-concurrency-repair/debug/universal-agent-runtime | cut -d' ' -f1)"`
4. `UAR_BASE_URL=http://127.0.0.1:60675 UAR_WORKSPACE_ID=uar-user-outcomes-007-supporting-workspace UAR_MODEL=liter/MiniMax-M3 node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/live-roster-routing.cucumber.mjs`, once (B4(a)), then review of the raw evidence.
5. B5(a): in the owned UAR log named by step 3's `log` field, read the `delegated output allowance bound` line whose `run_id` is the routed child's run (taken from the `liter-llm stream complete` line whose span names the specialist), and confirm `output_token_cap=8192` with `source=delegated_default`.

### D2(b) gate run — 2026-10-27 (operator-approved five commands, serial)

| Step | Result |
|---|---|
| 1 build | PASS, exit 0, 3m 49s. Binary `b4e68525240ce6f99028176503135ff831d022866bc2fb52ed5d737de8e688be`. Lib warnings unchanged from gate run 1 (`manager.rs` unused-fields warning moved 739 → 751 because of the inserted lines). |
| 2 `openspec validate raise-delegated-child-output-allowance` | PASS, exit 0, valid, no warnings |
| 3 owned restart | PASS. PID 24733 → 340; replaced `2ca5c057…`; config `8726e43a…`; store preserved; ready |
| 4 C34 | **PASS**, exit 0, 13/13 steps, 1m 4.930s. Raw evidence reviewed: genuine answers, no streamed reasoning, roster-bound routing. Receipt `.prometheus/verification/c34-live-roster-routing-macos-pass.json` |
| 5 allowance confirmation | CONFIRMED. Log line 1409: run `7b51ddb5-…` (the specialist, stream completed at line 1414, 144 events) `output_token_cap=8192 source=delegated_default` |

The exit criteria in d2b plan revision 2 are met under B4(a): steps 1–3 passed, C34 passed once with genuine non-reasoning answers, and step 5 confirmed 8,192. Not shown: that the 1,000 cap would have failed on this run.
