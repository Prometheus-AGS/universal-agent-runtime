//! Skills management API routes.
//!
//! REST endpoints for skills CRUD, matching configuration,
//! and per-agent skill bindings.

use crate::uar::runtime::skills::service::{SkillMatchingConfig, SkillService};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Build the skills API router.
///
/// Mount under `/api/uar/skills` with `Arc<SkillService>` as state.
pub fn build_router() -> Router<Arc<SkillService>> {
    Router::new()
        // Skills CRUD
        .route("/", get(list_skills))
        .route("/{id}", get(get_skill))
        .route("/{id}", delete(delete_skill))
        .route("/{id}/toggle", post(toggle_skill))
        .route("/match", get(match_skills))
        .route("/refresh", post(refresh_skills))
        // Matching configuration
        .route("/config", get(get_config))
        .route("/config", put(set_config))
}

/// Build the agent-skills binding router.
///
/// Mount under `/api/uar/agents` with `Arc<SkillService>` as state.
pub fn build_agent_skills_router() -> Router<Arc<SkillService>> {
    Router::new()
        .route("/{agent_id}/skills", get(get_agent_skills))
        .route("/{agent_id}/skills", put(set_agent_skills))
        .route("/{agent_id}/skills/{skill_id}", post(add_agent_skill))
        .route("/{agent_id}/skills/{skill_id}", delete(remove_agent_skill))
}

// --- Response types ---

#[derive(Serialize)]
struct SkillResponse {
    skill_id: String,
    title: String,
    description: String,
    enabled: bool,
    provider_id: String,
}

impl From<crate::uar::domain::skills::Skill> for SkillResponse {
    fn from(s: crate::uar::domain::skills::Skill) -> Self {
        Self {
            skill_id: s.skill_id,
            title: s.title,
            description: s.description,
            enabled: s.enabled,
            provider_id: s.provider_id,
        }
    }
}

#[derive(Deserialize)]
struct ToggleRequest {
    enabled: bool,
}

#[derive(Deserialize)]
struct MatchQuery {
    q: String,
    agent_id: Option<String>,
}

#[derive(Deserialize)]
struct AgentSkillsRequest {
    skill_ids: Vec<String>,
}

// --- Skills endpoints ---

async fn list_skills(State(service): State<Arc<SkillService>>) -> Json<Vec<SkillResponse>> {
    let skills = service.get_skills().await;
    Json(skills.into_iter().map(SkillResponse::from).collect())
}

async fn get_skill(
    State(service): State<Arc<SkillService>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let skills = service.get_skills().await;
    match skills.into_iter().find(|s| s.skill_id == id) {
        Some(skill) => Json(SkillResponse::from(skill)).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn delete_skill(
    State(service): State<Arc<SkillService>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    // Toggle to disabled (registry-level removal)
    if service.toggle_skill(&id, false).await {
        StatusCode::NO_CONTENT.into_response()
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}

async fn toggle_skill(
    State(service): State<Arc<SkillService>>,
    Path(id): Path<String>,
    Json(req): Json<ToggleRequest>,
) -> impl IntoResponse {
    if service.toggle_skill(&id, req.enabled).await {
        StatusCode::OK.into_response()
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}

async fn match_skills(
    State(service): State<Arc<SkillService>>,
    axum::extract::Query(params): axum::extract::Query<MatchQuery>,
) -> Json<Vec<SkillResponse>> {
    let matched = service
        .match_skills(&params.q, params.agent_id.as_deref())
        .await;
    Json(matched.into_iter().map(SkillResponse::from).collect())
}

async fn refresh_skills(State(service): State<Arc<SkillService>>) -> impl IntoResponse {
    match service.refresh().await {
        Ok(skills) => Json(serde_json::json!({
            "count": skills.len(),
            "skills": skills.into_iter().map(SkillResponse::from).collect::<Vec<_>>(),
        }))
        .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("{e:?}") })),
        )
            .into_response(),
    }
}

// --- Config endpoints ---

async fn get_config(State(service): State<Arc<SkillService>>) -> Json<SkillMatchingConfig> {
    Json(service.get_matching_config().await)
}

async fn set_config(
    State(service): State<Arc<SkillService>>,
    Json(config): Json<SkillMatchingConfig>,
) -> StatusCode {
    service.set_matching_config(config).await;
    StatusCode::OK
}

// --- Agent-skills binding endpoints ---

async fn get_agent_skills(
    State(service): State<Arc<SkillService>>,
    Path(agent_id): Path<String>,
) -> Json<Vec<String>> {
    Json(service.get_agent_skill_ids(&agent_id).await)
}

async fn set_agent_skills(
    State(service): State<Arc<SkillService>>,
    Path(agent_id): Path<String>,
    Json(req): Json<AgentSkillsRequest>,
) -> StatusCode {
    service.set_agent_skills(&agent_id, req.skill_ids).await;
    StatusCode::OK
}

async fn add_agent_skill(
    State(service): State<Arc<SkillService>>,
    Path((agent_id, skill_id)): Path<(String, String)>,
) -> StatusCode {
    service.add_skill_to_agent(&agent_id, &skill_id).await;
    StatusCode::CREATED
}

async fn remove_agent_skill(
    State(service): State<Arc<SkillService>>,
    Path((agent_id, skill_id)): Path<(String, String)>,
) -> StatusCode {
    service.remove_skill_from_agent(&agent_id, &skill_id).await;
    StatusCode::NO_CONTENT
}
