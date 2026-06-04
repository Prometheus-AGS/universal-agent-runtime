//! Eval result persistence + regression detection (EH4).
//!
//! Pure `summarize`/`compare` (per-scorer means and a delta-vs-baseline verdict)
//! plus a thin file layer that stores run results and a per-suite baseline under
//! a results directory. The CLI surface (EH5) calls these and records metrics.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::EvalResult;

/// Per-scorer mean score (0.0–1.0) over a run. `BTreeMap` for deterministic order.
pub type ScoreSummary = BTreeMap<String, f32>;

/// Mean score per scorer across all results in a run.
#[must_use]
pub fn summarize(results: &[EvalResult]) -> ScoreSummary {
    let mut sums: BTreeMap<String, (f64, u32)> = BTreeMap::new();
    for r in results {
        for s in &r.scores {
            let e = sums.entry(s.scorer.clone()).or_insert((0.0, 0));
            e.0 += f64::from(s.value);
            e.1 += 1;
        }
    }
    sums.into_iter()
        .map(|(k, (sum, n))| {
            #[expect(
                clippy::cast_possible_truncation,
                reason = "mean of 0..1 values fits f32"
            )]
            let mean = if n == 0 {
                0.0
            } else {
                (sum / f64::from(n)) as f32
            };
            (k, mean)
        })
        .collect()
}

/// One scorer's regression verdict vs a baseline.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RegressionEntry {
    pub scorer: String,
    pub current_mean: f32,
    pub baseline_mean: Option<f32>,
    pub delta: Option<f32>,
    pub regressed: bool,
}

/// The full regression report for a run vs its baseline.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RegressionReport {
    pub entries: Vec<RegressionEntry>,
    pub any_regressed: bool,
}

/// Compare a current summary to a baseline. A scorer regresses when its mean
/// drops below the baseline by more than `threshold`. With no baseline entry for
/// a scorer, it is not a regression (a run can establish a baseline).
#[must_use]
pub fn compare(
    current: &ScoreSummary,
    baseline: &ScoreSummary,
    threshold: f32,
) -> RegressionReport {
    let mut entries = Vec::with_capacity(current.len());
    let mut any = false;
    for (scorer, &cur) in current {
        let base = baseline.get(scorer).copied();
        let delta = base.map(|b| cur - b);
        let regressed = base.is_some_and(|b| (b - cur) > threshold);
        if regressed {
            any = true;
        }
        entries.push(RegressionEntry {
            scorer: scorer.clone(),
            current_mean: cur,
            baseline_mean: base,
            delta,
            regressed,
        });
    }
    RegressionReport {
        entries,
        any_regressed: any,
    }
}

/// Sanitize a suite name into a safe filename component.
fn safe_name(suite: &str) -> String {
    suite
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Write a run's results to `<dir>/<suite>-<ts>.json`, creating `dir` if needed.
pub fn save_results(
    dir: &Path,
    suite: &str,
    results: &[EvalResult],
    ts: &str,
) -> anyhow::Result<PathBuf> {
    std::fs::create_dir_all(dir)?;
    let path = dir.join(format!("{}-{}.json", safe_name(suite), safe_name(ts)));
    std::fs::write(&path, serde_json::to_string_pretty(results)?)?;
    Ok(path)
}

/// Path of a suite's baseline summary file.
fn baseline_path(dir: &Path, suite: &str) -> PathBuf {
    dir.join(format!("{}.baseline.json", safe_name(suite)))
}

/// Save a suite's baseline summary, creating `dir` if needed.
pub fn save_baseline(dir: &Path, suite: &str, summary: &ScoreSummary) -> anyhow::Result<()> {
    std::fs::create_dir_all(dir)?;
    std::fs::write(
        baseline_path(dir, suite),
        serde_json::to_string_pretty(summary)?,
    )?;
    Ok(())
}

/// Load a suite's baseline summary, or `None` if it does not exist.
pub fn load_baseline(dir: &Path, suite: &str) -> anyhow::Result<Option<ScoreSummary>> {
    let path = baseline_path(dir, suite);
    if !path.exists() {
        return Ok(None);
    }
    let body = std::fs::read_to_string(&path)?;
    Ok(Some(serde_json::from_str(&body)?))
}

#[cfg(test)]
mod tests {
    use super::{ScoreSummary, compare, load_baseline, save_baseline, save_results, summarize};
    use crate::uar::eval::{EvalResult, Score};

    fn result(case: &str, scores: &[(&str, f32)]) -> EvalResult {
        EvalResult {
            suite: "s".into(),
            case_id: case.into(),
            model: None,
            scores: scores
                .iter()
                .map(|(n, v)| Score::new(*n, *v, None))
                .collect(),
            run_at: "2026-06-03T00:00:00Z".into(),
        }
    }

    #[test]
    fn summarize_means_per_scorer() {
        let results = vec![
            result("c1", &[("acc", 1.0), ("q", 0.4)]),
            result("c2", &[("acc", 0.0), ("q", 0.6)]),
        ];
        let s = summarize(&results);
        assert!((s["acc"] - 0.5).abs() < 1e-6);
        assert!((s["q"] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn compare_detects_regression() {
        let mut base: ScoreSummary = ScoreSummary::new();
        base.insert("acc".into(), 0.9);
        let mut cur: ScoreSummary = ScoreSummary::new();
        cur.insert("acc".into(), 0.7); // dropped 0.2

        let report = compare(&cur, &base, 0.1);
        assert!(report.any_regressed);
        assert!(report.entries[0].regressed);

        // within threshold (0.05 drop, threshold 0.1) → no regression
        cur.insert("acc".into(), 0.85);
        assert!(!compare(&cur, &base, 0.1).any_regressed);
    }

    #[test]
    fn compare_no_baseline_is_clean() {
        let mut cur: ScoreSummary = ScoreSummary::new();
        cur.insert("acc".into(), 0.1);
        let report = compare(&cur, &ScoreSummary::new(), 0.1);
        assert!(!report.any_regressed);
        assert_eq!(report.entries[0].baseline_mean, None);
    }

    #[test]
    fn results_and_baseline_round_trip() {
        let dir = std::env::temp_dir().join("uar_eval_persist_test");
        let _ = std::fs::remove_dir_all(&dir);
        let results = vec![result("c1", &[("acc", 1.0)])];
        let path = save_results(&dir, "my suite", &results, "2026-06-03T00:00:00Z").unwrap();
        let reloaded: Vec<EvalResult> =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(reloaded, results);

        // baseline absent → None, then save → load round-trips
        assert!(load_baseline(&dir, "my suite").unwrap().is_none());
        let summary = summarize(&results);
        save_baseline(&dir, "my suite", &summary).unwrap();
        assert_eq!(load_baseline(&dir, "my suite").unwrap(), Some(summary));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
