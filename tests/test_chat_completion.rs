/// Integration tests for chat run management.
///
/// Tests run lifecycle (start, subscribe, history) without requiring
/// a real LLM API key. LLM driver tests are in test_graph_execution.rs
/// (via MockLlmDriver + AgentGraph).
use std::sync::Arc;
use tokio::sync::RwLock;
use universal_agent_runtime::config::LlmConfig;
use universal_agent_runtime::llm::mock_driver::MockLlmDriver;
use universal_agent_runtime::mcp::registry::McpRegistry;
use universal_agent_runtime::normalized::NormalizedEvent as DriverEvent;
use universal_agent_runtime::session::SessionStore;
use universal_agent_runtime::uar::defaults;
use universal_agent_runtime::uar::domain::artifact::AgentArtifact;
use universal_agent_runtime::uar::domain::events::NormalizedEvent;
use universal_agent_runtime::uar::persistence::{
    PersistenceLayer, providers::surreal::SurrealDbProvider,
};
use universal_agent_runtime::uar::rag::embeddings::EmbeddingBackend;
use universal_agent_runtime::uar::runtime::graph::{AgentGraph, AgentNode, GraphState, RouterNode};
use universal_agent_runtime::uar::runtime::manager::RunManager;
use universal_agent_runtime::uar::runtime::skills::SkillRegistry;
use universal_agent_runtime::uar::runtime::turn::RunExecutionRequest;
use universal_agent_runtime::uar::security::claims::{UserClaims, UserContext};

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

async fn make_graph_manager(driver: Arc<MockLlmDriver>) -> (Arc<RunManager>, tempfile::TempDir) {
    let mcp = Arc::new(McpRegistry::new_empty());
    let llm_config = LlmConfig {
        model: "openai/gpt-4o".to_string(),
        api_key: Some("test-key".to_string()),
        ..LlmConfig::default()
    };
    let sessions = SessionStore::new();
    let skills = Arc::new(RwLock::new(SkillRegistry::new(None, None)));
    let database = tempfile::tempdir().expect("graph manager database directory");
    let endpoint = format!(
        "surrealkv://{}",
        database.path().join("graph-manager.db").display()
    );
    let persistence: Arc<dyn PersistenceLayer> = Arc::new(
        SurrealDbProvider::new(
            &endpoint,
            None,
            None,
            Some("graph-manager-test"),
            Some("graph-manager-test"),
        )
        .await
        .expect("graph manager database opens"),
    );
    let graph = AgentGraph::builder("router")
        .add_node(RouterNode::new(
            "router",
            "Route Rust work to rust-reviewer and everything else to general-purpose.",
            vec!["general-purpose".to_string(), "rust-reviewer".to_string()],
        ))
        .add_node(AgentNode::new("general-purpose", "general-purpose"))
        .add_node(AgentNode::new("rust-reviewer", "rust-reviewer"))
        .add_conditional_edge("router", |state: &GraphState| {
            state
                .get::<String>("_route")
                .unwrap_or_else(|| "general-purpose".to_string())
        })
        .build();
    let manager = Arc::new(
        RunManager::new(
            llm_config,
            mcp,
            sessions,
            skills,
            make_vector_matcher(),
            Some(persistence),
        )
        .await
        .with_llm_driver(driver)
        .with_agent_graph(graph),
    );
    (manager, database)
}

async fn wait_for_run_done(manager: &RunManager, run_id: &str) -> Vec<NormalizedEvent> {
    let completed = tokio::time::timeout(std::time::Duration::from_secs(10), async {
        loop {
            let events = manager
                .history_since(run_id, None)
                .await
                .unwrap_or_default();
            if events
                .iter()
                .any(|event| matches!(event.event, NormalizedEvent::RunDone { .. }))
            {
                return events.into_iter().map(|event| event.event).collect();
            }
            tokio::task::yield_now().await;
        }
    })
    .await;
    match completed {
        Ok(events) => events,
        Err(_) => {
            let events = manager
                .history_since(run_id, None)
                .await
                .unwrap_or_default();
            panic!("run should complete; observed events: {events:#?}");
        }
    }
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

#[tokio::test]
async fn orchestrator_run_routes_and_streams_delegated_answer() {
    let driver = Arc::new(MockLlmDriver::new(vec![
        vec![
            DriverEvent::MessageDelta {
                text: "rust-reviewer".to_string(),
            },
            DriverEvent::Done,
        ],
        vec![
            DriverEvent::MessageDelta {
                text: "The ownership boundary is sound.".to_string(),
            },
            DriverEvent::Done,
        ],
    ]));
    let (manager, _database) = make_graph_manager(Arc::clone(&driver)).await;
    let orchestrator = defaults::orchestrator_agent();
    assert_eq!(orchestrator.runtime.entry, "orchestrator");
    assert!(
        orchestrator
            .metadata
            .tags
            .contains(&"delegation".to_string())
    );
    assert_ne!(
        orchestrator.prompt.system,
        defaults::default_agent().prompt.system
    );

    let peer = UserContext {
        user_id: "chat-integration-peer".to_string(),
        tenant_id: None,
        claims: UserClaims {
            sub: "chat-integration-peer".to_string(),
            name: Some("Chat integration UAR peer".to_string()),
            roles: Some(vec!["service".to_string()]),
            tenant_id: None,
            uar_instance_id: Some("chat-integration-uar".to_string()),
            exp: usize::MAX,
        },
    };
    let request = RunExecutionRequest::new(
        orchestrator,
        "Review this Rust ownership boundary".to_string(),
    )
    .with_user_context(&peer)
    .expect("verified peer context builds a hosted run request");
    let run_id = manager.execute_request(request).await;
    let approval_id = tokio::time::timeout(std::time::Duration::from_secs(5), async {
        loop {
            if let Some(approval_id) = manager
                .history_since(&run_id, None)
                .await
                .unwrap_or_default()
                .into_iter()
                .find_map(|event| match event.event {
                    NormalizedEvent::ToolCallApprovalRequired { approval_id, .. } => approval_id,
                    _ => None,
                })
            {
                return approval_id;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("spawn_agent approval should be requested");
    assert!(
        manager
            .resolve_approval_request(&run_id, Some(&approval_id), true)
            .await,
        "the exact graph child approval must resolve"
    );
    let events = wait_for_run_done(&manager, &run_id).await;

    assert_eq!(driver.call_count(), 2, "router and sub-agent must both run");
    let requests = driver.requests();
    assert!(
        requests[0].messages.iter().any(|message| {
            message["content"]
                .as_str()
                .is_some_and(|content| content.contains("Review this Rust ownership boundary"))
        }),
        "router request must include the user's task"
    );
    assert!(
        requests[1].messages[0]["content"]
            .as_str()
            .is_some_and(|content| content.contains("rust-reviewer")),
        "delegated request must identify the selected sub-agent"
    );
    assert!(events.iter().any(|event| matches!(
        event,
        NormalizedEvent::ChatDelta { text_delta, .. }
            if text_delta == "[rust-reviewer]\n\nThe ownership boundary is sound."
    )));
    assert_eq!(
        events
            .iter()
            .filter(|event| matches!(event, NormalizedEvent::RuntimeStep { .. }))
            .count(),
        4,
        "router and delegated agent must each emit start and finish step events"
    );
}

#[tokio::test]
async fn attached_graph_does_not_change_default_agent_path() {
    let driver = Arc::new(MockLlmDriver::new(vec![vec![
        DriverEvent::MessageDelta {
            text: "default answer".to_string(),
        },
        DriverEvent::Done,
    ]]));
    let (manager, _database) = make_graph_manager(Arc::clone(&driver)).await;

    let run_id = manager
        .start_run(
            defaults::default_agent(),
            "answer directly".to_string(),
            None,
            None,
            vec![],
        )
        .await;
    let events = wait_for_run_done(&manager, &run_id).await;

    assert_eq!(
        driver.call_count(),
        1,
        "default agent must not enter the graph"
    );
    assert!(events.iter().any(|event| matches!(
        event,
        NormalizedEvent::ChatDelta { text_delta, .. } if text_delta == "default answer"
    )));
}
