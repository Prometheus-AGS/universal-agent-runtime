//! Per-user settings API.
//!
//! Exposes endpoints for authenticated users to read and update their own
//! prompt-caching preferences.  All endpoints require a valid JWT (`sub` claim
//! must be present); anonymous requests are rejected with `401 Unauthorized`.
//!
//! Routes (mount under `/api/uar/user`):
//!
//! | Method | Path       | Description                           |
//! |--------|------------|---------------------------------------|
//! | GET    | `/settings` | Retrieve current user settings        |
//! | PUT    | `/settings` | Create or update user settings        |

use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode, header},
    response::IntoResponse,
    routing::{get, put},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::uar::{
    domain::prompt_caching::{CachingScope, UserPromptCachingSettings},
    runtime::user_settings_store::{
        PromptCachingPreferenceUpdate, UserSettingsStore, UserSettingsUpdate,
    },
    security::claims::UserContext,
};

// ---------------------------------------------------------------------------
// Router builder
// ---------------------------------------------------------------------------

/// Build the user settings router.
///
/// Mount under `/api/uar/user` with `Arc<UserSettingsStore>` as state.
pub fn build_router() -> Router<Arc<UserSettingsStore>> {
    Router::new()
        .route("/settings", get(get_settings))
        .route("/settings", put(update_settings))
}

// ---------------------------------------------------------------------------
// Request / Response types
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
struct UserSettingsResponse {
    user_id: String,
    prompt_caching_enabled: Option<bool>,
    preferred_scope: CachingScope,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl UserSettingsResponse {
    fn for_user(settings: UserPromptCachingSettings, user_id: String) -> Self {
        Self {
            user_id,
            prompt_caching_enabled: settings.prompt_caching_enabled,
            preferred_scope: settings.preferred_scope,
            updated_at: settings.updated_at,
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
enum NullablePromptCachingUpdate {
    #[default]
    Omitted,
    Value(Option<bool>),
}

impl<'de> Deserialize<'de> for NullablePromptCachingUpdate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Option::<bool>::deserialize(deserializer).map(Self::Value)
    }
}

#[derive(Debug, Deserialize)]
struct UpdateSettingsRequest {
    /// Whether prompt caching should be enabled for this user.
    ///
    /// Omit to preserve the stored value; pass `null` to reset to inherit.
    #[serde(default)]
    prompt_caching_enabled: NullablePromptCachingUpdate,
    /// Deprecated compatibility field. It no longer affects precedence.
    #[serde(default)]
    preferred_scope: Option<CachingScope>,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// Extract the authenticated user ID from request extensions.
///
/// Returns `None` when the request is anonymous (no JWT claims set by the
/// auth middleware).
fn require_jwt_user(
    headers: &HeaderMap,
    user_ctx: Option<axum::Extension<UserContext>>,
) -> Option<UserContext> {
    if headers.contains_key("x-api-key") {
        return None;
    }
    let authorization = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    let token = authorization.strip_prefix("Bearer ")?;
    if token.is_empty() {
        return None;
    }
    let ctx = user_ctx?.0;
    if ctx.user_id.is_empty()
        || ctx.user_id == "anonymous"
        || ctx.claims.sub.is_empty()
        || ctx.claims.sub == "anonymous"
    {
        None
    } else {
        Some(ctx)
    }
}

/// Build the storage key for a principal established by verified JWT claims.
///
/// Length-prefixing prevents tenant/subject delimiter collisions. Tenant is
/// included only when the verifier established one from the JWT.
#[must_use]
pub fn principal_storage_key(user_ctx: &UserContext) -> Option<String> {
    let subject = user_ctx.claims.sub.as_str();
    if subject.is_empty() || subject == "anonymous" {
        return None;
    }
    Some(match &user_ctx.tenant_id {
        Some(tenant_id) => format!(
            "v1:t:{}:{}:s:{}:{}",
            tenant_id.as_str().len(),
            tenant_id.as_str(),
            subject.len(),
            subject
        ),
        None => format!("v1:s:{}:{}", subject.len(), subject),
    })
}

fn persistence_error(operation: &'static str, error: anyhow::Error) -> axum::response::Response {
    tracing::error!(operation, error = %error, "User settings persistence failed");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": "User settings persistence unavailable" })),
    )
        .into_response()
}

async fn get_settings(
    State(store): State<Arc<UserSettingsStore>>,
    headers: HeaderMap,
    user_ctx: Option<axum::Extension<UserContext>>,
) -> impl IntoResponse {
    let Some(user_ctx) = require_jwt_user(&headers, user_ctx) else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "Authentication required" })),
        )
            .into_response();
    };
    let Some(principal_id) = principal_storage_key(&user_ctx) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };

    let settings = match store.get(&principal_id).await {
        Ok(Some(settings)) => settings,
        Ok(None) => UserPromptCachingSettings::new(&principal_id),
        Err(error) => return persistence_error("get", error),
    };

    Json(UserSettingsResponse::for_user(settings, user_ctx.user_id)).into_response()
}

async fn update_settings(
    State(store): State<Arc<UserSettingsStore>>,
    headers: HeaderMap,
    user_ctx: Option<axum::Extension<UserContext>>,
    Json(req): Json<UpdateSettingsRequest>,
) -> impl IntoResponse {
    let Some(user_ctx) = require_jwt_user(&headers, user_ctx) else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "Authentication required" })),
        )
            .into_response();
    };
    let Some(principal_id) = principal_storage_key(&user_ctx) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };

    let prompt_caching_enabled = match req.prompt_caching_enabled {
        NullablePromptCachingUpdate::Omitted => PromptCachingPreferenceUpdate::Preserve,
        NullablePromptCachingUpdate::Value(None) => PromptCachingPreferenceUpdate::Clear,
        NullablePromptCachingUpdate::Value(Some(enabled)) => {
            PromptCachingPreferenceUpdate::Set(enabled)
        }
    };

    let updated = match store
        .upsert(UserSettingsUpdate {
            principal_id,
            prompt_caching_enabled,
            preferred_scope: req.preferred_scope,
        })
        .await
    {
        Ok(updated) => updated,
        Err(error) => return persistence_error("update", error),
    };

    Json(UserSettingsResponse::for_user(updated, user_ctx.user_id)).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::uar::security::claims::{TenantId, UserClaims};

    fn user_context(subject: &str, tenant: Option<&str>) -> UserContext {
        UserContext {
            user_id: subject.to_owned(),
            tenant_id: tenant.map(TenantId::for_test),
            claims: UserClaims {
                sub: subject.to_owned(),
                name: None,
                roles: None,
                tenant_id: tenant.map(str::to_owned),
                exp: usize::MAX,
            },
        }
    }

    #[test]
    fn nullable_update_distinguishes_omitted_null_true_and_false() {
        let omitted: UpdateSettingsRequest = serde_json::from_str("{}").expect("omitted");
        assert_eq!(
            omitted.prompt_caching_enabled,
            NullablePromptCachingUpdate::Omitted
        );
        let cleared: UpdateSettingsRequest =
            serde_json::from_str(r#"{"prompt_caching_enabled":null}"#).expect("null");
        assert_eq!(
            cleared.prompt_caching_enabled,
            NullablePromptCachingUpdate::Value(None)
        );
        let enabled: UpdateSettingsRequest =
            serde_json::from_str(r#"{"prompt_caching_enabled":true}"#).expect("true");
        assert_eq!(
            enabled.prompt_caching_enabled,
            NullablePromptCachingUpdate::Value(Some(true))
        );
        let disabled: UpdateSettingsRequest =
            serde_json::from_str(r#"{"prompt_caching_enabled":false}"#).expect("false");
        assert_eq!(
            disabled.prompt_caching_enabled,
            NullablePromptCachingUpdate::Value(Some(false))
        );
    }

    #[test]
    fn principal_keys_isolate_same_subject_across_tenants_without_collisions() {
        let tenant_a = principal_storage_key(&user_context("sam", Some("a:b"))).expect("key A");
        let tenant_b = principal_storage_key(&user_context("sam", Some("a"))).expect("key B");
        let no_tenant = principal_storage_key(&user_context("sam", None)).expect("subject key");

        assert_eq!(tenant_a, "v1:t:3:a:b:s:3:sam");
        assert_ne!(tenant_a, tenant_b);
        assert_ne!(tenant_a, no_tenant);
        assert_ne!(tenant_b, no_tenant);
    }

    #[test]
    fn jwt_gate_rejects_api_key_only_context() {
        let ctx = user_context("sam", None);
        let mut headers = HeaderMap::new();
        headers.insert("x-api-key", "pat-value".parse().expect("header"));
        assert!(require_jwt_user(&headers, Some(axum::Extension(ctx))).is_none());
    }

    #[test]
    fn jwt_gate_rejects_ambiguous_bearer_and_api_key_context() {
        let ctx = user_context("sam", None);
        let mut headers = HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            "Bearer ambiguous".parse().expect("authorization"),
        );
        headers.insert("x-api-key", "pat-value".parse().expect("API key"));
        assert!(require_jwt_user(&headers, Some(axum::Extension(ctx))).is_none());
    }
}
