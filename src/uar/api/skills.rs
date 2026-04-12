//! Skills management API routes.
//!
//! REST endpoints for skills CRUD, matching configuration,
//! and per-agent skill bindings.

use crate::uar::domain::skills::{Skill, SkillExecutionConfig, SkillTriggers};
use crate::uar::runtime::skills::service::{SkillMatchingConfig, SkillService, SkillUpdate};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

/// Build the skills API router.
///
/// Mount under `/api/uar/skills` with `Arc<SkillService>` as state.
pub fn build_router() -> Router<Arc<SkillService>> {
    Router::new()
        // Skills CRUD
        .route("/", get(list_skills))
        .route("/", post(create_skill))
        .route("/{id}", get(get_skill))
        .route("/{id}", put(update_skill))
        .route("/{id}", delete(delete_skill))
        .route("/{id}/toggle", post(toggle_skill))
        .route("/match", get(match_skills))
        .route("/refresh", post(refresh_skills))
        // Matching configuration
        .route("/import", post(import_skill_from_disk))
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

// --- Request / Response types ---

#[derive(Serialize)]
struct SkillResponse {
    skill_id: String,
    title: String,
    description: String,
    version: String,
    enabled: bool,
    provider_id: String,
    triggers: crate::uar::domain::skills::SkillTriggers,
    preferred_tools: Vec<String>,
    prompt_overlay: String,
    execution_config: SkillExecutionConfig,
}

impl From<crate::uar::domain::skills::Skill> for SkillResponse {
    fn from(s: crate::uar::domain::skills::Skill) -> Self {
        Self {
            skill_id: s.skill_id,
            title: s.title,
            description: s.description,
            version: s.version,
            enabled: s.enabled,
            provider_id: s.provider_id,
            triggers: s.triggers,
            preferred_tools: s.preferred_tools,
            prompt_overlay: s.prompt_overlay,
            execution_config: s.execution_config,
        }
    }
}

/// Request body for creating a skill dynamically via the API.
#[derive(Deserialize)]
struct CreateSkillRequest {
    /// Human-readable skill name. The `skill_id` is derived from this.
    name: String,
    #[serde(default = "default_version")]
    version: String,
    description: String,
    #[serde(default)]
    triggers: SkillTriggers,
    #[serde(default)]
    prompt_overlay: String,
    #[serde(default)]
    preferred_tools: Vec<String>,
    #[serde(default = "default_enabled")]
    enabled: bool,
    #[serde(default)]
    execution_config: SkillExecutionConfig,
}

fn default_version() -> String {
    "1.0.0".to_string()
}

fn default_enabled() -> bool {
    true
}

#[derive(Deserialize)]
struct ToggleRequest {
    enabled: bool,
}

#[derive(Deserialize, Default)]
struct UpdateSkillRequest {
    version: Option<String>,
    title: Option<String>,
    description: Option<String>,
    triggers: Option<SkillTriggers>,
    prompt_overlay: Option<String>,
    preferred_tools: Option<Vec<String>>,
    enabled: Option<bool>,
    #[serde(default)]
    execution_config: Option<SkillExecutionConfig>,
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

// --- Import from disk types ---

#[derive(Debug, Deserialize)]
pub struct SkillImportRequest {
    pub path: String,
}

#[derive(Debug, Serialize)]
pub struct SkillImportResponse {
    pub parsed: ParsedSkillData,
    pub validation: ImportValidation,
}

#[derive(Debug, Serialize)]
pub struct ParsedSkillData {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub description: String,
    pub version: String,
    pub triggers: SkillTriggers,
    pub prompt_overlay: String,
    pub source: String,
    pub source_path: String,
    pub references: Vec<String>,
    pub detected_format: String,
}

#[derive(Debug, Serialize)]
pub struct ImportValidation {
    pub valid: bool,
    pub warnings: Vec<String>,
}

// --- Skills endpoints ---

async fn list_skills(State(service): State<Arc<SkillService>>) -> Json<Vec<SkillResponse>> {
    let skills = service.get_skills().await;
    Json(skills.into_iter().map(SkillResponse::from).collect())
}

async fn create_skill(
    State(service): State<Arc<SkillService>>,
    Json(req): Json<CreateSkillRequest>,
) -> impl IntoResponse {
    let skill_id = req.name.to_lowercase().replace(' ', "-");
    let skill = Skill {
        skill_id,
        version: req.version,
        title: req.name,
        description: req.description,
        triggers: req.triggers,
        prompt_overlay: req.prompt_overlay,
        preferred_tools: req.preferred_tools,
        mcp_config: None,
        constraints: Default::default(),
        enabled: req.enabled,
        provider_id: "api".to_string(),
        execution_config: req.execution_config,
    };

    match service.create_skill(skill).await {
        Ok(created) => (StatusCode::CREATED, Json(SkillResponse::from(created))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("{e:?}") })),
        )
            .into_response(),
    }
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
    match service.delete_skill_permanent(&id).await {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("{e:?}") })),
        )
            .into_response(),
    }
}

async fn update_skill(
    State(service): State<Arc<SkillService>>,
    Path(id): Path<String>,
    Json(req): Json<UpdateSkillRequest>,
) -> impl IntoResponse {
    let patch = SkillUpdate {
        version: req.version,
        title: req.title,
        description: req.description,
        triggers: req.triggers,
        prompt_overlay: req.prompt_overlay,
        preferred_tools: req.preferred_tools,
        enabled: req.enabled,
        execution_config: req.execution_config,
    };

    match service.update_skill(&id, patch).await {
        Ok(Some(skill)) => Json(SkillResponse::from(skill)).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("{e:?}") })),
        )
            .into_response(),
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

// --- Import from disk endpoint ---

async fn import_skill_from_disk(
    Json(req): Json<SkillImportRequest>,
) -> impl IntoResponse {
    let dir_path = PathBuf::from(&req.path);

    // Validate directory exists
    if !dir_path.is_dir() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Path is not a directory or does not exist" })),
        )
            .into_response();
    }

    let mut warnings: Vec<String> = Vec::new();

    // Scan for known subdirectories
    let mut references: Vec<String> = Vec::new();
    for subdir in &["references", "scripts", "assets"] {
        let sub = dir_path.join(subdir);
        if sub.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&sub) {
                for entry in entries.flatten() {
                    if let Some(name) = entry.file_name().to_str() {
                        references.push(format!("{subdir}/{name}"));
                    }
                }
            }
        }
    }

    // Detect format and parse
    let skill_md = dir_path.join("SKILL.md");
    let skills_md = dir_path.join("SKILLS.md");
    let claude_dir = dir_path.join(".claude");

    if skill_md.is_file() {
        // agentskills.io format
        match std::fs::read_to_string(&skill_md) {
            Ok(content) => {
                let (frontmatter, body) = parse_frontmatter(&content);
                let name = frontmatter
                    .get("name")
                    .cloned()
                    .unwrap_or_else(|| {
                        warnings.push("Missing 'name' in frontmatter; using directory name".to_string());
                        dir_path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown")
                            .to_string()
                    });
                let description = frontmatter
                    .get("description")
                    .cloned()
                    .unwrap_or_else(|| {
                        warnings.push("Missing 'description' in frontmatter".to_string());
                        String::new()
                    });
                let version = frontmatter
                    .get("version")
                    .cloned()
                    .unwrap_or_else(|| "1.0.0".to_string());

                // Parse triggers from frontmatter
                let triggers = parse_triggers_from_frontmatter(&frontmatter);

                let parsed = ParsedSkillData {
                    name,
                    title: frontmatter.get("title").cloned(),
                    description,
                    version,
                    triggers,
                    prompt_overlay: body,
                    source: "filesystem".to_string(),
                    source_path: req.path.clone(),
                    references,
                    detected_format: "agentskills-io".to_string(),
                };

                let validation = ImportValidation {
                    valid: warnings.is_empty() || warnings.iter().all(|w| !w.contains("Missing 'name'")),
                    warnings,
                };

                Json(SkillImportResponse { parsed, validation }).into_response()
            }
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Failed to read SKILL.md: {e}") })),
            )
                .into_response(),
        }
    } else if skills_md.is_file() {
        // Bundle catalog format
        match std::fs::read_to_string(&skills_md) {
            Ok(content) => {
                let (frontmatter, body) = parse_frontmatter(&content);
                let name = frontmatter
                    .get("name")
                    .cloned()
                    .unwrap_or_else(|| {
                        dir_path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown-bundle")
                            .to_string()
                    });

                warnings.push("Detected bundle catalog (SKILLS.md); contains multiple skills".to_string());

                let parsed = ParsedSkillData {
                    name,
                    title: frontmatter.get("title").cloned(),
                    description: frontmatter.get("description").cloned().unwrap_or_default(),
                    version: frontmatter.get("version").cloned().unwrap_or_else(|| "1.0.0".to_string()),
                    triggers: SkillTriggers::default(),
                    prompt_overlay: body,
                    source: "filesystem".to_string(),
                    source_path: req.path.clone(),
                    references,
                    detected_format: "bundle-catalog".to_string(),
                };

                let validation = ImportValidation {
                    valid: true,
                    warnings,
                };

                Json(SkillImportResponse { parsed, validation }).into_response()
            }
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Failed to read SKILLS.md: {e}") })),
            )
                .into_response(),
        }
    } else if claude_dir.is_dir() {
        // Claude Code plugin format
        warnings.push("Detected Claude Code plugin format (.claude/ directory)".to_string());

        let name = dir_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown-plugin")
            .to_string();

        let parsed = ParsedSkillData {
            name,
            title: None,
            description: "Claude Code plugin (auto-detected)".to_string(),
            version: "1.0.0".to_string(),
            triggers: SkillTriggers::default(),
            prompt_overlay: String::new(),
            source: "filesystem".to_string(),
            source_path: req.path.clone(),
            references,
            detected_format: "claude-code-plugin".to_string(),
        };

        let validation = ImportValidation {
            valid: true,
            warnings,
        };

        Json(SkillImportResponse { parsed, validation }).into_response()
    } else {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "No recognized skill format found. Expected SKILL.md, SKILLS.md, or .claude/ directory."
            })),
        )
            .into_response()
    }
}

/// Parse YAML frontmatter from a markdown file delimited by `---` markers.
///
/// Returns a map of key-value pairs and the markdown body.
fn parse_frontmatter(content: &str) -> (std::collections::HashMap<String, String>, String) {
    let mut map = std::collections::HashMap::new();
    let parts: Vec<&str> = content.splitn(3, "---").collect();

    if parts.len() < 3 {
        // No valid frontmatter; treat entire content as body
        return (map, content.to_string());
    }

    let yaml_section = parts[1].trim();
    let body = parts[2].trim().to_string();

    // Simple line-by-line key: value parsing
    for line in yaml_section.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim().to_string();
            let value = value.trim().trim_matches('"').trim_matches('\'').to_string();
            if !key.is_empty() {
                map.insert(key, value);
            }
        }
    }

    (map, body)
}

/// Extract trigger keywords from frontmatter.
///
/// Supports simple comma-separated values in a `triggers` or `keywords` field.
fn parse_triggers_from_frontmatter(
    frontmatter: &std::collections::HashMap<String, String>,
) -> SkillTriggers {
    let keywords = frontmatter
        .get("keywords")
        .or_else(|| frontmatter.get("triggers"))
        .map(|v| {
            v.trim_matches('[')
                .trim_matches(']')
                .split(',')
                .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let semantic = frontmatter.get("semantic").cloned();

    SkillTriggers { keywords, semantic }
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
