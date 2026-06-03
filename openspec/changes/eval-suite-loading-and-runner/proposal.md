# eval-suite-loading-and-runner

## Why

EH1 landed the eval domain + scorers but nothing loads suites or runs them. This change (EH2) adds the **suite loader** (golden files → `EvalSuite`) and the **runner** (execute each case → score → collect `EvalResult`s). It stays decoupled from the orchestrator (via a `CompletionProvider` seam) so it is fully unit-testable without a live LLM; EH5 wires the real provider.

## What Changes

- **Suite loader:** `load_suite(path) -> Result<EvalSuite>` reading `evals/<suite>.{yaml,yml,json}` (serde_yaml — already a dep — for YAML, serde_json for JSON; chosen by extension).
- **`CompletionProvider` seam:** `#[async_trait] trait CompletionProvider { async fn complete(&self, input: &str) -> Result<String> }` — the runner depends on this, not the orchestrator. (EH5 provides an impl wrapping `orchestrator.chat_non_streaming`; tests use a stub.)
- **Runner:** `Runner::run(suite, scorers, provider, model) -> Vec<EvalResult>` — for each case: get the output from the provider, run the configured scorers, build an `EvalResult` (with an RFC3339 `run_at`). A per-case completion **error is contained**: it records a failed `EvalResult` (a `completion` score of 0.0 with the error detail) and continues the suite.
- Unit tests with a stub provider (no live LLM): loader (YAML + JSON), per-case scoring/aggregation, error containment.

Out of scope: persistence + regression (EH4), CLI/HTTP surface (EH5), LLM-as-judge (deferred), full-agent-run mode.

## Capabilities

### Modified Capabilities
- **`eval-harness`** — delta `specs/eval-harness/spec.md`. Adds suite-loading and runner requirements (load from file; run cases through a completion provider; collect results; contain per-case errors).

## Impact

- **Affected code:** `src/uar/eval/` (new `runner.rs` with loader + `CompletionProvider` + `Runner`; re-export from `mod.rs`). Reuses `EvalCase/EvalSuite/EvalResult/Score/Scorer` (EH1), `serde_yaml`/`serde_json` (deps), `chrono` (dep) for `run_at`.
- **APIs / runtime:** none invoked yet (EH5 wires it). Additive.
- **Testability:** the `CompletionProvider` seam keeps the runner LLM-free in tests (Rule 30 — verifiable without a model).
- **KBD workflow state:** YES — EH2 of `uar-eval-harness` v1.
