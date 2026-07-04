## 1. Stage 01 validation

- [x] 1.1 `model_requirements` numeric bounds (`min_context == 0` warn,
      `max_cost_per_1m_input < 0.0` error)
- [x] 1.2 `prompt_dialect.dialect` must be a recognized name (error)
- [x] 1.3 `rag_configuration.enabled` with empty `knowledge_base_ids`
      (warn)
- [x] 1.4 `context_strategy` explicit-zero numeric fields (error)
- [x] 1.5 `api_harness.protocols`/`stream_mode` unrecognized values (warn)

## 2. Stage 08 emit

- [x] 2.1 `uses_any_v2_section()` helper checking all 5 sections for
      non-default values
- [x] 2.2 `schema` string bumped to `uar-agent-descriptor/v2` when true,
      unchanged (`/v1`) otherwise

## 3. Verify

- [x] 3.1 New full-pipeline round-trip test (parser → 8 stages → emit →
      sign) proving schema bump + field survival
- [x] 3.2 Existing v1-only end-to-end test still asserts `/v1` (no
      regression)
- [x] 3.3 `cargo test --lib compiler::` 31/31 green
- [x] 3.4 Full suite `cargo test --lib` 334/334 green

## 4. Follow-ups (not this change)

- [ ] CH-14: conformance harness — compare declared vs. actual runtime
      behavior for `model_requirements`/`prompt_dialect`/`context_strategy`.
- [ ] CH-15: agent template library — first real consumers of the v2
      sections in shipped `.agent.md` content.
