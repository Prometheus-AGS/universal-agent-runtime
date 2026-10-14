//! Admin REST endpoints for memory management.
//!
//! All endpoints require an admin-level JWT.  They allow platform administrators
//! to list, search, inspect, and delete memories across any scope.
//!
//! Routes (mounted at `/api/admin/memories` in `server.rs`):
//!
//! ```text
//! GET    /api/admin/memories         — paginated list (scope filters)
//! POST   /api/admin/memories         — add a memory
//! GET    /api/admin/memories/stats   — count + scope breakdown
//! GET    /api/admin/memories/:id     — get single memory by ID
//! PATCH  /api/admin/memories/:id     — replace a memory's content
//! DELETE /api/admin/memories/:id     — delete a single memory
//! DELETE /api/admin/memories         — bulk-delete by scope filter
//! GET    /api/admin/memories/search  — semantic/text search
//! ```

use crate::AppState;
use axum::{
    Json, Router,
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
};
use serde::Deserialize;
use serde_json::json;

// ---------------------------------------------------------------------------
// Query / request types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct ListMemoriesQuery {
    pub user_id: Option<String>,
    pub agent_id: Option<String>,
    pub session_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateMemoryRequest {
    pub content: String,
    pub user_id: Option<String>,
    pub agent_id: Option<String>,
    pub session_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMemoryRequest {
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct BulkDeleteQuery {
    pub user_id: Option<String>,
    pub agent_id: Option<String>,
    pub session_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SearchMemoriesQuery {
    pub q: String,
    pub user_id: Option<String>,
    pub agent_id: Option<String>,
    pub session_id: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    50
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn memory_to_json(m: &surreal_memory::Memory) -> serde_json::Value {
    let id =
        m.id.as_ref()
            .and_then(|r| crate::uar::memory::owned::record_id(r).ok())
            .unwrap_or_default();
    json!({
        "id": id,
        "content": m.content,
        "categories": m.categories,
        "scope": format!("{:?}", m.scope),
        "memory_type": format!("{:?}", m.memory_type),
        "user_id": m.user_id,
        "agent_id": m.agent_id,
        "session_id": m.session_id,
        "importance": m.importance,
        "created_at": m.created_at.to_rfc3339(),
    })
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// `GET /api/admin/memories` — paginated list filtered by scope
pub async fn list_memories_handler(
    State(state): State<AppState>,
    Extension(user): Extension<crate::uar::security::claims::UserContext>,
    Query(query): Query<ListMemoriesQuery>,
) -> impl IntoResponse {
    if user.trusted_host_id().is_none() {
        return (StatusCode::FORBIDDEN, "Trusted host memory administration required").into_response();
    }
    let Some(svc) = &state.memory_service else {
        // Disabled memory is a valid administrative state, not an outage.  A
        // settings client must be able to render the empty collection and let
        // the user enable it without producing a failed network request.
        return Json(json!({"enabled": false, "total": 0, "items": []})).into_response();
    };

    let storage = match svc.admin_storage(&user) {
        Ok(storage) => storage,
        Err(error) => return (StatusCode::FORBIDDEN, error.to_string()).into_response(),
    };

    match storage
        .get_all_memories(query.user_id.as_deref(), query.agent_id.as_deref(), query.session_id.as_deref())
        .await
    {
        Ok(memories) => {
            let memories = match svc.visible_memories(memories).await {
                Ok(memories) => memories,
                Err(error) => return (StatusCode::SERVICE_UNAVAILABLE, error.to_string()).into_response(),
            };
            let items: Vec<serde_json::Value> = memories.iter().map(memory_to_json).collect();
            Json(json!({
                "total": items.len(),
                "items": items,
            }))
            .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("{e}")})),
        )
            .into_response(),
    }
}

/// `GET /api/admin/memories/stats` — count + scope breakdown
pub async fn memory_stats_handler(State(state): State<AppState>, Extension(user): Extension<crate::uar::security::claims::UserContext>) -> impl IntoResponse {
    if user.trusted_host_id().is_none() {
        return (StatusCode::FORBIDDEN, "Trusted host memory administration required").into_response();
    }
    let Some(svc) = &state.memory_service else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"error": "Memory system not enabled"})),
        )
            .into_response();
    };

    let storage = match svc.admin_storage(&user) {
        Ok(storage) => storage,
        Err(error) => return (StatusCode::FORBIDDEN, error.to_string()).into_response(),
    };
    match storage.get_all_memories(None, None, None).await {
        Ok(memories) => {
            let memories = match svc.visible_memories(memories).await {
                Ok(memories) => memories,
                Err(error) => return (StatusCode::SERVICE_UNAVAILABLE, error.to_string()).into_response(),
            };
            let total = memories.len();
            let mut by_scope: std::collections::HashMap<String, usize> = Default::default();
            for m in &memories {
                *by_scope.entry(format!("{:?}", m.scope)).or_insert(0) += 1;
            }
            Json(json!({
                "total": total,
                "by_scope": by_scope,
            }))
            .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("{e}")})),
        )
            .into_response(),
    }
}

/// `GET /api/admin/memories/search?q=...` — text/semantic search
pub async fn search_memories_handler(
    State(state): State<AppState>,
    Extension(user): Extension<crate::uar::security::claims::UserContext>,
    Query(query): Query<SearchMemoriesQuery>,
) -> impl IntoResponse {
    if user.trusted_host_id().is_none() {
        return (StatusCode::FORBIDDEN, "Trusted host memory administration required").into_response();
    }
    let Some(svc) = &state.memory_service else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"error": "Memory system not enabled"})),
        )
            .into_response();
    };

    let storage = match svc.admin_storage(&user) {
        Ok(storage) => storage,
        Err(error) => return (StatusCode::FORBIDDEN, error.to_string()).into_response(),
    };
    match storage
        .search_memories(
            &query.q,
            query.user_id.as_deref(),
            query.agent_id.as_deref(),
            query.session_id.as_deref(),
            None,
            query.limit,
        )
        .await
    {
        Ok(results) => {
            let results = match svc.visible_memories(results).await {
                Ok(results) => results,
                Err(error) => return (StatusCode::SERVICE_UNAVAILABLE, error.to_string()).into_response(),
            };
            let items: Vec<serde_json::Value> = results.iter().map(memory_to_json).collect();
            Json(json!({ "items": items, "count": items.len() })).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("{e}")})),
        )
            .into_response(),
    }
}

/// `GET /api/admin/memories/:id`
pub async fn get_memory_handler(
    State(state): State<AppState>,
    Extension(user): Extension<crate::uar::security::claims::UserContext>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    if user.trusted_host_id().is_none() {
        return (StatusCode::FORBIDDEN, "Trusted host memory administration required").into_response();
    }
    let Some(svc) = &state.memory_service else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"error": "Memory system not enabled"})),
        )
            .into_response();
    };

    match crate::uar::admin::memory::get(Some(svc), &id, &user).await {
        Ok(Some(m)) => Json(memory_to_json(&m)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Memory not found"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("{e}")})),
        )
            .into_response(),
    }
}

/// `DELETE /api/admin/memories/:id`
pub async fn delete_memory_handler(
    State(state): State<AppState>,
    Extension(user): Extension<crate::uar::security::claims::UserContext>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    if user.trusted_host_id().is_none() {
        return (StatusCode::FORBIDDEN, "Trusted host memory administration required").into_response();
    }
    let Some(svc) = &state.memory_service else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"error": "Memory system not enabled"})),
        )
            .into_response();
    };

    match crate::uar::admin::memory::delete(Some(svc), &id, &user).await {
        Ok(_) => Json(json!({"status": "deleted", "id": id})).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("{e}")})),
        )
            .into_response(),
    }
}

/// `POST /api/admin/memories` — add a memory
///
/// Exists so the REMOTE container has the same memory capability the embedded
/// one does. Before this the HTTP surface was read-and-delete only, so a served
/// deployment could show a user what was remembered about them and let them
/// erase it, but never correct it — the two containers were not at parity.
pub async fn create_memory_handler(
    State(state): State<AppState>,
    Extension(user): Extension<crate::uar::security::claims::UserContext>,
    Json(body): Json<CreateMemoryRequest>,
) -> impl IntoResponse {
    if user.trusted_host_id().is_none() {
        return (StatusCode::FORBIDDEN, "Trusted host memory administration required").into_response();
    }
    let Some(svc) = &state.memory_service else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"error": "Memory system not enabled"})),
        )
            .into_response();
    };

    // An admin filter is never converted to a forged owner capability. Shared
    // creation remains explicit; per-owner creation uses the ordinary endpoint.
    if body.user_id.is_some() {
        return (StatusCode::BAD_REQUEST, "Use the authenticated owner memory endpoint for per-owner creation").into_response();
    }
    match crate::uar::admin::memory::add(
        Some(svc), body.content,
        crate::uar::domain::memory::MemoryScope::Global,
        crate::uar::domain::memory::MemoryType::default(),
        &user, body.agent_id.as_deref(), body.session_id.as_deref(),
    ).await
    {
        Ok(memory) => (StatusCode::CREATED, Json(memory_to_json(&memory))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("{e}")})),
        )
            .into_response(),
    }
}

/// `PATCH /api/admin/memories/{id}` — replace a memory's content
///
/// The service records a history entry, so an edit is auditable rather than
/// destructive: the previous text stays recoverable.
pub async fn update_memory_handler(
    State(state): State<AppState>,
    Extension(user): Extension<crate::uar::security::claims::UserContext>,
    Path(id): Path<String>,
    Json(body): Json<UpdateMemoryRequest>,
) -> impl IntoResponse {
    if user.trusted_host_id().is_none() {
        return (StatusCode::FORBIDDEN, "Trusted host memory administration required").into_response();
    }
    let Some(svc) = &state.memory_service else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"error": "Memory system not enabled"})),
        )
            .into_response();
    };

    match crate::uar::admin::memory::update(Some(svc), &id, body.content, &user).await {
        Ok(memory) => Json(memory_to_json(&memory)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("{e}")})),
        )
            .into_response(),
    }
}

/// `DELETE /api/admin/memories` — bulk delete by scope filters
pub async fn bulk_delete_memories_handler(
    State(state): State<AppState>,
    Extension(user): Extension<crate::uar::security::claims::UserContext>,
    Query(query): Query<BulkDeleteQuery>,
) -> impl IntoResponse {
    if user.trusted_host_id().is_none() {
        return (StatusCode::FORBIDDEN, "Trusted host memory administration required").into_response();
    }
    let Some(svc) = &state.memory_service else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"error": "Memory system not enabled"})),
        )
            .into_response();
    };

    let storage = match svc.admin_storage(&user) {
        Ok(storage) => storage,
        Err(error) => return (StatusCode::FORBIDDEN, error.to_string()).into_response(),
    };
    match storage
        .get_all_memories(query.user_id.as_deref(), query.agent_id.as_deref(), query.session_id.as_deref())
        .await
    {
        Ok(memories) => {
            let mut deleted = 0;
            for memory in memories {
                let id = match memory.id.as_ref().map(crate::uar::memory::owned::record_id).transpose() {
                    Ok(Some(id)) => id,
                    _ => return (StatusCode::INTERNAL_SERVER_ERROR, "Memory identity unavailable").into_response(),
                };
                if let Err(error) = crate::uar::admin::memory::delete(Some(svc), &id, &user).await {
                    return (StatusCode::SERVICE_UNAVAILABLE, error.to_string()).into_response();
                }
                deleted += 1;
            }
            Json(json!({ "deleted": deleted })).into_response()
        },
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("{e}")})),
        )
            .into_response(),
    }
}

/// The second credential proves the mapping target. It is never persisted or
/// printed; this request deliberately has no Debug implementation.
#[derive(Deserialize)]
#[expect(missing_debug_implementations, reason = "Contains a migration owner bearer credential that must never be logged")]
pub struct ImportLegacyMemoryRequest {
    pub id: String,
    pub evidence: String,
    pub owner_bearer: Option<String>,
}

#[cfg(feature = "surreal-backend")]
pub async fn import_legacy_memory_handler(
    State(state): State<AppState>,
    Extension(administrator): Extension<crate::uar::security::claims::UserContext>,
    Json(body): Json<ImportLegacyMemoryRequest>,
) -> impl IntoResponse {
    if administrator.trusted_host_id().is_none() {
        return (StatusCode::FORBIDDEN, "Trusted host memory administration required").into_response();
    }
    let Some(service) = &state.memory_service else {
        return (StatusCode::SERVICE_UNAVAILABLE, "Memory system not enabled").into_response();
    };
    let owner = if let Some(bearer) = body.owner_bearer.as_deref() {
        match crate::uar::security::middleware::authenticate_bearer_context(&state.config.security, bearer).await {
            Ok(owner) => owner,
            Err(status) => return (status, "Verified migration owner credential required").into_response(),
        }
    } else { administrator.clone() };
    match service.import_legacy_memory(&administrator, &body.id, &owner, &body.evidence).await {
        Ok(memory) => (StatusCode::CREATED, Json(memory_to_json(&memory))).into_response(),
        Err(error) => (StatusCode::CONFLICT, Json(json!({ "error": error.to_string() }))).into_response(),
    }
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn build_router() -> Router<AppState> {
    let router = Router::new()
        .route(
            "/",
            get(list_memories_handler)
                .post(create_memory_handler)
                .delete(bulk_delete_memories_handler),
        )
        .route("/stats", get(memory_stats_handler))
        .route("/search", get(search_memories_handler))
        .route(
            "/{id}",
            get(get_memory_handler)
                .patch(update_memory_handler)
                .delete(delete_memory_handler),
        );
    #[cfg(feature = "surreal-backend")]
    let router = router.route("/import-legacy", axum::routing::post(import_legacy_memory_handler));
    router
}
