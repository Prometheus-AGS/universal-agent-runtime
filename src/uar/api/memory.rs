//! HTTP REST handlers for legacy memory endpoints.
//!
//! These endpoints (`POST /memory`, `GET /memory`) are kept for backward
//! compatibility. New integrations should use the MCP memory tools at
//! `/mcp/memory` which provide the full surreal-memory feature set.

use crate::AppState;
use axum::{
    Json,
    extract::{Query, State},
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
    Json(payload): Json<SaveMemoryRequest>,
) -> impl IntoResponse {
    let Some(svc) = &state.memory_service else {
        return (StatusCode::SERVICE_UNAVAILABLE, "Memory system not enabled").into_response();
    };

    let memory = Memory::new(
        payload.content,
        payload.user_id,
        payload.agent_id,
        None,
        payload.categories.unwrap_or_default(),
    );

    match svc.storage().add_memory(memory).await {
        Ok(stored) => {
            let id = stored
                .id
                .as_ref()
                .map(|r| {
                    serde_json::to_value(r)
                        .ok()
                        .and_then(|v| v.as_str().map(str::to_string))
                        .unwrap_or_default()
                })
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
    Query(query): Query<SearchMemoryQuery>,
) -> impl IntoResponse {
    let Some(svc) = &state.memory_service else {
        return (StatusCode::SERVICE_UNAVAILABLE, "Memory system not enabled").into_response();
    };

    let limit = query.limit.unwrap_or(10);

    match svc
        .storage()
        .search_memories(
            &query.q,
            query.user_id.as_deref(),
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
