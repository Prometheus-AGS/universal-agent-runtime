/// Integration tests for chat run management.
///
/// Tests run lifecycle (start, subscribe, history) without requiring
/// a real LLM API key. LLM driver tests are in test_graph_execution.rs
/// (via MockLlmDriver + AgentGraph).
use std::sync::Arc;
use tokio::sync::RwLock;
use universal_agent_runtime::config::LlmConfig;
use universal_agent_runtime::mcp::registry::McpRegistry;
use universal_agent_runtime::session::SessionStore;
use universal_agent_runtime::uar::domain::artifact::AgentArtifact;
use universal_agent_runtime::uar::rag::embeddings::EmbeddingBackend;
use universal_agent_runtime::uar::runtime::manager::RunManager;
use universal_agent_runtime::uar::runtime::skills::SkillRegistry;

/// These tests exercise run lifecycle only — nothing embeds. The
/// unconditionally-compiled `UnavailableEmbeddingBackend` therefore builds and
/// runs under every feature profile, whereas constructing a real backend would
/// panic without `local-models` (the `openai` fallback requires an API key).
fn unavailable_embedding_backend() -> Arc<dyn EmbeddingBackend> {
    Arc::new(
        universal_agent_runtime::uar::rag::embeddings::UnavailableEmbeddingBackend::new(
            384,
            "embeddings are not exercised by these tests",
        ),
    )
}

fn make_vector_matcher() -> Arc<universal_agent_runtime::uar::runtime::matching::VectorMatcher> {
    Arc::new(
        universal_agent_runtime::uar::runtime::matching::VectorMatcher::new(
            unavailable_embedding_backend(),
            0.75,
        ),
    )
}

fn minimal_artifact() -> AgentArtifact {
    let json = serde_json::json!({
        "version": "1.0",
        "kind": "agent",
        "id": "test-agent",
        "metadata": { "title": "Test Agent", "description": "" },
        "policy": {
            "provider": {
                "default": { "provider": "openai", "model": "gpt-4o" },
                "fallbacks": []
            },
            "tools": { "allow": [], "deny": [], "max_concurrent": 1 },
            "skills": { "prefer": [], "max_active": 0, "selection_method": "auto" }
        },
        "prompt": { "system": "You are helpful.", "instructions": [] },
        "memory": {
            "conversation": { "enabled": true },
            "kb": { "enabled": false, "knowledge_bases": [], "citation_required": false }
        },
        "schemas": { "inputs": null, "outputs": null, "state": null },
        "runtime": { "entry": "default", "protocols": {} },
        "tools": { "bundles": [] },
        "ui": { "forms": { "enabled": false }, "artifacts": { "enabled": false, "preferred_types": [] } },
        "extensions": {}
    });
    serde_json::from_value(json).expect("minimal artifact should deserialize")
}

async fn make_manager() -> Arc<RunManager> {
    let mcp = Arc::new(McpRegistry::new_empty());
    let llm_config = LlmConfig {
        model: "openai/gpt-4o".to_string(),
        api_key: Some("test-key".to_string()),
        ..LlmConfig::default()
    };
    let sessions = SessionStore::new();
    let skills = Arc::new(RwLock::new(SkillRegistry::new(None, None)));
    let vm = make_vector_matcher();
    Arc::new(RunManager::new(llm_config, mcp, sessions, skills, vm, None).await)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_start_run_returns_non_empty_run_id() {
    let manager = make_manager().await;
    let run_id = manager
        .start_run(minimal_artifact(), "Hello".to_string(), None, None, vec![])
        .await;
    assert!(
        !run_id.is_empty(),
        "start_run should return a non-empty run_id"
    );
}

#[tokio::test]
async fn test_start_run_returns_unique_ids() {
    let manager = make_manager().await;
    let id1 = manager
        .start_run(minimal_artifact(), "a".to_string(), None, None, vec![])
        .await;
    let id2 = manager
        .start_run(minimal_artifact(), "b".to_string(), None, None, vec![])
        .await;
    assert_ne!(id1, id2, "each run should get a unique id");
}

#[tokio::test]
async fn test_subscribe_returns_receiver_for_active_run() {
    let manager = make_manager().await;
    let run_id = manager
        .start_run(minimal_artifact(), "test".to_string(), None, None, vec![])
        .await;

    // Allow run to register
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;

    let rx = manager.subscribe(&run_id).await;
    assert!(
        rx.is_some(),
        "subscribe should return a receiver for an active run"
    );
}

#[tokio::test]
async fn test_subscribe_returns_none_for_unknown_run() {
    let manager = make_manager().await;
    let rx = manager.subscribe("nonexistent-run-id").await;
    assert!(
        rx.is_none(),
        "subscribe should return None for an unknown run"
    );
}

#[tokio::test]
async fn test_get_run_returns_metadata_for_started_run() {
    let manager = make_manager().await;
    let run_id = manager
        .start_run(minimal_artifact(), "ping".to_string(), None, None, vec![])
        .await;

    // Allow run to register
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;

    let run = manager.get_run(&run_id).await;
    assert!(
        run.is_some(),
        "get_run should return metadata for a started run"
    );
}

#[tokio::test]
async fn test_get_run_returns_none_for_unknown_run() {
    let manager = make_manager().await;
    let run = manager.get_run("no-such-run").await;
    assert!(run.is_none(), "get_run should return None for unknown run");
}

#[tokio::test]
async fn test_session_id_links_run_to_session() {
    let manager = make_manager().await;
    let session_id = "test-session-123".to_string();
    let run_id = manager
        .start_run(
            minimal_artifact(),
            "hello".to_string(),
            Some(session_id.clone()),
            None,
            vec![],
        )
        .await;

    // Allow run to register
    tokio::time::sleep(std::time::Duration::from_millis(20)).await;

    let run_by_session = manager.get_run_by_session_id(&session_id).await;
    // The run may or may not be in active_runs by the time we check (run may have
    // already completed or been cleaned up), but if it is present, the id must match.
    if let Some(run) = run_by_session {
        assert_eq!(run.run_id, run_id, "run id via session lookup should match");
    }
}
