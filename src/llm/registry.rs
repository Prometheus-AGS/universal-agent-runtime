//! Provider registry for multi-provider LLM management.
//!
//! This module provides a [`ProviderRegistry`] that maps provider IDs to their
//! connection configurations, enabling per-agent provider selection with
//! fallback chains.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;

use super::{LlmProtocol, LlmSettings, Provider};

// =============================================================================
// PROVIDER CONFIGURATION TYPES
// =============================================================================

/// Configuration for a single LLM provider.
///
/// A provider represents an API endpoint (e.g., OpenAI, Groq, Azure)
/// that can serve one or more models.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Unique identifier (e.g., "openai", "groq-fast", "azure-prod").
    pub id: String,
    /// Human-friendly display name.
    #[serde(default)]
    pub display_name: String,
    /// Base URL for the API (e.g., `https://api.openai.com`).
    pub base_url: String,
    /// Optional API key for authentication.
    #[serde(default)]
    pub api_key: Option<String>,
    /// Protocol to use (`auto`, `chat`, `responses`).
    #[serde(default)]
    pub protocol: ProtocolSetting,
    /// Default model to use when none is specified.
    #[serde(default)]
    pub default_model: Option<String>,
    /// Available models for this provider.
    #[serde(default)]
    pub models: Vec<ModelConfig>,
    /// Whether this provider is enabled.
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_enabled() -> bool {
    true
}

/// Protocol setting for config deserialization.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProtocolSetting {
    #[default]
    Auto,
    Chat,
    Responses,
}

impl From<ProtocolSetting> for LlmProtocol {
    fn from(s: ProtocolSetting) -> Self {
        match s {
            ProtocolSetting::Auto => LlmProtocol::Auto,
            ProtocolSetting::Chat => LlmProtocol::Chat,
            ProtocolSetting::Responses => LlmProtocol::Responses,
        }
    }
}

impl From<LlmProtocol> for ProtocolSetting {
    fn from(p: LlmProtocol) -> Self {
        match p {
            LlmProtocol::Auto => ProtocolSetting::Auto,
            LlmProtocol::Chat => ProtocolSetting::Chat,
            LlmProtocol::Responses => ProtocolSetting::Responses,
        }
    }
}

/// Configuration for a single model within a provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// Model identifier (e.g., "gpt-4o", "llama-3.3-70b-versatile").
    pub id: String,
    /// Human-friendly display name.
    #[serde(default)]
    pub display_name: Option<String>,
    /// Maximum context window in tokens.
    #[serde(default)]
    pub context_window: Option<u32>,
    /// Whether this model supports vision/image inputs.
    #[serde(default)]
    pub supports_vision: bool,
    /// Whether this model supports tool/function calling.
    #[serde(default = "default_supports_tools")]
    pub supports_tools: bool,
    /// Maximum output tokens.
    #[serde(default)]
    pub max_output_tokens: Option<u32>,
}

fn default_supports_tools() -> bool {
    true
}

// =============================================================================
// PROVIDER REGISTRY
// =============================================================================

/// Registry that maps provider IDs to their connection configurations.
///
/// The registry supports:
/// - Multiple named providers (e.g., "openai", "groq", "azure-prod")
/// - A designated default provider
/// - Resolution of `ProviderSelection` → `LlmSettings`
/// - Fallback chains via `ProviderPolicy`
#[derive(Debug)]
pub struct ProviderRegistry {
    providers: RwLock<HashMap<String, ProviderConfig>>,
    default_id: RwLock<Option<String>>,
}

impl ProviderRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            providers: RwLock::new(HashMap::new()),
            default_id: RwLock::new(None),
        }
    }

    /// Seed the registry with the global `LlmSettings` as the "default" provider.
    ///
    /// This maintains backward compatibility — env-var-based config is
    /// automatically available as a named provider.
    pub async fn seed_from_settings(&self, settings: &LlmSettings) {
        let provider_id = detect_provider_id(&settings.base_url);

        let config = ProviderConfig {
            id: provider_id.clone(),
            display_name: format!("{} (default)", provider_id),
            base_url: settings.base_url.clone(),
            api_key: settings.api_key.clone(),
            protocol: settings.protocol.into(),
            default_model: Some(settings.model.clone()),
            models: vec![ModelConfig {
                id: settings.model.clone(),
                display_name: None,
                context_window: None,
                supports_vision: Provider::supports_vision(&settings.model),
                supports_tools: true,
                max_output_tokens: None,
            }],
            enabled: true,
        };

        let mut providers = self.providers.write().await;
        providers.insert(provider_id.clone(), config);
        drop(providers);

        let mut default = self.default_id.write().await;
        *default = Some(provider_id);
    }

    /// Seed providers from config-file definitions.
    pub async fn seed_from_configs(&self, configs: Vec<ProviderConfig>) {
        let mut providers = self.providers.write().await;
        for config in configs {
            tracing::info!(
                provider_id = %config.id,
                base_url = %config.base_url,
                models = config.models.len(),
                "Registered provider from config"
            );
            providers.insert(config.id.clone(), config);
        }
    }

    /// Register a new provider.
    pub async fn register(&self, config: ProviderConfig) -> anyhow::Result<()> {
        let mut providers = self.providers.write().await;
        tracing::info!(provider_id = %config.id, "Registering provider");
        providers.insert(config.id.clone(), config);
        Ok(())
    }

    /// Get a provider by ID.
    pub async fn get(&self, id: &str) -> Option<ProviderConfig> {
        let providers = self.providers.read().await;
        providers.get(id).cloned()
    }

    /// List all providers.
    pub async fn list(&self) -> Vec<ProviderConfig> {
        let providers = self.providers.read().await;
        providers.values().cloned().collect()
    }

    /// Remove a provider by ID.
    pub async fn remove(&self, id: &str) -> anyhow::Result<()> {
        let mut providers = self.providers.write().await;
        providers.remove(id);

        // Clear default if it was the removed provider
        let mut default = self.default_id.write().await;
        if default.as_deref() == Some(id) {
            *default = None;
        }

        Ok(())
    }

    /// Update an existing provider.
    pub async fn update(&self, config: ProviderConfig) -> anyhow::Result<()> {
        let mut providers = self.providers.write().await;
        if !providers.contains_key(&config.id) {
            anyhow::bail!("Provider '{}' not found", config.id);
        }
        providers.insert(config.id.clone(), config);
        Ok(())
    }

    /// Get the default provider ID.
    pub async fn default_id(&self) -> Option<String> {
        self.default_id.read().await.clone()
    }

    /// Set the default provider.
    pub async fn set_default(&self, id: &str) -> anyhow::Result<()> {
        let providers = self.providers.read().await;
        if !providers.contains_key(id) {
            anyhow::bail!("Provider '{}' not found", id);
        }
        drop(providers);

        let mut default = self.default_id.write().await;
        *default = Some(id.to_string());
        Ok(())
    }

    /// Resolve a provider selection to `LlmSettings`.
    ///
    /// Looks up the provider by `selection.provider`, then uses
    /// `selection.model` (or the provider's default model).
    pub async fn resolve(&self, provider_id: &str, model: &str) -> Option<LlmSettings> {
        let providers = self.providers.read().await;
        let config = providers.get(provider_id)?;

        if !config.enabled {
            tracing::debug!(provider_id, "Provider is disabled, skipping");
            return None;
        }

        let resolved_model = if model.is_empty() {
            config
                .default_model
                .clone()
                .unwrap_or_else(|| model.to_string())
        } else {
            model.to_string()
        };

        let provider = Provider::detect_from_url(&config.base_url);

        Some(LlmSettings {
            base_url: config.base_url.clone(),
            api_key: config.api_key.clone(),
            model: resolved_model,
            protocol: config.protocol.clone().into(),
            provider,
            parallel_tool_calls: None,
            deployment_name: None,
            api_version: None,
        })
    }

    /// Resolve using the agent's provider policy.
    ///
    /// Tries the default provider selection first, then iterates through
    /// fallbacks. Returns `None` if no provider can be resolved.
    pub async fn resolve_from_policy(
        &self,
        policy: &crate::uar::domain::artifact::ProviderPolicy,
    ) -> Option<LlmSettings> {
        // Try primary provider
        if let Some(settings) = self
            .resolve(&policy.default.provider, &policy.default.model)
            .await
        {
            tracing::debug!(
                provider = %policy.default.provider,
                model = %policy.default.model,
                "Resolved primary provider"
            );
            return Some(settings);
        }

        // Try fallbacks in order
        for (i, fallback) in policy.fallbacks.iter().enumerate() {
            if let Some(settings) = self.resolve(&fallback.provider, &fallback.model).await {
                tracing::info!(
                    provider = %fallback.provider,
                    model = %fallback.model,
                    fallback_index = i,
                    "Resolved fallback provider"
                );
                return Some(settings);
            }
        }

        tracing::warn!(
            primary = %policy.default.provider,
            fallbacks = policy.fallbacks.len(),
            "No provider could be resolved from policy"
        );
        None
    }

    /// Get the list of models for a specific provider.
    pub async fn models(&self, provider_id: &str) -> Option<Vec<ModelConfig>> {
        let providers = self.providers.read().await;
        providers.get(provider_id).map(|p| p.models.clone())
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// HELPERS
// =============================================================================

/// Detect a short provider ID from a base URL.
fn detect_provider_id(base_url: &str) -> String {
    let lower = base_url.to_lowercase();

    if lower.contains("openai.com") && !lower.contains("azure") {
        "openai".to_string()
    } else if lower.contains("azure.com") || lower.contains("openai.azure.com") {
        "azure".to_string()
    } else if lower.contains("openrouter.ai") {
        "openrouter".to_string()
    } else if lower.contains("together.ai") || lower.contains("together.xyz") {
        "together".to_string()
    } else if lower.contains("groq.com") {
        "groq".to_string()
    } else if lower.contains("anthropic.com") {
        "anthropic".to_string()
    } else if lower.contains("googleapis.com") || lower.contains("generativelanguage") {
        "google".to_string()
    } else {
        "default".to_string()
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_config(id: &str, base_url: &str) -> ProviderConfig {
        ProviderConfig {
            id: id.to_string(),
            display_name: id.to_string(),
            base_url: base_url.to_string(),
            api_key: Some("test-key".to_string()),
            protocol: ProtocolSetting::Auto,
            default_model: Some("test-model".to_string()),
            models: vec![],
            enabled: true,
        }
    }

    #[tokio::test]
    async fn test_register_and_get() {
        let registry = ProviderRegistry::new();
        let config = make_test_config("openai", "https://api.openai.com");
        registry.register(config).await.unwrap();

        let result = registry.get("openai").await;
        assert!(result.is_some());
        assert_eq!(result.unwrap().base_url, "https://api.openai.com");
    }

    #[tokio::test]
    async fn test_resolve() {
        let registry = ProviderRegistry::new();
        let config = make_test_config("groq", "https://api.groq.com/openai");
        registry.register(config).await.unwrap();

        let settings = registry.resolve("groq", "llama-3.3-70b").await;
        assert!(settings.is_some());
        let s = settings.unwrap();
        assert_eq!(s.model, "llama-3.3-70b");
        assert_eq!(s.base_url, "https://api.groq.com/openai");
    }

    #[tokio::test]
    async fn test_resolve_uses_default_model() {
        let registry = ProviderRegistry::new();
        let config = make_test_config("openai", "https://api.openai.com");
        registry.register(config).await.unwrap();

        let settings = registry.resolve("openai", "").await;
        assert!(settings.is_some());
        assert_eq!(settings.unwrap().model, "test-model");
    }

    #[tokio::test]
    async fn test_resolve_disabled_provider() {
        let registry = ProviderRegistry::new();
        let mut config = make_test_config("disabled", "https://example.com");
        config.enabled = false;
        registry.register(config).await.unwrap();

        let settings = registry.resolve("disabled", "model").await;
        assert!(settings.is_none());
    }

    #[tokio::test]
    async fn test_resolve_unknown_provider() {
        let registry = ProviderRegistry::new();
        let settings = registry.resolve("nonexistent", "model").await;
        assert!(settings.is_none());
    }

    #[tokio::test]
    async fn test_seed_from_settings() {
        let settings = LlmSettings {
            base_url: "https://api.openai.com".to_string(),
            api_key: Some("sk-test".to_string()),
            model: "gpt-4o".to_string(),
            protocol: LlmProtocol::Auto,
            provider: Provider::OpenAI,
            parallel_tool_calls: None,
            deployment_name: None,
            api_version: None,
        };

        let registry = ProviderRegistry::new();
        registry.seed_from_settings(&settings).await;

        let default_id = registry.default_id().await;
        assert_eq!(default_id, Some("openai".to_string()));

        let resolved = registry.resolve("openai", "gpt-4o").await;
        assert!(resolved.is_some());
    }

    #[tokio::test]
    async fn test_list_and_remove() {
        let registry = ProviderRegistry::new();
        registry
            .register(make_test_config("a", "https://a.com"))
            .await
            .unwrap();
        registry
            .register(make_test_config("b", "https://b.com"))
            .await
            .unwrap();

        assert_eq!(registry.list().await.len(), 2);

        registry.remove("a").await.unwrap();
        assert_eq!(registry.list().await.len(), 1);
        assert!(registry.get("a").await.is_none());
    }

    #[test]
    fn test_detect_provider_id() {
        assert_eq!(detect_provider_id("https://api.openai.com"), "openai");
        assert_eq!(
            detect_provider_id("https://my-resource.openai.azure.com"),
            "azure"
        );
        assert_eq!(detect_provider_id("https://openrouter.ai"), "openrouter");
        assert_eq!(detect_provider_id("https://api.groq.com"), "groq");
        assert_eq!(detect_provider_id("https://api.together.ai"), "together");
        assert_eq!(detect_provider_id("https://custom.llm.dev"), "default");
    }
}
