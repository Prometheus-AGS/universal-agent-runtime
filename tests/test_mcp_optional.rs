/// Tests that verify the server and orchestrator function correctly without
/// an `mcp.json` file (MCP is purely optional).
use std::sync::Arc;
use tokio::sync::RwLock;
use universal_agent_runtime::config::LlmConfig;
use universal_agent_runtime::mcp::registry::McpRegistry;
use universal_agent_runtime::session::SessionStore;
use universal_agent_runtime::uar::runtime::manager::RunManager;
use universal_agent_runtime::uar::runtime::skills::SkillRegistry;

/// Matches the value this test used before `VectorMatcher::new` changed shape.
/// Nothing here is threshold-sensitive — no matching is ever performed.
const VECTOR_THRESHOLD: f32 = 0.75;

/// Dimension reported by the placeholder backend. Never used to produce a
/// vector: the backend errors on `embed` by design.
const EMBEDDING_DIMENSION: usize = 384;

/// These tests only assert that `RunManager` *constructs* without MCP — none of
/// them embeds anything. `UnavailableEmbeddingBackend` is therefore the correct
/// backend rather than a stand-in: it is compiled unconditionally (no feature
/// gate), so this test builds under every profile, and it mirrors the fallback
/// `server.rs` installs when a real backend cannot be built.
fn make_vector_matcher() -> Arc<universal_agent_runtime::uar::runtime::matching::VectorMatcher> {
    let backend = Arc::new(
        universal_agent_runtime::uar::rag::embeddings::UnavailableEmbeddingBackend::new(
            EMBEDDING_DIMENSION,
            "embeddings are not exercised by the MCP-optional tests",
        ),
    );
    Arc::new(
        universal_agent_runtime::uar::runtime::matching::VectorMatcher::new(
            backend,
            VECTOR_THRESHOLD,
        ),
    )
}

#[tokio::test]
async fn test_run_manager_constructs_without_mcp_registry_entries() {
    // McpRegistry::new_empty() simulates no mcp.json configuration.
    let mcp = Arc::new(McpRegistry::new_empty());
    let llm_config = LlmConfig {
        model: "openai/gpt-4o".to_string(),
        api_key: Some("test-key".to_string()),
        ..LlmConfig::default()
    };
    let sessions = SessionStore::new();
    let skills = Arc::new(RwLock::new(SkillRegistry::new(None, None)));
    let vm = make_vector_matcher();

    // Should not panic or error
    let manager = RunManager::new(llm_config, mcp, sessions, skills, vm, None).await;

    // Registry and model resolution still work
    let resolved = manager.resolve_default_model().await;
    assert!(resolved.is_some(), "model should resolve even without MCP");
}

#[tokio::test]
async fn test_mcp_registry_new_empty_has_no_tools() {
    let mcp = McpRegistry::new_empty();
    let tools = mcp.tools();
    assert!(
        tools.is_empty(),
        "empty MCP registry should report no tools"
    );
}

#[tokio::test]
async fn test_mcp_registry_survives_concurrent_reads() {
    // Verify the empty registry is Send + Sync and handles concurrent access.
    let mcp = Arc::new(McpRegistry::new_empty());

    let handles: Vec<_> = (0..10)
        .map(|_| {
            let mcp = Arc::clone(&mcp);
            tokio::spawn(async move {
                let tools = mcp.tools();
                assert!(tools.is_empty());
            })
        })
        .collect();

    for h in handles {
        h.await.expect("concurrent read should not panic");
    }
}

#[tokio::test]
async fn test_skill_registry_works_without_filesystem_provider() {
    // Skills are also optional — verify registry works with no providers.
    let registry = SkillRegistry::new(None, None);
    let all = registry.list();
    assert!(all.is_empty(), "new registry should have no skills");
}
