# Shared skill matching source trace after Gate014

Phase: production-readiness-476-capability-dispatch / Execute.
Verification: CLOSED_OPERATOR_LOCK. This is bounded source diagnosis, not an independent review or acceptance gate. This writer changed only this report. No product source, fixture, service, config, credential, or other-chat state was changed by this task.

## Observed failure and historical limits

The retained Gate014 receipt and diagnostic transcript report 29 passed, 2 failed, and 2 historically ignored on the inventoried uncommitted repair tree based on b3e5eda9729daf2a160d75aaea04f43486dd8f40. Both failing fixtures emitted skill_matching_unavailable before RunStart. The graph fixture observed executions=0 and provider_calls=0. Its original capture received no state. These are historical observations from the authorized diagnostic command, not a new run by this writer or acceptance for the next fixture repair.

The uncomfortable correction is that earlier native capability/provisional-completion source findings were not exercised by these failed runs. The opaque graph timeout did not establish slow graph/checkpoint execution. Neither default TF-IDF nor the unavailable embedding fixture backend explains the selected-owner failure below.

## Deterministic source contradiction

1. RunManager::new calls with_classifier_config; supplying Some(persistence) constructs a SettingsManager using that same persistence (src/uar/runtime/manager.rs:1386). The constructor still initializes skill_service=None (:1440). Both failing fixture constructors supplied durable persistence and an isolated SkillRegistry without with_skill_service (tests/context_history_integrity.rs, pending manager around :787 and graph manager around :2261 in the inspected pre-repair source).
2. The run captures skills through RunSkillBindings::capture at manager.rs:5695. With service=None, src/uar/runtime/turn/bindings.rs:177-194 clones the legacy registry and sets matching=None. Capturing an empty registry succeeds; emptiness does not manufacture a SkillMatchingSnapshot.
3. manager.rs:5820 chooses the engine from SettingsManager whenever present. SettingsManager::skill_matching_engine_for_admission (src/uar/settings/manager.rs:523-527) reads committed skills.matching.engine and uses unwrap_or_default for absence. get_typed_for_admission (:503-518) reads persistence directly and propagates storage/deserialization failure; the default is not a cached classifier preference.
4. SkillMatchingEngine::default is Service (src/uar/runtime/skills/service.rs:52-56). With no committed override in these fresh isolated fixture stores, Service is selected. ClassifierConfig's default Tfidf creates the startup classifier but does not select IntentClassifier as matching owner.
5. manager.rs:5832 requires service_config before checking whether Service's algorithm needs LLM. Since skill_bindings.matching=None, it returns skill_service_resource_required. This precedes auxiliary capture, provider opening, vector matching, final tool admission, graph attachment, and RunStart.
6. The common matching-result error branch at manager.rs:5990-6024 records Error, emits skill_matching_unavailable, emits RunDone, removes cancellation, and returns. Pre-admission projection explains the actual diagnostic message "Run admission failed before protected content publication" while preserving the public code. This is consistent with both retained failures.

## Actual Service/Keyword fixture setup

Use the real SkillService with the same durable fixture persistence. SkillService::new (src/uar/runtime/skills/service.rs:268-279) creates its registry and defaults matching_config to Service/Keyword (algorithm default :43-46; config default :136-146). Explicit Service/Keyword configuration makes this fixture's dependency ownership intentional. Pass Arc::clone(service.registry()) to RunManager and install the same Arc<SkillService> through with_skill_service (manager.rs:2055-2057). The registry getter is service.rs:1104.

This gives RunSkillBindings::capture a real matching snapshot. SkillService::matching_snapshot (service/selection.rs:189-205) reconciles durable selection, clones the service registry, captures config and scoped bindings, and pins any already configured local matcher. RunSkillBindings then uses that same captured snapshot registry for matching, catalog/activation, and candidate relookup. The fixture must not use a separately constructed registry that silently diverges from the service.

The checked snapshot path (service.rs:232-256) clones the captured registry and invokes match_in_registry_checked. Keyword at :915-917 calls keyword_match (:975-989) over eligible skills. It does not dispatch an LLM or call VectorMatcher. Empty fixture skills therefore produce an ordinary empty candidate set through the selected matching operation, not an early skip or synthetic vector success. registry.rs:135-159 still consults durable list_skills during selection reconciliation and preserves storage failure as a refusal; there is no bypass of persistence.

The source telemetry at manager.rs:6050 onward can prove selected requested_method/effective_method="skill_service.keyword" and null requested/effective model in a subsequent authorized run, in addition to the existing protected pending/resume assertions and exact graph/provider counts. Those are proposed fixture assertions, not observations made here.

## Recommendation and remaining uncertainty

The actual service/shared-registry/Keyword setup repairs the demonstrated missing dependency. Do not add a production fallback to IntentClassifier, skip matching for empty registries, relax admission, supply fabricated embedding success, or change the unavailable backend solely to evade this refusal. The default Service requirement is explicit and the fixture should supply it.

Source reasoning does not establish that the pending stream, durable persistence, graph capture, checkpoint denials, or all later boundaries pass after setup repair. Those remain UNVERIFIED pending a newly inventoried source tree and the operator's exact named gate approval. Earlier Gate014 compile success applies only to its historical source and does not compile the next fixture edits.

## Commands and ownership

This task used only cat/sed/rg source and retained-artifact reads with the production-readiness-476 worktree explicitly supplied, then wrote this report. No tests, Cargo, check, fmt, lint, typecheck, compiler/import probe, build, review, API/container call, git diff --check, preflight, commit, hook, or delegation ran. Exact anchors and conclusions were sent directly to the lead and pending_checkpoint_fixture_repair teammate. Fixture implementation belongs to that teammate; authoritative ledger/inventory and renewed gate proposal belong to the lead.
