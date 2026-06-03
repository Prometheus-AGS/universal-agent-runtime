# eval-domain-and-rule-scorers

## Why

`uar-eval-harness` goal **S1** is greenfield — there is no eval domain model or scorers anywhere in compiled `src/` (the dead `src/testing/` tree is CI analytics, not eval). This change lays the **foundation layer**: the typed domain (cases, suites, scores, results), a `Scorer` trait, and a set of rule-based scorers. It deliberately ships no runner, IO, or surface — those are EH2/EH4/EH5 — so it stays pure and fully unit-testable.

## What Changes

- New module `src/uar/eval/` (registered via `pub mod eval;` in `uar/mod.rs`):
  - **Domain:** `EvalCase { id, input, expected: Option<String>, metadata }`, `EvalSuite { name, cases }`, `Score { scorer, value: f32 (0.0–1.0), detail: Option<String> }`, `EvalResult { suite, case_id, model, scores, run_at }` (serde-serializable for later file persistence).
  - **`Scorer` trait:** `fn name(&self) -> &str` + `async fn score(&self, case: &EvalCase, output: &str) -> Score` (async to allow LLM-judge scorers later; rule scorers are sync internally).
  - **Rule-based scorers:** `ExactMatch`, `Contains`, `Regex` (substring/manual — no regex dep unless already present), `JsonValid`, `NonEmpty`, and a `Sycophancy` adapter wrapping `quality::detect` (`value = 1.0 − sycophancy_score`; clean text ⇒ 1.0).
- Unit tests for each scorer (positive/negative) and the domain (serde round-trip).

No runner, no file/HTTP/CLI surface, no persistence, no LLM call — purely additive types + scorers. Nothing calls them yet (EH2 wires the runner), so there is zero runtime behavior change.

## Capabilities

### New Capabilities
- **`eval-harness`** — `specs/eval-harness/spec.md`. Defines the eval domain model and the `Scorer` contract: scorers map a `(case, output)` to a normalized 0.0–1.0 `Score`; the built-in rule scorers' semantics; deterministic, no-IO scoring.

## Impact

- **Affected code:** new `src/uar/eval/` (`mod.rs` + `domain.rs` + `scorers.rs` or similar); `src/uar/mod.rs` (`pub mod eval;`). Reuses `crate::uar::quality::detect` for the sycophancy adapter. No new dependency for v1 (rule scorers use std + `serde_json`; `Regex` uses substring/manual matching to avoid adding the `regex` crate).
- **APIs / runtime:** none — additive types/scorers, not yet invoked.
- **Determinism (Rule 35):** rule scorers are pure and deterministic; the `Score` value is always clamped to 0.0–1.0.
- **KBD workflow state:** YES — EH1, foundation of `uar-eval-harness` v1.
