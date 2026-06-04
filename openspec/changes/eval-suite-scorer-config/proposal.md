# eval-suite-scorer-config

## Why

EH5 shipped the eval CLI with a **hardcoded scorer heuristic** (`[NonEmpty, Sycophancy]`, plus `[ExactMatch, Contains]` when every case has `expected`). A suite can't ask for `json_valid`, a specific `pattern_match`, or (soon) an `llm_judge`. This change (EHH3) lets a suite **declare its scorers**, and centralizes scorer construction in a factory — the foundation the LLM-judge scorer (EHH2) and the starter suite + CI gate (EHH1) build on.

## What Changes

- **`ScorerSpec`** — a serde-tagged enum (`#[serde(tag = "type", rename_all = "snake_case")]`) over the rule scorers: `exact_match`, `contains`, `json_valid`, `non_empty`, `pattern_match { pattern, mode }`, `sycophancy`. (The `llm_judge` variant is added by EHH2.)
- **`EvalSuite.scorers: Vec<ScorerSpec>`** with `#[serde(default)]` — existing suites (no `scorers:` key) deserialize unchanged.
- **`build_scorers(suite) -> Vec<Arc<dyn Scorer>>`** factory: maps each `ScorerSpec` to its scorer; when `suite.scorers` is empty, falls back to the existing heuristic (`default_scorers`) so behavior is preserved.
- **CLI** calls `build_scorers` instead of the inline `select_scorers`; the heuristic moves into the factory.
- `PatternMode` gains `Serialize`/`Deserialize`/`PartialEq` (+ snake_case) so it can appear in a spec.

Out of scope: the `llm_judge` scorer (EHH2); per-case scorer overrides (deferred).

## Capabilities

### Modified Capabilities
- **`eval-harness`** — delta `specs/eval-harness/spec.md`. Adds suite-declared scorer configuration with a backward-compatible default.

## Impact

- **Affected code:** `src/uar/eval/mod.rs` (`EvalSuite.scorers`, `PatternMode` derives, re-exports), new `src/uar/eval/scorer_spec.rs` (`ScorerSpec` + `build_scorers` + `default_scorers`), `src/uar/eval/cli.rs` (call the factory; drop inline `select_scorers`), `src/uar/eval/runner.rs` (test literal gains the field).
- **Behavior preservation (Rule 32):** a suite without a `scorers:` key produces exactly the EH5 heuristic; serde default keeps deserialization unchanged.
- **No new dependency** (Rule 27): reuses `serde`.
- **KBD workflow state:** YES — EHH3, round 1 of `eval-harness-hardening`.
