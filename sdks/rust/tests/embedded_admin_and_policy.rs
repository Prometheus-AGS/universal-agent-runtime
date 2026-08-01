//! Tests for the embedded run-policy resolution + admin surface
//! (change `embedded-run-policy-and-admin-surface`).
//!
//! These exercise the transport-free policy core and the persistence-backed
//! settings + agent stores through the runtime crate directly (the SDK dev-dep
//! enables `in-memory-backend`), so they run with no HTTP service and no real
//! database.
#![cfg(feature = "embedded")]

use std::sync::Arc;

use universal_agent_runtime::uar::context::ContextStrategy;
use universal_agent_runtime::uar::domain::agent_store;
use universal_agent_runtime::uar::domain::artifact::AgentArtifact;
use universal_agent_runtime::uar::domain::policy::{
    resolve_effective_run_policy_core, ModelRoute, PolicyResolutionContext, PolicyUniverse,
    RunPolicy,
};
use universal_agent_runtime::uar::persistence::providers::memory::InMemoryProvider;
use universal_agent_runtime::uar::persistence::PersistenceLayer;
use universal_agent_runtime::uar::settings::manager::SettingsManager;

fn persistence() -> Arc<dyn PersistenceLayer> {
    Arc::new(InMemoryProvider::new())
}

/// A SettingsManager on in-memory persistence with the `run_policy` namespace
/// seeded (so `run_policy.global` can be read/written).
async fn settings() -> (Arc<dyn PersistenceLayer>, SettingsManager) {
    let persistence = persistence();
    let manager = SettingsManager::new(Arc::clone(&persistence));
    manager
        .ensure_run_policy_seed()
        .await
        .expect("seed run_policy namespace");
    (persistence, manager)
}

/// A valid agent (from the runtime's own `default_agent()`) re-identified by
/// `id`. Its provider/model default is already empty, so it contributes no model
/// scope and the Global (or conversation) scope decides.
fn agent(id: &str) -> AgentArtifact {
    let mut agent = universal_agent_runtime::uar::defaults::default_agent();
    agent.id = id.to_string();
    agent
}

fn model(provider: &str, model: &str) -> ModelRoute {
    ModelRoute {
        provider_id: provider.into(),
        model_id: model.into(),
    }
}

fn ctx<'a>(manager: Option<&'a SettingsManager>) -> PolicyResolutionContext<'a> {
    PolicyResolutionContext {
        settings_manager: manager,
        universe: PolicyUniverse::default(),
        default_context_strategy: ContextStrategy::Auto,
    }
}

// 4.1 — Global default model is applied on the embedded runtime.
#[tokio::test]
async fn global_default_model_is_applied_when_agent_and_conversation_set_none() {
    let (_p, manager) = settings().await;
    manager
        .set_value(
            "run_policy.global",
            serde_json::json!({ "model": { "provider_id": "openai", "model_id": "gpt-global" } }),
        )
        .await
        .expect("write run_policy.global");

    let effective =
        resolve_effective_run_policy_core(ctx(Some(&manager)), &agent("a"), None, None).await;

    assert_eq!(effective.model, Some(model("openai", "gpt-global")));
}

// 4.2 — Conversation overrides the global default (precedence conv > global).
#[tokio::test]
async fn conversation_overrides_global_default() {
    let (_p, manager) = settings().await;
    manager
        .set_value(
            "run_policy.global",
            serde_json::json!({ "model": { "provider_id": "openai", "model_id": "gpt-global" } }),
        )
        .await
        .expect("write run_policy.global");

    let conversation = RunPolicy {
        model: Some(model("anthropic", "claude-conv")),
        ..RunPolicy::default()
    };
    let effective =
        resolve_effective_run_policy_core(ctx(Some(&manager)), &agent("a"), Some(conversation), None)
            .await;

    assert_eq!(effective.model, Some(model("anthropic", "claude-conv")));
}

// 4.3 — No settings manager falls back to agent+conversation without error.
#[tokio::test]
async fn missing_settings_manager_falls_back_without_error() {
    let conversation = RunPolicy {
        model: Some(model("groq", "llama-conv")),
        ..RunPolicy::default()
    };
    let effective =
        resolve_effective_run_policy_core(ctx(None), &agent("a"), Some(conversation), None).await;

    // No global scope available; conversation still applies, no panic/error.
    assert_eq!(effective.model, Some(model("groq", "llama-conv")));
}

// 4.4 — Parity: two calls with identical inputs yield an identical policy (the
// core is the single shared resolver for service + embedded paths).
#[tokio::test]
async fn identical_inputs_yield_identical_effective_policy() {
    let (_p, manager) = settings().await;
    manager
        .set_value(
            "run_policy.global",
            serde_json::json!({ "model": { "provider_id": "openai", "model_id": "gpt-global" } }),
        )
        .await
        .expect("write run_policy.global");

    let conversation = RunPolicy {
        agent_id: Some("orchestrator-agent".into()),
        ..RunPolicy::default()
    };
    let a = resolve_effective_run_policy_core(
        ctx(Some(&manager)),
        &agent("a"),
        Some(conversation.clone()),
        None,
    )
    .await;
    let b = resolve_effective_run_policy_core(
        ctx(Some(&manager)),
        &agent("a"),
        Some(conversation),
        None,
    )
    .await;

    assert_eq!(a.model, b.model);
    assert_eq!(a.agent_id, b.agent_id);
    assert_eq!(a.chat_mode, b.chat_mode);
}

// 4.5 — Settings admin round-trips run_policy.global.
#[tokio::test]
async fn settings_admin_round_trips_run_policy_global() {
    let (_p, manager) = settings().await;
    let value =
        serde_json::json!({ "model": { "provider_id": "mistral", "model_id": "large" } });
    manager
        .set_value("run_policy.global", value.clone())
        .await
        .expect("set_value");

    let read = manager
        .get_value("run_policy.global")
        .await
        .expect("global should be present");
    assert_eq!(
        read.get("model").and_then(|m| m.get("model_id")),
        Some(&serde_json::json!("large"))
    );

    // The registered type is discoverable (settings_snapshot's `types` source).
    let types = manager.list_types().await.expect("list_types");
    assert!(types.iter().any(|t| t.key.starts_with("run_policy")));
}

// 4.6 — Agent CRUD round-trips against in-process persistence.
#[tokio::test]
async fn agent_crud_round_trips_against_in_memory_persistence() {
    let persistence = persistence();
    let store = persistence.as_ref();

    agent_store::upsert_agent(store, &agent("office-assistant"))
        .await
        .expect("upsert");

    let listed = agent_store::list_agents(store).await.expect("list");
    assert!(listed.iter().any(|a| a.id == "office-assistant"));

    let got = agent_store::get_agent(store, "office-assistant")
        .await
        .expect("get");
    assert!(got.is_some());

    agent_store::delete_agent(store, "office-assistant")
        .await
        .expect("delete");
    let gone = agent_store::get_agent(store, "office-assistant")
        .await
        .expect("get after delete");
    assert!(gone.is_none());
}
