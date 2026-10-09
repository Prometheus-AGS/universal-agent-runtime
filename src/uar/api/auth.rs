//! REST endpoints for API key management.
//!
//! Mounted at `/api/uar/auth`:
//!
//! | Method | Path | Auth | Description |
//! |--------|------|------|-------------|
//! | POST | `/keys` | JWT | Create a new API key |
//! | GET | `/keys` | JWT | List your API keys |
//! | DELETE | `/keys/{id}` | JWT | Revoke an API key |
//! | POST | `/exchange` | API Key header | Exchange API key for JWT |

use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{delete, post},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

use crate::uar::security::api_keys::{ApiKeyAuthorityError, ApiKeyService, CreateKeyRequest};
use crate::uar::security::claims::UserContext;

/// Shared state for the auth router.
#[derive(Clone, Debug)]
pub struct AuthApiState {
    pub api_key_service: Arc<ApiKeyService>,
}

/// Build the auth router.
pub fn build_router() -> Router<Arc<AuthApiState>> {
    Router::new()
        .route("/keys", post(create_key).get(list_keys))
        .route("/keys/{id}", delete(revoke_key))
        .route("/exchange", post(exchange_key))
}

// ─────────────────────────────────────────────────────────────────────────────
// Handlers
// ─────────────────────────────────────────────────────────────────────────────

/// POST /api/uar/auth/keys — Create a new API key.
///
/// Requires JWT auth. The raw key is returned once and never stored.
async fn create_key(
    State(state): State<Arc<AuthApiState>>,
    user: Option<axum::Extension<UserContext>>,
    Json(body): Json<CreateKeyRequest>,
) -> impl IntoResponse {
    let Some(axum::Extension(user)) = user else {
        return (StatusCode::UNAUTHORIZED, Json(json!({"error":"verified caller identity required"}))).into_response();
    };
    match state.api_key_service.create_key(&user, body).await {
        Ok(resp) => (StatusCode::CREATED, Json(json!(resp))).into_response(),
        Err(error) => key_authority_error(error),
    }
}

/// GET /api/uar/auth/keys — List API keys for the current user.
async fn list_keys(
    State(state): State<Arc<AuthApiState>>,
    user: Option<axum::Extension<UserContext>>,
) -> impl IntoResponse {
    let Some(axum::Extension(user)) = user else {
        return key_authority_error(ApiKeyAuthorityError::CallerRequired.into());
    };
    match state.api_key_service.list_keys(&user).await {
        Ok(keys) => (StatusCode::OK, Json(json!({ "keys": keys }))).into_response(),
        Err(error) => key_authority_error(error),
    }
}

/// DELETE /api/uar/auth/keys/{id} — Revoke an API key.
async fn revoke_key(
    State(state): State<Arc<AuthApiState>>,
    Path(id): Path<String>,
    user: Option<axum::Extension<UserContext>>,
) -> impl IntoResponse {
    let Some(axum::Extension(user)) = user else {
        return key_authority_error(ApiKeyAuthorityError::CallerRequired.into());
    };
    match state.api_key_service.revoke_key(&user, &id).await {
        Ok(true) => (StatusCode::OK, Json(json!({ "revoked": true, "id": id }))).into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "key not found", "id": id })),
        )
            .into_response(),
        Err(error) => key_authority_error(error),
    }
}

/// Request body for key exchange.
#[derive(Debug, Deserialize)]
struct ExchangeRequest {
    /// The raw API key to exchange.
    api_key: String,
}

/// Response for successful key exchange.
#[derive(Debug, Serialize)]
struct ExchangeResponse {
    /// Short-lived JWT token.
    token: String,
    /// Token type (always "Bearer").
    token_type: String,
    /// Expiry in seconds from now.
    expires_in: u64,
}

/// POST /api/uar/auth/exchange — Exchange an API key for a JWT.
///
/// Accepts the key via JSON body OR `X-API-Key` header.
async fn exchange_key(
    State(state): State<Arc<AuthApiState>>,
    headers: HeaderMap,
    body: Option<Json<ExchangeRequest>>,
) -> impl IntoResponse {
    // Prefer header, fall back to body
    let raw_key = if let Some(key) = headers
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .map(str::to_string)
    {
        key
    } else if let Some(Json(b)) = body {
        b.api_key
    } else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "provide api_key in body or X-API-Key header" })),
        )
            .into_response();
    };

    match state.api_key_service.exchange_with_expiry(&raw_key).await {
        Ok(Some(issued)) => (
            StatusCode::OK,
            Json(json!(ExchangeResponse {
                token: issued.token,
                token_type: "Bearer".to_string(),
                expires_in: issued.expires_in,
            })),
        )
            .into_response(),
        Ok(None) => (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "invalid or expired API key" })),
        )
            .into_response(),
        Err(error) => key_authority_error(error),
    }
}

fn key_authority_error(error: anyhow::Error) -> axum::response::Response {
    let (status, message) = match error.downcast_ref::<ApiKeyAuthorityError>() {
        Some(ApiKeyAuthorityError::CallerRequired) => (StatusCode::UNAUTHORIZED, "verified caller identity required"),
        Some(ApiKeyAuthorityError::DelegationDenied) => (StatusCode::FORBIDDEN, "requested API-key authority is not delegable"),
        Some(ApiKeyAuthorityError::ReissueRequired) => (StatusCode::UNAUTHORIZED, "API key requires reissue with verified identity metadata"),
        Some(ApiKeyAuthorityError::UnsupportedExchange) => (StatusCode::NOT_IMPLEMENTED, "API-key exchange is unsupported by the configured verifier"),
        Some(ApiKeyAuthorityError::InvalidConfiguration) => (StatusCode::SERVICE_UNAVAILABLE, "API-key authority configuration is invalid"),
        Some(ApiKeyAuthorityError::InvalidLifetime) => (StatusCode::BAD_REQUEST, "API key lifetime must be a positive representable duration"),
        None => (StatusCode::INTERNAL_SERVER_ERROR, "API-key operation failed"),
    };
    (status, Json(json!({"error":message}))).into_response()
}
