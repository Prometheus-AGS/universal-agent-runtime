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
    claims::{CredentialPrincipal, UserContext},
    credentials::{
        CredentialEncryption, CredentialMetadata, CredentialRecord, CredentialScope,
        CredentialMigrationError, ProviderService,
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
        .route("/principal", get(credential_principal))
        .route("/migrate-legacy", axum::routing::post(migrate_legacy))
        .route("/{provider}", get(read_credential))
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
#[serde(deny_unknown_fields)]
struct PutCredentialRequest {
    /// Raw provider API key. Accepted on write only; encrypted at rest.
    api_key: String,
}

/// Extract the authenticated user id (rejecting anonymous).
fn require_principal(user_ctx: Option<axum::Extension<UserContext>>) -> Option<CredentialPrincipal> {
    CredentialPrincipal::from_verified_context(&user_ctx?.0).ok()
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
    let Some(principal) = require_principal(user_ctx) else {
        return unauthorized();
    };
    let Some(service) = state else {
        return service_unavailable();
    };

    match service.store().list_principal(&principal, CredentialScope::User, principal.subject()).await {
        Ok(rows) => {
            let views: Vec<CredentialView> = rows
                .iter()
                .map(|r| CredentialView::from(CredentialMetadata::from(r)))
                .collect();
            Json(views).into_response()
        }
        Err(_) => {
            tracing::error!("list credentials failed");
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
    let Some(principal) = require_principal(user_ctx) else {
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
        scope_id: principal.subject().to_owned(),
        provider_id: provider.clone(),
        api_key_encrypted: encrypted,
        api_key_hint: CredentialEncryption::key_hint(&req.api_key, 4),
        created_at: now,
        updated_at: now,
    };

    match service.store().put_principal(&principal, record.clone()).await {
        Ok(()) => (
            StatusCode::OK,
            Json(CredentialView::from(CredentialMetadata::from(&record))),
        )
            .into_response(),
        Err(_) => {
            tracing::error!("store credential failed");
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
    let Some(principal) = require_principal(user_ctx) else {
        return unauthorized();
    };
    let Some(service) = state else {
        return service_unavailable();
    };

    match service
        .store()
        .delete_principal(&principal, CredentialScope::User, principal.subject(), &provider)
        .await
    {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "No credential for that provider" })),
        )
            .into_response(),
        Err(_) => {
            tracing::error!("delete credential failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Failed to delete credential" })),
            )
                .into_response()
        }
    }
}

async fn credential_principal(user_ctx: Option<axum::Extension<UserContext>>) -> axum::response::Response {
    let Some(principal) = require_principal(user_ctx) else { return unauthorized(); };
    Json(serde_json::json!({ "owner_key": principal.storage_key(), "issuer": principal.issuer(),
        "tenant_id": principal.tenant_id(), "subject": principal.subject() })).into_response()
}

async fn read_credential(
    State(state): State<CredentialApiState>,
    user_ctx: Option<axum::Extension<UserContext>>,
    Path(provider): Path<String>,
) -> axum::response::Response {
    let Some(principal) = require_principal(user_ctx) else { return unauthorized(); };
    let Some(service) = state else { return service_unavailable(); };
    match service.store().get_principal(&principal, CredentialScope::User, principal.subject(), &provider).await {
        Ok(Some(row)) => Json(CredentialView::from(CredentialMetadata::from(&row))).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(_) => StatusCode::SERVICE_UNAVAILABLE.into_response(),
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyMappingRequest {
    scope: CredentialScope,
    legacy_scope_id: String,
    provider_id: String,
    reason: String,
}

async fn migrate_legacy(
    State(state): State<CredentialApiState>,
    user_ctx: Option<axum::Extension<UserContext>>,
    Json(request): Json<LegacyMappingRequest>,
) -> axum::response::Response {
    let Some(principal) = require_principal(user_ctx) else { return unauthorized(); };
    let Some(service) = state else { return service_unavailable(); };
    match service.migrate_legacy_credential(&principal, request.scope, &request.legacy_scope_id,
        &request.provider_id, &request.reason).await {
        Ok(receipt) => Json(receipt).into_response(),
        Err(error) => {
            let (status, code) = match error.downcast_ref::<CredentialMigrationError>() {
                Some(CredentialMigrationError::MappingDenied) => (StatusCode::FORBIDDEN, "credential_mapping_denied"),
                Some(CredentialMigrationError::SourceMissing) => (StatusCode::NOT_FOUND, "legacy_credential_missing"),
                Some(CredentialMigrationError::Conflict) => (StatusCode::CONFLICT, "credential_mapping_conflict"),
                None => (StatusCode::SERVICE_UNAVAILABLE, "credential_migration_unavailable"),
            };
            (status, Json(serde_json::json!({ "error": { "code": code } }))).into_response()
        }
    }
}
