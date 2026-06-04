//! Suite-declared scorer configuration + the scorer factory (EHH3).
//!
//! A suite can declare which scorers apply to its cases via [`ScorerSpec`];
//! [`build_scorers`] turns that declaration into concrete scorers, falling back
//! to a default heuristic when a suite declares none (preserving prior behavior).

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use super::{
    CompletionProvider, Contains, EvalSuite, ExactMatch, JsonValid, LlmJudge, NonEmpty,
    PatternMatch, PatternMode, Scorer, Sycophancy,
};

/// A declarative scorer entry in a suite file. Serde-tagged by `type`
/// (`snake_case`), e.g. `{ "type": "pattern_match", "pattern": "ERROR", "mode": "starts_with" }`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ScorerSpec {
    ExactMatch,
    Contains,
    JsonValid,
    NonEmpty,
    PatternMatch {
        pattern: String,
        mode: PatternMode,
    },
    Sycophancy,
    /// LLM-as-judge: grade against `rubric` via the run's completion provider.
    /// Advisory (not part of the hard regression gate).
    LlmJudge {
        rubric: String,
    },
}

impl ScorerSpec {
    /// Construct the concrete scorer for this spec. `provider` is used only by
    /// the `llm_judge` variant; rule scorers ignore it.
    fn build(&self, provider: &Arc<dyn CompletionProvider>) -> Arc<dyn Scorer> {
        match self {
            ScorerSpec::ExactMatch => Arc::new(ExactMatch),
            ScorerSpec::Contains => Arc::new(Contains),
            ScorerSpec::JsonValid => Arc::new(JsonValid),
            ScorerSpec::NonEmpty => Arc::new(NonEmpty),
            ScorerSpec::PatternMatch { pattern, mode } => Arc::new(PatternMatch {
                pattern: pattern.clone(),
                mode: *mode,
            }),
            ScorerSpec::Sycophancy => Arc::new(Sycophancy),
            ScorerSpec::LlmJudge { rubric } => {
                Arc::new(LlmJudge::new(Arc::clone(provider), rubric.clone()))
            }
        }
    }
}

/// Build the scorers for a suite: its declared [`ScorerSpec`]s, or — when it
/// declares none — the default set from [`default_scorers`]. `provider` backs
/// any `llm_judge` scorers.
#[must_use]
pub fn build_scorers(
    suite: &EvalSuite,
    provider: &Arc<dyn CompletionProvider>,
) -> Vec<Arc<dyn Scorer>> {
    if suite.scorers.is_empty() {
        return default_scorers(suite);
    }
    suite.scorers.iter().map(|s| s.build(provider)).collect()
}

/// Default scorer set when a suite declares none: quality scorers always, plus
/// expected-based scorers when every case carries an `expected` output.
#[must_use]
pub fn default_scorers(suite: &EvalSuite) -> Vec<Arc<dyn Scorer>> {
    let mut scorers: Vec<Arc<dyn Scorer>> = Vec::new();
    if !suite.cases.is_empty() && suite.cases.iter().all(|c| c.expected.is_some()) {
        scorers.push(Arc::new(ExactMatch));
        scorers.push(Arc::new(Contains));
    }
    scorers.push(Arc::new(NonEmpty));
    scorers.push(Arc::new(Sycophancy));
    scorers
}

#[cfg(test)]
mod tests {
    use super::{ScorerSpec, build_scorers, default_scorers};
    use crate::uar::eval::{CompletionProvider, EvalCase, EvalSuite, PatternMode};
    use async_trait::async_trait;
    use std::sync::Arc;

    struct StubProvider;
    #[async_trait]
    impl CompletionProvider for StubProvider {
        async fn complete(&self, _input: &str) -> anyhow::Result<String> {
            Ok(r#"{"score": 1.0}"#.to_string())
        }
    }

    fn provider() -> Arc<dyn CompletionProvider> {
        Arc::new(StubProvider)
    }

    fn suite(expecteds: &[Option<&str>], scorers: Vec<ScorerSpec>) -> EvalSuite {
        EvalSuite {
            name: "s".into(),
            cases: expecteds
                .iter()
                .enumerate()
                .map(|(i, e)| EvalCase {
                    id: format!("c{i}"),
                    input: "in".into(),
                    expected: e.map(str::to_string),
                    metadata: serde_json::Map::new(),
                })
                .collect(),
            scorers,
        }
    }

    #[test]
    fn default_set_includes_expected_based_only_when_all_have_expected() {
        // all have expected → exact+contains+nonempty+sycophancy = 4
        assert_eq!(
            default_scorers(&suite(&[Some("a"), Some("b")], vec![])).len(),
            4
        );
        // some missing expected → only nonempty+sycophancy = 2
        assert_eq!(default_scorers(&suite(&[Some("a"), None], vec![])).len(), 2);
        // empty suite → 2 (no expected-based)
        assert_eq!(default_scorers(&suite(&[], vec![])).len(), 2);
    }

    #[test]
    fn build_scorers_uses_declaration_when_present() {
        // Declared scorers are used verbatim (count == declared length),
        // independent of the expected-based heuristic.
        let s = suite(
            &[Some("a")],
            vec![ScorerSpec::JsonValid, ScorerSpec::NonEmpty],
        );
        assert_eq!(build_scorers(&s, &provider()).len(), 2);
    }

    #[test]
    fn build_scorers_includes_llm_judge_when_declared() {
        let s = suite(
            &[None],
            vec![
                ScorerSpec::NonEmpty,
                ScorerSpec::LlmJudge {
                    rubric: "ok?".into(),
                },
            ],
        );
        assert_eq!(build_scorers(&s, &provider()).len(), 2);
    }

    #[test]
    fn build_scorers_falls_back_to_default_when_empty() {
        let s = suite(&[Some("a"), Some("b")], vec![]);
        assert_eq!(
            build_scorers(&s, &provider()).len(),
            default_scorers(&s).len()
        );
    }

    #[test]
    fn suite_without_scorers_field_deserializes_to_empty() {
        // Backward compatibility: no `scorers` key → empty (serde default) → heuristic.
        let s: EvalSuite =
            serde_json::from_str(r#"{"name":"x","cases":[{"id":"a","input":"i"}]}"#).unwrap();
        assert!(s.scorers.is_empty());
        assert!(!build_scorers(&s, &provider()).is_empty());
    }

    #[test]
    fn llm_judge_spec_round_trips() {
        let spec = ScorerSpec::LlmJudge {
            rubric: "Is it correct?".into(),
        };
        let json = serde_json::to_string(&spec).unwrap();
        assert!(json.contains("\"type\":\"llm_judge\""));
        let back: ScorerSpec = serde_json::from_str(&json).unwrap();
        assert_eq!(spec, back);
    }

    #[test]
    fn pattern_match_spec_round_trips() {
        let spec = ScorerSpec::PatternMatch {
            pattern: "ERROR".into(),
            mode: PatternMode::StartsWith,
        };
        let json = serde_json::to_string(&spec).unwrap();
        assert!(json.contains("\"type\":\"pattern_match\""));
        assert!(json.contains("\"mode\":\"starts_with\""));
        let back: ScorerSpec = serde_json::from_str(&json).unwrap();
        assert_eq!(spec, back);
    }
}
