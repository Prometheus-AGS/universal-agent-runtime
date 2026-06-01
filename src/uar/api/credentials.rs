//! Per-user provider credential API.
//!
//! Authenticated users manage their own encrypted provider API keys. Raw keys
//! are accepted on write only and are **never** returned; reads expose masked
//! metadata (provider, last-4 hint, timestamps). All endpoints require a valid
//! JWT; anonymous requests are rejected with `401`.
//!
//! Routes (mount under `/api/uar/credentials`, `AppState` as state):
//!
//! | Method | Path           | Description                          |
//! |--------|----------------|--------------------------------------|
//! | GET    | `/`            | List the caller's credentials (masked) |
//! | PUT    | `/{provider}`  | Store or rotate a provider key       |
//! | DELETE | `/{provider}`  | Delete a provider key                |

use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, put},
};
use serde::{Deserialize, Serialize};

use crate::uar::security::{
    claims::UserContext,
    credentials::{
        CredentialEncryption, CredentialMetadata, CredentialRecord, CredentialScope,
        ProviderService,
    },
};

/// State for the credentials API: the optional multi-tenant credential service.
/// `None` ⇒ multi-tenant credentials are disabled on this server (→ `503`).
///
/// Narrow by design (not the full `AppState`) so the router is unit-testable in
/// isolation and decoupled from unrelated application state.
pub type CredentialApiState = Option<Arc<ProviderService>>;

/// Build the credentials router. Mount under `/api/uar/credentials` with
/// `AppState.provider_service` as state.
pub fn build_router() -> Router<CredentialApiState> {
    Router::new()
        .route("/", get(list_credentials))
        .route("/{provider}", put(put_credential))
        .route("/{provider}", delete(delete_credential))
}

#[derive(Debug, Serialize)]
struct CredentialView {
    provider_id: String,
    api_key_hint: String,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<CredentialMetadata> for CredentialView {
    fn from(m: CredentialMetadata) -> Self {
        Self {
            provider_id: m.provider_id,
            api_key_hint: m.api_key_hint,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}

#[derive(Debug, Deserialize)]
struct PutCredentialRequest {
    /// Raw provider API key. Accepted on write only; encrypted at rest.
    api_key: String,
}

/// Extract the authenticated user id (rejecting anonymous).
fn require_user_id(user_ctx: Option<axum::Extension<UserContext>>) -> Option<String> {
    let ctx = user_ctx?.0;
    if ctx.user_id.is_empty() || ctx.user_id == "anonymous" {
        None
    } else {
        Some(ctx.user_id)
    }
}

fn unauthorized() -> axum::response::Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(serde_json::json!({ "error": "Authentication required" })),
    )
        .into_response()
}

fn service_unavailable() -> axum::response::Response {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({
            "error": "Multi-tenant credentials are not enabled on this server"
        })),
    )
        .into_response()
}

async fn list_credentials(
    State(state): State<CredentialApiState>,
    user_ctx: Option<axum::Extension<UserContext>>,
) -> impl IntoResponse {
    let Some(user_id) = require_user_id(user_ctx) else {
        return unauthorized();
    };
    let Some(service) = state else {
        return service_unavailable();
    };

    match service.store().list(CredentialScope::User, &user_id).await {
        Ok(rows) => {
            let views: Vec<CredentialView> = rows
                .iter()
                .map(|r| CredentialView::from(CredentialMetadata::from(r)))
                .collect();
            Json(views).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "list credentials failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Failed to list credentials" })),
            )
                .into_response()
        }
    }
}

async fn put_credential(
    State(state): State<CredentialApiState>,
    user_ctx: Option<axum::Extension<UserContext>>,
    Path(provider): Path<String>,
    Json(req): Json<PutCredentialRequest>,
) -> impl IntoResponse {
    let Some(user_id) = require_user_id(user_ctx) else {
        return unauthorized();
    };
    let Some(service) = state else {
        return service_unavailable();
    };

    let encrypted = match service.encryption().encrypt(&req.api_key) {
        Ok(c) => c,
        Err(e) => {
            tracing::error!(error = %e, "encrypt credential failed");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Failed to store credential" })),
            )
                .into_response();
        }
    };
    let now = chrono::Utc::now();
    let record = CredentialRecord {
        scope: CredentialScope::User,
        scope_id: user_id,
        provider_id: provider.clone(),
        api_key_encrypted: encrypted,
        api_key_hint: CredentialEncryption::key_hint(&req.api_key, 4),
        created_at: now,
        updated_at: now,
    };

    match service.store().put(record.clone()).await {
        Ok(()) => (
            StatusCode::OK,
            Json(CredentialView::from(CredentialMetadata::from(&record))),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, "store credential failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Failed to store credential" })),
            )
                .into_response()
        }
    }
}

async fn delete_credential(
    State(state): State<CredentialApiState>,
    user_ctx: Option<axum::Extension<UserContext>>,
    Path(provider): Path<String>,
) -> impl IntoResponse {
    let Some(user_id) = require_user_id(user_ctx) else {
        return unauthorized();
    };
    let Some(service) = state else {
        return service_unavailable();
    };

    match service
        .store()
        .delete(CredentialScope::User, &user_id, &provider)
        .await
    {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "No credential for that provider" })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, "delete credential failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Failed to delete credential" })),
            )
                .into_response()
        }
    }
}
