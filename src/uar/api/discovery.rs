//! Unified discovery endpoints for agents, sessions, skills, and tools.

use crate::AppState;
use crate::uar::api::a2a::registry::AgentInfo;
use crate::uar::domain::agent_store;
use crate::uar::domain::artifact::AgentArtifact;
use crate::uar::domain::policy::{
    ChatMode, ConversationPolicyRecord, EffectiveRunPolicy, ModelRoute, PolicyResolutionContext,
    PolicyUniverse, ResourceSelection, RunPolicy, SelectionMode, ToolApprovalPolicy,
    resolve_effective_run_policy_core,
};
use crate::uar::domain::runs::RunStatus;
use crate::uar::domain::skills::{SkillConstraints, SkillTriggers};
use crate::uar::security::claims::UserContext;
use axum::{
    Json, Router,
    extract::{Extension, Path, State},
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
    Extension(user): Extension<UserContext>,
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

    let Some(run) = state
        .run_manager
        .get_run_by_session_id_for_user(&user.user_id, &session_id)
        .await
    else {
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

    let agent = match resolve_agent_artifact(&state, &run.agent_id).await {
        Ok(agent) => agent,
        Err(error) => return error.into_response(),
    };
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

async fn resolve_agent_artifact(
    state: &AppState,
    agent_id: &str,
) -> Result<Option<AgentArtifact>, (StatusCode, String)> {
    if let Some(persistence) = &state.persistence {
        let persisted = persistence.load_agent(agent_id).await.map_err(|_| {
            (
                StatusCode::SERVICE_UNAVAILABLE,
                "Agent policy storage is unavailable".to_string(),
            )
        })?;
        if persisted.is_some() {
            return Ok(persisted);
        }
    }
    Ok(match agent_id {
        "default-agent" => Some(crate::uar::defaults::default_agent()),
        "orchestrator-agent" => Some(crate::uar::defaults::orchestrator_agent()),
        _ => None,
    })
}

/// Public wrapper used by the chat handler to resolve an agent by id.
///
/// Retains the legacy unknown-ID fallback, honoring a persisted default agent.
///
/// # Errors
/// Returns 503 if storage fails, rather than dropping persisted restrictions.
pub async fn resolve_agent_for_run(
    state: &AppState,
    agent_id: &str,
) -> Result<AgentArtifact, (StatusCode, String)> {
    if let Some(agent) = resolve_agent_artifact(state, agent_id).await? {
        return Ok(agent);
    }
    tracing::warn!(agent_id = %agent_id, "Agent not found — falling back to default-agent");
    Ok(resolve_agent_artifact(state, "default-agent")
        .await?
        .unwrap_or_else(crate::uar::defaults::default_agent))
}

// =========================================================================
// Agent CRUD handlers
// =========================================================================

/// GET /api/agents/{id} — read a persisted definition without catalog fallbacks.
///
/// # Errors
/// Returns 503 when storage is unavailable and 404 for an absent definition.
pub async fn get_persisted_agent(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<AgentArtifact>, (StatusCode, String)> {
    let persistence = state.persistence.as_ref().ok_or((
        StatusCode::SERVICE_UNAVAILABLE,
        "Agent storage is unavailable".to_string(),
    ))?;
    let agent = agent_store::get_agent(persistence.as_ref(), &id)
        .await
        .map_err(|_| {
            (
                StatusCode::SERVICE_UNAVAILABLE,
                "Agent storage is unavailable".to_string(),
            )
        })?
        .ok_or((
            StatusCode::NOT_FOUND,
            "Persisted agent not found".to_string(),
        ))?;
    Ok(Json(agent))
}

/// POST /api/agents — create a new agent
pub async fn create_agent(
    State(state): State<AppState>,
    Json(agent): Json<AgentArtifact>,
) -> Result<(StatusCode, Json<AgentArtifact>), (StatusCode, String)> {
    let persistence = state.persistence.as_ref().ok_or((
        StatusCode::SERVICE_UNAVAILABLE,
        "No persistence layer".to_string(),
    ))?;

    let saved = agent_store::create_agent(persistence.as_ref(), agent)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((StatusCode::CREATED, Json(saved)))
}

/// PUT /api/agents/{id} — full replacement update
pub async fn update_agent_full(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(agent): Json<AgentArtifact>,
) -> Result<Json<AgentArtifact>, (StatusCode, String)> {
    let persistence = state.persistence.as_ref().ok_or((
        StatusCode::SERVICE_UNAVAILABLE,
        "No persistence layer".to_string(),
    ))?;

    let saved = agent_store::replace_agent(persistence.as_ref(), id, agent)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(saved))
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

    let saved = agent_store::patch_agent(persistence.as_ref(), &id, &patch)
        .await
        .map_err(patch_error_response)?;

    Ok(Json(saved))
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

    agent_store::delete_agent(persistence.as_ref(), &id)
        .await
        .map_err(delete_error_response)?;

    Ok(StatusCode::NO_CONTENT)
}

/// Map an [`agent_store::AgentStoreError`] from a patch to the HTTP status the
/// previous inline handler produced (404 not found, 400 invalid, 500 backend).
fn patch_error_response(error: agent_store::AgentStoreError) -> (StatusCode, String) {
    match error {
        agent_store::AgentStoreError::Conflict => (
            StatusCode::CONFLICT,
            "Agent changed; reload before saving".to_string(),
        ),
        agent_store::AgentStoreError::NotFound(id) => {
            (StatusCode::NOT_FOUND, format!("Agent '{id}' not found"))
        }
        agent_store::AgentStoreError::Invalid(message) => (StatusCode::BAD_REQUEST, message),
        agent_store::AgentStoreError::Protected(id) => (
            StatusCode::FORBIDDEN,
            format!("Agent '{id}' is built in and cannot be deleted"),
        ),
        agent_store::AgentStoreError::Backend(error) => {
            (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
        }
    }
}

/// Map an [`agent_store::AgentStoreError`] from a delete to the HTTP status the
/// previous inline handler produced (403 for built-in agents, 500 otherwise).
fn delete_error_response(error: agent_store::AgentStoreError) -> (StatusCode, String) {
    match error {
        agent_store::AgentStoreError::Conflict => (
            StatusCode::CONFLICT,
            "Agent changed; reload before saving".to_string(),
        ),
        agent_store::AgentStoreError::Protected(id) => (
            StatusCode::FORBIDDEN,
            format!("Agent '{id}' is built in and cannot be deleted"),
        ),
        agent_store::AgentStoreError::Backend(error) => {
            (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
        }
        agent_store::AgentStoreError::NotFound(id) => {
            (StatusCode::NOT_FOUND, format!("Agent '{id}' not found"))
        }
        agent_store::AgentStoreError::Invalid(message) => (StatusCode::BAD_REQUEST, message),
    }
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
    /// Presentation assignment. Omission preserves the saved selection; an explicit
    /// `inherit` selection removes the override without copying inherited IDs.
    #[serde(default)]
    pub presentations: Option<ResourceSelection>,
    /// Tool approval policy: "auto" | "ask" | "deny".
    #[serde(default)]
    pub tool_approval: Option<String>,
    /// Per-session prompt-caching override. `None` inherits user/global policy.
    #[serde(default)]
    pub prompt_caching_enabled: Option<bool>,
}

impl AgentSessionConfig {
    fn into_run_policy(self) -> RunPolicy {
        let model = self.model.as_deref().and_then(|value| {
            value
                .split_once('/')
                .map(|(provider_id, model_id)| ModelRoute {
                    provider_id: provider_id.to_string(),
                    model_id: model_id.to_string(),
                })
        });
        RunPolicy {
            chat_mode: Some(ChatMode::Agent),
            agent_id: Some(self.agent_id),
            model,
            tools: optional_selection(self.tools),
            skills: optional_selection(self.skills),
            knowledge_bases: optional_selection(self.knowledge_bases),
            mcp_servers: optional_selection(self.mcp_servers),
            presentations: self.presentations.unwrap_or_default(),
            prompt_caching_enabled: self.prompt_caching_enabled,
            tool_approval: parse_tool_approval(self.tool_approval.as_deref()),
            ..RunPolicy::default()
        }
    }

    fn from_run_policy(policy: &RunPolicy) -> Self {
        Self {
            agent_id: policy
                .agent_id
                .clone()
                .unwrap_or_else(|| "default-agent".to_string()),
            model: policy
                .model
                .as_ref()
                .map(|route| format!("{}/{}", route.provider_id, route.model_id)),
            tools: selected_ids(&policy.tools),
            skills: selected_ids(&policy.skills),
            knowledge_bases: selected_ids(&policy.knowledge_bases),
            mcp_servers: selected_ids(&policy.mcp_servers),
            presentations: Some(policy.presentations.clone()),
            prompt_caching_enabled: policy.prompt_caching_enabled,
            tool_approval: match policy.tool_approval {
                ToolApprovalPolicy::Inherit => None,
                ToolApprovalPolicy::Auto => Some("auto".into()),
                ToolApprovalPolicy::Ask => Some("ask".into()),
                ToolApprovalPolicy::Deny => Some("deny".into()),
            },
        }
    }
}

fn optional_selection(ids: Option<Vec<String>>) -> ResourceSelection {
    ids.map(ResourceSelection::selected).unwrap_or_default()
}

fn selected_ids(selection: &ResourceSelection) -> Option<Vec<String>> {
    (selection.mode == SelectionMode::Selected).then(|| selection.ids.clone())
}

fn parse_tool_approval(value: Option<&str>) -> ToolApprovalPolicy {
    match value {
        Some("ask") => ToolApprovalPolicy::Ask,
        Some("deny") => ToolApprovalPolicy::Deny,
        Some("auto") => ToolApprovalPolicy::Auto,
        _ => ToolApprovalPolicy::Inherit,
    }
}

pub(crate) async fn load_conversation_policy(
    state: &AppState,
    user: &UserContext,
    conversation_id: &str,
) -> Option<RunPolicy> {
    load_conversation_policy_with_status(state, user, conversation_id)
        .await
        .0
}

fn conversation_policy_owner_key(user: &UserContext) -> String {
    crate::uar::runtime::actor::messages::ActorOwner::from_verified_context(user)
        .map(|owner| owner.presentation_owner_key())
        .unwrap_or_else(|_| user.user_id.clone())
}

fn conversation_policy_cache_key(user: &UserContext, conversation_id: &str) -> String {
    let key = crate::uar::persistence::tenant_storage_key(
        &conversation_policy_owner_key(user),
        conversation_id,
    );
    if crate::uar::runtime::actor::messages::ActorOwner::from_verified_context(user).is_ok() {
        // Legacy keys always start with the numeric subject length.
        format!("principal-policy:{key}")
    } else {
        key
    }
}

async fn persist_policy_for_user(
    persistence: &dyn crate::uar::persistence::PersistenceLayer,
    user: &UserContext,
    record: &ConversationPolicyRecord,
    expected: Option<&RunPolicy>,
) -> anyhow::Result<bool> {
    if crate::uar::runtime::actor::messages::ActorOwner::from_verified_context(user).is_ok() {
        persistence
            .save_principal_conversation_policy(record, expected)
            .await
    } else {
        persistence.save_conversation_policy(record).await?;
        Ok(true)
    }
}

async fn principal_policy_before_write(
    persistence: &dyn crate::uar::persistence::PersistenceLayer,
    user: &UserContext,
    conversation_id: &str,
) -> anyhow::Result<Option<RunPolicy>> {
    let Ok(owner) = crate::uar::runtime::actor::messages::ActorOwner::from_verified_context(user)
    else {
        return Ok(None);
    };
    let key = owner.presentation_owner_key();
    let record = persistence
        .load_principal_conversation_policy(&key, conversation_id)
        .await?;
    if let Some(record) = &record {
        anyhow::ensure!(
            record.owner_id == key && record.conversation_id == conversation_id,
            "Conversation policy partition mismatch"
        );
    }
    Ok(record.map(|record| record.policy))
}

fn policy_write_conflict() -> axum::response::Response {
    (
        StatusCode::CONFLICT,
        Json(json!({"error": "conversation policy changed; reload before saving"})),
    )
        .into_response()
}

async fn load_conversation_policy_with_status(
    state: &AppState,
    user: &UserContext,
    conversation_id: &str,
) -> (Option<RunPolicy>, bool) {
    let mut unavailable = false;
    let verified =
        crate::uar::runtime::actor::messages::ActorOwner::from_verified_context(user).ok();
    if let Some(persistence) = &state.persistence {
        match crate::uar::domain::policy::load_owner_scoped_conversation_policy(
            persistence.as_ref(),
            &user.user_id,
            conversation_id,
            verified.as_ref(),
        )
        .await
        {
            Ok((policy, true)) => return (policy, false),
            Ok((Some(policy), false)) => return (Some(policy), false),
            Ok((None, false)) => {}
            Err(error) => {
                unavailable = true;
                tracing::warn!(%error, %conversation_id, "failed to load conversation policy")
            }
        }
    }
    let sessions = state.agent_sessions.read().await;
    let scoped_key = conversation_policy_cache_key(user, conversation_id);
    let policy = if let Some(config) = sessions.get(&scoped_key) {
        Some(config.clone().into_run_policy())
    } else {
        sessions
            .get(&crate::uar::persistence::tenant_storage_key(
                &user.user_id,
                conversation_id,
            ))
            .cloned()
            .map(|config| {
                let mut policy = config.into_run_policy();
                policy.presentations = ResourceSelection::default();
                policy
            })
    };
    (policy, unavailable)
}

async fn policy_universe(
    state: &AppState,
    owner_id: &str,
    verified_owner: Option<&crate::uar::runtime::actor::messages::ActorOwner>,
) -> PolicyUniverse {
    let disabled_skills = match &state.settings_manager {
        Some(manager) => manager
            .list_namespace_with_meta("skill_config")
            .await
            .into_iter()
            .filter(|entry| {
                entry
                    .setting
                    .data
                    .get("enabled")
                    .and_then(serde_json::Value::as_bool)
                    == Some(false)
            })
            .filter_map(|entry| {
                entry
                    .setting
                    .key
                    .strip_prefix("skill_config.")
                    .map(str::to_owned)
            })
            .collect::<std::collections::BTreeSet<_>>(),
        None => std::collections::BTreeSet::new(),
    };
    let skills = state
        .skill_service
        .get_skills()
        .await
        .into_iter()
        .filter(|skill| skill.enabled)
        .filter(|skill| {
            let short_id = skill
                .skill_id
                .rsplit("::")
                .next()
                .unwrap_or(&skill.skill_id);
            !disabled_skills.contains(short_id) && !disabled_skills.contains(&skill.skill_id)
        })
        .map(|skill| skill.skill_id)
        .collect();
    let mut tools = state
        .mcp
        .tools()
        .iter()
        .map(|(name, _)| name.clone())
        .collect::<std::collections::BTreeSet<_>>();
    let (mcp_servers, catalog_tools) = state.run_manager.mcp_policy_inventory(None).await;
    tools.extend(catalog_tools);
    for tool in state.native_skill_registry.openai_tools_json().await {
        if let Some(name) = tool
            .get("function")
            .and_then(|function| function.get("name"))
            .and_then(serde_json::Value::as_str)
        {
            tools.insert(name.to_string());
        }
    }
    let mut knowledge_bases = std::collections::BTreeSet::new();
    if let Some(persistence) = &state.persistence
        && let Ok(records) = persistence.list_knowledge_bases(owner_id).await
    {
        for knowledge_base in records {
            knowledge_bases.insert(knowledge_base.id);
            knowledge_bases.insert(knowledge_base.name);
        }
    }
    let (presentations, presentation_warnings) =
        crate::uar::persistence::presentations::eligible_presentations(
            state.persistence.as_ref(),
            verified_owner,
        )
        .await;
    PolicyUniverse {
        skills,
        tools,
        mcp_servers,
        knowledge_bases,
        presentations,
        presentation_warnings,
    }
}

/// Resolve the effective policy that UAR will use for a conversation and turn.
///
/// Thin service-path wrapper: it builds the [`PolicyUniverse`] and the
/// conversation scope from `AppState` (the conversation scope still consults the
/// in-memory session map as a fallback), then delegates precedence resolution to
/// the transport-free [`resolve_effective_run_policy_core`] shared with the
/// embedded runtime. Behavior is identical to the previous inline resolution.
pub async fn resolve_effective_run_policy(
    state: &AppState,
    user: &UserContext,
    conversation_id: &str,
    agent: &AgentArtifact,
    turn: Option<RunPolicy>,
) -> EffectiveRunPolicy {
    let owner_id = &user.user_id;
    let verified_owner =
        crate::uar::runtime::actor::messages::ActorOwner::from_verified_context(user).ok();
    let (conversation, unavailable) =
        load_conversation_policy_with_status(state, user, conversation_id).await;
    let mut universe = policy_universe(state, owner_id, verified_owner.as_ref()).await;
    if unavailable {
        universe.presentations.clear();
        universe
            .presentation_warnings
            .push("Conversation policy could not be loaded; Presentation access is closed".into());
    }
    if state.run_manager.uses_agent_graph(agent) {
        universe.tools.extend(
            crate::uar::runtime::thread::control::AGENT_TOOL_NAMES
                .into_iter()
                .map(str::to_owned),
        );
    }
    let ctx = PolicyResolutionContext {
        settings_manager: state.settings_manager.as_deref(),
        universe,
        default_context_strategy: state.config.context_strategy.clone(),
    };
    resolve_effective_run_policy_core(ctx, agent, conversation, turn).await
}

/// POST /api/sessions/{id}/agent-config
pub async fn save_agent_session_config(
    State(state): State<AppState>,
    Extension(user): Extension<UserContext>,
    Path(session_id): Path<String>,
    Json(mut config): Json<AgentSessionConfig>,
) -> impl IntoResponse {
    let Some(persistence) = &state.persistence else {
        let key = conversation_policy_cache_key(&user, &session_id);
        let mut sessions = state.agent_sessions.write().await;
        if config.presentations.is_none() {
            config.presentations = Some(
                sessions
                    .get(&key)
                    .and_then(|saved| saved.presentations.clone())
                    .unwrap_or_default(),
            );
        }
        sessions.insert(key, config.clone());
        return (StatusCode::OK, Json(config)).into_response();
    };
    let expected =
        match principal_policy_before_write(persistence.as_ref(), &user, &session_id).await {
            Ok(expected) => expected,
            Err(error) => {
                tracing::error!(%error, %session_id, "failed to read policy write baseline");
                return (
                    StatusCode::SERVICE_UNAVAILABLE,
                    Json(json!({"error": "conversation policy unavailable"})),
                )
                    .into_response();
            }
        };
    if config.presentations.is_none() {
        let (existing, unavailable) = if expected.is_some() {
            (expected.clone(), false)
        } else {
            load_conversation_policy_with_status(&state, &user, &session_id).await
        };
        if unavailable {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({"error": "cannot preserve Presentation assignment while policy is unavailable"})),
            ).into_response();
        }
        config.presentations = Some(
            existing
                .map(|policy| policy.presentations)
                .unwrap_or_default(),
        );
    }
    let policy = config.clone().into_run_policy();
    match persist_policy_for_user(
        persistence.as_ref(),
        &user,
        &ConversationPolicyRecord::new_for_user(
            conversation_policy_owner_key(&user),
            session_id.clone(),
            policy,
        ),
        expected.as_ref(),
    )
    .await
    {
        Ok(true) => {}
        Ok(false) => return policy_write_conflict(),
        Err(error) => {
            tracing::error!(%error, %session_id, "failed to persist conversation policy");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "failed to persist conversation policy"})),
            )
                .into_response();
        }
    }
    state.agent_sessions.write().await.insert(
        conversation_policy_cache_key(&user, &session_id),
        config.clone(),
    );
    (StatusCode::OK, Json(config)).into_response()
}

/// GET /api/sessions/{id}/agent-config
pub async fn get_agent_session_config(
    State(state): State<AppState>,
    Extension(user): Extension<UserContext>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    let (policy, unavailable) =
        load_conversation_policy_with_status(&state, &user, &session_id).await;
    if unavailable {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"error": "conversation policy unavailable"})),
        )
            .into_response();
    }
    if let Some(policy) = policy {
        return (
            StatusCode::OK,
            Json(AgentSessionConfig::from_run_policy(&policy)),
        )
            .into_response();
    }
    StatusCode::NO_CONTENT.into_response()
}

#[derive(Debug, Serialize)]
struct EffectivePromptCachingResponse {
    enabled: bool,
    source: crate::uar::domain::prompt_caching::PromptCachingSource,
    session_override: Option<bool>,
    user_override: Option<bool>,
    global_default: bool,
}

/// GET /api/uar/sessions/{id}/prompt-caching
pub async fn get_effective_prompt_caching(
    State(state): State<AppState>,
    Extension(user): Extension<UserContext>,
    headers: axum::http::HeaderMap,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    let session_override = load_conversation_policy(&state, &user, &session_id)
        .await
        .and_then(|policy| policy.prompt_caching_enabled);
    let global_default = match &state.settings_manager {
        Some(manager) => manager
            .get_typed::<bool>("prompt_caching.enabled")
            .await
            .ok()
            .flatten()
            .unwrap_or(false),
        None => false,
    };
    let jwt_principal = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .filter(|value| value.starts_with("Bearer "))
        .filter(|_| !headers.contains_key("x-api-key"))
        .and_then(|_| super::user_settings::principal_storage_key(&user));
    let user_override = if let Some(principal_id) = jwt_principal {
        match state
            .user_settings_store
            .caching_enabled_for(&principal_id)
            .await
        {
            Ok(value) => value,
            Err(error) => {
                tracing::error!(%error, %session_id, "failed to resolve user prompt-caching preference");
                return (
                    StatusCode::SERVICE_UNAVAILABLE,
                    Json(json!({"error": "User settings persistence unavailable"})),
                )
                    .into_response();
            }
        }
    } else {
        None
    };
    let effective = crate::uar::domain::prompt_caching::resolve_effective_caching(
        None,
        session_override,
        user_override,
        global_default,
    );

    Json(EffectivePromptCachingResponse {
        enabled: effective.enabled,
        source: effective.source,
        session_override,
        user_override,
        global_default,
    })
    .into_response()
}

/// GET /api/sessions/{id}/effective-config
///
/// Returns the agent definition merged with any per-session overrides.
pub async fn get_effective_config(
    State(state): State<AppState>,
    Extension(user): Extension<UserContext>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    let Some(policy) = load_conversation_policy(&state, &user, &session_id).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "No agent session config found for this session"})),
        )
            .into_response();
    };

    // Load the base agent definition from persistence (or built-in defaults).
    let agent_id = policy
        .agent_id
        .clone()
        .unwrap_or_else(|| "default-agent".to_string());
    let agent = match resolve_agent_for_run(&state, &agent_id).await {
        Ok(agent) => agent,
        Err(error) => return error.into_response(),
    };
    let effective = resolve_effective_run_policy(&state, &user, &session_id, &agent, None).await;
    (
        StatusCode::OK,
        Json(json!({
            "session_id": session_id,
            "agent": agent,
            "requested_policy": policy,
            "effective_policy": effective,
        })),
    )
        .into_response()
}

/// Conversation write compatibility: old clients do not know the Presentation field.
#[derive(Debug, Deserialize)]
pub struct ConversationPolicyUpdate {
    /// Existing policy fields keep their full-replacement semantics.
    #[serde(flatten)]
    pub policy: RunPolicy,
    /// Missing/null preserves saved intent; explicit Inherit resets it.
    #[serde(default)]
    pub presentations: Option<ResourceSelection>,
}

/// PUT `/api/uar/conversations/{id}/policy`.
pub async fn save_conversation_policy(
    State(state): State<AppState>,
    Extension(user): Extension<UserContext>,
    Path(conversation_id): Path<String>,
    Json(update): Json<ConversationPolicyUpdate>,
) -> impl IntoResponse {
    let Some(persistence) = &state.persistence else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"error": "persistence not configured"})),
        )
            .into_response();
    };
    let expected =
        match principal_policy_before_write(persistence.as_ref(), &user, &conversation_id).await {
            Ok(expected) => expected,
            Err(error) => {
                tracing::error!(%error, %conversation_id, "failed to read policy write baseline");
                return (
                    StatusCode::SERVICE_UNAVAILABLE,
                    Json(json!({"error": "conversation policy unavailable"})),
                )
                    .into_response();
            }
        };
    let mut policy = update.policy;
    policy.presentations = match update.presentations {
        Some(selection) => selection,
        None => {
            let (existing, unavailable) = if expected.is_some() {
                (expected.clone(), false)
            } else {
                load_conversation_policy_with_status(&state, &user, &conversation_id).await
            };
            if unavailable {
                return (
                    StatusCode::SERVICE_UNAVAILABLE,
                    Json(json!({"error": "cannot preserve Presentation assignment while policy is unavailable"})),
                ).into_response();
            }
            existing
                .map(|policy| policy.presentations)
                .unwrap_or_default()
        }
    };
    let record = ConversationPolicyRecord::new_for_user(
        conversation_policy_owner_key(&user),
        conversation_id.clone(),
        policy,
    );
    match persist_policy_for_user(persistence.as_ref(), &user, &record, expected.as_ref()).await {
        Ok(true) => (StatusCode::OK, Json(record)).into_response(),
        Ok(false) => policy_write_conflict(),
        Err(error) => {
            tracing::error!(%error, %conversation_id, "failed to persist conversation policy");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "failed to persist conversation policy"})),
            )
                .into_response()
        }
    }
}

/// GET `/api/uar/conversations/{id}/policy`.
pub async fn get_conversation_policy(
    State(state): State<AppState>,
    Extension(user): Extension<UserContext>,
    Path(conversation_id): Path<String>,
) -> impl IntoResponse {
    let (policy, unavailable) =
        load_conversation_policy_with_status(&state, &user, &conversation_id).await;
    if unavailable {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"error": "conversation policy unavailable"})),
        )
            .into_response();
    }
    match policy {
        Some(policy) => (StatusCode::OK, Json(policy)).into_response(),
        // Missing means "inherit global/agent policy", not an exceptional
        // resource failure. Returning JSON null keeps first-run web/mobile
        // clients quiet while preserving the same typed optional contract.
        None => (StatusCode::OK, Json(serde_json::Value::Null)).into_response(),
    }
}

/// DELETE `/api/uar/conversations/{id}/policy`.
pub async fn delete_conversation_policy(
    State(state): State<AppState>,
    Extension(user): Extension<UserContext>,
    Path(conversation_id): Path<String>,
) -> impl IntoResponse {
    let key = conversation_policy_owner_key(&user);
    let verified =
        crate::uar::runtime::actor::messages::ActorOwner::from_verified_context(&user).is_ok();
    let result = if let Some(persistence) = &state.persistence {
        if verified {
            let expected =
                match principal_policy_before_write(persistence.as_ref(), &user, &conversation_id)
                    .await
                {
                    Ok(expected) => expected,
                    Err(error) => {
                        tracing::error!(%error, %conversation_id, "failed to read reset baseline");
                        return (
                            StatusCode::SERVICE_UNAVAILABLE,
                            Json(json!({"error": "conversation policy unavailable"})),
                        )
                            .into_response();
                    }
                };
            persistence
                .save_principal_conversation_policy(
                    &ConversationPolicyRecord::new_for_user(
                        &key,
                        &conversation_id,
                        RunPolicy::default(),
                    ),
                    expected.as_ref(),
                )
                .await
        } else {
            persistence
                .delete_conversation_policy(&key, &conversation_id)
                .await
                .map(|()| true)
        }
    } else if verified {
        // A verified reset needs a durable shadow to keep legacy fallback suppressed.
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"error": "persistence not configured"})),
        )
            .into_response();
    } else {
        Ok(true)
    };
    if matches!(result, Ok(false)) {
        return policy_write_conflict();
    }
    if let Err(error) = result {
        tracing::error!(%error, %conversation_id, "failed to delete conversation policy");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "failed to delete conversation policy"})),
        )
            .into_response();
    }
    state
        .agent_sessions
        .write()
        .await
        .remove(&conversation_policy_cache_key(&user, &conversation_id));
    StatusCode::NO_CONTENT.into_response()
}
