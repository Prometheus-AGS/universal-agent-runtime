## Context

EH2 yields `Vec<EvalResult>` (each with `scores: Vec<Score{scorer,value}>`). Deps: `serde_json`, `std::fs`. D1 chose file storage; D5 chose delta-vs-baseline gating. Metrics live in `metrics.rs`.

## Goals / Non-Goals
**Goals:** pure summary + comparison; file persistence of results + baseline; metrics. Testable without IO for the pure core.
**Non-Goals:** CLI/HTTP surface (EH5), SurrealDB, absolute-floor gating, trend dashboards.

## Decisions
- **D1 — `ScoreSummary` = `BTreeMap<String, f32>`** (scorer → mean, 0..1; BTree for deterministic ordering/serde). `summarize(&[EvalResult])` averages each scorer's values across cases (ignores scorers absent from a case).
- **D2 — pure `compare(current, baseline, threshold) -> RegressionReport`**: per-scorer `RegressionEntry { scorer, current_mean, baseline_mean: Option<f32>, delta: Option<f32>, regressed }`; `regressed = baseline.map_or(false, |b| (b - current) > threshold)`. `RegressionReport { entries, any_regressed }`. No baseline entry ⇒ not regressed.
- **D3 — file layout (D1):** `save_results(dir, suite, results, ts)` → `<dir>/<suite>-<ts>.json` (sanitize suite for filename); `save_baseline(dir, suite, &ScoreSummary)` / `load_baseline(dir, suite) -> Result<Option<ScoreSummary>>` → `<dir>/<suite>.baseline.json` (None when absent). `std::fs`; create dir if missing.
- **D4 — metrics:** `record_eval_score(suite, scorer, mean)` (gauge), `record_eval_regression()` (counter) in `metrics.rs`; called by the caller (EH5) after compare, not inside the pure fns.
- **D5 — file location:** `src/uar/eval/persistence.rs`; re-export `ScoreSummary, RegressionReport, RegressionEntry, summarize, compare, save_results, save_baseline, load_baseline` from `mod.rs`.

## Risks / Trade-offs
- **[Filename sanitization]** suite names with path chars → Mitigation: replace non-alphanumeric/`-`/`_` with `_` in the filename component.
- **[Threshold semantics]** delta-only (not absolute floor) → Mitigation: D5 decision; absolute-floor is a documented later option.
- **[Mean ignores per-case variance]** a v1 simplification → Mitigation: acceptable for a regression gate; distributions are a later enhancement.

## Migration Plan
1. `persistence.rs`: `ScoreSummary`/`RegressionReport`/`RegressionEntry` + `summarize`/`compare` (pure) + file IO; re-export.
2. Add metric recorders to `metrics.rs`.
3. Unit tests (pure): summarize means; compare pass/regress/no-baseline; results+baseline round-trip via a temp dir.
4. `cargo check`/`clippy`/tests.
- Rollback: additive; revert removes persistence. EH1/EH2 unaffected.

## Open Questions
- Keep last-N result files or unbounded? Defer (EH5/ops concern); v1 writes one file per run.
