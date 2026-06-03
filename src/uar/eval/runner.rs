//! Eval suite loading + the runner (EH2).
//!
//! Loads golden suites from JSON/YAML and runs each case through a pluggable
//! [`CompletionProvider`], applying the configured scorers. The runner depends
//! on the provider abstraction (not a specific LLM client) so it is testable
//! without a live model; the orchestrator-backed provider is wired by the CLI.

use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;

use super::{EvalResult, EvalSuite, Score, Scorer};

/// Load an [`EvalSuite`] from a golden file, choosing the parser by extension
/// (`.json`, `.yaml`, `.yml`). Returns an error (never panics) on a missing
/// file, an unparseable body, or an unknown extension.
pub fn load_suite(path: &Path) -> anyhow::Result<EvalSuite> {
    let body = std::fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("failed to read eval suite '{}': {e}", path.display()))?;
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default()
        .to_lowercase();
    match ext.as_str() {
        "json" => serde_json::from_str(&body)
            .map_err(|e| anyhow::anyhow!("invalid JSON eval suite '{}': {e}", path.display())),
        "yaml" | "yml" => serde_yaml::from_str(&body)
            .map_err(|e| anyhow::anyhow!("invalid YAML eval suite '{}': {e}", path.display())),
        other => Err(anyhow::anyhow!(
            "unsupported eval suite extension '{other}' for '{}': use .json/.yaml/.yml",
            path.display()
        )),
    }
}

/// Produces a model output for an eval case's input. Implemented by the CLI over
/// `Orchestrator::chat_non_streaming`; stubbed in tests.
#[async_trait]
pub trait CompletionProvider: Send + Sync {
    async fn complete(&self, input: &str) -> anyhow::Result<String>;
}

/// Runs an [`EvalSuite`] through a [`CompletionProvider`] and scores each case.
#[derive(Debug, Default)]
pub struct Runner;

impl Runner {
    /// Run every case: obtain the output from `provider`, apply `scorers`, and
    /// collect one [`EvalResult`] per case. A per-case completion error is
    /// contained (recorded as a failed result) so the suite continues.
    pub async fn run(
        &self,
        suite: &EvalSuite,
        scorers: &[Arc<dyn Scorer>],
        provider: &dyn CompletionProvider,
        model: Option<&str>,
    ) -> Vec<EvalResult> {
        let mut results = Vec::with_capacity(suite.cases.len());
        for case in &suite.cases {
            let run_at = chrono::Utc::now().to_rfc3339();
            let case_scores = match provider.complete(&case.input).await {
                Ok(output) => {
                    let mut acc = Vec::with_capacity(scorers.len());
                    for scorer in scorers {
                        acc.push(scorer.score(case, &output).await);
                    }
                    acc
                }
                Err(e) => vec![Score::new("completion", 0.0, Some(e.to_string()))],
            };
            results.push(EvalResult {
                suite: suite.name.clone(),
                case_id: case.id.clone(),
                model: model.map(str::to_string),
                scores: case_scores,
                run_at,
            });
        }
        results
    }
}

#[cfg(test)]
mod tests {
    use super::{CompletionProvider, Runner, load_suite};
    use crate::uar::eval::{Contains, EvalCase, EvalSuite, ExactMatch, Scorer};
    use async_trait::async_trait;
    use std::path::Path;
    use std::sync::Arc;

    struct Echo;
    #[async_trait]
    impl CompletionProvider for Echo {
        async fn complete(&self, input: &str) -> anyhow::Result<String> {
            Ok(format!("echo: {input}"))
        }
    }

    struct Failing;
    #[async_trait]
    impl CompletionProvider for Failing {
        async fn complete(&self, _input: &str) -> anyhow::Result<String> {
            Err(anyhow::anyhow!("provider down"))
        }
    }

    fn suite() -> EvalSuite {
        EvalSuite {
            name: "s".into(),
            cases: vec![
                EvalCase {
                    id: "c1".into(),
                    input: "hello".into(),
                    expected: Some("echo: hello".into()),
                    metadata: serde_json::Map::new(),
                },
                EvalCase {
                    id: "c2".into(),
                    input: "world".into(),
                    expected: Some("nope".into()),
                    metadata: serde_json::Map::new(),
                },
            ],
        }
    }

    #[test]
    fn loads_json_suite() {
        let dir = std::env::temp_dir();
        let path = dir.join("uar_eval_test_suite.json");
        std::fs::write(
            &path,
            r#"{"name":"j","cases":[{"id":"a","input":"x","expected":"x"}]}"#,
        )
        .unwrap();
        let s = load_suite(&path).expect("json loads");
        assert_eq!(s.name, "j");
        assert_eq!(s.cases.len(), 1);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn loads_yaml_suite() {
        let dir = std::env::temp_dir();
        let path = dir.join("uar_eval_test_suite.yaml");
        std::fs::write(&path, "name: y\ncases:\n  - id: a\n    input: x\n").unwrap();
        let s = load_suite(&path).expect("yaml loads");
        assert_eq!(s.name, "y");
        assert_eq!(s.cases.len(), 1);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn missing_file_errors() {
        assert!(load_suite(Path::new("/no/such/eval.json")).is_err());
    }

    #[tokio::test]
    async fn runs_cases_and_scores() {
        let scorers: Vec<Arc<dyn Scorer>> = vec![Arc::new(ExactMatch), Arc::new(Contains)];
        let results = Runner
            .run(&suite(), &scorers, &Echo, Some("test/model"))
            .await;
        assert_eq!(results.len(), 2);
        // c1 expected == output ("echo: hello") -> exact 1.0, contains 1.0
        assert_eq!(results[0].scores.len(), 2);
        assert_eq!(results[0].scores[0].value, 1.0);
        assert_eq!(results[0].model.as_deref(), Some("test/model"));
        // c2 expected "nope" not matched
        assert_eq!(results[1].scores[0].value, 0.0);
    }

    #[tokio::test]
    async fn contains_provider_error() {
        let scorers: Vec<Arc<dyn Scorer>> = vec![Arc::new(ExactMatch)];
        let results = Runner.run(&suite(), &scorers, &Failing, None).await;
        // Both cases still produce results, each a contained completion failure.
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].scores.len(), 1);
        assert_eq!(results[0].scores[0].scorer, "completion");
        assert_eq!(results[0].scores[0].value, 0.0);
    }
}
