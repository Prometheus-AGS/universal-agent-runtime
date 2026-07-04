//! End-to-end integration coverage for the eval run pipeline (EHH4).
//!
//! Drives the public pipeline — `load_suite` → `build_scorers` → `Runner::run`
//! → `summarize` → persist → `compare` — with a deterministic recorded provider,
//! so the composition is verified without a live model. The only piece not
//! covered here is the live-orchestrator construction in `cli::run_suite`
//! (exercised by the nightly real-model job).

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;

use crate::uar::eval::{
    CompletionProvider, EvalResult, Runner, ScoreSummary, build_scorers, compare, load_baseline,
    load_suite, save_baseline, save_results, summarize,
};

/// A deterministic [`CompletionProvider`] returning canned outputs keyed by
/// input. A missing key errors, exercising the runner's contained-failure path.
struct RecordedProvider {
    responses: HashMap<String, String>,
}

#[async_trait]
impl CompletionProvider for RecordedProvider {
    async fn complete(&self, input: &str) -> anyhow::Result<String> {
        self.responses
            .get(input)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("no recorded response for {input}"))
    }
}

/// A provider that always returns the same canned string (for structural tests
/// that assert wiring, not answer correctness).
struct ConstProvider(&'static str);

#[async_trait]
impl CompletionProvider for ConstProvider {
    async fn complete(&self, _input: &str) -> anyhow::Result<String> {
        Ok(self.0.to_string())
    }
}

fn unique_dir(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("uar_eval_itest_{tag}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

const SUITE_JSON: &str = r#"{
  "name": "itest",
  "cases": [
    { "id": "c1", "input": "ping", "expected": "pong" },
    { "id": "c2", "input": "q",    "expected": "yes"  }
  ],
  "scorers": [
    { "type": "exact_match" },
    { "type": "contains" },
    { "type": "non_empty" }
  ]
}"#;

#[tokio::test]
async fn full_pipeline_runs_scores_persists_and_compares() {
    let dir = unique_dir("full");
    let suite_path = dir.join("itest.json");
    std::fs::write(&suite_path, SUITE_JSON).unwrap();

    // load
    let suite = load_suite(&suite_path).expect("suite loads");
    assert_eq!(suite.cases.len(), 2);
    assert_eq!(suite.scorers.len(), 3);

    // recorded outputs: c1 matches expected ("pong"), c2 does not ("no" != "yes")
    let provider: Arc<dyn CompletionProvider> = Arc::new(RecordedProvider {
        responses: HashMap::from([
            ("ping".to_string(), "pong".to_string()),
            ("q".to_string(), "no".to_string()),
        ]),
    });

    // run + score
    let scorers = build_scorers(&suite, &provider);
    assert_eq!(scorers.len(), 3);
    let results = Runner
        .run(&suite, &scorers, provider.as_ref(), Some("test/model"))
        .await;
    assert_eq!(results.len(), 2);
    // every case scored by all three scorers
    assert!(results.iter().all(|r| r.scores.len() == 3));

    // summarize: exact_match 0.5 (1 of 2), contains 0.5, non_empty 1.0
    let summary = summarize(&results);
    assert!((summary["exact_match"] - 0.5).abs() < 1e-6);
    assert!((summary["contains"] - 0.5).abs() < 1e-6);
    assert!((summary["non_empty"] - 1.0).abs() < 1e-6);

    // persist results + reload unchanged
    let path = save_results(&dir, &suite.name, &results, "2026-06-04T00:00:00Z").unwrap();
    let reloaded: Vec<EvalResult> =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(reloaded, results);

    // baseline round-trip
    assert!(load_baseline(&dir, &suite.name).unwrap().is_none());
    save_baseline(&dir, &suite.name, &summary).unwrap();
    assert_eq!(
        load_baseline(&dir, &suite.name).unwrap(),
        Some(summary.clone())
    );

    // compare: no baseline = clean; equal = clean; drop beyond threshold = regressed
    assert!(!compare(&summary, &ScoreSummary::new(), 0.1).any_regressed);
    assert!(!compare(&summary, &summary, 0.1).any_regressed);
    let mut higher = summary.clone();
    higher.insert("exact_match".to_string(), 0.9); // current 0.5 → drop 0.4 > 0.1
    assert!(compare(&summary, &higher, 0.1).any_regressed);

    let _ = std::fs::remove_dir_all(&dir);
}

/// Tier-1 CI guard (EHH1): the *shipped* `evals/starter.yaml` must parse, declare
/// scorers, and run through the harness — deterministically, no model call. This
/// fails the build if the suite file rots or the wiring breaks.
#[tokio::test]
async fn starter_suite_is_valid_and_runs() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("evals/starter.yaml");
    let suite = load_suite(&path).expect("shipped evals/starter.yaml loads");
    assert_eq!(suite.name, "starter");
    assert!(!suite.cases.is_empty(), "starter suite has cases");
    assert!(!suite.scorers.is_empty(), "starter suite declares scorers");

    let provider: Arc<dyn CompletionProvider> = Arc::new(ConstProvider("a deterministic answer"));
    let scorers = build_scorers(&suite, &provider);
    assert_eq!(scorers.len(), suite.scorers.len());

    let results = Runner
        .run(&suite, &scorers, provider.as_ref(), Some("test/model"))
        .await;
    assert_eq!(results.len(), suite.cases.len());
    // Every case was scored by every declared scorer.
    assert!(
        results
            .iter()
            .all(|r| r.scores.len() == suite.scorers.len())
    );
}

/// Tier-1 CI guard (CH-17): each targeted suite's shipped YAML must parse
/// AND, run through its *real* fixture provider (not a canned stand-in —
/// these suites test actual runtime decision code), score 1.0 on every
/// case. This fails the build if a suite file and its provider's fixture
/// data (skills, seeded catalog providers) drift out of sync.
#[tokio::test]
async fn targeted_suites_are_valid_and_score_perfectly() {
    use crate::uar::eval::{ContextEfficiencyProvider, RoutingProvider, SkillActivationProvider};

    let cases: Vec<(&str, Arc<dyn CompletionProvider>)> = vec![
        (
            "skill-activation",
            Arc::new(SkillActivationProvider::new().await),
        ),
        ("routing-accuracy", Arc::new(RoutingProvider::new().await)),
        (
            "context-efficiency",
            Arc::new(ContextEfficiencyProvider) as Arc<dyn CompletionProvider>,
        ),
    ];

    for (name, provider) in cases {
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("evals/{name}.yaml"));
        let suite = load_suite(&path).unwrap_or_else(|e| panic!("evals/{name}.yaml loads: {e}"));
        assert_eq!(suite.name, name);
        assert!(!suite.cases.is_empty(), "{name} suite has cases");

        let scorers = build_scorers(&suite, &provider);
        let results = Runner
            .run(&suite, &scorers, provider.as_ref(), None)
            .await;
        assert_eq!(results.len(), suite.cases.len());

        for r in &results {
            for score in &r.scores {
                assert!(
                    (score.value - 1.0).abs() < f32::EPSILON,
                    "{name}/{}: scorer '{}' expected 1.0, got {} (detail: {:?})",
                    r.case_id,
                    score.scorer,
                    score.value,
                    score.detail
                );
            }
        }
    }
}

#[tokio::test]
async fn provider_failure_is_contained_and_run_completes() {
    let dir = unique_dir("fail");
    let suite_path = dir.join("itest.json");
    std::fs::write(&suite_path, SUITE_JSON).unwrap();
    let suite = load_suite(&suite_path).unwrap();

    // Only c1 has a recorded output; c2 ("q") is missing → provider errors.
    let provider: Arc<dyn CompletionProvider> = Arc::new(RecordedProvider {
        responses: HashMap::from([("ping".to_string(), "pong".to_string())]),
    });
    let scorers = build_scorers(&suite, &provider);
    let results = Runner.run(&suite, &scorers, provider.as_ref(), None).await;

    // Both cases still produce a result; c2's is a single contained completion failure.
    assert_eq!(results.len(), 2);
    let c2 = results.iter().find(|r| r.case_id == "c2").unwrap();
    assert_eq!(c2.scores.len(), 1);
    assert_eq!(c2.scores[0].scorer, "completion");
    assert_eq!(c2.scores[0].value, 0.0);

    let _ = std::fs::remove_dir_all(&dir);
}
