//! Admin REST endpoints for memory management.
//!
//! All endpoints require an admin-level JWT.  They allow platform administrators
//! to list, search, inspect, and delete memories across any scope.
//!
//! Routes (mounted at `/api/admin/memories` in `server.rs`):
//!
//! ```text
//! GET    /api/admin/memories         — paginated list (scope filters)
//! GET    /api/admin/memories/stats   — count + scope breakdown
//! GET    /api/admin/memories/:id     — get single memory by ID
//! DELETE /api/admin/memories/:id     — delete a single memory
//! DELETE /api/admin/memories         — bulk-delete by scope filter
//! GET    /api/admin/memories/search  — semantic/text search
//! ```

use crate::AppState;
use axum::{
    Json, Router,
    extract::{Path, Query, State},
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
            .and_then(|r| serde_json::to_value(r).ok())
            .and_then(|v| v.as_str().map(str::to_string))
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

/// Build an anonymous-style UserContext for admin queries (no scope enforcement).
fn admin_ctx(user_id: Option<&str>) -> crate::uar::security::claims::UserContext {
    use crate::uar::security::claims::{UserClaims, UserContext};
    let uid = user_id.unwrap_or("anonymous").to_string();
    UserContext {
        user_id: uid.clone(),
        claims: UserClaims {
            sub: uid,
            name: None,
            roles: None,
            exp: 0,
        },
    }
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// `GET /api/admin/memories` — paginated list filtered by scope
pub async fn list_memories_handler(
    State(state): State<AppState>,
    Query(query): Query<ListMemoriesQuery>,
) -> impl IntoResponse {
    let Some(svc) = &state.memory_service else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"error": "Memory system not enabled"})),
        )
            .into_response();
    };

    let ctx = admin_ctx(query.user_id.as_deref());

    match svc
        .list(&ctx, query.agent_id.as_deref(), query.session_id.as_deref())
        .await
    {
        Ok(memories) => {
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
pub async fn memory_stats_handler(State(state): State<AppState>) -> impl IntoResponse {
    let Some(svc) = &state.memory_service else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"error": "Memory system not enabled"})),
        )
            .into_response();
    };

    let ctx = admin_ctx(None);
    match svc.list(&ctx, None, None).await {
        Ok(memories) => {
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
    Query(query): Query<SearchMemoriesQuery>,
) -> impl IntoResponse {
    let Some(svc) = &state.memory_service else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"error": "Memory system not enabled"})),
        )
            .into_response();
    };

    let ctx = admin_ctx(query.user_id.as_deref());
    match svc
        .search(
            &query.q,
            &ctx,
            query.agent_id.as_deref(),
            query.session_id.as_deref(),
            query.limit,
            None,
        )
        .await
    {
        Ok(results) => {
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
    Path(id): Path<String>,
) -> impl IntoResponse {
    let Some(svc) = &state.memory_service else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"error": "Memory system not enabled"})),
        )
            .into_response();
    };

    match svc.get(&id).await {
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
    Path(id): Path<String>,
) -> impl IntoResponse {
    let Some(svc) = &state.memory_service else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"error": "Memory system not enabled"})),
        )
            .into_response();
    };

    match svc.delete(&id).await {
        Ok(_) => Json(json!({"status": "deleted", "id": id})).into_response(),
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
    Query(query): Query<BulkDeleteQuery>,
) -> impl IntoResponse {
    let Some(svc) = &state.memory_service else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"error": "Memory system not enabled"})),
        )
            .into_response();
    };

    let ctx = admin_ctx(query.user_id.as_deref());
    match svc
        .delete_all(&ctx, query.agent_id.as_deref(), query.session_id.as_deref())
        .await
    {
        Ok(deleted) => Json(json!({ "deleted": deleted })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("{e}")})),
        )
            .into_response(),
    }
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn build_router() -> Router<AppState> {
    Router::new()
        .route(
            "/",
            get(list_memories_handler).delete(bulk_delete_memories_handler),
        )
        .route("/stats", get(memory_stats_handler))
        .route("/search", get(search_memories_handler))
        .route(
            "/:id",
            get(get_memory_handler).delete(delete_memory_handler),
        )
}
