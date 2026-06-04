# Tasks — eval-suite-scorer-config

## 1. Domain (mod.rs)
- [x] 1.1 Add `Serialize, Deserialize, PartialEq` + `#[serde(rename_all="snake_case")]` to `PatternMode`
- [x] 1.2 Add `#[serde(default)] pub scorers: Vec<ScorerSpec>` to `EvalSuite`
- [x] 1.3 `mod scorer_spec;` + `pub use scorer_spec::{ScorerSpec, build_scorers};`

## 2. Factory (scorer_spec.rs)
- [x] 2.1 `ScorerSpec` tagged enum (exact_match/contains/json_valid/non_empty/pattern_match/sycophancy)
- [x] 2.2 `build_scorers(&EvalSuite) -> Vec<Arc<dyn Scorer>>` — empty → `default_scorers`, else map specs
- [x] 2.3 `default_scorers(&EvalSuite)` — EH5 heuristic moved verbatim
- [x] 2.4 Tests: spec→scorer mapping; empty→heuristic; serde default (no `scorers` key); `pattern_match` round-trip

## 3. CLI (cli.rs)
- [x] 3.1 `run_suite` calls `build_scorers(&suite_obj)`
- [x] 3.2 Remove `select_scorers` + its test (moved); prune now-unused imports

## 4. Tests fix (runner.rs)
- [x] 4.1 `suite()` literal gains `scorers: Vec::new()`

## 5. Validation (gate)
- [x] 5.1 `cargo check --features postgres-backend` clean; zero new warnings
- [x] 5.2 `cargo clippy` — no new warnings in touched code
- [x] 5.3 `cargo test --features postgres-backend --lib eval::` green
- [x] 5.4 `openspec validate eval-suite-scorer-config --strict`; update progress

## Notes
- Behavior preserved when a suite declares no scorers (Rule 32). No new dep. `llm_judge` + per-case overrides deferred (EHH2 / later).
