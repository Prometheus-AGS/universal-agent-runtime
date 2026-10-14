//! HTTP REST handlers for legacy memory endpoints.
//!
//! These endpoints (`POST /memory`, `GET /memory`) are kept for backward
//! compatibility. New integrations should use the MCP memory tools at
//! `/mcp/memory` which provide the full surreal-memory feature set.

use crate::AppState;
use crate::uar::security::claims::UserContext;
use axum::{
    Json,
    extract::{Extension, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use serde_json::json;
use surreal_memory::Memory;

#[derive(Debug, Deserialize)]
pub struct SaveMemoryRequest {
    pub content: String,
    pub categories: Option<Vec<String>>,
    pub agent_id: Option<String>,
    pub user_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SearchMemoryQuery {
    pub q: String,
    pub agent_id: Option<String>,
    pub user_id: Option<String>,
    pub limit: Option<usize>,
}

pub async fn save_memory_handler(
    State(state): State<AppState>,
    Extension(user_ctx): Extension<UserContext>,
    Json(payload): Json<SaveMemoryRequest>,
) -> impl IntoResponse {
    let Some(svc) = &state.memory_service else {
        return (StatusCode::SERVICE_UNAVAILABLE, "Memory system not enabled").into_response();
    };

    let owned = match svc.for_context(&user_ctx) {
        Ok(owned) => owned,
        Err(_) => return (StatusCode::UNAUTHORIZED, "Verified memory owner required").into_response(),
    };
    if let Err(error) = owned.check_filter(payload.user_id.as_deref()) {
        return (StatusCode::FORBIDDEN, error.to_string()).into_response();
    }
    let memory = Memory::new(
        payload.content,
        None,
        payload.agent_id,
        None,
        payload.categories.unwrap_or_default(),
    );

    match owned.add_memory(memory).await {
        Ok(stored) => {
            let id = stored
                .id
                .as_ref()
                .map(|r| crate::uar::memory::owned::record_id(r).unwrap_or_default())
                .unwrap_or_default();
            Json(json!({ "status": "success", "id": id })).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Save failed: {e}"),
        )
            .into_response(),
    }
}

pub async fn search_memory_handler(
    State(state): State<AppState>,
    Extension(user_ctx): Extension<UserContext>,
    Query(query): Query<SearchMemoryQuery>,
) -> impl IntoResponse {
    let Some(svc) = &state.memory_service else {
        return (StatusCode::SERVICE_UNAVAILABLE, "Memory system not enabled").into_response();
    };

    let owned = match svc.for_context(&user_ctx) {
        Ok(owned) => owned,
        Err(_) => return (StatusCode::UNAUTHORIZED, "Verified memory owner required").into_response(),
    };
    if let Err(error) = owned.check_filter(query.user_id.as_deref()) {
        return (StatusCode::FORBIDDEN, error.to_string()).into_response();
    }
    let limit = query.limit.unwrap_or(10);

    match owned
        .search_memories(
            &query.q,
            None,
            query.agent_id.as_deref(),
            None,
            None,
            limit,
        )
        .await
    {
        Ok(results) => {
            let out: Vec<serde_json::Value> = results
                .iter()
                .map(|m| {
                    json!({
                        "content": m.content,
                        "categories": m.categories,
                        "scope": format!("{:?}", m.scope),
                        "agent_id": m.agent_id,
                        "user_id": m.user_id,
                    })
                })
                .collect();
            Json(json!(out)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Search failed: {e}"),
        )
            .into_response(),
    }
}

/// Authenticated owner controls; mounted beneath `/api/memory` by the host.
pub fn controls_router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/policy", axum::routing::get(get_policy).put(set_policy))
        .route("/records", axum::routing::get(export_records))
        .route("/records/{id}", axum::routing::get(get_record).patch(update_record).delete(delete_record))
        .route("/records/{id}/history", axum::routing::get(record_history))
        .route("/records/{id}/control", axum::routing::get(record_control))
}

async fn get_policy(State(state): State<AppState>, Extension(user): Extension<UserContext>) -> axum::response::Response {
    let Some(service) = &state.memory_service else { return StatusCode::SERVICE_UNAVAILABLE.into_response(); };
    match service.capture_policy(&user).await {
        Ok(policy) => Json(policy).into_response(),
        Err(error) => (StatusCode::SERVICE_UNAVAILABLE, error.to_string()).into_response(),
    }
}
async fn set_policy(State(state): State<AppState>, Extension(user): Extension<UserContext>, Json(update): Json<crate::uar::memory::controls::CapturePolicyUpdate>) -> axum::response::Response {
    if let Err(error) = update.validate() { return (StatusCode::BAD_REQUEST, error.to_string()).into_response(); }
    let Some(service) = &state.memory_service else { return StatusCode::SERVICE_UNAVAILABLE.into_response(); };
    if update.enabled && !service.config().auto_capture { return (StatusCode::CONFLICT, "Host capture is disabled").into_response(); }
    match service.set_capture_policy(&user, update).await {
        Ok(policy) => Json(policy).into_response(),
        Err(error) => (StatusCode::SERVICE_UNAVAILABLE, error.to_string()).into_response(),
    }
}
async fn export_records(State(state): State<AppState>, Extension(user): Extension<UserContext>) -> axum::response::Response {
    let Some(service) = &state.memory_service else { return StatusCode::SERVICE_UNAVAILABLE.into_response(); };
    match service.list(&user, None, None).await {
        Ok(memories) => Json(json!({"memories":memories})).into_response(),
        Err(error) => (StatusCode::SERVICE_UNAVAILABLE, error.to_string()).into_response(),
    }
}
async fn get_record(State(state): State<AppState>, Extension(user): Extension<UserContext>, axum::extract::Path(id): axum::extract::Path<String>) -> axum::response::Response {
    let Some(service) = &state.memory_service else { return StatusCode::SERVICE_UNAVAILABLE.into_response(); };
    match service.get(&id, &user).await {
        Ok(Some(memory)) => Json(memory).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(error) => (StatusCode::SERVICE_UNAVAILABLE, error.to_string()).into_response(),
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UpdateRecord { content: String }
async fn update_record(State(state): State<AppState>, Extension(user): Extension<UserContext>, axum::extract::Path(id): axum::extract::Path<String>, Json(update): Json<UpdateRecord>) -> axum::response::Response {
    let Some(service) = &state.memory_service else { return StatusCode::SERVICE_UNAVAILABLE.into_response(); };
    match service.update(&id, update.content, &user).await {
        Ok(memory) => Json(memory).into_response(),
        Err(error) => (StatusCode::CONFLICT, error.to_string()).into_response(),
    }
}
async fn delete_record(State(state): State<AppState>, Extension(user): Extension<UserContext>, axum::extract::Path(id): axum::extract::Path<String>) -> axum::response::Response {
    let Some(service) = &state.memory_service else { return StatusCode::SERVICE_UNAVAILABLE.into_response(); };
    match service.delete(&id, &user).await {
        Ok(()) => Json(json!({"id":id,"deleted":true,"purge":"pending"})).into_response(),
        Err(error) => (StatusCode::CONFLICT, error.to_string()).into_response(),
    }
}
async fn record_history(State(state): State<AppState>, Extension(user): Extension<UserContext>, axum::extract::Path(id): axum::extract::Path<String>) -> axum::response::Response {
    let Some(service) = &state.memory_service else { return StatusCode::SERVICE_UNAVAILABLE.into_response(); };
    match service.history(&id, &user).await {
        Ok(history) => Json(json!({"history":history})).into_response(),
        Err(error) => (StatusCode::NOT_FOUND, error.to_string()).into_response(),
    }
}

async fn record_control(State(state): State<AppState>, Extension(user): Extension<UserContext>, axum::extract::Path(id): axum::extract::Path<String>) -> axum::response::Response {
    let Some(service) = &state.memory_service else { return StatusCode::SERVICE_UNAVAILABLE.into_response(); };
    match service.memory_control_status(&user, &id).await {
        Ok(Some(control)) => Json(control).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(error) => (StatusCode::SERVICE_UNAVAILABLE, error.to_string()).into_response(),
    }
}
