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
    http::StatusCode,
    response::IntoResponse,
    routing::{get, put},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::uar::{
    domain::prompt_caching::{CachingScope, UserPromptCachingSettings},
    runtime::user_settings_store::{UserSettingsStore, UserSettingsUpdate},
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

impl From<UserPromptCachingSettings> for UserSettingsResponse {
    fn from(s: UserPromptCachingSettings) -> Self {
        Self {
            user_id: s.user_id,
            prompt_caching_enabled: s.prompt_caching_enabled,
            preferred_scope: s.preferred_scope,
            updated_at: s.updated_at,
        }
    }
}

#[derive(Debug, Deserialize)]
struct UpdateSettingsRequest {
    /// Whether prompt caching should be enabled for this user.
    ///
    /// Pass `null` (or omit) to reset to "inherit from global setting".
    #[serde(default)]
    prompt_caching_enabled: Option<bool>,
    /// Which scope to use when no session-level override is present.
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
fn require_user_id(user_ctx: Option<axum::Extension<UserContext>>) -> Option<String> {
    let ctx = user_ctx?.0;
    // "anonymous" is the sentinel value used by the middleware when no JWT is
    // present; treat it as unauthenticated.
    if ctx.user_id.is_empty() || ctx.user_id == "anonymous" {
        None
    } else {
        Some(ctx.user_id)
    }
}

async fn get_settings(
    State(store): State<Arc<UserSettingsStore>>,
    user_ctx: Option<axum::Extension<UserContext>>,
) -> impl IntoResponse {
    let Some(user_id) = require_user_id(user_ctx) else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "Authentication required" })),
        )
            .into_response();
    };

    let settings = store
        .get(&user_id)
        .await
        .unwrap_or_else(|| UserPromptCachingSettings::new(&user_id));

    Json(UserSettingsResponse::from(settings)).into_response()
}

async fn update_settings(
    State(store): State<Arc<UserSettingsStore>>,
    user_ctx: Option<axum::Extension<UserContext>>,
    Json(req): Json<UpdateSettingsRequest>,
) -> impl IntoResponse {
    let Some(user_id) = require_user_id(user_ctx) else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "Authentication required" })),
        )
            .into_response();
    };

    let updated = store
        .upsert(UserSettingsUpdate {
            user_id,
            prompt_caching_enabled: req.prompt_caching_enabled,
            preferred_scope: req.preferred_scope,
        })
        .await;

    Json(UserSettingsResponse::from(updated)).into_response()
}
