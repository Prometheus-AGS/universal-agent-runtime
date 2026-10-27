## 1. Implementation (not authorized until operator approval; see plan revision 2)

- [x] 1.0 D0 (if approved): observe the classifier reply shape, 3 calls, content discarded
- [x] 1.1 T1 Inline-reasoning splitter `src/llm/inline_reasoning.rs` with unit tests (authored, not run)
- [x] 1.2 T2 RouterNode registered-agent mode: answer-part exact match, D4 normalization if approved, `graph_routing_classifier_incomplete`, content-free diagnostics; unit test module
- [x] 1.3 T4 AgentNode: contribution is the answer part only; `delegated_answer_missing: reasoning_only`; unit test module
- [x] 1.4 T5 Integration tests in `tests/test_chat_completion.rs`
- [x] 1.5 T6 C34 oracle: separate the "selected then failed" case; exact refusal message
- [x] 1.6 T7 Gotcha entry; implementation-complete inventory

## 2. Verification (operator-approved exact commands only; plan revision 2 lists them)

- [x] 2.1 Build the owned binary (PASS, `2ca5c057…`)
- [x] ~~2.2 `--lib` unit tests: inline_reasoning, router_node, agent_node~~ removed by the operator on 2026-10-27: unit tests are not completion evidence. It had run (20/20 after a test-only repair); not counted.
- [x] ~~2.3 `--test test_chat_completion -- orchestrator`~~ removed by the operator on 2026-10-27: mock-LLM tests are not completion evidence. Not run.
- [x] 2.4 `openspec validate fix-orchestrator-routing-reasoning` (PASS, 1 length warning)
- [x] 2.5 Owned restart onto the new binary SHA (PASS, PID 24733)
- [x] 2.6 Rerun only C34 once; review the raw evidence (attempt 4 FAIL: `delegated_answer_missing: reasoning_only`; continued under `raise-delegated-child-output-allowance`)
