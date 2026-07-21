//! Provider registry for multi-provider LLM management.
//!
//! This module provides a [`ProviderRegistry`] that maps provider IDs to their
//! connection configurations, enabling per-agent provider selection with
//! fallback chains. It integrates with `LlmConfig` for the liter-llm client
//! and enriches provider records with data from the compile-time [`ModelCatalog`].

use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;

use super::catalog::{ModelCatalog, ProviderInfo};
use crate::config::LlmConfig;

/// JSON `null` or absent field → empty string (admin UI may send `base_url: null` when the
/// catalog has no default URL and the user leaves the override blank).
fn deserialize_string_default<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<String>::deserialize(deserializer).map(|o| o.unwrap_or_default())
}

// =============================================================================
// PROVIDER CONFIGURATION TYPES
// =============================================================================

/// Configuration for a single LLM provider.
///
/// A provider represents an API endpoint (e.g., OpenAI, Groq, Azure)
/// that can serve one or more models.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ProviderConfig {
    /// Unique identifier (e.g., "openai", "groq-fast", "azure-prod").
    pub id: String,
    /// Human-friendly display name.
    #[serde(default, deserialize_with = "deserialize_string_default")]
    pub display_name: String,
    /// Base URL for the API (e.g., `https://api.openai.com`).
    ///
    /// May be omitted in API requests; [`enrich_provider_config`] fills this from the
    /// embedded catalog or a built-in fallback per provider id.
    #[serde(default, deserialize_with = "deserialize_string_default")]
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
#[derive(Debug, Clone, Default, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ProtocolSetting {
    #[default]
    Auto,
    Chat,
    Responses,
}

/// Configuration for a single model within a provider.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
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
    /// Whether this model exposes a distinct reasoning/thinking capability.
    #[serde(default)]
    pub supports_reasoning: bool,
    /// Whether this model/runtime can enforce structured JSON output.
    #[serde(default)]
    pub supports_structured_output: bool,
    /// Whether this model/runtime can emit incremental response chunks.
    #[serde(default = "default_supports_streaming")]
    pub supports_streaming: bool,
    /// Maximum output tokens.
    #[serde(default)]
    pub max_output_tokens: Option<u32>,
    /// Whether UAR may route runs to this model.
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_supports_tools() -> bool {
    true
}

fn default_supports_streaming() -> bool {
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
/// - Resolution of provider/model → `LlmConfig`
/// - Fallback chains via `ProviderPolicy`
#[derive(Debug)]
pub struct ProviderRegistry {
    providers: RwLock<HashMap<String, ProviderConfig>>,
    default_id: RwLock<Option<String>>,
    /// Per-provider health/cooldown tracking, shared with `ModelRouter` and
    /// every `Orchestrator` (CH-03).
    health: std::sync::Arc<super::health::ProviderHealthMonitor>,
}

impl ProviderRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            providers: RwLock::new(HashMap::new()),
            default_id: RwLock::new(None),
            health: std::sync::Arc::new(super::health::ProviderHealthMonitor::new()),
        }
    }

    /// Shared provider-health monitor (CH-03): consulted by `resolve_to_llm_config`
    /// below and by `ModelRouter::route`, and updated by `Orchestrator` on every
    /// driver success/failure.
    #[must_use]
    pub fn health(&self) -> &std::sync::Arc<super::health::ProviderHealthMonitor> {
        &self.health
    }

    /// Seed the registry from the global `LlmConfig`.
    ///
    /// Extracts the provider from the `model` field's `provider/model` format,
    /// enriches the entry with catalog metadata (display name, base URL, model
    /// list), and sets it as the default provider.
    pub async fn seed_from_llm_config(&self, config: &LlmConfig) {
        let (provider_id, model_id) = split_model_string(&config.model);
        let catalog = ModelCatalog::global();
        let catalog_provider = catalog.provider(&provider_id);

        // Store only the operator-configured model in `models`, not the full
        // catalog. The full catalog is available via `/api/models` (browse path).
        // This ensures the admin UI "configured models" section reflects reality.
        let catalog_model = catalog_provider.and_then(|p| {
            p.models
                .iter()
                .find(|m| m.id == model_id)
                .map(|m| ModelConfig {
                    id: m.id.clone(),
                    display_name: if m.name.is_empty() {
                        None
                    } else {
                        Some(m.name.clone())
                    },
                    context_window: if m.limits.context_window > 0 {
                        Some(u32::try_from(m.limits.context_window).unwrap_or(u32::MAX))
                    } else {
                        None
                    },
                    supports_vision: m.modalities.input.iter().any(|s| s == "image"),
                    supports_tools: m.capabilities.tool_call,
                    supports_reasoning: m.capabilities.reasoning,
                    supports_structured_output: m.capabilities.structured_output,
                    supports_streaming: m.capabilities.streaming,
                    max_output_tokens: if m.limits.max_output > 0 {
                        Some(u32::try_from(m.limits.max_output).unwrap_or(u32::MAX))
                    } else {
                        None
                    },
                    enabled: true,
                })
        });
        let models = if let Some(m) = catalog_model {
            vec![m]
        } else {
            // Model not in catalog (custom/local) — store a minimal entry.
            vec![ModelConfig {
                id: model_id.clone(),
                display_name: None,
                context_window: None,
                supports_vision: false,
                supports_tools: true,
                supports_reasoning: false,
                supports_structured_output: false,
                supports_streaming: true,
                max_output_tokens: None,
                enabled: true,
            }]
        };

        // Resolve api_key: explicit config → per-provider shortcut key.
        let api_key = config
            .api_key
            .clone()
            .or_else(|| config.provider_keys.get(&provider_id).cloned());

        let display_name = catalog_provider
            .map(|p| p.display_name.clone())
            .unwrap_or_else(|| format!("{provider_id} (default)"));

        let base_url = config
            .base_url
            .clone()
            .or_else(|| catalog_provider.and_then(|p| p.base_url.clone()))
            .unwrap_or_default();

        let mut pc = ProviderConfig {
            id: provider_id.clone(),
            display_name,
            base_url,
            api_key,
            protocol: ProtocolSetting::Auto,
            default_model: Some(model_id),
            models,
            enabled: true,
        };
        enrich_provider_config(&mut pc);

        let mut providers = self.providers.write().await;
        providers.insert(provider_id.clone(), pc);
        drop(providers);

        let mut default = self.default_id.write().await;
        *default = Some(provider_id);
    }

    /// Seed providers from config-file definitions.
    /// Seed providers from the YAML `providers:` array.
    ///
    /// If the array contains exactly one provider that entry becomes the
    /// registry default (unless `seed_from_llm_config` was already called and
    /// set a default from `llm.model`). This ensures YAML-only deployments that
    /// never set `UAR_LLM__MODEL` still surface a correct default in the UI.
    pub async fn seed_from_configs(&self, configs: Vec<ProviderConfig>) {
        let mut providers = self.providers.write().await;
        let mut first_id: Option<String> = None;
        for config in configs {
            tracing::info!(
                provider_id = %config.id,
                base_url = %config.base_url,
                models = config.models.len(),
                "Registered provider from config"
            );
            if first_id.is_none() {
                first_id = Some(config.id.clone());
            }
            providers.insert(config.id.clone(), config);
        }
        drop(providers);

        // Set as default if no default has been established yet (e.g. only
        // config-file providers, no `llm.model` env var).
        if let Some(id) = first_id {
            let mut default = self.default_id.write().await;
            if default.is_none() {
                tracing::info!(provider_id = %id, "Setting registry default from config providers array");
                *default = Some(id);
            }
        }
    }

    /// Register a new provider (internal / seeding use).
    pub async fn register(&self, config: ProviderConfig) -> anyhow::Result<()> {
        let mut providers = self.providers.write().await;
        tracing::info!(provider_id = %config.id, "Registering provider");
        providers.insert(config.id.clone(), config);
        Ok(())
    }

    /// Register a custom provider that is not (or is not yet) in the built-in catalog.
    ///
    /// Use this for custom endpoints such as local Ollama instances, corporate
    /// proxy servers, or third-party compatible APIs. If the provider ID matches
    /// a catalog entry and the caller did not supply any models, the catalog's
    /// model list is used to enrich the entry automatically.
    ///
    /// # Errors
    ///
    /// Returns an error if `config.id` is empty.
    pub async fn register_custom_provider(&self, mut config: ProviderConfig) -> anyhow::Result<()> {
        if config.id.is_empty() {
            anyhow::bail!("Custom provider ID must not be empty");
        }

        enrich_provider_config(&mut config);

        let mut providers = self.providers.write().await;

        // Preserve existing default_model if the new config doesn't specify one.
        // This prevents UI-driven re-registration from losing the model set at startup.
        if config.default_model.is_none() {
            if let Some(existing) = providers.get(&config.id) {
                if existing.default_model.is_some() {
                    tracing::debug!(
                        provider_id = %config.id,
                        preserved_model = ?existing.default_model,
                        "Preserving existing default_model during re-registration"
                    );
                    config.default_model = existing.default_model.clone();
                }
            }
        }

        // Preserve existing API key if the new config doesn't have one.
        if config.api_key.is_none() {
            if let Some(existing) = providers.get(&config.id) {
                config.api_key = existing.api_key.clone();
            }
        }

        tracing::info!(
            provider_id = %config.id,
            base_url = %config.base_url,
            models = config.models.len(),
            default_model = ?config.default_model,
            "Registering custom provider"
        );

        providers.insert(config.id.clone(), config);
        Ok(())
    }

    /// Get a provider by ID.
    pub async fn get(&self, id: &str) -> Option<ProviderConfig> {
        let providers = self.providers.read().await;
        providers.get(id).cloned()
    }

    /// Check if a provider is configured (exists and enabled).
    pub async fn is_configured(&self, id: &str) -> bool {
        let providers = self.providers.read().await;
        providers.get(id).is_some_and(|p| p.enabled)
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

    /// Return `(provider_id, model_id)` for the configured default provider,
    /// or `None` if no default is set or the provider has no `default_model`.
    pub async fn default_model(&self) -> Option<(String, String)> {
        let provider_id = self.default_id.read().await.clone()?;
        let providers = self.providers.read().await;
        let config = providers.get(&provider_id)?;
        let model_id = config.default_model.clone()?;
        Some((provider_id, model_id))
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

    /// Resolve a provider/model pair into an `LlmConfig` for the liter-llm driver.
    pub async fn resolve_to_llm_config(&self, provider_id: &str, model: &str) -> Option<LlmConfig> {
        let providers = self.providers.read().await;
        let config = providers.get(provider_id)?;

        if !config.enabled {
            tracing::debug!(provider_id, "Provider is disabled, skipping");
            return None;
        }

        if !self.health.is_available(provider_id).await {
            tracing::debug!(provider_id, "Provider is in a failover cooldown, skipping");
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

        if config
            .models
            .iter()
            .find(|candidate| candidate.id == resolved_model)
            .is_some_and(|candidate| !candidate.enabled)
        {
            tracing::debug!(provider_id, model = %resolved_model, "Model is disabled, skipping");
            return None;
        }

        // When base_url is explicitly set, the provider routing is already handled
        // and the API expects just the model ID (e.g., "gpt-4o" not "openai/gpt-4o").
        // Only use provider/model format when liter-llm needs to auto-detect the provider.
        let has_explicit_base_url = !config.base_url.is_empty();
        let model_for_driver = if has_explicit_base_url {
            resolved_model
        } else {
            format!("{provider_id}/{resolved_model}")
        };

        Some(LlmConfig {
            model: model_for_driver,
            api_key: config.api_key.clone(),
            base_url: if config.base_url.is_empty() {
                None
            } else {
                Some(config.base_url.clone())
            },
            ..LlmConfig::default()
        })
    }

    /// Resolve using the agent's provider policy into an `LlmConfig`.
    pub async fn resolve_llm_config_from_policy(
        &self,
        policy: &crate::uar::domain::artifact::ProviderPolicy,
    ) -> Option<LlmConfig> {
        if let Some(cfg) = self
            .resolve_to_llm_config(&policy.default.provider, &policy.default.model)
            .await
        {
            tracing::debug!(
                provider = %policy.default.provider,
                model = %policy.default.model,
                "Resolved primary provider"
            );
            return Some(cfg);
        }

        for (i, fallback) in policy.fallbacks.iter().enumerate() {
            if let Some(cfg) = self
                .resolve_to_llm_config(&fallback.provider, &fallback.model)
                .await
            {
                tracing::info!(
                    provider = %fallback.provider,
                    model = %fallback.model,
                    fallback_index = i,
                    "Resolved fallback provider"
                );
                return Some(cfg);
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

/// Default API base URL when the catalog omits `base_url` (common for OpenAI-compatible hosts).
#[must_use]
pub(crate) fn fallback_base_url(provider_id: &str) -> Option<&'static str> {
    match provider_id {
        "openai" => Some("https://api.openai.com/v1"),
        "anthropic" => Some("https://api.anthropic.com"),
        "groq" => Some("https://api.groq.com/openai/v1"),
        "together" => Some("https://api.together.xyz/v1"),
        "openrouter" => Some("https://openrouter.ai/api/v1"),
        "google" | "gemini" => Some("https://generativelanguage.googleapis.com/v1beta/openai"),
        "mistral" => Some("https://api.mistral.ai/v1"),
        "cohere" => Some("https://api.cohere.com/v2"),
        "deepseek" => Some("https://api.deepseek.com/v1"),
        "moonshot" | "moonshotai" => Some("https://api.moonshot.cn/v1"),
        "alibaba" => Some("https://dashscope-intl.aliyuncs.com/compatible-mode/v1"),
        "alibaba-cn" => Some("https://dashscope.aliyuncs.com/compatible-mode/v1"),
        "fireworks-ai" | "fireworks_ai" => Some("https://api.fireworks.ai/inference/v1"),
        "minimax" => Some("https://api.minimax.io/v1"),
        "xai" | "x-ai" => Some("https://api.x.ai/v1"),
        "perplexity" => Some("https://api.perplexity.ai"),
        _ => None,
    }
}

/// Fill display name, models, and `base_url` from the embedded catalog and known fallbacks.
///
/// Used by the admin API when clients omit `base_url` or model lists.
pub(crate) fn enrich_provider_config(config: &mut ProviderConfig) {
    let catalog = ModelCatalog::global();
    if config.models.is_empty() {
        if let Some(catalog_provider) = catalog.provider(&config.id) {
            config.models = models_from_catalog(catalog_provider);

            if config.display_name.is_empty() {
                config.display_name = catalog_provider.display_name.clone();
            }
            if config.base_url.is_empty() {
                if let Some(ref url) = catalog_provider.base_url {
                    config.base_url.clone_from(url);
                }
            }

            tracing::debug!(
                provider_id = %config.id,
                catalog_models = config.models.len(),
                "Auto-enriched provider from catalog"
            );
        }
    }

    if config.base_url.is_empty() {
        if let Some(url) = fallback_base_url(&config.id) {
            config.base_url = url.to_string();
        }
    }
}

/// Convert a catalog [`ProviderInfo`] model list into the registry's [`ModelConfig`] format.
fn models_from_catalog(provider: &ProviderInfo) -> Vec<ModelConfig> {
    provider
        .models
        .iter()
        .map(|m| ModelConfig {
            id: m.id.clone(),
            display_name: if m.name.is_empty() {
                None
            } else {
                Some(m.name.clone())
            },
            context_window: if m.limits.context_window > 0 {
                u32::try_from(m.limits.context_window).ok()
            } else {
                None
            },
            supports_vision: m.modalities.input.iter().any(|i| i == "image"),
            supports_tools: m.capabilities.tool_call,
            supports_reasoning: m.capabilities.reasoning,
            supports_structured_output: m.capabilities.structured_output,
            supports_streaming: m.capabilities.streaming,
            max_output_tokens: if m.limits.max_output > 0 {
                u32::try_from(m.limits.max_output).ok()
            } else {
                None
            },
            enabled: true,
        })
        .collect()
}

/// Split a `provider/model` string into `(provider_id, model_id)`.
///
/// If no slash is present, uses URL-based detection as the provider and the
/// entire string as the model.
fn split_model_string(model: &str) -> (String, String) {
    if let Some((provider, model_id)) = model.split_once('/') {
        (provider.to_string(), model_id.to_string())
    } else {
        // No provider prefix — infer from model name patterns.
        // liter-llm requires "provider/model" format; "default" is not a valid provider.
        let inferred = if model.starts_with("gpt-")
            || model.starts_with("o1")
            || model.starts_with("o3")
            || model.starts_with("o4")
            || model.starts_with("chatgpt-")
        {
            "openai"
        } else if model.starts_with("claude-") {
            "anthropic"
        } else if model.starts_with("gemini-") || model.starts_with("gemma-") {
            "google"
        } else if model.starts_with("llama")
            || model.starts_with("mixtral")
            || model.starts_with("mistral")
        {
            "groq"
        } else {
            // Fall back to openai-compatible as the most common default
            "openai"
        };
        tracing::debug!(
            model,
            inferred_provider = inferred,
            "No provider prefix in model name, inferred provider"
        );
        (inferred.to_string(), model.to_string())
    }
}

/// Public wrapper for `split_model_string` (used by `resolve_requested_model` in `server.rs`).
pub fn split_model_string_pub(model: &str) -> (String, String) {
    split_model_string(model)
}

/// Detect a short provider ID from a base URL.
#[allow(dead_code)]
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
    async fn test_resolve_to_llm_config() {
        let registry = ProviderRegistry::new();
        let config = make_test_config("groq", "https://api.groq.com/openai");
        registry.register(config).await.unwrap();

        let llm = registry
            .resolve_to_llm_config("groq", "llama-3.3-70b")
            .await;
        assert!(llm.is_some());
        let c = llm.unwrap();
        // base_url is set explicitly, so driver gets plain model (no provider prefix)
        assert_eq!(c.model, "llama-3.3-70b");
    }

    #[tokio::test]
    async fn test_resolve_uses_default_model() {
        let registry = ProviderRegistry::new();
        let config = make_test_config("openai", "https://api.openai.com");
        registry.register(config).await.unwrap();

        let llm = registry.resolve_to_llm_config("openai", "").await;
        assert!(llm.is_some());
        // base_url is set explicitly, so driver gets plain model (no provider prefix)
        assert_eq!(llm.unwrap().model, "test-model");
    }

    #[tokio::test]
    async fn test_resolve_disabled_provider() {
        let registry = ProviderRegistry::new();
        let mut config = make_test_config("disabled", "https://example.com");
        config.enabled = false;
        registry.register(config).await.unwrap();

        let llm = registry.resolve_to_llm_config("disabled", "model").await;
        assert!(llm.is_none());
    }

    #[tokio::test]
    async fn test_resolve_disabled_model() {
        let registry = ProviderRegistry::new();
        let mut config = make_test_config("openai", "https://api.openai.com");
        config.models = vec![ModelConfig {
            id: "test-model".to_string(),
            display_name: Some("Test model".to_string()),
            context_window: Some(8_192),
            supports_vision: false,
            supports_tools: true,
            supports_reasoning: false,
            supports_structured_output: false,
            supports_streaming: true,
            max_output_tokens: Some(1_024),
            enabled: false,
        }];
        registry.register(config).await.unwrap();

        assert!(
            registry
                .resolve_to_llm_config("openai", "test-model")
                .await
                .is_none()
        );
    }

    #[tokio::test]
    async fn test_resolve_unknown_provider() {
        let registry = ProviderRegistry::new();
        let llm = registry.resolve_to_llm_config("nonexistent", "model").await;
        assert!(llm.is_none());
    }

    #[tokio::test]
    async fn test_seed_from_llm_config() {
        let config = LlmConfig {
            model: "openai/gpt-4o".to_string(),
            api_key: Some("sk-test".to_string()),
            ..LlmConfig::default()
        };

        let registry = ProviderRegistry::new();
        registry.seed_from_llm_config(&config).await;

        let default_id = registry.default_id().await;
        assert_eq!(default_id, Some("openai".to_string()));
    }

    #[tokio::test]
    async fn test_seed_from_llm_config_enriches_provider_base_url() {
        let config = LlmConfig {
            model: "alibaba/qwen3.6-plus".to_string(),
            api_key: Some("test-key".to_string()),
            ..LlmConfig::default()
        };

        let registry = ProviderRegistry::new();
        registry.seed_from_llm_config(&config).await;

        let provider = registry.get("alibaba").await.expect("provider seeded");
        assert_eq!(
            provider.base_url,
            "https://dashscope-intl.aliyuncs.com/compatible-mode/v1"
        );

        let llm = registry
            .resolve_to_llm_config("alibaba", "qwen3.6-plus")
            .await
            .expect("provider resolves");
        assert_eq!(llm.model, "qwen3.6-plus");
        assert_eq!(
            llm.base_url.as_deref(),
            Some("https://dashscope-intl.aliyuncs.com/compatible-mode/v1")
        );
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

    #[test]
    fn test_enrich_fills_base_url_when_missing() {
        let mut config = ProviderConfig {
            id: "openai".to_string(),
            display_name: "OpenAI".to_string(),
            base_url: String::new(),
            api_key: None,
            protocol: ProtocolSetting::Auto,
            default_model: None,
            models: vec![],
            enabled: true,
        };
        enrich_provider_config(&mut config);
        assert_eq!(config.base_url, "https://api.openai.com/v1");
        assert!(
            !config.models.is_empty(),
            "catalog should hydrate models for openai"
        );
    }

    #[test]
    fn test_fallback_base_urls_for_catalog_aliases() {
        assert_eq!(
            fallback_base_url("moonshotai"),
            Some("https://api.moonshot.cn/v1")
        );
        assert_eq!(
            fallback_base_url("alibaba"),
            Some("https://dashscope-intl.aliyuncs.com/compatible-mode/v1")
        );
        assert_eq!(
            fallback_base_url("alibaba-cn"),
            Some("https://dashscope.aliyuncs.com/compatible-mode/v1")
        );
        assert_eq!(
            fallback_base_url("fireworks-ai"),
            Some("https://api.fireworks.ai/inference/v1")
        );
        assert_eq!(
            fallback_base_url("minimax"),
            Some("https://api.minimax.io/v1")
        );
        assert_eq!(
            fallback_base_url("deepseek"),
            Some("https://api.deepseek.com/v1")
        );
    }

    #[test]
    fn test_split_model_string() {
        let (p, m) = split_model_string("openai/gpt-4o");
        assert_eq!(p, "openai");
        assert_eq!(m, "gpt-4o");

        let (p, m) = split_model_string("llama3");
        assert_eq!(p, "groq");
        assert_eq!(m, "llama3");
    }

    #[test]
    fn provider_config_deserializes_null_base_url_and_display_name() {
        let j = r#"{"id":"openai","display_name":null,"base_url":null,"protocol":"auto","enabled":true}"#;
        let c: ProviderConfig = serde_json::from_str(j).expect("deserialize");
        assert_eq!(c.id, "openai");
        assert_eq!(c.display_name, "");
        assert_eq!(c.base_url, "");
    }
}
