/// Integration tests for provider registration and model resolution.
///
/// Tests `ProviderRegistry::default_model()` and `RunManager::resolve_default_model()`.
use std::sync::Arc;
use tokio::sync::RwLock;
use universal_agent_runtime::config::LlmConfig;
use universal_agent_runtime::llm::{ProviderRegistry, registry::ProviderConfig};
use universal_agent_runtime::mcp::registry::McpRegistry;
use universal_agent_runtime::session::SessionStore;
use universal_agent_runtime::uar::runtime::manager::RunManager;
use universal_agent_runtime::uar::runtime::skills::SkillRegistry;

/// These tests only exercise provider/model resolution — nothing embeds. The
/// unconditionally-compiled `UnavailableEmbeddingBackend` therefore builds and
/// runs under every feature profile, whereas constructing a real backend would
/// panic without `local-models` (the `openai` fallback requires an API key).
fn make_vector_matcher() -> Arc<universal_agent_runtime::uar::runtime::matching::VectorMatcher> {
    let backend: Arc<dyn universal_agent_runtime::uar::rag::embeddings::EmbeddingBackend> =
        Arc::new(
            universal_agent_runtime::uar::rag::embeddings::UnavailableEmbeddingBackend::new(
                384,
                "embeddings are not exercised by these tests",
            ),
        );
    Arc::new(universal_agent_runtime::uar::runtime::matching::VectorMatcher::new(backend, 0.75))
}

async fn make_manager(model: &str) -> RunManager {
    let llm_config = LlmConfig {
        model: model.to_string(),
        api_key: Some("test-key".to_string()),
        ..LlmConfig::default()
    };
    let mcp = Arc::new(McpRegistry::new_empty());
    let sessions = SessionStore::new();
    let skills = Arc::new(RwLock::new(SkillRegistry::new(None, None)));
    let vm = make_vector_matcher();
    RunManager::new(llm_config, mcp, sessions, skills, vm, None).await
}

// ── ProviderRegistry::default_model ──────────────────────────────────────────

#[tokio::test]
async fn test_default_model_returns_none_when_no_default_set() {
    let registry = ProviderRegistry::new();
    assert!(
        registry.default_model().await.is_none(),
        "fresh registry should have no default model"
    );
}

#[tokio::test]
async fn test_default_model_roundtrip_via_register() {
    let registry = ProviderRegistry::new();

    let config = ProviderConfig {
        id: "openai".to_string(),
        display_name: "OpenAI".to_string(),
        api_key: Some("sk-test".to_string()),
        default_model: Some("gpt-4o".to_string()),
        enabled: true,
        base_url: String::new(),
        protocol: universal_agent_runtime::llm::registry::ProtocolSetting::Auto,
        models: vec![],
    };

    registry
        .register(config)
        .await
        .expect("register should succeed");
    registry
        .set_default("openai")
        .await
        .expect("set_default should succeed");

    let result = registry.default_model().await;
    assert_eq!(
        result,
        Some(("openai".to_string(), "gpt-4o".to_string())),
        "default_model should return registered provider/model"
    );
}

#[tokio::test]
async fn test_default_model_returns_none_when_provider_has_no_default_model() {
    let registry = ProviderRegistry::new();

    let config = ProviderConfig {
        id: "groq".to_string(),
        display_name: "Groq".to_string(),
        api_key: Some("gsk_test".to_string()),
        default_model: None, // intentionally no default model
        enabled: true,
        base_url: String::new(),
        protocol: universal_agent_runtime::llm::registry::ProtocolSetting::Auto,
        models: vec![],
    };

    registry
        .register(config)
        .await
        .expect("register should succeed");
    registry
        .set_default("groq")
        .await
        .expect("set_default should succeed");

    let result = registry.default_model().await;
    assert!(
        result.is_none(),
        "provider without default_model should return None"
    );
}

// ── RunManager::resolve_default_model ────────────────────────────────────────

#[tokio::test]
async fn test_run_manager_resolve_from_llm_config_provider_slash_model() {
    let manager = make_manager("anthropic/claude-sonnet-4-6").await;
    let resolved = manager.resolve_default_model().await;
    assert_eq!(
        resolved,
        Some(("anthropic".to_string(), "claude-sonnet-4-6".to_string())),
        "should split provider/model on first slash"
    );
}

#[tokio::test]
async fn test_run_manager_resolve_from_llm_config_model_only() {
    let manager = make_manager("gpt-4o").await;
    let resolved = manager.resolve_default_model().await;
    assert_eq!(
        resolved,
        Some(("default".to_string(), "gpt-4o".to_string())),
        "model without slash should use 'default' as provider"
    );
}

#[tokio::test]
async fn test_run_manager_resolve_returns_none_when_empty() {
    let llm_config = LlmConfig {
        model: "".to_string(),
        api_key: None,
        ..LlmConfig::default()
    };
    let mcp = Arc::new(McpRegistry::new_empty());
    let sessions = SessionStore::new();
    let skills = Arc::new(RwLock::new(SkillRegistry::new(None, None)));
    let vm = make_vector_matcher();
    let manager = RunManager::new(llm_config, mcp, sessions, skills, vm, None).await;

    let resolved = manager.resolve_default_model().await;
    assert!(
        resolved.is_none(),
        "empty model string should resolve to None"
    );
}

#[tokio::test]
async fn test_run_manager_prefers_registry_over_llm_config() {
    let manager = make_manager("openai/gpt-3.5-turbo").await;

    // Registry has a different default
    let registry = Arc::new(ProviderRegistry::new());
    let config = ProviderConfig {
        id: "anthropic".to_string(),
        display_name: "Anthropic".to_string(),
        api_key: Some("sk-ant-test".to_string()),
        default_model: Some("claude-opus-4-6".to_string()),
        enabled: true,
        base_url: String::new(),
        protocol: universal_agent_runtime::llm::registry::ProtocolSetting::Auto,
        models: vec![],
    };
    registry
        .register(config)
        .await
        .expect("register should succeed");
    registry
        .set_default("anthropic")
        .await
        .expect("set_default should succeed");

    let manager = manager.with_provider_registry(registry);

    let resolved = manager.resolve_default_model().await;
    assert_eq!(
        resolved,
        Some(("anthropic".to_string(), "claude-opus-4-6".to_string())),
        "provider registry should take precedence over llm_config"
    );
}
