## Context

EH5's `cli.rs::select_scorers` is a hardcoded heuristic; `EvalSuite { name, cases }` has no scorer field. The `Scorer` trait is async and object-safe; rule scorers are unit structs (`ExactMatch`, `Contains`, `JsonValid`, `NonEmpty`, `PatternMatch{pattern,mode}`, `Sycophancy`). `PatternMode` is `{Contains,StartsWith,EndsWith}` (currently only `Debug,Clone,Copy`).

## Goals / Non-Goals
**Goals:** suite-declared scorers; a single factory; preserve EH5 behavior when undeclared.
**Non-Goals:** the `llm_judge` scorer (EHH2); per-case overrides.

## Decisions
- **D1 — `ScorerSpec` shape:** serde-tagged enum, `#[serde(tag = "type", rename_all = "snake_case")]`, in a new `src/uar/eval/scorer_spec.rs`. Variants: `ExactMatch`, `Contains`, `JsonValid`, `NonEmpty`, `PatternMatch { pattern: String, mode: PatternMode }`, `Sycophancy`. (`LlmJudge` added by EHH2.) Tagged (not untagged) → clear YAML/JSON authoring + good error messages.
- **D2 — `PatternMode` serde:** add `Serialize, Deserialize, PartialEq` + `#[serde(rename_all = "snake_case")]` so `mode: starts_with` parses.
- **D3 — `EvalSuite.scorers`:** `#[serde(default)] pub scorers: Vec<ScorerSpec>`. NOTE: `serde(default)` only affects *deserialization*; struct literals must still set the field — update the two test literals (`runner.rs`, and the moved `cli.rs` test).
- **D4 — factory:** `build_scorers(suite: &EvalSuite) -> Vec<Arc<dyn Scorer>>`. Empty `suite.scorers` → `default_scorers(suite)` (the EH5 heuristic, moved here verbatim). Non-empty → map each spec to its scorer. No provider param this change (rule scorers need none); EHH2 will extend the signature for `llm_judge`.
- **D5 — CLI:** `run_suite` calls `build_scorers(&suite_obj)`; delete `cli.rs::select_scorers` and move its unit test into `scorer_spec.rs` (testing `default_scorers`/`build_scorers`).

## Risks / Trade-offs
- **[struct-literal breakage]** adding a field breaks `EvalSuite { .. }` literals → Mitigation: only two, both in tests; set `scorers: Vec::new()`. Verified by `cargo test`.
- **[behavior drift]** the heuristic must remain identical when undeclared → Mitigation: move it verbatim; keep the existing test (now over `default_scorers`).
- **[signature churn for EHH2]** `build_scorers` gains a `provider` param in EHH2 → accepted; localized to the factory + its one caller.

## Migration Plan
1. `mod.rs`: `PatternMode` derives; `EvalSuite.scorers` (serde default); `mod scorer_spec; pub use scorer_spec::{ScorerSpec, build_scorers};`.
2. `scorer_spec.rs`: `ScorerSpec`, `build_scorers`, `default_scorers` (moved heuristic) + tests.
3. `cli.rs`: call `build_scorers`; remove `select_scorers` + its test + now-unused imports.
4. `runner.rs`: test literal gains `scorers: Vec::new()`.
5. Verify: check/clippy/test `eval::`, `openspec validate --strict`.
- Rollback: additive (new field defaults empty, new module); revert restores EH5.

## Open Questions
- None blocking. Per-case scorer overrides remain deferred.
