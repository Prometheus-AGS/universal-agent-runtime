# eval-persistence-and-regression

## Why

EH2 produces `EvalResult`s in memory but nothing persists them or detects regressions over time. This change (EH4) adds **file-based result persistence**, a named **baseline**, and a **delta-vs-baseline** regression verdict (decision D5) — the core that makes the harness a CI gate. The comparison logic is a pure function (unit-tested); IO is a thin layer.

## What Changes

- **Summary (pure):** `summarize(&[EvalResult]) -> ScoreSummary` = per-scorer mean over the run (0.0–1.0).
- **Comparison (pure):** `compare(current: &ScoreSummary, baseline: &ScoreSummary, threshold: f32) -> RegressionReport` — per-scorer `{ current_mean, baseline_mean, delta, regressed }` where `regressed = (baseline_mean − current_mean) > threshold`; `any_regressed` rolls up. No baseline ⇒ no regressions (first run establishes one).
- **Persistence (files, D1):** `save_results(dir, suite, results, ts) -> PathBuf` writes `evals/results/<suite>-<ts>.json`; `save_baseline(dir, suite, summary)` / `load_baseline(dir, suite) -> Option<ScoreSummary>` at `evals/results/<suite>.baseline.json`.
- **Metrics:** `record_eval_score(suite, scorer, mean)` (gauge `uar_eval_score{suite,scorer}`) and `record_eval_regression()` (`uar_eval_regressions_total`).

Out of scope: the CLI/HTTP surface that calls these (EH5); SurrealDB storage; absolute-floor gating (D5 chose delta-vs-baseline).

## Capabilities

### Modified Capabilities
- **`eval-harness`** — delta `specs/eval-harness/spec.md`. Adds result persistence, baseline storage, and delta-vs-baseline regression detection.

## Impact

- **Affected code:** `src/uar/eval/` (new `persistence.rs`: summary/compare/file IO; re-export from `mod.rs`), `src/uar/telemetry/metrics.rs` (`record_eval_score`, `record_eval_regression`). Reuses `EvalResult/Score` (EH1), `serde_json`. No new dependency.
- **APIs / runtime:** none invoked yet (EH5 wires the runner+persistence to a surface). Additive.
- **Determinism (Rule 35):** `summarize`/`compare` are pure and deterministic; regression verdict is a simple threshold on means.
- **KBD workflow state:** YES — EH4 of `uar-eval-harness` v1.
