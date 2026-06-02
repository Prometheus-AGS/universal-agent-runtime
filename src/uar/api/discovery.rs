//! Unified discovery endpoints for agents, sessions, skills, and tools.

use crate::AppState;
use crate::uar::api::a2a::registry::AgentInfo;
use crate::uar::domain::artifact::AgentArtifact;
use crate::uar::domain::runs::RunStatus;
use crate::uar::domain::skills::{SkillConstraints, SkillTriggers};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

pub fn build_router() -> Router<AppState> {
    Router::new()
        .route("/agents", get(list_agents))
        .route(
            "/sessions/{session_id}/agent",
            get(current_agent_by_session),
        )
        .route("/skills", get(list_skills))
        .route("/tools", get(list_tools))
}

#[derive(Debug, Serialize)]
struct AgentsCatalogResponse {
    runtime_agents: Vec<AgentArtifact>,
    federated_agents: Vec<AgentInfo>,
}

#[derive(Debug, Serialize)]
struct SessionAgentResponse {
    session_id: String,
    run_id: String,
    agent_id: String,
    status: RunStatus,
    agent: Option<AgentArtifact>,
}

#[derive(Debug, Serialize)]
struct SkillFullResponse {
    skill_id: String,
    version: String,
    title: String,
    description: String,
    triggers: SkillTriggers,
    prompt_overlay: String,
    preferred_tools: Vec<String>,
    constraints: SkillConstraints,
    enabled: bool,
    provider_id: String,
    mcp_servers: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ToolsCatalogResponse {
    mcp_servers: Vec<String>,
    tools: Vec<ToolEntry>,
    built_in_tools: Vec<BuiltInToolEntry>,
}

#[derive(Debug, Serialize)]
struct ToolEntry {
    namespaced_name: String,
    name: String,
    source: String,
    server: Option<String>,
    description: Option<String>,
    input_schema: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct BuiltInToolEntry {
    name: String,
    description: Option<String>,
    parameters: serde_json::Value,
    source: &'static str,
}

pub async fn list_agents(State(state): State<AppState>) -> impl IntoResponse {
    let mut runtime_agents = match &state.persistence {
        Some(persistence) => persistence.list_agents().await.unwrap_or_default(),
        None => Vec::new(),
    };
    ensure_builtin_agent(&mut runtime_agents, crate::uar::defaults::default_agent());
    ensure_builtin_agent(
        &mut runtime_agents,
        crate::uar::defaults::orchestrator_agent(),
    );

    let federated_agents = state
        .federated_agent_registry
        .list_agents()
        .await
        .unwrap_or_default();

    Json(AgentsCatalogResponse {
        runtime_agents,
        federated_agents,
    })
}

async fn current_agent_by_session(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    if Uuid::parse_str(&session_id).is_err() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": {
                    "message": "session_id must be a valid UUID",
                    "type": "invalid_request_error",
                    "param": "session_id",
                    "code": "invalid_session_id"
                }
            })),
        )
            .into_response();
    }

    let Some(run) = state.run_manager.get_run_by_session_id(&session_id).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": {
                    "message": "No run found for session",
                    "type": "not_found_error",
                    "param": "session_id",
                    "code": "session_not_found"
                }
            })),
        )
            .into_response();
    };

    let agent = resolve_agent_artifact(&state, &run.agent_id).await;
    Json(SessionAgentResponse {
        session_id,
        run_id: run.run_id,
        agent_id: run.agent_id,
        status: run.status,
        agent,
    })
    .into_response()
}

async fn list_skills(State(state): State<AppState>) -> impl IntoResponse {
    let skills = state.skill_service.get_skills().await;
    let response = skills
        .into_iter()
        .map(|skill| SkillFullResponse {
            skill_id: skill.skill_id,
            version: skill.version,
            title: skill.title,
            description: skill.description,
            triggers: skill.triggers,
            prompt_overlay: skill.prompt_overlay,
            preferred_tools: skill.preferred_tools,
            constraints: skill.constraints,
            enabled: skill.enabled,
            provider_id: skill.provider_id,
            mcp_servers: skill
                .mcp_config
                .as_ref()
                .map(|cfg| cfg.mcp_servers.keys().cloned().collect())
                .unwrap_or_default(),
        })
        .collect::<Vec<_>>();

    Json(response)
}

pub async fn list_tools(State(state): State<AppState>) -> impl IntoResponse {
    let mcp_servers = state.mcp.server_names();
    let tools = state
        .mcp
        .tools()
        .iter()
        .map(|(namespaced_name, tool)| {
            let (source, server, name) = if state.mcp.is_native_tool(namespaced_name) {
                ("native_tool".to_string(), None, tool.name.to_string())
            } else if let Some((server, raw_name)) = state.mcp.resolve_mcp_tool(namespaced_name) {
                ("mcp_server".to_string(), Some(server), raw_name)
            } else {
                ("unknown".to_string(), None, tool.name.to_string())
            };

            ToolEntry {
                namespaced_name: namespaced_name.clone(),
                name,
                source,
                server,
                description: tool.description.as_ref().map(ToString::to_string),
                input_schema: serde_json::Value::Object(tool.input_schema.as_ref().clone()),
            }
        })
        .collect::<Vec<_>>();

    let built_in_tools = state
        .native_skill_registry
        .openai_tools_json()
        .await
        .into_iter()
        .map(|tool| BuiltInToolEntry {
            name: tool
                .get("function")
                .and_then(|f| f.get("name"))
                .and_then(serde_json::Value::as_str)
                .unwrap_or("unknown")
                .to_string(),
            description: tool
                .get("function")
                .and_then(|f| f.get("description"))
                .and_then(serde_json::Value::as_str)
                .map(ToString::to_string),
            parameters: tool
                .get("function")
                .and_then(|f| f.get("parameters"))
                .cloned()
                .unwrap_or_else(|| serde_json::json!({})),
            source: "built_in_native_skill",
        })
        .collect::<Vec<_>>();

    Json(ToolsCatalogResponse {
        mcp_servers,
        tools,
        built_in_tools,
    })
}

fn ensure_builtin_agent(agents: &mut Vec<AgentArtifact>, candidate: AgentArtifact) {
    if !agents.iter().any(|a| a.id == candidate.id) {
        agents.push(candidate);
    }
}

async fn resolve_agent_artifact(state: &AppState, agent_id: &str) -> Option<AgentArtifact> {
    if agent_id == "default-agent" {
        return Some(crate::uar::defaults::default_agent());
    }
    if agent_id == "orchestrator-agent" {
        return Some(crate::uar::defaults::orchestrator_agent());
    }
    let persistence = state.persistence.as_ref()?;
    persistence.load_agent(agent_id).await.ok().flatten()
}

/// Public wrapper used by the chat handler to resolve an agent by id.
///
/// Falls back to `default_agent()` if not found so the caller always gets
/// a valid artifact without an extra error-handling layer.
pub async fn resolve_agent_for_run(state: &AppState, agent_id: &str) -> AgentArtifact {
    resolve_agent_artifact(state, agent_id)
        .await
        .unwrap_or_else(|| {
            tracing::warn!(
                agent_id = %agent_id,
                "Agent not found — falling back to default-agent"
            );
            crate::uar::defaults::default_agent()
        })
}

// =========================================================================
// Agent CRUD handlers
// =========================================================================

/// POST /api/agents — create a new agent
pub async fn create_agent(
    State(state): State<AppState>,
    Json(mut agent): Json<AgentArtifact>,
) -> Result<(StatusCode, Json<AgentArtifact>), (StatusCode, String)> {
    if agent.id.is_empty() {
        agent.id = Uuid::new_v4().to_string();
    }
    agent.kind = "agent".to_string();

    let persistence = state.persistence.as_ref().ok_or((
        StatusCode::SERVICE_UNAVAILABLE,
        "No persistence layer".to_string(),
    ))?;

    persistence
        .save_agent(&agent)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((StatusCode::CREATED, Json(agent)))
}

/// PUT /api/agents/{id} — full replacement update
pub async fn update_agent_full(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(mut agent): Json<AgentArtifact>,
) -> Result<Json<AgentArtifact>, (StatusCode, String)> {
    agent.id = id;

    let persistence = state.persistence.as_ref().ok_or((
        StatusCode::SERVICE_UNAVAILABLE,
        "No persistence layer".to_string(),
    ))?;

    persistence
        .save_agent(&agent)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(agent))
}

/// PATCH /api/agents/{id} — partial update (merge fields into existing agent)
pub async fn patch_agent(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(patch): Json<serde_json::Value>,
) -> Result<Json<AgentArtifact>, (StatusCode, String)> {
    let persistence = state.persistence.as_ref().ok_or((
        StatusCode::SERVICE_UNAVAILABLE,
        "No persistence layer".to_string(),
    ))?;

    let existing = persistence
        .load_agent(&id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, format!("Agent '{id}' not found")))?;

    let mut base = serde_json::to_value(&existing)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    json_merge(&mut base, &patch);

    let mut agent: AgentArtifact = serde_json::from_value(base).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            format!("Invalid agent after merge: {e}"),
        )
    })?;
    agent.id = id;

    persistence
        .save_agent(&agent)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(agent))
}

/// DELETE /api/agents/{id}
pub async fn delete_agent(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let persistence = state.persistence.as_ref().ok_or((
        StatusCode::SERVICE_UNAVAILABLE,
        "No persistence layer".to_string(),
    ))?;

    persistence
        .delete_agent(&id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

// =========================================================================
// Tool execution handler
// =========================================================================

#[derive(Debug, Deserialize)]
pub struct ExecuteToolRequest {
    #[serde(default)]
    arguments: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct ExecuteToolResponse {
    result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    duration_ms: u128,
    success: bool,
}

/// POST /api/tools/{name}/execute — execute a registered tool by namespaced name.
pub async fn execute_tool(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(body): Json<ExecuteToolRequest>,
) -> impl IntoResponse {
    // Verify the tool exists before attempting execution.
    let tool_exists = state
        .mcp
        .tools()
        .iter()
        .any(|(ns_name, _)| ns_name == &name);

    if !tool_exists {
        return (
            StatusCode::NOT_FOUND,
            Json(ExecuteToolResponse {
                result: None,
                error: Some(format!("unknown tool: {name}")),
                duration_ms: 0,
                success: false,
            }),
        )
            .into_response();
    }

    let start = std::time::Instant::now();
    match state.mcp.call_namespaced_tool(&name, body.arguments).await {
        Ok(result) => {
            let duration_ms = start.elapsed().as_millis();
            (
                StatusCode::OK,
                Json(ExecuteToolResponse {
                    result: Some(result),
                    error: None,
                    duration_ms,
                    success: true,
                }),
            )
                .into_response()
        }
        Err(err) => {
            let duration_ms = start.elapsed().as_millis();
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ExecuteToolResponse {
                    result: None,
                    error: Some(err.to_string()),
                    duration_ms,
                    success: false,
                }),
            )
                .into_response()
        }
    }
}

// =========================================================================
// Agent session config endpoints
// =========================================================================

/// Per-session agent configuration overrides.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSessionConfig {
    /// The agent definition this session is based on.
    pub agent_id: String,
    /// Override the model used by this agent for this session.
    #[serde(default)]
    pub model: Option<String>,
    /// Override the set of tools available to this agent.
    #[serde(default)]
    pub tools: Option<Vec<String>>,
    /// Override the set of skills available to this agent.
    #[serde(default)]
    pub skills: Option<Vec<String>>,
    /// Override the knowledge bases attached to this agent.
    #[serde(default)]
    pub knowledge_bases: Option<Vec<String>>,
    /// Override the MCP servers available to this agent.
    #[serde(default)]
    pub mcp_servers: Option<Vec<String>>,
    /// Tool approval policy: "auto" | "ask" | "deny".
    #[serde(default)]
    pub tool_approval: Option<String>,
}

/// POST /api/sessions/{id}/agent-config
pub async fn save_agent_session_config(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    Json(config): Json<AgentSessionConfig>,
) -> impl IntoResponse {
    state
        .agent_sessions
        .write()
        .await
        .insert(session_id, config.clone());
    (StatusCode::OK, Json(config))
}

/// GET /api/sessions/{id}/agent-config
pub async fn get_agent_session_config(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    let sessions = state.agent_sessions.read().await;
    match sessions.get(&session_id) {
        Some(config) => {
            (StatusCode::OK, Json(serde_json::to_value(config).unwrap())).into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "No agent session config found for this session"})),
        )
            .into_response(),
    }
}

/// GET /api/sessions/{id}/effective-config
///
/// Returns the agent definition merged with any per-session overrides.
pub async fn get_effective_config(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    let sessions = state.agent_sessions.read().await;
    let Some(config) = sessions.get(&session_id) else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "No agent session config found for this session"})),
        )
            .into_response();
    };

    // Load the base agent definition from persistence (or built-in defaults).
    let agent = resolve_agent_artifact(&state, &config.agent_id).await;

    // Build effective config: agent defaults + session overrides.
    let mut effective = serde_json::to_value(&agent).unwrap_or_else(|_| serde_json::json!({}));
    if let Some(model) = &config.model {
        effective["model_override"] = json!(model);
    }
    if let Some(tools) = &config.tools {
        effective["tools_override"] = json!(tools);
    }
    if let Some(skills) = &config.skills {
        effective["skills_override"] = json!(skills);
    }
    if let Some(kbs) = &config.knowledge_bases {
        effective["knowledge_bases_override"] = json!(kbs);
    }
    if let Some(mcp_servers) = &config.mcp_servers {
        effective["mcp_servers_override"] = json!(mcp_servers);
    }
    if let Some(tool_approval) = &config.tool_approval {
        effective["tool_approval_override"] = json!(tool_approval);
    }
    effective["session_id"] = json!(session_id);
    effective["agent_id"] = json!(config.agent_id);

    (StatusCode::OK, Json(effective)).into_response()
}

/// RFC 7396 JSON Merge Patch: recursively merge `patch` into `target`.
fn json_merge(target: &mut serde_json::Value, patch: &serde_json::Value) {
    if let serde_json::Value::Object(patch_map) = patch {
        if !target.is_object() {
            *target = serde_json::Value::Object(serde_json::Map::new());
        }
        let target_map = target.as_object_mut().unwrap();
        for (key, value) in patch_map {
            if value.is_null() {
                target_map.remove(key);
            } else if value.is_object() {
                let entry = target_map
                    .entry(key.clone())
                    .or_insert(serde_json::Value::Object(serde_json::Map::new()));
                json_merge(entry, value);
            } else {
                target_map.insert(key.clone(), value.clone());
            }
        }
    } else {
        *target = patch.clone();
    }
}
