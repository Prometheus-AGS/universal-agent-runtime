# Tasks — eval-persistence-and-regression

## 0. Bootstrap
- [x] 0.1 Confirm EH1/EH2 types (EvalResult/Score) + serde_json; metrics.rs recorder pattern
- [x] 0.2 `cargo check --features postgres-backend` green on branch base

## 1. Pure summary + comparison (src/uar/eval/persistence.rs)
- [x] 1.1 `pub type ScoreSummary = BTreeMap<String,f32>` + `pub fn summarize(&[EvalResult]) -> ScoreSummary` (per-scorer mean)
- [x] 1.2 `RegressionEntry { scorer, current_mean, baseline_mean: Option<f32>, delta: Option<f32>, regressed }` + `RegressionReport { entries, any_regressed }`
- [x] 1.3 `pub fn compare(current, baseline, threshold) -> RegressionReport` — regressed = baseline−current > threshold; no baseline ⇒ not regressed

## 2. File persistence
- [x] 2.1 `save_results(dir, suite, &[EvalResult], ts) -> Result<PathBuf>` → `<dir>/<sanitized-suite>-<ts>.json` (create dir)
- [x] 2.2 `save_baseline(dir, suite, &ScoreSummary) -> Result<()>` + `load_baseline(dir, suite) -> Result<Option<ScoreSummary>>` (None when missing)
- [x] 2.3 Re-export the above from `mod.rs`

## 3. Metrics
- [x] 3.1 `record_eval_score(suite, scorer, mean: f64)` (gauge `uar_eval_score{suite,scorer}`) + `record_eval_regression()` (`uar_eval_regressions_total`) in metrics.rs

## 4. Tests (pure + temp-dir IO)
- [x] 4.1 `summarize`: multi-case mean per scorer
- [x] 4.2 `compare`: regression (delta>threshold), within-threshold no-regress, no-baseline no-regress
- [x] 4.3 results save + reload round-trip; baseline save/load + missing → None (temp dir)

## 5. Validation (gate)
- [x] 5.1 `cargo check --features postgres-backend` clean; zero new warnings
- [x] 5.2 `cargo clippy` — no new warnings in the eval module
- [x] 5.3 `cargo test --features postgres-backend --lib eval::` — new tests pass; full suite unaffected
- [x] 5.4 `openspec validate eval-persistence-and-regression --strict`; update `.kbd-orchestrator` progress

## Notes
- Pure summarize/compare; thin file IO (D1). Delta-vs-baseline gate (D5). Metrics recorded by the caller (EH5), not the pure fns. No new dependency.
