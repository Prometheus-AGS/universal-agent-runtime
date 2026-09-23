## Why

the-boss will expose UAR's skill-matching algorithm as an app-wide setting, and every exposed option must work (the-boss decision D3, revision 3). Two of the five `SkillMatchingAlgorithm` values do not. `Llm` logs "not yet implemented" and runs keyword matching (`src/uar/runtime/skills/service.rs:697-701`). `LocalEmbedding` is the same code path as `Embedding` (`service.rs:703`): it calls the shared remote-capable `VectorMatcher` through `registry.find_candidates`, and with persistence off that call silently degrades to keyword (`src/uar/runtime/skills/registry.rs:297-327`). `SkillMatchingConfig::model_name` (`service.rs:57-58`) is never read. A user who picks either option today gets keyword matching and no signal that it happened.

A second, older matcher is dead on every production path. `RunSkillBindings::capture` returns `matching: Some(..)` whenever a `SkillService` exists (`src/uar/runtime/turn/bindings.rs:88-106`), and both hosts attach one (`src/server.rs:1164`, `src/embedded.rs:417`), so the legacy intent-classifier branch (`src/uar/runtime/manager.rs:2891-2939`) never runs outside tests. `RunManager::new` passes `ClassifierConfig::default()` (`manager.rs:703-713`) and ignores `config.intent_classifier` (`src/config.rs:251`). The settings UI still offers six classifier backends for it (`src/uar/settings/manager.rs:1592-1640`, `src/uar/api/settings.rs:132-139`). Changing them does nothing.

## What Changes

- Implement `SkillMatchingAlgorithm::Llm` in `SkillService`. Reuse the legacy prompt builder and JSON parser (`src/uar/runtime/matching/intent/llm.rs:51-75`, `:118-154`). Run the classification call on the run's captured model binding, not on a freshly built `Orchestrator` (`llm.rs:91-94` bypasses credential binding, `bindings.rs:108-127`). Honour `model_name` by rebinding the captured driver (`LlmDriver::with_bound_model`, `src/llm/mod.rs:356`). A malformed or failed reply falls back to keyword matching, and the fallback is recorded.
- Implement a distinct on-device `SkillMatchingAlgorithm::LocalEmbedding`: a fastembed backend owned by `SkillService` (feature `local-models`), an in-memory skill-vector cache rebuilt on reconciliation, no call to the shared embedding backend, and no dependency on persistence. When the local model is unavailable it falls back to keyword, never to the remote backend.
- Record the method that actually produced each match (`skill_service.llm`, `skill_service.llm.keyword_fallback`, and the local-embedding equivalents) in the existing `selection_method` field of `agui.skill.activated` (`src/uar/api/sse.rs:484-500`) and the shadow-candidate record (`manager.rs:3170-3176`).
- Mark the legacy intent classifier and its `intent_classifier` settings namespace as legacy: schema marker and description, doc comments. It is not removed from UAR and not implemented further. **BREAKING** for nobody at runtime (it already has no effect); the settings schema gains a legacy marker that UIs can read.
- **Behaviour change under `Llm` only (operator decision 2026-09-23: approved, task 0.3):** the root run's model binding is captured before skill matching so the classifier can use it. A matched skill's `preferred_model` can then only rebind within the captured provider; a cross-provider preferred model is ignored and recorded. Under every other algorithm the ordering is unchanged. A skill author who relied on a cross-provider `preferred_model` loses that override under `Llm` and sees a recorded reason, not an error.

Not in scope: exposing any option in the-boss (the-boss side), the `Embedding`/`Hybrid` keyword degradation with persistence off, and a live-model accuracy claim. The falsifier tests use a deterministic mock driver, so they prove the plumbing, not the model's judgement.

## Capabilities

### New Capabilities
- `skill-matching-algorithms`: what each configurable skill-matching algorithm does, which model or embedding backend it may use, how it fails, how the used method is reported, and the legacy status of the intent-classifier namespace.

### Modified Capabilities
(none — `skill-activation-runtime` keeps its threshold, margin and activation-mode requirements; this change only supplies the scores.)

## Impact

- **Code:** `src/uar/runtime/skills/service.rs` (algorithm dispatch, snapshot, local cache), `src/uar/runtime/turn/bindings.rs` (snapshot carries the cache), `src/uar/runtime/manager.rs` (Llm-only capture ordering, driver hand-off to matching), a new matcher module beside `service.rs` that owns the reused prompt/parser, `src/uar/eval/targeted.rs` (algorithm-selectable provider), `src/server.rs` / `src/embedded.rs` (attach the local backend), `src/uar/settings/manager.rs` (legacy marker).
- **APIs:** `GET/PUT /api/uar/skills/config` and `/api/skills/config` (`src/uar/api/skills.rs:43-44`, `:639-648`) keep their shape; `llm` and `local_embedding` now change behaviour. `selection_method` gains fallback values.
- **Provider compatibility:** the classification call uses whatever driver the run captured, including its budget wrapper; `model_name` works only for drivers that implement `with_bound_model` (liter and Anthropic drivers do: `src/llm/liter_driver.rs:533`, `src/llm/anthropic_driver.rs:295`). Other drivers fall back to keyword with a recorded reason.
- **Cost and latency:** `Llm` adds one model call per matched turn, charged to the run's budget, before the first token.
- **Runtime UX / realtime state:** users see which method matched in the activation event; no new event types.
- **Dependencies:** none new; fastembed is already optional under `local-models` (`Cargo.toml:184`, `:380-382`).
- **KBD workflow state:** YES. This change is a D3 build item for child phase `the-boss-universal-agent-runtime` (skills-mini repo). Its stop rule needs dated start/stop entries in that child's task log before work begins (tasks 0.1-0.2).
