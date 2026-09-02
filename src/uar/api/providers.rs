//! Provider management API routes.
//!
//! RESTful endpoints for managing LLM provider configurations.

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};

use futures::StreamExt;

use crate::llm::registry::{ModelConfig, ProviderConfig, ProviderRegistry, enrich_provider_config};
use crate::llm::{LlmDriver, LlmRequest};
use crate::uar::settings::manager::SettingsManager;

/// Shared state for provider routes: in-memory registry plus optional settings DB sync.
#[derive(Clone, Debug)]
pub struct ProviderApiState {
    pub registry: Arc<ProviderRegistry>,
    pub settings_manager: Option<Arc<SettingsManager>>,
}

/// Build the providers API router.
///
/// Mounted at `/api/uar/providers`.
pub fn build_router() -> Router<ProviderApiState> {
    Router::new()
        .route("/", get(list_providers).post(create_provider))
        .route("/enabled", get(list_enabled_providers))
        .route("/health", get(provider_health))
        .route(
            "/{id}",
            get(get_provider)
                .put(update_provider)
                .delete(delete_provider),
        )
        .route("/{id}/models", get(list_models))
        .route("/{id}/test", post(test_provider))
        .route("/{id}/default", post(set_default))
}

async fn persist_provider_config(
    settings_manager: Option<&Arc<SettingsManager>>,
    cfg: &ProviderConfig,
) -> Result<(), String> {
    if let Some(m) = settings_manager {
        m.upsert_provider_config(cfg)
            .await
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

// =============================================================================
// HANDLERS
// =============================================================================

/// List all registered providers.
async fn list_providers(State(state): State<ProviderApiState>) -> Json<ProvidersResponse> {
    let providers = state.registry.list().await;
    let default_id = state.registry.default_id().await;
    Json(ProvidersResponse {
        providers: providers.into_iter().map(ProviderView::from).collect(),
        default_id,
    })
}

/// List only provider/model routes UAR may use for inference.
async fn list_enabled_providers(State(state): State<ProviderApiState>) -> Json<ProvidersResponse> {
    let providers = state
        .registry
        .list()
        .await
        .into_iter()
        .filter(|provider| provider.enabled)
        .map(|mut provider| {
            provider.models.retain(|model| model.enabled);
            ProviderView::from(provider)
        })
        .collect();
    let default_id = state.registry.default_id().await;
    Json(ProvidersResponse {
        providers,
        default_id,
    })
}

/// Provider health snapshot (CH-03): current failover-cooldown state per
/// provider, for Runtime Console surfacing.
async fn provider_health(State(state): State<ProviderApiState>) -> Json<ProviderHealthResponse> {
    let providers = state
        .registry
        .health()
        .snapshot()
        .await
        .into_iter()
        .map(|(id, s)| {
            (
                id,
                ProviderHealthEntry {
                    healthy: s.healthy,
                    consecutive_errors: s.consecutive_errors,
                    cooldown_remaining_secs: s.cooldown_remaining_secs,
                },
            )
        })
        .collect();
    Json(ProviderHealthResponse { providers })
}

/// Get a single provider by ID.
async fn get_provider(
    State(state): State<ProviderApiState>,
    Path(id): Path<String>,
) -> Result<Json<ProviderView>, StatusCode> {
    state
        .registry
        .get(&id)
        .await
        .map(ProviderView::from)
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

/// Create/register a new provider.
async fn create_provider(
    State(state): State<ProviderApiState>,
    Json(config): Json<ProviderConfig>,
) -> Result<(StatusCode, Json<ProviderView>), (StatusCode, Json<ErrorResponse>)> {
    if state.registry.get(&config.id).await.is_some() {
        return Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                error: format!("Provider '{}' already exists", config.id),
            }),
        ));
    }

    let id = config.id.clone();
    state
        .registry
        .register_custom_provider(config)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
        })?;

    let response = state.registry.get(&id).await.ok_or((
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
            error: "Provider missing after registration".to_string(),
        }),
    ))?;

    if let Err(e) = persist_provider_config(state.settings_manager.as_ref(), &response).await {
        let _ = state.registry.remove(&id).await;
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to persist provider: {e}"),
            }),
        ));
    }

    Ok((StatusCode::CREATED, Json(ProviderView::from(response))))
}

/// Update an existing provider.
async fn update_provider(
    State(state): State<ProviderApiState>,
    Path(id): Path<String>,
    Json(mut config): Json<ProviderConfig>,
) -> Result<Json<ProviderView>, (StatusCode, Json<ErrorResponse>)> {
    config.id = id;
    enrich_provider_config(&mut config);
    if config.api_key.is_none()
        && let Some(existing) = state.registry.get(&config.id).await
    {
        config.api_key = existing.api_key;
    }
    let response = config.clone();
    state.registry.update(config).await.map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;

    if let Err(e) = persist_provider_config(state.settings_manager.as_ref(), &response).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to persist provider: {e}"),
            }),
        ));
    }

    Ok(Json(ProviderView::from(response)))
}

/// Delete a provider by ID.
async fn delete_provider(
    State(state): State<ProviderApiState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    if state.registry.get(&id).await.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("Provider '{id}' not found"),
            }),
        ));
    }

    let was_default = state.registry.default_id().await.as_deref() == Some(id.as_str());

    state.registry.remove(&id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;

    if let Some(ref sm) = state.settings_manager {
        sm.delete_provider_config(&id).await.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to delete persisted provider: {e}"),
                }),
            )
        })?;
        if was_default {
            let _ = sm.set_default_provider_id("").await;
        }
    }

    Ok(StatusCode::NO_CONTENT)
}

/// List models for a specific provider.
async fn list_models(
    State(state): State<ProviderApiState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<ModelConfig>>, StatusCode> {
    state
        .registry
        .models(&id)
        .await
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

/// Set a provider as the default.
async fn set_default(
    State(state): State<ProviderApiState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    if state.registry.get(&id).await.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("Provider '{id}' not found"),
            }),
        ));
    }

    if let Some(ref sm) = state.settings_manager {
        sm.set_default_provider_id(&id).await.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to persist default provider: {e}"),
                }),
            )
        })?;
    }

    state.registry.set_default(&id).await.map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;

    Ok(StatusCode::OK)
}

#[derive(Debug, Default, Deserialize)]
struct TestProviderRequest {
    #[serde(default)]
    model: Option<String>,
}

#[derive(Serialize)]
struct TestProviderResponse {
    ok: bool,
    provider_id: String,
    model_id: String,
    latency_ms: u128,
    received_text: bool,
}

/// Validate persisted credentials and endpoint routing with a minimal inference
/// request. Secrets are never echoed in the response or logs.
async fn test_provider(
    State(state): State<ProviderApiState>,
    Path(id): Path<String>,
    Json(request): Json<TestProviderRequest>,
) -> Result<Json<TestProviderResponse>, (StatusCode, Json<ErrorResponse>)> {
    let provider = state.registry.get(&id).await.ok_or((
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
            error: format!("Provider '{id}' not found"),
        }),
    ))?;
    let model = request
        .model
        .or_else(|| provider.default_model.clone())
        .or_else(|| {
            provider
                .models
                .iter()
                .find(|model| model.enabled)
                .map(|model| model.id.clone())
        })
        .ok_or((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "No enabled model is configured for this provider".to_string(),
            }),
        ))?;
    let llm_config = state
        .registry
        .resolve_to_llm_config(&id, &model)
        .await
        .ok_or((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Provider or model is disabled, unavailable, or incomplete".to_string(),
            }),
        ))?;
    if llm_config
        .api_key
        .as_deref()
        .is_none_or(|key| key.trim().is_empty())
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Provider credential is not configured".to_string(),
            }),
        ));
    }

    let driver = crate::llm::LiterLlmDriver::new(
        crate::config::build_client_config(&llm_config),
        llm_config.model,
        None,
    )
    .map_err(|error| {
        (
            StatusCode::BAD_GATEWAY,
            Json(ErrorResponse {
                error: format!("Provider client initialization failed: {error}"),
            }),
        )
    })?;
    let started = Instant::now();
    let mut validation_request = LlmRequest {
        messages: vec![serde_json::json!({
            "role": "user",
            "content": "Reply with OK."
        })],
        tools: Vec::new(),
        cache_strategy: None,
        thinking_config: None,
        anthropic_system: None,
        extra_params: None,
    };
    crate::uar::runtime::context::normalize::normalize_provider_messages(
        &mut validation_request.messages,
    )
    .map_err(|error| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Provider validation history failed: {error}"),
            }),
        )
    })?;
    let stream = tokio::time::timeout(Duration::from_secs(30), driver.stream(validation_request))
        .await
        .map_err(|_| {
            (
                StatusCode::GATEWAY_TIMEOUT,
                Json(ErrorResponse {
                    error: "Provider validation timed out".to_string(),
                }),
            )
        })?
        .map_err(|error| {
            (
                StatusCode::BAD_GATEWAY,
                Json(ErrorResponse {
                    error: format!("Provider validation failed: {error}"),
                }),
            )
        })?;
    let events = stream.collect::<Vec<_>>().await;
    let received_text = events.into_iter().any(|event| {
        matches!(
            event,
            Ok(crate::normalized::NormalizedEvent::MessageDelta { ref text }) if !text.is_empty()
        )
    });
    if !received_text {
        return Err((
            StatusCode::BAD_GATEWAY,
            Json(ErrorResponse {
                error: "Provider returned no text".to_string(),
            }),
        ));
    }

    Ok(Json(TestProviderResponse {
        ok: true,
        provider_id: id,
        model_id: model,
        latency_ms: started.elapsed().as_millis(),
        received_text,
    }))
}

// =============================================================================
// RESPONSE TYPES
// =============================================================================

#[derive(Serialize)]
struct ProvidersResponse {
    providers: Vec<ProviderView>,
    default_id: Option<String>,
}

#[derive(Serialize)]
struct ProviderView {
    id: String,
    display_name: String,
    base_url: String,
    protocol: crate::llm::registry::ProtocolSetting,
    default_model: Option<String>,
    models: Vec<ModelConfig>,
    enabled: bool,
    credential_configured: bool,
}

impl From<ProviderConfig> for ProviderView {
    fn from(provider: ProviderConfig) -> Self {
        Self {
            id: provider.id,
            display_name: provider.display_name,
            base_url: provider.base_url,
            protocol: provider.protocol,
            default_model: provider.default_model,
            models: provider.models,
            enabled: provider.enabled,
            credential_configured: provider
                .api_key
                .as_deref()
                .is_some_and(|key| !key.trim().is_empty()),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct ErrorResponse {
    error: String,
}

/// One provider's health entry in `GET /health` (CH-03).
#[derive(Serialize)]
struct ProviderHealthEntry {
    healthy: bool,
    consecutive_errors: u32,
    cooldown_remaining_secs: Option<u64>,
}

#[derive(Serialize)]
struct ProviderHealthResponse {
    providers: std::collections::HashMap<String, ProviderHealthEntry>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::AppConfig,
        uar::persistence::{PersistenceLayer, providers::surreal::SurrealDbProvider},
    };
    use serde_json::json;

    async fn test_persistence() -> (Arc<dyn PersistenceLayer>, tempfile::TempDir) {
        let dir = tempfile::tempdir().expect("provider API test tempdir must be creatable");
        let url = format!("surrealkv://{}", dir.path().to_string_lossy());
        let provider = SurrealDbProvider::new(&url, None, None, None, None)
            .await
            .expect("provider API test database must start");
        (Arc::new(provider), dir)
    }

    fn test_config() -> AppConfig {
        serde_json::from_value(json!({
            "server": {
                "port": 3000,
                "host": "127.0.0.1"
            },
            "security": {
                "jwt_required": false,
                "jwt_secret": "test-secret"
            },
            "resilience": {
                "rate_limit_enabled": false,
                "requests_per_second": 5.0,
                "burst_size": 10.0
            },
            "persistence": {
                "provider": "surreal",
                "database_url": "memory://provider-api-test",
                "vector_dimension": 384,
                "external_cache_enabled": false
            }
        }))
        .expect("minimal provider API test config must deserialize")
    }

    fn provider(id: &str) -> ProviderConfig {
        ProviderConfig {
            id: id.to_string(),
            display_name: id.to_string(),
            base_url: format!("https://{id}.example.test/v1"),
            api_key: None,
            protocol: crate::llm::registry::ProtocolSetting::Auto,
            default_model: None,
            models: Vec::new(),
            enabled: true,
        }
    }

    async fn registry_with_two_providers() -> Arc<ProviderRegistry> {
        let registry = Arc::new(ProviderRegistry::new());
        registry
            .register(provider("provider-a"))
            .await
            .expect("provider-a must register");
        registry
            .register(provider("provider-b"))
            .await
            .expect("provider-b must register");
        registry
            .set_default("provider-a")
            .await
            .expect("provider-a must become the initial default");
        registry
    }

    #[tokio::test]
    async fn set_default_persistence_failure_preserves_live_default() {
        let (persistence, _dir) = test_persistence().await;
        let settings_manager = Arc::new(SettingsManager::new(persistence));
        let registry = registry_with_two_providers().await;
        let durable_before = settings_manager.get_default_provider_id().await;

        let error = set_default(
            State(ProviderApiState {
                registry: Arc::clone(&registry),
                settings_manager: Some(Arc::clone(&settings_manager)),
            }),
            Path("provider-b".to_string()),
        )
        .await
        .expect_err("an uninitialized settings manager must reject the durable write");

        assert_eq!(error.0, StatusCode::INTERNAL_SERVER_ERROR);
        assert!(
            error
                .1
                .0
                .error
                .contains("Failed to persist default provider")
        );
        assert_eq!(registry.default_id().await.as_deref(), Some("provider-a"));
        assert_eq!(
            settings_manager.get_default_provider_id().await,
            durable_before
        );
    }

    #[tokio::test]
    async fn set_default_missing_provider_preserves_defaults() {
        let (persistence, _dir) = test_persistence().await;
        let settings_manager = Arc::new(SettingsManager::new(persistence));
        let registry = registry_with_two_providers().await;
        let durable_before = settings_manager.get_default_provider_id().await;

        let error = set_default(
            State(ProviderApiState {
                registry: Arc::clone(&registry),
                settings_manager: Some(Arc::clone(&settings_manager)),
            }),
            Path("missing-provider".to_string()),
        )
        .await
        .expect_err("an unregistered provider must be rejected");

        assert_eq!(error.0, StatusCode::NOT_FOUND);
        assert_eq!(registry.default_id().await.as_deref(), Some("provider-a"));
        assert_eq!(
            settings_manager.get_default_provider_id().await,
            durable_before
        );
    }

    #[tokio::test]
    async fn set_default_success_survives_fresh_manager() {
        let (persistence, _dir) = test_persistence().await;
        let settings_manager = Arc::new(SettingsManager::new(Arc::clone(&persistence)));
        settings_manager
            .initialize(&test_config())
            .await
            .expect("settings must initialize");
        settings_manager
            .set_default_provider_id("provider-a")
            .await
            .expect("initial durable default must persist");
        let registry = registry_with_two_providers().await;

        let status = match set_default(
            State(ProviderApiState {
                registry: Arc::clone(&registry),
                settings_manager: Some(Arc::clone(&settings_manager)),
            }),
            Path("provider-b".to_string()),
        )
        .await
        {
            Ok(status) => status,
            Err((status, Json(error))) => {
                panic!("default selection failed with {status}: {}", error.error)
            }
        };

        assert_eq!(status, StatusCode::OK);
        assert_eq!(registry.default_id().await.as_deref(), Some("provider-b"));

        let fresh_manager = SettingsManager::new(persistence);
        fresh_manager
            .initialize(&test_config())
            .await
            .expect("fresh settings manager must reconstruct persisted state");
        assert_eq!(
            fresh_manager.get_default_provider_id().await.as_deref(),
            Some("provider-b")
        );
    }
}
