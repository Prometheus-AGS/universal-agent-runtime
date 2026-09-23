## 0. Stop rule, recording and operator gates (before any code)

The-boss decision D3 caps effort. Effort is focused working days recorded by the implementer in the KBD task log of child phase `the-boss-universal-agent-runtime` (prometheus-skills-mini repo, `.kbd-orchestrator/phases/the-boss-shipping-and-settings/children/the-boss-universal-agent-runtime/`). No task-log file exists there yet; task 0.1 creates `effort-log.md` (append-only). This path is a proposal — the operator may name another.

| Item | Estimate (upper bound) | Stop at (1.5×) |
|---|---|---|
| `Llm` matching (groups 2, 3, 6) | 1 day | 1.5 days |
| `LocalEmbedding` matching (groups 4, 5, 6) | 2 days | 3 days |
| Shared cap with `agentic-chunking` (3 days, stop 4.5) | — | 8 days total across all three |

An item whose recorded effort reaches its stop value without its gate passing stops: it is hidden from the-boss's settings and recorded as follow-up work. Reaching the 8-day cap stops every item still open. The phase's adversarial review checks the log at each threshold.

- [ ] 0.1 Create the task-log file and write a dated start entry for this change before any other task (`YYYY-MM-DD HH:MM start skill-matching-llm-and-local-embedding/<item>`); verify the entry exists and predates the first code commit.
- [ ] 0.2 Record a dated stop entry at the end of every working session, per item (`YYYY-MM-DD HH:MM stop <item> <hours>h gate=<pass|fail|open>`); verify running totals against the table above after each entry and stop the item at its threshold.
- [ ] 0.3 Operator gate: confirm design D2 (capture the root binding before matching under `llm`; cross-provider skill `preferred_model` is then ignored and recorded) or choose design Alternative A; record the answer in `.prometheus/decisions.md` and update `design.md` if Alternative A is chosen.
  Operator decision 2026-09-23: approved — design D2. Under `llm` the run's model binding is captured before skill matching; a matched skill's cross-provider `preferred_model` is ignored and recorded. `design.md` D2 carries the decision. Still open: the `.prometheus/decisions.md` entry, which the documents-only edit of this change did not write.

## 1. Contract tests — written first, must fail before and pass after

Each test names the falsifier or gate it serves. Record the failing run (before) and the passing run (after) in the evidence folder. Nothing here is compiled in this planning session.

- [ ] 1.1 `llm_matching_picks_paraphrased_skills_keyword_misses` (in `src/uar/eval/targeted.rs` tests; **D3 falsifier 1**). Load the six cases of `evals/skill-activation.yaml` and at least five cases of a new `evals/skill-activation-paraphrase.yaml` whose inputs share no keyword with their expected skill. Assert: keyword picks the expected skill in ≤ 1 paraphrase case; `llm` with a deterministic scripted mock driver picks it in ≥ 4 of 5; `llm` still answers all six original cases, including `no-match-query` → `none`. Fails before: `llm` runs keyword.
- [ ] 1.2 `llm_matching_malformed_reply_falls_back_to_keyword` (service tests; **D3 falsifier 1, fallback clause**). Mock replies `not json`. Assert the result equals the keyword result and the reported method is `skill_service.llm.keyword_fallback`. The equality half passes before; the method assertion fails before (`skill_service.llm`).
- [ ] 1.3 `llm_matching_discards_ineligible_ids_and_clamps_scores` (service tests; **A-3 boundary**). Reply names one eligible paraphrase target at 0.9, one disabled skill, one unknown id, and one score of 7.0. Assert only the eligible target is matched and no score exceeds 1.0. Fails before.
- [ ] 1.4 `llm_matching_uses_run_bound_driver` (`tests/skill_activation_runtime.rs`). With the `matching_manager` harness and a request-recording driver, assert the first recorded request is the classification prompt on the run's injected driver and that no other driver instance received a request. Fails before: no classification request exists.
- [ ] 1.5 `llm_matching_honours_model_name_by_rebinding` (`tests/skill_activation_runtime.rs`). Test-local driver that implements `with_bound_model` and records the bound model. Set `model_name` to a model under the captured provider. Assert the classification request went through the rebound client with that model. Fails before.
- [ ] 1.6 `llm_matching_unbindable_model_name_falls_back_to_keyword` (`tests/skill_activation_runtime.rs`). `MockLlmDriver` (no rebinding) with `model_name` set. Assert zero classification requests, keyword result, method `skill_service.llm.keyword_fallback`. Fails before on the method assertion.
- [ ] 1.7 `local_embedding_never_calls_shared_backend` (service tests; **D3 falsifier 2, first clause**). Persistence on (temporary SurrealKV, as in `standard_skill_reconciliation_does_not_invoke_embeddings`, `service.rs:1062-1119`), shared `VectorMatcher` over `RecordingEmbeddingBackend` (`service.rs:953-977`), deterministic local backend double. Match three inputs with `local_embedding`. Assert the recording backend's batch list is empty and the local double was called. Fails before: `find_candidates` embeds through the shared matcher.
- [ ] 1.8 `local_embedding_matches_with_persistence_off` (service tests; **D3 falsifier 2, second clause**). `SkillService::new(None, None)` plus the local double. Assert a paraphrased input selects the expected skill. Fails before: keyword fallback misses the paraphrase.
- [ ] 1.9 `local_embedding_cache_follows_reconciliation` (service tests). Reconcile a skill whose description changes; assert the next match uses the new description's vector and the old text is not embedded again. Fails before.
- [ ] 1.10 `local_embedding_unavailable_falls_back_to_keyword_not_remote` (service tests). No local backend. Assert keyword result, method `skill_service.local_embedding.keyword_fallback`, zero shared-backend requests. Fails before on the method assertion.
- [ ] 1.11 `local_embedding_real_fastembed_paraphrase` (`#[cfg(feature = "local-models")]`, service tests). Real `FastEmbedBackend` from the repository model assets, persistence off. Assert the paraphrase suite's expected skill ranks first for at least 4 of 5 cases. Fails before.
- [ ] 1.12 `changing_matching_algorithm_changes_activated_skill` (`tests/skill_activation_runtime.rs`; **D3 behaviour gate, review finding 1**). Two skills: A triggered by keyword `deploy`, B described as shipping a release. Legacy overlay mode. Run 1 (keyword) on "please deploy the release" loads A's body. Set the algorithm to `llm` through the `/api/uar/skills/config` router (not by calling the service directly), mock ranks B first; run 2 loads B's body, not A's, and its `agui.skill.activated` event reports `skill_service.llm`. Repeat with `local_embedding` and the local double: run 3 loads B's body and reports `skill_service.local_embedding`. Fails before: runs 2 and 3 load A.
- [ ] 1.13 `intent_classifier_namespace_is_marked_legacy` (settings manager tests). Assert the `intent_classifier` schema has `x-uar-legacy: true` and names the skill-matching configuration. Fails before.
- [ ] 1.14 `llm_matching_ignores_cross_provider_preferred_model` (`tests/skill_activation_runtime.rs`; **operator decision 2026-09-23, design D2**). Algorithm `llm`; the mock ranks a skill whose `preferred_model` names a provider outside the run's captured grant. Assert the run executes on the captured primary, no driver for the preferred provider is built or called, and `skill_preferred_model_ignored` is recorded. Second case: a `preferred_model` under the captured provider is applied by rebinding. Fails before: no classification call exists and the preferred model re-routes the run.

## 2. LLM matcher

- [ ] 2.1 Move the prompt builder, fence-stripping parser and id mapping out of `src/uar/runtime/matching/intent/llm.rs` into a matcher module beside `service.rs`; keep `LlmClassifier` compiling by calling the moved functions. Verify by static inspection that `LlmClassifier`'s behaviour is unchanged.
- [ ] 2.2 Add the matching context (optional run driver handle, local cache handle) to `match_in_registry` and `SkillMatchingSnapshot::match_skills_scoped`; implement the `Llm` arm with eligibility intersection and score clamping (design D1, D4). Verify with 1.2 and 1.3 once group 3 is wired.
- [ ] 2.3 Implement `model_name` rebinding and the `model_unbindable` / `no_run_binding` fallbacks (design D3). Verify with 1.5 and 1.6 once group 3 is wired.

## 3. Run wiring for `llm`

- [ ] 3.1 In `RunManager` root-run assembly, capture the model binding before matching when the algorithm is `llm` (design D2 as confirmed in 0.3); pass `primary()` to the snapshot; for child runs pass the inherited binding's primary. Verify by inspection that other algorithms keep today's order.
- [ ] 3.2 Apply matched-skill `preferred_model` by rebinding within the captured grant under `llm`; record `skill_preferred_model_ignored` otherwise. Verify with 1.14.
- [ ] 3.3 Report the used method in `skill_selection_method` and the shadow-candidate record (design D6). Verify with 1.2, 1.6, 1.12.

## 4. Local-embedding matcher

- [ ] 4.1 Add the optional local `EmbeddingBackend` and the shared cache to `SkillService`; carry the cache `Arc` in the snapshot (design D5). Verify by inspection that the shared `VectorMatcher` is not reachable from the `LocalEmbedding` arm.
- [ ] 4.2 Rebuild the cache after `initialize`, `refresh`, `reconcile_config_skills`, `reconcile_standard_agent_skills`; invalidate on create, update and toggle; embed missing or stale entries at match time. Verify with 1.9.
- [ ] 4.3 Implement the `LocalEmbedding` arm and its keyword fallback. Verify with 1.7, 1.8, 1.10.
- [ ] 4.4 Attach a `FastEmbedBackend` as the local backend in `src/server.rs` and `src/embedded.rs` under `local-models`. Verify by inspection that no host passes the shared backend as the local one.

## 5. Eval provider

- [ ] 5.1 Add an algorithm-selecting constructor to `SkillActivationProvider` that accepts a driver and a local backend; keep `SkillActivationProvider::new()` unchanged for the existing suite. Verify with 1.1.
- [ ] 5.2 Add `evals/skill-activation-paraphrase.yaml` (≥ 5 cases, no keyword overlap with the expected skill's triggers) and review each case against `fixture_skills()` (`targeted.rs:67-97`). Verify 1.1's keyword ≤ 1 assertion holds.

## 6. Legacy marking

- [ ] 6.1 Add the legacy marker and description to the `intent_classifier` schema and doc comments to `ClassifierConfig` and `RunManager::with_classifier_config` (design D7). Verify with 1.13.

## 7. Completion gate (once, after groups 2-6 are complete)

- [ ] 7.1 Run the smallest integration gate that exercises the production path: `cargo test --features local-models --test skill_activation_runtime`, then `cargo test --features local-models --lib -- skills::service eval::targeted settings::manager`, serialized in one target directory. Verify all tests in group 1 pass; save logs to the evidence folder.
- [ ] 7.2 Record the gate result in the task log (0.2) and in `openspec/changes/skill-matching-llm-and-local-embedding/verification.md`, naming which claims are mock-driver claims.
- [ ] 7.3 If an item stopped under the stop rule, record it as follow-up work and tell the-boss integration to hide that option.
- [ ] 7.4 `openspec validate skill-matching-llm-and-local-embedding --strict` passes.
