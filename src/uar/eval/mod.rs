//! Evaluation harness — domain model + scorers (foundation layer).
//!
//! Defines eval cases/suites/scores/results and a [`Scorer`] contract with
//! rule-based scorers. This module performs no IO and runs nothing on its own;
//! the suite loader, runner, persistence, and CLI surface are separate changes.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub mod cli;
mod persistence;
mod runner;
pub use persistence::{
    RegressionEntry, RegressionReport, ScoreSummary, compare, load_baseline, save_baseline,
    save_results, summarize,
};
pub use runner::{CompletionProvider, Runner, load_suite};

/// A single evaluation case: an input and an optional expected output.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EvalCase {
    pub id: String,
    pub input: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected: Option<String>,
    #[serde(default)]
    pub metadata: serde_json::Map<String, serde_json::Value>,
}

/// A named collection of eval cases (loaded from a golden suite file by EH2).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EvalSuite {
    pub name: String,
    pub cases: Vec<EvalCase>,
}

/// A normalized score in the range 0.0–1.0 produced by a [`Scorer`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Score {
    pub scorer: String,
    pub value: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

impl Score {
    /// Construct a score, clamping `value` into 0.0–1.0 so every scorer is
    /// normalized by construction.
    #[must_use]
    pub fn new(scorer: impl Into<String>, value: f32, detail: Option<String>) -> Self {
        Self {
            scorer: scorer.into(),
            value: value.clamp(0.0, 1.0),
            detail,
        }
    }
}

/// The scores for one case in a run (persisted to a file by EH4).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EvalResult {
    pub suite: String,
    pub case_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    pub scores: Vec<Score>,
    /// RFC3339 timestamp, set by the runner.
    pub run_at: String,
}

/// Maps a `(case, output)` pair to a normalized [`Score`].
///
/// Async so that LLM-as-judge scorers (a later change) fit without trait churn;
/// rule-based scorers compute synchronously.
#[async_trait]
pub trait Scorer: Send + Sync {
    /// Stable scorer name (used as the `Score.scorer` label).
    fn name(&self) -> &'static str;
    /// Score `output` for `case`. Rule scorers are deterministic and do no IO.
    async fn score(&self, case: &EvalCase, output: &str) -> Score;
}

// ─────────────────────────────────────────────────────────────────────────────
// Rule-based scorers
// ─────────────────────────────────────────────────────────────────────────────

/// 1.0 when `output` equals the case's `expected`, else 0.0 (0.0 if no expected).
#[derive(Debug)]
pub struct ExactMatch;
#[async_trait]
impl Scorer for ExactMatch {
    fn name(&self) -> &'static str {
        "exact_match"
    }
    async fn score(&self, case: &EvalCase, output: &str) -> Score {
        let v = f32::from(u8::from(case.expected.as_deref() == Some(output)));
        Score::new(self.name(), v, None)
    }
}

/// 1.0 when `output` contains the case's `expected` substring, else 0.0.
#[derive(Debug)]
pub struct Contains;
#[async_trait]
impl Scorer for Contains {
    fn name(&self) -> &'static str {
        "contains"
    }
    async fn score(&self, case: &EvalCase, output: &str) -> Score {
        let hit = case.expected.as_deref().is_some_and(|e| output.contains(e));
        Score::new(self.name(), f32::from(u8::from(hit)), None)
    }
}

/// 1.0 when `output` parses as valid JSON, else 0.0.
#[derive(Debug)]
pub struct JsonValid;
#[async_trait]
impl Scorer for JsonValid {
    fn name(&self) -> &'static str {
        "json_valid"
    }
    async fn score(&self, _case: &EvalCase, output: &str) -> Score {
        let ok = serde_json::from_str::<serde_json::Value>(output.trim()).is_ok();
        Score::new(self.name(), f32::from(u8::from(ok)), None)
    }
}

/// 1.0 when `output` is non-empty after trimming, else 0.0.
#[derive(Debug)]
pub struct NonEmpty;
#[async_trait]
impl Scorer for NonEmpty {
    fn name(&self) -> &'static str {
        "non_empty"
    }
    async fn score(&self, _case: &EvalCase, output: &str) -> Score {
        Score::new(
            self.name(),
            f32::from(u8::from(!output.trim().is_empty())),
            None,
        )
    }
}

/// How a [`PatternMatch`] matches its literal `pattern` against the output.
#[derive(Debug, Clone, Copy)]
pub enum PatternMode {
    Contains,
    StartsWith,
    EndsWith,
}

/// Literal substring/anchor match (no `regex` dependency — covers the common
/// contains/format checks; a true-regex scorer is a later option).
#[derive(Debug)]
pub struct PatternMatch {
    pub pattern: String,
    pub mode: PatternMode,
}
#[async_trait]
impl Scorer for PatternMatch {
    fn name(&self) -> &'static str {
        "pattern_match"
    }
    async fn score(&self, _case: &EvalCase, output: &str) -> Score {
        let hit = match self.mode {
            PatternMode::Contains => output.contains(&self.pattern),
            PatternMode::StartsWith => output.trim_start().starts_with(&self.pattern),
            PatternMode::EndsWith => output.trim_end().ends_with(&self.pattern),
        };
        Score::new(self.name(), f32::from(u8::from(hit)), None)
    }
}

/// Quality scorer: higher value = less sycophantic (`1.0 - sycophancy_score`),
/// derived from the existing rule-based sycophancy detector. Uses a default
/// detector config so eval scoring is independent of runtime gating.
#[derive(Debug)]
pub struct Sycophancy;
#[async_trait]
impl Scorer for Sycophancy {
    fn name(&self) -> &'static str {
        "sycophancy"
    }
    async fn score(&self, _case: &EvalCase, output: &str) -> Score {
        let cfg = crate::config::SycophancyConfig::default();
        match crate::uar::quality::detect(&cfg, output) {
            Some(outcome) => {
                let detail = if outcome.classifications.is_empty() {
                    None
                } else {
                    Some(
                        outcome
                            .classifications
                            .iter()
                            .map(|c| c.pattern_id.as_str())
                            .collect::<Vec<_>>()
                            .join(","),
                    )
                };
                Score::new(self.name(), 1.0 - outcome.score, detail)
            }
            // Empty/clean input — treat as fully clean.
            None => Score::new(self.name(), 1.0, None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Contains, EvalCase, EvalResult, ExactMatch, JsonValid, NonEmpty, PatternMatch, PatternMode,
        Score, Scorer, Sycophancy,
    };

    fn case(expected: Option<&str>) -> EvalCase {
        EvalCase {
            id: "c1".into(),
            input: "in".into(),
            expected: expected.map(str::to_string),
            metadata: serde_json::Map::new(),
        }
    }

    #[test]
    fn score_clamps_to_unit_range() {
        assert_eq!(Score::new("s", 2.0, None).value, 1.0);
        assert_eq!(Score::new("s", -1.0, None).value, 0.0);
        assert_eq!(Score::new("s", 0.5, None).value, 0.5);
    }

    #[test]
    fn eval_result_round_trips() {
        let r = EvalResult {
            suite: "suite".into(),
            case_id: "c1".into(),
            model: Some("openai/gpt-4o".into()),
            scores: vec![Score::new("exact_match", 1.0, None)],
            run_at: "2026-06-03T00:00:00Z".into(),
        };
        let json = serde_json::to_string(&r).unwrap();
        let back: EvalResult = serde_json::from_str(&json).unwrap();
        assert_eq!(r, back);
    }

    #[tokio::test]
    async fn exact_and_contains() {
        let c = case(Some("hello world"));
        assert_eq!(ExactMatch.score(&c, "hello world").await.value, 1.0);
        assert_eq!(ExactMatch.score(&c, "hello").await.value, 0.0);
        assert_eq!(Contains.score(&c, "say hello world now").await.value, 1.0);
        assert_eq!(Contains.score(&c, "goodbye").await.value, 0.0);
    }

    #[tokio::test]
    async fn json_and_non_empty() {
        let c = case(None);
        assert_eq!(JsonValid.score(&c, "{\"a\":1}").await.value, 1.0);
        assert_eq!(JsonValid.score(&c, "not json").await.value, 0.0);
        assert_eq!(NonEmpty.score(&c, "  x ").await.value, 1.0);
        assert_eq!(NonEmpty.score(&c, "   ").await.value, 0.0);
    }

    #[tokio::test]
    async fn pattern_modes() {
        let c = case(None);
        let starts = PatternMatch {
            pattern: "ERROR".into(),
            mode: PatternMode::StartsWith,
        };
        assert_eq!(starts.score(&c, "ERROR: boom").await.value, 1.0);
        assert_eq!(starts.score(&c, "boom ERROR").await.value, 0.0);
        let ends = PatternMatch {
            pattern: "done".into(),
            mode: PatternMode::EndsWith,
        };
        assert_eq!(ends.score(&c, "all done").await.value, 1.0);
    }

    #[tokio::test]
    async fn sycophancy_scorer_is_normalized_and_clean_is_high() {
        let c = case(None);
        let s = Sycophancy
            .score(&c, "The capital of France is Paris.")
            .await;
        assert!((0.0..=1.0).contains(&s.value));
        // A neutral factual statement should not score as heavily sycophantic.
        assert!(s.value >= 0.5);
        // Empty input is treated as fully clean.
        assert_eq!(Sycophancy.score(&c, "").await.value, 1.0);
    }
}
