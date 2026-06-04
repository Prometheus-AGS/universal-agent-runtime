//! LLM-as-judge scorer (EHH2).
//!
//! Grades a candidate output against a rubric by prompting a
//! [`CompletionProvider`] for a JSON verdict, then parsing it deterministically.
//! Judge scores are **advisory** (phase decision D-B): they are reported and
//! persisted like any score, but the hard regression gate uses rule scorers.

use std::sync::Arc;

use async_trait::async_trait;
use serde::Deserialize;

use super::{CompletionProvider, EvalCase, Score, Scorer};

/// Stable scorer name / `Score.scorer` label.
const NAME: &str = "llm_judge";

/// An LLM-as-judge scorer. Holds the provider it calls (captured at
/// construction) since `Scorer::score` only receives `(case, output)`.
pub struct LlmJudge {
    provider: Arc<dyn CompletionProvider>,
    rubric: String,
}

impl std::fmt::Debug for LlmJudge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LlmJudge")
            .field("rubric", &self.rubric)
            .finish_non_exhaustive()
    }
}

impl LlmJudge {
    /// Construct a judge over `provider`, grading against `rubric`.
    #[must_use]
    pub fn new(provider: Arc<dyn CompletionProvider>, rubric: impl Into<String>) -> Self {
        Self {
            provider,
            rubric: rubric.into(),
        }
    }
}

#[async_trait]
impl Scorer for LlmJudge {
    fn name(&self) -> &'static str {
        NAME
    }

    async fn score(&self, case: &EvalCase, output: &str) -> Score {
        let prompt = judge_prompt(&self.rubric, &case.input, output);
        match self.provider.complete(&prompt).await {
            Ok(resp) => parse_verdict(&resp),
            Err(e) => Score::new(NAME, 0.0, Some(format!("judge provider error: {e}"))),
        }
    }
}

/// Build the judge prompt: instruct a JSON-only verdict.
fn judge_prompt(rubric: &str, input: &str, output: &str) -> String {
    format!(
        "You are grading a candidate answer against a rubric. Respond with ONLY a \
JSON object and nothing else, in the form {{\"score\": <number between 0.0 and 1.0>, \
\"reason\": \"<short explanation>\"}}.\n\n\
Rubric:\n{rubric}\n\n\
Input:\n{input}\n\n\
Candidate answer:\n{output}\n"
    )
}

/// Extract the substring from the first `{` to the last `}` (tolerant of models
/// that wrap JSON in prose or code fences).
fn extract_json_object(resp: &str) -> Option<&str> {
    let start = resp.find('{')?;
    let end = resp.rfind('}')?;
    (end > start).then(|| &resp[start..=end])
}

/// The judge's parsed verdict. `score` is `f32` so deserialization needs no
/// truncating cast; `reason` is optional.
#[derive(Debug, Deserialize)]
struct Verdict {
    score: f32,
    #[serde(default)]
    reason: String,
}

/// Parse a judge response into a [`Score`]. Any failure (no JSON, parse error)
/// is contained as a `0.0` score with a detail — never panics.
fn parse_verdict(resp: &str) -> Score {
    let parsed = extract_json_object(resp).and_then(|j| serde_json::from_str::<Verdict>(j).ok());
    if let Some(v) = parsed {
        let value = v.score.clamp(0.0, 1.0);
        let detail = (!v.reason.trim().is_empty()).then_some(v.reason);
        Score::new(NAME, value, detail)
    } else {
        let preview: String = resp.trim().chars().take(120).collect();
        Score::new(
            NAME,
            0.0,
            Some(format!("unparseable judge verdict: {preview}")),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{LlmJudge, parse_verdict};
    use crate::uar::eval::{CompletionProvider, EvalCase, Scorer};
    use async_trait::async_trait;
    use std::sync::Arc;

    fn case() -> EvalCase {
        EvalCase {
            id: "c1".into(),
            input: "What is 2+2?".into(),
            expected: None,
            metadata: serde_json::Map::new(),
        }
    }

    #[test]
    fn parses_clean_json() {
        let s = parse_verdict(r#"{"score": 0.8, "reason": "mostly correct"}"#);
        assert!((s.value - 0.8).abs() < 1e-6);
        assert_eq!(s.detail.as_deref(), Some("mostly correct"));
        assert_eq!(s.scorer, "llm_judge");
    }

    #[test]
    fn parses_json_wrapped_in_prose() {
        let s = parse_verdict("Sure! Here is my verdict:\n```json\n{\"score\": 1.0}\n```\nThanks");
        assert!((s.value - 1.0).abs() < 1e-6);
    }

    #[test]
    fn clamps_out_of_range_score() {
        assert!((parse_verdict(r#"{"score": 1.7}"#).value - 1.0).abs() < 1e-6);
        assert!((parse_verdict(r#"{"score": -0.5}"#).value - 0.0).abs() < 1e-6);
    }

    #[test]
    fn malformed_is_contained_at_zero() {
        let s = parse_verdict("I cannot produce JSON, sorry.");
        assert_eq!(s.value, 0.0);
        assert!(s.detail.unwrap().contains("unparseable"));
        // total garbage with no braces
        assert_eq!(parse_verdict("").value, 0.0);
    }

    struct StubProvider(&'static str);
    #[async_trait]
    impl CompletionProvider for StubProvider {
        async fn complete(&self, _input: &str) -> anyhow::Result<String> {
            Ok(self.0.to_string())
        }
    }

    struct FailingProvider;
    #[async_trait]
    impl CompletionProvider for FailingProvider {
        async fn complete(&self, _input: &str) -> anyhow::Result<String> {
            Err(anyhow::anyhow!("model down"))
        }
    }

    #[tokio::test]
    async fn scores_via_provider() {
        let judge = LlmJudge::new(
            Arc::new(StubProvider(r#"{"score": 0.6, "reason": "ok"}"#)),
            "Is the answer correct?",
        );
        let s = judge.score(&case(), "4").await;
        assert!((s.value - 0.6).abs() < 1e-6);
    }

    #[tokio::test]
    async fn provider_error_is_contained() {
        let judge = LlmJudge::new(Arc::new(FailingProvider), "rubric");
        let s = judge.score(&case(), "4").await;
        assert_eq!(s.value, 0.0);
        assert!(s.detail.unwrap().contains("provider error"));
    }
}
