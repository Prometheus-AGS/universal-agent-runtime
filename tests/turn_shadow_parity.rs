//! Corpus-driven parity checks for the typed turn assembler's shadow mode.

use std::collections::BTreeSet;
use std::sync::Arc;

use tokio::sync::RwLock;
use universal_agent_runtime::config::{HarnessConfig, HarnessMode, LlmConfig};
use universal_agent_runtime::llm::mock_driver::MockLlmDriver;
use universal_agent_runtime::mcp::registry::McpRegistry;
use universal_agent_runtime::normalized::NormalizedEvent as DriverEvent;
use universal_agent_runtime::session::SessionStore;
use universal_agent_runtime::uar::defaults::default_agent;
use universal_agent_runtime::uar::domain::events::{MemoryItem, NormalizedEvent as RunEvent};
use universal_agent_runtime::uar::rag::embeddings::{
    EmbeddingBackend, UnavailableEmbeddingBackend,
};
use universal_agent_runtime::uar::runtime::manager::RunManager;
use universal_agent_runtime::uar::runtime::matching::VectorMatcher;
use universal_agent_runtime::uar::runtime::skills::SkillRegistry;

async fn shadow_manager(driver: Arc<MockLlmDriver>) -> Arc<RunManager> {
    let embeddings: Arc<dyn EmbeddingBackend> = Arc::new(UnavailableEmbeddingBackend::new(
        384,
        "embeddings are outside this parity boundary",
    ));
    Arc::new(
        RunManager::new(
            LlmConfig {
                model: "openai/gpt-4o".to_string(),
                api_key: Some("shadow-parity-fixture-key".to_string()),
                ..LlmConfig::default()
            },
            Arc::new(McpRegistry::new_empty()),
            SessionStore::new(),
            Arc::new(RwLock::new(SkillRegistry::default())),
            Arc::new(VectorMatcher::new(embeddings, 0.75)),
            None,
        )
        .await
        .with_llm_driver(driver)
        .with_harness_config(HarnessConfig {
            mode: HarnessMode::Shadow,
            ..HarnessConfig::default()
        }),
    )
}

async fn completed_events(manager: &RunManager, run_id: &str) -> Vec<RunEvent> {
    tokio::time::timeout(std::time::Duration::from_secs(2), async {
        loop {
            let events = manager
                .history_since(run_id, None)
                .await
                .expect("started run keeps event history");
            if events
                .iter()
                .any(|event| matches!(event.event, RunEvent::RunDone { .. }))
            {
                return events.into_iter().map(|event| event.event).collect();
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("shadow run completes through the mock driver")
}

#[tokio::test]
async fn shadow_corpus_has_no_unexpected_differences_and_dispatches_only_legacy() {
    let corpus: serde_json::Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/turn_parity/requests.json"
    )))
    .expect("parity corpus is valid JSON");
    let allowlist: serde_json::Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/turn_parity/intentional_deltas.json"
    )))
    .expect("intentional delta allowlist is valid JSON");
    let mut observed_allowlist_entries = BTreeSet::new();
    let mut request_reports = Vec::new();

    for case in corpus["cases"]
        .as_array()
        .expect("parity corpus cases are an array")
    {
        let case_id = case["id"].as_str().expect("case id is text");
        let driver = Arc::new(MockLlmDriver::new(vec![vec![DriverEvent::Done]]));
        let manager = shadow_manager(Arc::clone(&driver)).await;
        let mut artifact = default_agent();
        artifact.prompt.instructions = case["instructions"]
            .as_array()
            .expect("instructions are an array")
            .iter()
            .map(|value| value.as_str().expect("instruction is text").to_string())
            .collect();
        let memory_hits = case["memory"].as_str().map_or_else(Vec::new, |value| {
            vec![MemoryItem {
                key: format!("{case_id}-memory"),
                value: value.to_string(),
                source: "memory_context".to_string(),
                scope: Some("session".to_string()),
                memory_type: Some("semantic".to_string()),
                importance: Some(0.8),
            }]
        });
        let run_id = manager
            .start_run(
                artifact,
                case["input"]
                    .as_str()
                    .expect("case input is text")
                    .to_string(),
                Some(format!("shadow-{case_id}")),
                Some("shadow-parity-owner".to_string()),
                memory_hits,
            )
            .await;
        let events = completed_events(&manager, &run_id).await;

        assert_eq!(
            driver.requests().len(),
            1,
            "{case_id}: shadow mode must dispatch only the legacy request"
        );
        let reports = events
            .iter()
            .filter_map(|event| match event {
                RunEvent::Artifact { artifact, .. }
                    if artifact.artifact_type == "provider_event"
                        && artifact.title == "resolved_step" =>
                {
                    let content: serde_json::Value =
                        serde_json::from_str(&artifact.content).ok()?;
                    content["payload"]["manifest"]["shadow"]
                        .as_object()
                        .cloned()
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert!(!reports.is_empty(), "{case_id}: shadow report is emitted");
        let mut case_allowlist_entries = BTreeSet::new();
        let mut case_unexpected_differences = 0_u64;
        let resolved_step_count = reports.len();
        for report in reports {
            let unexpected = report["unexpected_difference_count"]
                .as_u64()
                .expect("unexpected difference count is an integer");
            assert_eq!(
                unexpected, 0,
                "{case_id}: unexpected shadow difference: {report:?}"
            );
            assert_eq!(report["dispatched_path"], "legacy");
            for difference in report["differences"]
                .as_array()
                .expect("shadow differences are an array")
            {
                if let Some(id) = difference["allowlist_entry"].as_str() {
                    observed_allowlist_entries.insert(id.to_string());
                    case_allowlist_entries.insert(id.to_string());
                }
            }
            case_unexpected_differences = case_unexpected_differences.saturating_add(unexpected);
        }
        request_reports.push(serde_json::json!({
            "id": case_id,
            "dispatched_path": "legacy",
            "dispatched_request_count": driver.requests().len(),
            "resolved_step_count": resolved_step_count,
            "unexpected_difference_count": case_unexpected_differences,
            "allowlisted_differences": case_allowlist_entries.into_iter().collect::<Vec<_>>(),
        }));
    }

    let allowlist_entries = allowlist["entries"]
        .as_array()
        .expect("allowlist entries are an array");
    for entry in allowlist_entries {
        let id = entry["id"].as_str().expect("allowlist id is text");
        assert!(
            observed_allowlist_entries.contains(id),
            "allowlisted difference '{id}' was not observed by the corpus"
        );
    }

    let observed_report = serde_json::json!({
        "schema_version": 1,
        "harness_mode": "shadow",
        "corpus_size": request_reports.len(),
        "intentional_delta_count": allowlist_entries.len(),
        "requests": request_reports,
        "totals": {
            "unexpected_difference_count": 0,
            "observed_allowlisted_differences": observed_allowlist_entries.into_iter().collect::<Vec<_>>(),
        },
    });
    let checked_in_report: serde_json::Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/openspec/changes/archive/2026-09-04-typed-turn-assembly/parity-report.json"
    )))
    .expect("checked-in parity report is valid JSON");
    assert_eq!(
        observed_report, checked_in_report,
        "checked-in parity report must match the executed shadow corpus"
    );
}
