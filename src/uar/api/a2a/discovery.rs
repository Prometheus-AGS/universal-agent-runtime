//! A2A Agent Discovery & Registry API.
//!
//! Provides REST endpoints for registering and discovering federated agents:
//! - `POST /a2a/registry/register` — Register or update an agent
//! - `GET  /a2a/registry/agents`   — List all registered agents
//! - `GET  /a2a/registry/agents/{id}` — Get a specific agent

use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::registry::{AgentInfo, AgentRegistry};

// ─────────────────────────────────────────────────────────────────────────────
// State
// ─────────────────────────────────────────────────────────────────────────────

/// Shared state for the discovery API.
#[derive(Debug)]
pub struct DiscoveryApiState {
    pub registry: Arc<dyn AgentRegistry>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Request / Response DTOs
// ─────────────────────────────────────────────────────────────────────────────

/// Request body for registering an agent.
#[derive(Debug, Deserialize)]
pub struct RegisterAgentRequest {
    /// Optional ID — if omitted, a new UUID is generated.
    pub id: Option<String>,
    pub name: String,
    pub description: String,
    pub base_url: String,
    #[serde(default)]
    pub capabilities: Vec<String>,
}

/// Response after registering an agent.
#[derive(Debug, Serialize)]
pub struct RegisterAgentResponse {
    pub id: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// Router
// ─────────────────────────────────────────────────────────────────────────────

/// Build the discovery router.
pub fn build_discovery_router() -> Router<Arc<DiscoveryApiState>> {
    Router::new()
        .route("/register", post(register_agent))
        .route("/agents", get(list_agents))
        .route("/agents/{id}", get(get_agent))
        .route("/skills", get(list_skills))
}

// ─────────────────────────────────────────────────────────────────────────────
// Handlers
// ─────────────────────────────────────────────────────────────────────────────

/// POST /a2a/registry/register
async fn register_agent(
    State(state): State<Arc<DiscoveryApiState>>,
    Json(req): Json<RegisterAgentRequest>,
) -> impl IntoResponse {
    let now = Utc::now();
    let agent = AgentInfo {
        id: req.id.unwrap_or_else(|| Uuid::new_v4().to_string()),
        name: req.name,
        description: req.description,
        base_url: req.base_url,
        capabilities: req.capabilities,
        created_at: now,
        updated_at: now,
    };

    match state.registry.register_agent(agent).await {
        Ok(id) => (StatusCode::OK, Json(RegisterAgentResponse { id })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to register agent: {e}"),
        )
            .into_response(),
    }
}

/// GET /a2a/registry/agents
async fn list_agents(State(state): State<Arc<DiscoveryApiState>>) -> impl IntoResponse {
    match state.registry.list_agents().await {
        Ok(agents) => (StatusCode::OK, Json(agents)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to list agents: {e}"),
        )
            .into_response(),
    }
}

/// GET /a2a/registry/agents/{id}
async fn get_agent(
    State(state): State<Arc<DiscoveryApiState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.registry.get_agent(&id).await {
        Ok(Some(agent)) => (StatusCode::OK, Json(agent)).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, format!("Agent '{id}' not found")).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to get agent: {e}"),
        )
            .into_response(),
    }
}

/// GET /a2a/registry/skills
async fn list_skills(State(state): State<Arc<DiscoveryApiState>>) -> impl IntoResponse {
    match state.registry.list_skills().await {
        Ok(skills) => (StatusCode::OK, Json(skills)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to list skills: {e}"),
        )
            .into_response(),
    }
}
