//! A real LLM driver for integration tests, backed by the local Ollama install.
//!
//! # Why this exists
//!
//! `EmbeddedRuntime::build()` requires an LLM driver (`E_EMBEDDED_LOCAL_DRIVER_
//! REQUIRED`). Tests that only care about *skills* previously satisfied that
//! with `MockLlmDriver`, which proves the builder accepts a driver but proves
//! nothing about whether the runtime can actually talk to a model.
//!
//! Ollama gives us a real driver with no cloud dependency, no API key, and no
//! per-call cost — so an integration test can exercise the genuine code path
//! (`LiterLlmDriver` → OpenAI-compatible REST) instead of a stub.
//!
//! # Why Ollama specifically
//!
//! It serves an **OpenAI-compatible** API at `/v1` (verified: `GET /v1/models`
//! returns 200), which is the protocol `LiterLlmDriver` already speaks. No new
//! driver was needed — Ollama registers as a custom provider with a `base_url`,
//! exactly as `LlmRegistry::register_custom_provider` documents for "local
//! Ollama instances".
//!
//! # Skipping
//!
//! Ollama is a developer convenience, not a build requirement. When it is not
//! running, helpers here return `None` and the calling test **skips loudly** —
//! never silently, because a silent skip lets coverage rot unnoticed.

#![allow(dead_code)] // Not every test binary uses every helper.

use std::sync::Arc;

use universal_agent_runtime::llm::LlmDriver;
use universal_agent_runtime::llm::liter_driver::LiterLlmDriver;
use universal_agent_runtime::llm::registry::{ModelConfig, ProtocolSetting, ProviderConfig};

/// Where Ollama's OpenAI-compatible API lives.
///
/// Overridable so CI can point at a shared instance without editing tests.
pub fn base_url() -> String {
    std::env::var("OLLAMA_BASE_URL").unwrap_or_else(|_| "http://127.0.0.1:11434/v1".to_string())
}

/// The model tests use.
///
/// `llama3.2:1b` by default — the smallest local model, chosen so a test that
/// does run a completion finishes in seconds rather than minutes. Tests that
/// only need a *constructible* driver never call it at all.
pub fn model() -> String {
    std::env::var("OLLAMA_TEST_MODEL").unwrap_or_else(|_| "llama3.2:1b".to_string())
}

/// Is Ollama reachable right now?
///
/// A cheap, bounded probe — 2s so a missing service costs a moment, not a
/// hung test run.
pub async fn is_available() -> bool {
    let url = format!("{}/models", base_url());
    match reqwest::Client::new()
        .get(&url)
        .timeout(std::time::Duration::from_secs(2))
        .send()
        .await
    {
        Ok(resp) => resp.status().is_success(),
        Err(_) => false,
    }
}

/// A driver pointed at the local Ollama, or `None` when it is not running.
///
/// # Errors
///
/// Returns `None` rather than erroring: an absent Ollama is a skip condition,
/// not a failure. Construction problems that are *not* about availability still
/// panic, because those indicate a genuine bug in how we build the driver.
pub fn driver() -> Option<Arc<dyn LlmDriver>> {
    let llm = universal_agent_runtime::config::LlmConfig {
        // Ollama ignores the key, but the client requires a non-empty string.
        api_key: Some("ollama-local".to_string()),
        base_url: Some(base_url()),
        model: model(),
        ..Default::default()
    };

    let config = universal_agent_runtime::config::build_client_config(&llm);
    let driver = LiterLlmDriver::new(config, model(), Some(false))
        .expect("building a LiterLlmDriver against a local base_url must not fail");

    Some(Arc::new(driver) as Arc<dyn LlmDriver>)
}

/// A `ProviderConfig` describing the local Ollama instance.
///
/// Matches what `EmbeddedRuntimeBuilder::local_provider` expects.
pub fn provider() -> ProviderConfig {
    let m = model();
    ProviderConfig {
        id: "ollama-local".to_string(),
        display_name: "Local Ollama".to_string(),
        base_url: base_url(),
        api_key: None,
        protocol: ProtocolSetting::Auto,
        default_model: Some(m.clone()),
        models: vec![ModelConfig {
            id: m.clone(),
            display_name: Some(m),
            context_window: Some(8_192),
            supports_vision: false,
            supports_tools: true,
            supports_reasoning: false,
            supports_structured_output: true,
            supports_streaming: true,
            max_output_tokens: Some(2_048),
            enabled: true,
        }],
        enabled: true,
    }
}

/// Print the canonical skip message.
///
/// Kept in one place so every skip reads the same and names the fix.
pub fn skip_notice(test_name: &str) {
    eprintln!(
        "SKIPPED {test_name}: Ollama is not reachable at {}.\n\
         Start it with `ollama serve` and pull the test model:\n\
         \x20   ollama pull {}\n\
         This test exercises a REAL LLM driver; it must not be reported as \
         passing when it never ran.",
        base_url(),
        model()
    );
}
