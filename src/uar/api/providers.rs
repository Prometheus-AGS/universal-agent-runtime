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

use crate::llm::registry::{ModelConfig, ProviderConfig, ProviderRegistry};

/// Build the providers API router.
///
/// Mounted at `/api/uar/providers`.
pub fn build_router() -> Router<Arc<ProviderRegistry>> {
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

// =============================================================================
// HANDLERS
// =============================================================================

/// List all registered providers.
async fn list_providers(State(registry): State<Arc<ProviderRegistry>>) -> Json<ProvidersResponse> {
    let providers = registry.list().await;
    let default_id = registry.default_id().await;
    Json(ProvidersResponse {
        providers,
        default_id,
    })
}

/// Get a single provider by ID.
async fn get_provider(
    State(registry): State<Arc<ProviderRegistry>>,
    Path(id): Path<String>,
) -> Result<Json<ProviderConfig>, StatusCode> {
    registry
        .get(&id)
        .await
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

/// Create/register a new provider.
async fn create_provider(
    State(registry): State<Arc<ProviderRegistry>>,
    Json(config): Json<ProviderConfig>,
) -> Result<(StatusCode, Json<ProviderConfig>), (StatusCode, Json<ErrorResponse>)> {
    // Check if provider already exists
    if registry.get(&config.id).await.is_some() {
        return Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                error: format!("Provider '{}' already exists", config.id),
            }),
        ));
    }

    let response = config.clone();
    registry.register(config).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;

    Ok((StatusCode::CREATED, Json(response)))
}

/// Update an existing provider.
async fn update_provider(
    State(registry): State<Arc<ProviderRegistry>>,
    Path(id): Path<String>,
    Json(mut config): Json<ProviderConfig>,
) -> Result<Json<ProviderConfig>, (StatusCode, Json<ErrorResponse>)> {
    config.id = id; // Ensure ID in body matches path
    let response = config.clone();
    registry.update(config).await.map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;

    Ok(Json(response))
}

/// Delete a provider by ID.
async fn delete_provider(
    State(registry): State<Arc<ProviderRegistry>>,
    Path(id): Path<String>,
) -> StatusCode {
    match registry.remove(&id).await {
        Ok(()) => StatusCode::NO_CONTENT,
        Err(_) => StatusCode::NOT_FOUND,
    }
}

/// List models for a specific provider.
async fn list_models(
    State(registry): State<Arc<ProviderRegistry>>,
    Path(id): Path<String>,
) -> Result<Json<Vec<ModelConfig>>, StatusCode> {
    registry
        .models(&id)
        .await
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

/// Set a provider as the default.
async fn set_default(
    State(registry): State<Arc<ProviderRegistry>>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    registry.set_default(&id).await.map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;
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
