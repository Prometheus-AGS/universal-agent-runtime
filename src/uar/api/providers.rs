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

use crate::llm::registry::{ModelConfig, ProviderConfig, ProviderRegistry, enrich_provider_config};
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
        .route(
            "/{id}",
            get(get_provider)
                .put(update_provider)
                .delete(delete_provider),
        )
        .route("/{id}/models", get(list_models))
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
        providers,
        default_id,
    })
}

/// Get a single provider by ID.
async fn get_provider(
    State(state): State<ProviderApiState>,
    Path(id): Path<String>,
) -> Result<Json<ProviderConfig>, StatusCode> {
    state
        .registry
        .get(&id)
        .await
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

/// Create/register a new provider.
async fn create_provider(
    State(state): State<ProviderApiState>,
    Json(config): Json<ProviderConfig>,
) -> Result<(StatusCode, Json<ProviderConfig>), (StatusCode, Json<ErrorResponse>)> {
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

    Ok((StatusCode::CREATED, Json(response)))
}

/// Update an existing provider.
async fn update_provider(
    State(state): State<ProviderApiState>,
    Path(id): Path<String>,
    Json(mut config): Json<ProviderConfig>,
) -> Result<Json<ProviderConfig>, (StatusCode, Json<ErrorResponse>)> {
    config.id = id;
    enrich_provider_config(&mut config);
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

    Ok(Json(response))
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
    state.registry.set_default(&id).await.map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;

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

    Ok(StatusCode::OK)
}

// =============================================================================
// RESPONSE TYPES
// =============================================================================

#[derive(Serialize)]
struct ProvidersResponse {
    providers: Vec<ProviderConfig>,
    default_id: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct ErrorResponse {
    error: String,
}
