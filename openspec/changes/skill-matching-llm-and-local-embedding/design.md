## Context

See `proposal.md` for why. Current seams, read before this design was written:

- `SkillService::match_in_registry` (`src/uar/runtime/skills/service.rs:672-737`) is a static function over a registry and a config. It has no model handle and no embedding backend of its own. `SkillMatchingSnapshot` (`:130-157`) copies the registry and config per run and calls it.
- Skill matching for a root run happens at `src/uar/runtime/manager.rs:2866-2890`. The run's model binding is captured later, at `manager.rs:3231-3386`, because a matched skill's `preferred_model` can change the route (`manager.rs:3262-3336`). The capture comment is explicit: "Capture once before any summarization or model execution" (`manager.rs:3368-3370`). Child runs already hold their bindings before matching (`inherited.models`, `manager.rs:3231-3232`).
- `RunModelBindings::primary()` returns the captured driver wrapped in the run's `ModelCallBudget` (`bindings.rs:206-209`). `for_policy` shows the rebinding pattern (`bindings.rs:228-316`): `with_bound_model` on a driver whose grant covers the target provider.
- The legacy LLM classifier (`src/uar/runtime/matching/intent/llm.rs`) has a usable prompt (`:51-75`), fence-stripping parser (`:118-138`) and id-to-skill mapping (`:141-154`). Its call site builds a new `Orchestrator` from an `LlmConfig` (`:91-94`) — unusable here.
- The legacy local-embedding classifier (`intent/local_embedding.rs:14-80`) has the cache-plus-cosine shape but embeds through the shared `VectorMatcher`, which may be a remote backend. Only the shape is reused.
- `FastEmbedBackend` (`src/uar/rag/embeddings/fastembed.rs:21-70`) loads BGE-small assets from disk (`UAR_MODELS_DIR`, the configured `models_dir`, `/app/models`, or `src/uar/runtime/matching/models`); it never downloads at runtime.
- A run-level test harness with a mock driver exists: `matching_manager` in `tests/skill_activation_runtime.rs:133-176`, and `keyword_threshold_and_activation_mode_control_body_loading` (`:623-673`) shows how to assert body loading from `driver.requests()`.

## Goals / Non-Goals

**Goals:**
- `llm` and `local_embedding` produce their own scores, through the same `SkillMatchResult::resolve` threshold, margin and `top_k` rules as the other algorithms.
- The classification call is a model call of the run: same credentials, same budget, same cancellation.
- The used method, including fallbacks, is visible per activation.

**Non-Goals:**
- Improving `Embedding` or `Hybrid` (both still degrade to keyword with persistence off, `registry.rs:297-327`).
- Per-algorithm default thresholds. One threshold applies to whichever algorithm is selected; see Risks.
- A timeout on the classification call. No hang has been observed; the run's budget deadline already applies (`cost_budget.rs:804-821`).
- Removing the legacy intent classifier.

## Decisions

### D1 — Matching takes an optional run model handle
`match_in_registry` gains a matching context: an optional run-scoped driver handle and the local embedding cache. `SkillMatchingSnapshot::match_skills_scoped` takes the handle from the manager. `SkillService::match_skills` (API and eval path) takes none, so `llm` without a handle falls back to keyword with reason `no_run_binding`. The eval provider (`src/uar/eval/targeted.rs:34-116`) gets a constructor that selects the algorithm and supplies a driver, so the falsifier runs through the same function the runtime uses.
*Alternative:* store a driver in `SkillService`. Rejected: it would be a process-global client outside the run's credential grant and budget — the defect the spike found in `llm.rs:91-94`.

### D2 — Under `llm`, capture the root binding before matching (operator decision 2026-09-23: approved)
For root runs with algorithm `llm`, the manager captures `RunModelBindings` from the policy route before skill matching and passes `primary()` (or a `model_name` rebind of it) to the matcher. After matching, a matched skill's `preferred_model` is applied by rebinding the captured primary when the grant covers that provider; otherwise it is ignored and recorded (`skill_preferred_model_ignored`). For every other algorithm, and for child runs, ordering is unchanged. This keeps "capture once" and "run on the run's bound driver".
*Alternative A:* capture a separate matching-only binding before matching, then capture the execution binding as today. Preserves cross-provider skill model overrides, but makes two captures per turn, resolves credentials twice, and contradicts the "capture once" invariant. *Alternative B:* match with the host's primary driver outside the run. Rejected: no run budget or grant. D2 changes existing behaviour for one edge (cross-provider `preferred_model` under `llm`).
*Operator decision 2026-09-23: approved — D2 as written.* Under `llm` the root binding is captured before skill matching; a matched skill's cross-provider `preferred_model` is ignored and recorded as `skill_preferred_model_ignored`. Alternative A is not built.

### D3 — `model_name` is honoured by rebinding, never by constructing
`model_name` is a model id qualified as `provider/model`. It is bound with `with_bound_model` on the captured client whose grant targets that provider. If no captured grant covers it, or the driver refuses rebinding (the trait default bails, `src/llm/mod.rs:356-358`; the test `MockLlmDriver` does not implement it), matching falls back to keyword with reason `model_unbindable` and sends no request.

### D4 — The reply is untrusted input
User text and skill descriptions go into the prompt, so the reply can be steered (prompt-injection surface at a real boundary). The parser keeps only ids in the eligible set it was given, discards non-finite scores, clamps to 0.0-1.0, and never widens eligibility; the manager's later intersection with `effective_policy.skills` (`manager.rs:2940-2954`) stays as a second barrier. Replies are not logged verbatim.

### D5 — Local embedding owns its backend and cache
`SkillService` gains an optional local `EmbeddingBackend` (the host passes a `FastEmbedBackend` under `local-models`) and a cache `Arc<RwLock<HashMap<skill_id, (text_hash, Vec<f32>)>>>`. The embedded text is title, description and keywords — the same fields the LLM prompt lists. The cache is rebuilt after `initialize`, `refresh`, `reconcile_config_skills` and `reconcile_standard_agent_skills`; create, update and toggle invalidate the affected entry, and a missing or hash-mismatched entry is embedded on first match, so a stale vector is never scored. The snapshot carries an `Arc` of the cache, not a copy. Cosine and ranking reuse the `local_embedding.rs` shape.
*Alternative:* reuse `VectorMatcher` with a second, local instance. Rejected: `find_candidates` requires persistence (`registry.rs:297`) and its cosine search lives in the database.

### D6 — Fallback reasons are method suffixes
`selection_method` stays a string. Values: `skill_service.llm`, `skill_service.llm.keyword_fallback`, `skill_service.local_embedding`, `skill_service.local_embedding.keyword_fallback`. A `tracing::warn!` with a reason code (`parse_error`, `call_failed`, `model_unbindable`, `no_run_binding`, `local_model_unavailable`) accompanies each fallback. No new event type.

### D7 — Legacy marking is metadata only
The `intent_classifier` schema (`src/uar/settings/manager.rs:1592-1610`) gets `"x-uar-legacy": true` and a description naming `/api/uar/skills/config` as the effective control; `ClassifierConfig` and `RunManager::with_classifier_config` get doc comments stating the branch is test-only in production hosts. Reads and writes keep working.

## Risks / Trade-offs

- [Mock-driver falsifier proves plumbing, not accuracy] → State it in the evidence. Optionally record one live-model run of the paraphrase suite as information, not as a gate.
- [One threshold for three score scales] → LLM scores, BGE cosine and keyword scores differ in scale. The default 0.5 may over-accept under `local_embedding`. Document it beside the setting; the-boss shows the threshold next to the algorithm.
- [Extra latency and cost per turn under `llm`] → Charged to the run budget and visible in usage; documented in the setting's description.
- [D2 changes cross-provider skill model overrides under `llm`] → Accepted by the operator on 2026-09-23 (task 0.3). The reason is recorded when it happens; contract test 1.14 holds the behaviour.
- [Local model assets missing in a packaged sidecar] → Keyword fallback with reason; the release change (`sidecar-release-pipeline`) packages the model assets.
- [Cache memory] → 384 floats per skill; 2,000 skills is about 3 MB.

## Migration Plan

No data migration. Existing `keyword`, `embedding` and `hybrid` users see no change. Anyone who had selected `llm` or `local_embedding` starts getting those algorithms on upgrade. Rollback is a code revert; the stored config values stay valid.

## Open Questions

- Should `local_embedding` scores be rescaled so one threshold means roughly the same thing across algorithms? Deferrable: it changes defaults, not the specs or tasks.
