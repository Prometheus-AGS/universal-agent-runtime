//! A2UI REST routes.
//!
//! Mounts at `/api/uar/a2ui` in the main router.
//!
//! | Method | Path | Description |
//! |--------|------|-------------|
//! | GET | `/api/uar/a2ui/schemas` | List all registered artifact schemas |
//! | GET | `/api/uar/a2ui/schemas/{schema_id}` | Get a single schema by ID |
//! | POST | `/api/uar/runs/{run_id}/artifact-response` | Submit user response to an input artifact |
//! | POST | `/api/uar/runs/{run_id}/a2ui/test-trigger` | Trigger a real artifact input request for testing |
//! | POST | `/api/uar/runs/{run_id}/a2ui/surface-test-trigger` | Trigger a real A2UI surface state-patch event for testing (Change 20) |
//! | GET | `/api/uar/runs/{run_id}/a2ui/surface-replay` | Replay every surface state-patch published for this run so far (Change 20 late-join reattach) |

use std::sync::Arc;

use axum::{
    Extension, Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};

use super::design_systems::{
    store::SharedDesignSystemStore,
    types::{Component as LibraryComponent, Renderers},
};
use super::protocol::parse_message;
use super::realtime::{
    A2uiReplayBackbone, A2uiWireKind, InMemoryReplayBackbone, surface_message_to_state_patch,
};
use super::registry::A2uiRegistry;
use super::schema::ArtifactSchema;
use crate::uar::{
    domain::events::{ArtifactPayload, NormalizedEvent},
    persistence::PersistenceLayer,
    runtime::manager::RunManager,
    security::claims::UserContext,
};

// ── State ─────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct A2uiApiState {
    pub registry: Arc<A2uiRegistry>,
    pub run_manager: Arc<RunManager>,
    /// Durable-replay backbone for A2UI surface state patches (Change 20,
    /// `a2ui-realtime-backbone-from-flint-realtime-fabric`). In-memory for
    /// now — see `realtime.rs` module docs for the deferred
    /// `flint-realtime-fabric`-backed implementation.
    pub realtime_backbone: Arc<InMemoryReplayBackbone>,
    /// UAR-owned durable A2UI component library. Clients may promote a
    /// rendered artifact into this library, but never own the canonical data.
    pub design_system_store: SharedDesignSystemStore,
    /// Agent definitions are consulted before deleting schemas they reference.
    pub persistence: Option<Arc<dyn PersistenceLayer>>,
}

// ── Request / response types ──────────────────────────────────────────────────

/// Response submitted by the user after completing an A2UI artifact form.
#[derive(Debug, Deserialize)]
pub struct ArtifactResponsePayload {
    /// The artifact ID that generated this input request (from the SSE event).
    pub artifact_id: String,
    /// The user's response data. Shape depends on `artifact_type`:
    /// - `form`: `{ "field_name": "value", ... }`
    /// - `confirm`: `{ "accepted": true | false }`
    /// - `select`: `{ "value": "selected_option_value" }`
    /// - `text_input`: `{ "text": "user input" }`
    pub response: serde_json::Value,
    #[serde(flatten)]
    pub host: ContinuationHostInput,
}

#[derive(Debug, Default, Deserialize)]
pub struct ContinuationHostInput {
    #[serde(default)]
    pub artifact: Option<crate::uar::domain::artifact::AgentArtifact>,
    #[serde(default, alias = "runCredentials")]
    pub run_credentials: Option<Vec<crate::uar::runtime::turn::host::RunCredentialInput>>,
    #[serde(default, alias = "mcpServers")]
    pub mcp_servers: Option<Vec<crate::uar::runtime::turn::host::RunMcpServerInput>>,
    #[serde(default, alias = "workingDirectory")]
    pub working_directory: Option<std::path::PathBuf>,
    #[serde(default, alias = "reasoningEffort")]
    pub reasoning_effort: Option<String>,
}

#[derive(Debug, Serialize)]
struct ArtifactResponseAck {
    run_id: String,
    artifact_id: String,
    continuation_run_id: String,
    status: &'static str,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct A2uiActionPayload {
    pub surface_id: String,
    pub name: String,
    pub source_component_id: String,
    #[serde(default)]
    pub timestamp: Option<String>,
    #[serde(default)]
    pub context: serde_json::Value,
    #[serde(default)]
    pub a2ui_client_data_model: Option<serde_json::Value>,
    #[serde(flatten)]
    pub host: ContinuationHostInput,
}

#[derive(Debug, Serialize)]
struct A2uiContinuationAck {
    run_id: String,
    continuation_run_id: String,
    status: &'static str,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromoteComponentPayload {
    pub title: String,
    pub source: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CustomSchemaCreatePayload {
    schema: ArtifactSchema,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CustomSchemaUpdatePayload {
    expected_revision: String,
    schema: ArtifactSchema,
}

#[derive(Debug, Serialize)]
struct CustomSchemaRecord {
    schema: ArtifactSchema,
    revision: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct UpdateComponentPayload {
    expected_revision: String,
    title: String,
    source: String,
    #[serde(default)]
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RevisionQuery {
    expected_revision: String,
}

#[derive(Debug, Serialize)]
struct BuiltinComponent {
    id: &'static str,
    title: &'static str,
    category: &'static str,
    builtin: bool,
}

/// Request body for `POST /api/uar/runs/{run_id}/a2ui/test-trigger`.
///
/// Mirrors [`ArtifactPayload`] minus `artifact_id` (generated server-side)
/// and `language` (not needed for input-request artifact types).
#[derive(Debug, Deserialize)]
pub struct TestTriggerPayload {
    pub artifact_type: String,
    pub title: String,
    pub content: String,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

/// Request body for `POST /api/uar/runs/{run_id}/a2ui/surface-test-trigger`
/// (Change 20). `kind` is one of `"createSurface" | "updateComponents" |
/// "updateDataModel" | "deleteSurface"`.
#[derive(Debug, Deserialize)]
pub struct SurfaceTestTriggerPayload {
    pub surface_id: String,
    pub kind: String,
    #[serde(default)]
    pub payload: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct SurfaceTestTriggerAck {
    run_id: String,
    surface_id: String,
    status: &'static str,
}

#[derive(Debug, Serialize)]
struct TestTriggerAck {
    run_id: String,
    artifact_id: String,
    status: &'static str,
}

async fn continue_run(
    state: &A2uiApiState,
    run: &crate::uar::domain::runs::Run,
    interaction: serde_json::Value,
    user: &UserContext,
    host: ContinuationHostInput,
) -> Result<String, axum::response::Response> {
    let source_marker = run
        .context
        .get("host_resources")
        .cloned()
        .and_then(|value| serde_json::from_value(value).ok())
        .unwrap_or_default();
    let mut request = state
        .run_manager
        .prepare_interaction_request(&run.run_id, interaction, user, host.artifact)
        .await
        .map_err(|error| {
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({ "error": error })),
            )
                .into_response()
        })?;
    crate::uar::api::routes::attach_host_resources(
        &state.run_manager,
        user,
        &mut request,
        host.run_credentials,
        host.mcp_servers,
        host.working_directory,
        host.reasoning_effort,
        None,
    )
    .await
    .map_err(|error| error.into_response())?;
    crate::uar::api::routes::require_matching_host_resources(&source_marker, &request)
        .map_err(|error| error.into_response())?;
    Ok(state.run_manager.execute_request(request).await)
}

// ── Handlers ──────────────────────────────────────────────────────────────────

async fn surface_run(
    state: &A2uiApiState,
    user: &UserContext,
    run_id: &str,
) -> Result<crate::uar::domain::runs::Run, axum::response::Response> {
    let Some((run, snapshot)) = state
        .run_manager
        .presentation_run_for_user(user, run_id)
        .await
    else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "Admitted run not found"
            })),
        )
            .into_response());
    };
    if !snapshot.selection().allows_surfaces() {
        return Err((
            StatusCode::CONFLICT,
            Json(serde_json::json!({
                "error": "This run permits text output only", "code": "presentation_output_ceiling"
            })),
        )
            .into_response());
    }
    Ok(run)
}

/// `GET /api/uar/a2ui/schemas`
///
/// Returns all schemas registered in the A2UI registry (built-ins + user-defined).
async fn list_schemas(State(state): State<A2uiApiState>) -> impl IntoResponse {
    let mut schemas = state.registry.list().await;
    if let Ok(components) = state.design_system_store.list_components().await {
        schemas.extend(
            components
                .into_iter()
                .filter(|component| component.category == "schema")
                .filter_map(|component| {
                    serde_json::from_value::<ArtifactSchema>(component.schema).ok()
                }),
        );
        schemas.sort_by(|left, right| left.schema_id.cmp(&right.schema_id));
    }
    Json(schemas)
}

/// `GET /api/uar/a2ui/schemas/{schema_id}`
///
/// Returns a single schema by its `schema_id` (e.g. `a2ui/form`).
/// URL-encodes slashes: `a2ui%2Fform`.
async fn get_schema(
    State(state): State<A2uiApiState>,
    Path(schema_id): Path<String>,
) -> impl IntoResponse {
    if let Some(schema) = state.registry.get(&schema_id).await {
        return (StatusCode::OK, Json(schema)).into_response();
    }
    match state
        .design_system_store
        .get_component_by_slug(&schema_id)
        .await
    {
        Ok(Some(component)) if component.category == "schema" => {
            match serde_json::from_value::<ArtifactSchema>(component.schema) {
                Ok(schema) => (StatusCode::OK, Json(schema)).into_response(),
                Err(error) => {
                    tracing::error!(schema_id, %error, "stored A2UI schema is invalid");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({ "error": "Stored schema is invalid" })),
                    )
                        .into_response()
                }
            }
        }
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": format!("schema '{}' not found", schema_id) })),
        )
            .into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": error.to_string() })),
        )
            .into_response(),
    }
}

fn component_revision(component: &LibraryComponent) -> String {
    component
        .updated_at
        .to_rfc3339_opts(chrono::SecondsFormat::Nanos, true)
}

fn parse_revision(value: &str) -> Result<chrono::DateTime<chrono::Utc>, axum::response::Response> {
    chrono::DateTime::parse_from_rfc3339(value)
        .map(|value| value.with_timezone(&chrono::Utc))
        .map_err(|_| {
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({ "error": "expected_revision is invalid" })),
            )
                .into_response()
        })
}

async fn validate_custom_schema(
    state: &A2uiApiState,
    schema: &ArtifactSchema,
) -> Result<(), axum::response::Response> {
    let id = schema.schema_id.trim();
    if id.is_empty()
        || id.len() > 256
        || schema.title.trim().is_empty()
        || schema.title.len() > 256
        || schema.description.len() > 4096
        || !schema.json_schema.is_object()
        || serde_json::to_vec(&schema.json_schema).map_or(true, |value| value.len() > 262_144)
    {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({ "error": "Custom schema fields are invalid" })),
        )
            .into_response());
    }
    if schema.builtin || id.starts_with("a2ui/") || state.registry.contains(id).await {
        return Err((
            StatusCode::CONFLICT,
            Json(serde_json::json!({ "error": "Built-in schemas are read-only" })),
        )
            .into_response());
    }
    Ok(())
}

fn schema_record(component: &LibraryComponent) -> Result<CustomSchemaRecord, serde_json::Error> {
    Ok(CustomSchemaRecord {
        schema: serde_json::from_value(component.schema.clone())?,
        revision: component_revision(component),
    })
}

async fn list_custom_schemas(State(state): State<A2uiApiState>) -> impl IntoResponse {
    match state.design_system_store.list_components().await {
        Ok(components) => {
            let records = components
                .iter()
                .filter(|component| component.category == "schema")
                .filter_map(|component| schema_record(component).ok())
                .collect::<Vec<_>>();
            (StatusCode::OK, Json(serde_json::json!(records))).into_response()
        }
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": error.to_string() })),
        )
            .into_response(),
    }
}

async fn get_custom_schema(
    State(state): State<A2uiApiState>,
    Path(schema_id): Path<String>,
) -> impl IntoResponse {
    match state
        .design_system_store
        .get_component_by_slug(&schema_id)
        .await
    {
        Ok(Some(component)) if component.category == "schema" => match schema_record(&component) {
            Ok(record) => (StatusCode::OK, Json(serde_json::json!(record))).into_response(),
            Err(error) => {
                tracing::error!(schema_id, %error, "stored A2UI schema is invalid");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": "Stored schema is invalid" })),
                )
                    .into_response()
            }
        },
        Ok(_) => StatusCode::NOT_FOUND.into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": error.to_string() })),
        )
            .into_response(),
    }
}

async fn create_custom_schema(
    State(state): State<A2uiApiState>,
    Json(payload): Json<CustomSchemaCreatePayload>,
) -> impl IntoResponse {
    if let Err(response) = validate_custom_schema(&state, &payload.schema).await {
        return response;
    }
    match state
        .design_system_store
        .get_component_by_slug(&payload.schema.schema_id)
        .await
    {
        Ok(Some(_)) => {
            return (
                StatusCode::CONFLICT,
                Json(serde_json::json!({ "error": "A schema with this ID already exists" })),
            )
                .into_response();
        }
        Ok(None) => {}
        Err(error) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": error.to_string() })),
            )
                .into_response();
        }
    }
    let now = chrono::Utc::now();
    let component = LibraryComponent {
        id: uuid::Uuid::new_v4().to_string(),
        slug: payload.schema.schema_id.clone(),
        primitive_type: "ArtifactSchema".to_string(),
        category: "schema".to_string(),
        schema: match serde_json::to_value(&payload.schema) {
            Ok(value) => value,
            Err(error) => {
                return (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    Json(serde_json::json!({ "error": error.to_string() })),
                )
                    .into_response();
            }
        },
        description: Some(payload.schema.description.clone()),
        usage_examples: None,
        renderers: Renderers::default(),
        created_at: now,
        updated_at: now,
    };
    match state
        .design_system_store
        .put_component(component.clone())
        .await
    {
        Ok(()) => (
            StatusCode::CREATED,
            Json(serde_json::json!(
                schema_record(&component).expect("new schema serializes")
            )),
        )
            .into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": error.to_string() })),
        )
            .into_response(),
    }
}

async fn update_custom_schema(
    State(state): State<A2uiApiState>,
    Path(schema_id): Path<String>,
    Json(mut payload): Json<CustomSchemaUpdatePayload>,
) -> impl IntoResponse {
    payload.schema.schema_id = schema_id.clone();
    if let Err(response) = validate_custom_schema(&state, &payload.schema).await {
        return response;
    }
    let expected = match parse_revision(&payload.expected_revision) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let current = match state
        .design_system_store
        .get_component_by_slug(&schema_id)
        .await
    {
        Ok(Some(component)) if component.category == "schema" => component,
        Ok(_) => return StatusCode::NOT_FOUND.into_response(),
        Err(error) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": error.to_string() })),
            )
                .into_response();
        }
    };
    let updated = LibraryComponent {
        schema: serde_json::to_value(&payload.schema).expect("ArtifactSchema serializes"),
        description: Some(payload.schema.description.clone()),
        updated_at: chrono::Utc::now(),
        ..current
    };
    match state
        .design_system_store
        .replace_component_if_unchanged(expected, updated.clone())
        .await
    {
        Ok(true) => (
            StatusCode::OK,
            Json(serde_json::json!(
                schema_record(&updated).expect("schema serializes")
            )),
        )
            .into_response(),
        Ok(false) => (
            StatusCode::CONFLICT,
            Json(serde_json::json!({ "error": "Schema changed; reload before saving" })),
        )
            .into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": error.to_string() })),
        )
            .into_response(),
    }
}

async fn delete_custom_schema(
    State(state): State<A2uiApiState>,
    Path(schema_id): Path<String>,
    Query(query): Query<RevisionQuery>,
) -> impl IntoResponse {
    if state.registry.contains(&schema_id).await || schema_id.starts_with("a2ui/") {
        return (
            StatusCode::CONFLICT,
            Json(serde_json::json!({ "error": "Built-in schemas are read-only" })),
        )
            .into_response();
    }
    if let Some(persistence) = &state.persistence {
        match persistence.list_agents().await {
            Ok(agents)
                if agents.iter().any(|agent| {
                    agent
                        .ui
                        .artifacts
                        .preferred_types
                        .iter()
                        .any(|value| value == &schema_id)
                }) =>
            {
                return (
                    StatusCode::CONFLICT,
                    Json(serde_json::json!({ "error": "Schema is referenced by an agent definition" })),
                )
                    .into_response();
            }
            Ok(_) => {}
            Err(error) => {
                tracing::error!(schema_id, %error, "schema reference check failed");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": "Schema references could not be checked" })),
                )
                    .into_response();
            }
        }
    }
    let expected = match parse_revision(&query.expected_revision) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let current = match state
        .design_system_store
        .get_component_by_slug(&schema_id)
        .await
    {
        Ok(Some(component)) if component.category == "schema" => component,
        Ok(_) => return StatusCode::NOT_FOUND.into_response(),
        Err(error) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": error.to_string() })),
            )
                .into_response();
        }
    };
    match state
        .design_system_store
        .delete_component_if_unchanged(&current.id, expected)
        .await
    {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => (
            StatusCode::CONFLICT,
            Json(serde_json::json!({ "error": "Schema changed; reload before deleting" })),
        )
            .into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": error.to_string() })),
        )
            .into_response(),
    }
}

/// `POST /api/uar/runs/{run_id}/artifact-response`
///
/// Receives the user's response to an `agui.artifact_input_request` event and
/// injects it back into the paused agent run as a tool result, allowing the
/// agent to continue execution.
async fn submit_artifact_response(
    State(state): State<A2uiApiState>,
    Extension(user): Extension<UserContext>,
    Path(run_id): Path<String>,
    Json(payload): Json<ArtifactResponsePayload>,
) -> impl IntoResponse {
    // Verify the run exists and is active.
    let run = match surface_run(&state, &user, &run_id).await {
        Ok(run) => run,
        Err(response) => return response,
    };

    // Preserve the response on the source run for every live/late subscriber.
    let tool_result_event = NormalizedEvent::ToolEnd {
        run_id: run.run_id.clone(),
        call_index: 0,
        tool_call_id: payload.artifact_id.clone(),
        tool: "a2ui.collect_input".to_string(),
        output: payload.response.clone(),
        ok: true,
    };

    state
        .run_manager
        .emit_to_run(&run.run_id, tool_result_event)
        .await;

    // Resume agent execution through a real continuation run in the same
    // conversation. The previous implementation stopped at the synthetic
    // ToolEnd above and therefore never actually returned control to the LLM.
    let interaction = serde_json::json!({
        "artifactId": payload.artifact_id,
        "response": payload.response,
    });
    let continuation_run_id =
        match continue_run(&state, &run, interaction, &user, payload.host).await {
            Ok(value) => value,
            Err(response) => return response,
        };

    (
        StatusCode::OK,
        Json(ArtifactResponseAck {
            run_id: run.run_id,
            artifact_id: payload.artifact_id,
            continuation_run_id,
            status: "accepted",
        }),
    )
        .into_response()
}

fn message_values(value: serde_json::Value) -> Result<Vec<serde_json::Value>, String> {
    match value {
        serde_json::Value::Array(values) => Ok(values),
        serde_json::Value::Object(mut value) if value.contains_key("messages") => value
            .remove("messages")
            .and_then(|value| value.as_array().cloned())
            .ok_or_else(|| "messages must be an array".to_string()),
        value @ serde_json::Value::Object(_) => Ok(vec![value]),
        _ => Err("A2UI payload must be a message, array, or { messages } object".to_string()),
    }
}

async fn publish_messages(
    state: &A2uiApiState,
    run_id: &str,
    body: serde_json::Value,
) -> Result<(String, Vec<String>), String> {
    let values = message_values(body)?;
    if values.is_empty() {
        return Err("at least one A2UI message is required".to_string());
    }
    let validated = values
        .into_iter()
        .map(parse_message)
        .collect::<Result<Vec<_>, _>>()?;
    let mut surface_ids = Vec::new();
    let mut source_lines = Vec::new();
    for message in validated {
        if !surface_ids.contains(&message.surface_id) {
            surface_ids.push(message.surface_id.clone());
        }
        let op = surface_message_to_state_patch(&message.surface_id, message.kind, message.payload);
        state.realtime_backbone.publish(run_id, op.clone());
        state
            .run_manager
            .emit_to_run(
                run_id,
                NormalizedEvent::StatePatch {
                    run_id: run_id.to_string(),
                    patch: vec![op],
                },
            )
            .await;
        source_lines.push(message.raw.to_string());
    }

    let artifact_id = format!("a2ui:{}", uuid::Uuid::new_v4());
    state
        .run_manager
        .emit_to_run(
            run_id,
            NormalizedEvent::ArtifactDisplay {
                run_id: run_id.to_string(),
                artifact: ArtifactPayload {
                    artifact_id: artifact_id.clone(),
                    artifact_type: "a2ui".to_string(),
                    title: "Interactive UI".to_string(),
                    content: source_lines.join("\n"),
                    language: Some("application/a2ui+json".to_string()),
                    metadata: serde_json::json!({
                        "profile": "uar.a2ui/1",
                        "surfaceIds": surface_ids,
                    }),
                },
            },
        )
        .await;
    Ok((artifact_id, surface_ids))
}

/// Accept validated A2UI messages from the orchestrator/tool loop and publish
/// both replayable state patches and a directly renderable AG-UI artifact.
async fn submit_messages(
    State(state): State<A2uiApiState>,
    Extension(user): Extension<UserContext>,
    Path(run_id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Err(response) = surface_run(&state, &user, &run_id).await {
        return response;
    }
    match publish_messages(&state, &run_id, body).await {
        Ok((artifact_id, surface_ids)) => (
            StatusCode::ACCEPTED,
            Json(serde_json::json!({
                "runId": run_id,
                "artifactId": artifact_id,
                "surfaceIds": surface_ids,
                "status": "published",
            })),
        )
            .into_response(),
        Err(error) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": error })),
        )
            .into_response(),
    }
}

fn json_contains_string(value: &serde_json::Value, expected: &str) -> bool {
    match value {
        serde_json::Value::String(value) => value == expected,
        serde_json::Value::Array(values) => values
            .iter()
            .any(|value| json_contains_string(value, expected)),
        serde_json::Value::Object(values) => values
            .values()
            .any(|value| json_contains_string(value, expected)),
        _ => false,
    }
}

/// Receive a standard A2UI action, validate it against the published surface,
/// persist/broadcast the action state, and resume the agent in a continuation run.
async fn submit_action(
    State(state): State<A2uiApiState>,
    Extension(user): Extension<UserContext>,
    Path(run_id): Path<String>,
    Json(payload): Json<A2uiActionPayload>,
) -> impl IntoResponse {
    let run = match surface_run(&state, &user, &run_id).await {
        Ok(run) => run,
        Err(response) => return response,
    };
    let surface_path = format!("/a2ui/surfaces/{}", payload.surface_id);
    let replay = state.realtime_backbone.replay(&run_id);
    let surface_exists = replay
        .iter()
        .any(|op| op.path.starts_with(&surface_path) && op.op != "remove");
    let action_exists = replay.iter().any(|op| {
        op.path.starts_with(&surface_path)
            && op
                .value
                .as_ref()
                .is_some_and(|value| json_contains_string(value, &payload.name))
    });
    if !surface_exists || !action_exists {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({
                "error": "action is not declared by the active A2UI surface"
            })),
        )
            .into_response();
    }

    let interaction = serde_json::json!({
        "surfaceId": payload.surface_id,
        "name": payload.name,
        "sourceComponentId": payload.source_component_id,
        "timestamp": payload.timestamp,
        "context": payload.context,
        "a2uiClientDataModel": payload.a2ui_client_data_model,
    });
    let action_op = crate::uar::domain::events::StatePatchOp {
        op: "add".to_string(),
        path: format!("/a2ui/actions/{}", uuid::Uuid::new_v4()),
        value: Some(interaction.clone()),
    };
    state.realtime_backbone.publish(&run_id, action_op.clone());
    state
        .run_manager
        .emit_to_run(
            &run_id,
            NormalizedEvent::StatePatch {
                run_id: run_id.clone(),
                patch: vec![action_op],
            },
        )
        .await;
    let continuation_run_id =
        match continue_run(&state, &run, interaction, &user, payload.host).await {
            Ok(value) => value,
            Err(response) => return response,
        };
    (
        StatusCode::ACCEPTED,
        Json(A2uiContinuationAck {
            run_id,
            continuation_run_id,
            status: "continued",
        }),
    )
        .into_response()
}

async fn list_library_components(State(state): State<A2uiApiState>) -> impl IntoResponse {
    match state.design_system_store.list_components().await {
        Ok(components) => (
            StatusCode::OK,
            Json(serde_json::json!(
                components
                    .into_iter()
                    .filter(|component| component.category != "schema")
                    .collect::<Vec<_>>()
            )),
        )
            .into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": error.to_string() })),
        )
            .into_response(),
    }
}

async fn list_builtin_components() -> impl IntoResponse {
    Json([
        BuiltinComponent {
            id: "Text",
            title: "Text",
            category: "content",
            builtin: true,
        },
        BuiltinComponent {
            id: "Button",
            title: "Button",
            category: "action",
            builtin: true,
        },
        BuiltinComponent {
            id: "TextField",
            title: "Text field",
            category: "input",
            builtin: true,
        },
        BuiltinComponent {
            id: "CheckBox",
            title: "Checkbox",
            category: "input",
            builtin: true,
        },
        BuiltinComponent {
            id: "ChoicePicker",
            title: "Choice picker",
            category: "input",
            builtin: true,
        },
        BuiltinComponent {
            id: "Row",
            title: "Row",
            category: "layout",
            builtin: true,
        },
        BuiltinComponent {
            id: "Column",
            title: "Column",
            category: "layout",
            builtin: true,
        },
        BuiltinComponent {
            id: "Card",
            title: "Card",
            category: "layout",
            builtin: true,
        },
        BuiltinComponent {
            id: "Divider",
            title: "Divider",
            category: "layout",
            builtin: true,
        },
    ])
}

fn parse_component_source(
    source: &str,
) -> Result<Vec<super::protocol::ValidatedA2uiMessage>, String> {
    let values = source
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            serde_json::from_str::<serde_json::Value>(line).map_err(|error| error.to_string())
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(parse_message)
        .collect::<Result<Vec<_>, _>>()?;
    if values.is_empty() {
        return Err("A2UI source is empty".to_string());
    }
    Ok(values)
}

fn component_slug(title: &str) -> String {
    title
        .to_ascii_lowercase()
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

async fn get_library_component(
    State(state): State<A2uiApiState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.design_system_store.get_component(&id).await {
        Ok(Some(component)) if component.category != "schema" => {
            (StatusCode::OK, Json(serde_json::json!(component))).into_response()
        }
        Ok(_) => StatusCode::NOT_FOUND.into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": error.to_string() })),
        )
            .into_response(),
    }
}

async fn update_library_component(
    State(state): State<A2uiApiState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateComponentPayload>,
) -> impl IntoResponse {
    let expected = match parse_revision(&payload.expected_revision) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let messages = match parse_component_source(&payload.source) {
        Ok(value) => value,
        Err(error) => {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({ "error": error })),
            )
                .into_response();
        }
    };
    let current = match state.design_system_store.get_component(&id).await {
        Ok(Some(component)) if component.category != "schema" => component,
        Ok(_) => return StatusCode::NOT_FOUND.into_response(),
        Err(error) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": error.to_string() })),
            )
                .into_response();
        }
    };
    let slug = component_slug(&payload.title);
    if slug.is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({ "error": "Component title is required" })),
        )
            .into_response();
    }
    match state.design_system_store.get_component_by_slug(&slug).await {
        Ok(Some(existing)) if existing.id != current.id => {
            return (
                StatusCode::CONFLICT,
                Json(serde_json::json!({ "error": "A component with this title already exists" })),
            )
                .into_response();
        }
        Ok(_) => {}
        Err(error) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": error.to_string() })),
            )
                .into_response();
        }
    }
    let updated = LibraryComponent {
        slug,
        schema: serde_json::json!({
            "profile": "uar.a2ui/1",
            "messages": messages.iter().map(|message| &message.raw).collect::<Vec<_>>(),
        }),
        description: payload.description,
        usage_examples: Some(serde_json::json!({ "source": payload.source })),
        updated_at: chrono::Utc::now(),
        ..current
    };
    match state
        .design_system_store
        .replace_component_if_unchanged(expected, updated.clone())
        .await
    {
        Ok(true) => (StatusCode::OK, Json(serde_json::json!(updated))).into_response(),
        Ok(false) => (
            StatusCode::CONFLICT,
            Json(serde_json::json!({ "error": "Component changed; reload before saving" })),
        )
            .into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": error.to_string() })),
        )
            .into_response(),
    }
}

async fn delete_library_component(
    State(state): State<A2uiApiState>,
    Path(id): Path<String>,
    Query(query): Query<RevisionQuery>,
) -> impl IntoResponse {
    let expected = match parse_revision(&query.expected_revision) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let current = match state.design_system_store.get_component(&id).await {
        Ok(Some(component)) if component.category != "schema" => component,
        Ok(_) => return StatusCode::NOT_FOUND.into_response(),
        Err(error) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": error.to_string() })),
            )
                .into_response();
        }
    };
    match state
        .design_system_store
        .component_reference_count(&id)
        .await
    {
        Ok(0) => {}
        Ok(_) => {
            return (
                StatusCode::CONFLICT,
                Json(serde_json::json!({ "error": "Component is referenced by a design system" })),
            )
                .into_response();
        }
        Err(error) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": error.to_string() })),
            )
                .into_response();
        }
    }
    match state
        .design_system_store
        .delete_component_if_unchanged(&current.id, expected)
        .await
    {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => (
            StatusCode::CONFLICT,
            Json(serde_json::json!({ "error": "Component changed; reload before deleting" })),
        )
            .into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": error.to_string() })),
        )
            .into_response(),
    }
}

async fn promote_library_component(
    State(state): State<A2uiApiState>,
    Json(payload): Json<PromoteComponentPayload>,
) -> impl IntoResponse {
    let values = match parse_component_source(&payload.source) {
        Ok(values) => values,
        Err(error) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": error })),
            )
                .into_response();
        }
    };
    let slug = component_slug(&payload.title);
    if !slug.is_empty() {
        match state.design_system_store.get_component_by_slug(&slug).await {
            Ok(Some(_)) => {
                return (
                    StatusCode::CONFLICT,
                    Json(serde_json::json!({ "error": "A component with this title already exists" })),
                )
                    .into_response();
            }
            Ok(None) => {}
            Err(error) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": error.to_string() })),
                )
                    .into_response();
            }
        }
    }
    let now = chrono::Utc::now();
    let component = LibraryComponent {
        id: uuid::Uuid::new_v4().to_string(),
        slug: if slug.is_empty() {
            format!("a2ui-{}", uuid::Uuid::new_v4())
        } else {
            slug
        },
        primitive_type: "A2uiSurface".to_string(),
        category: "artifact".to_string(),
        schema: serde_json::json!({
            "profile": "uar.a2ui/1",
            "messages": values.iter().map(|message| &message.raw).collect::<Vec<_>>(),
        }),
        description: payload.description,
        usage_examples: Some(serde_json::json!({ "source": payload.source })),
        renderers: Renderers {
            react: true,
            flutter: true,
            htmx: false,
        },
        created_at: now,
        updated_at: now,
    };
    match state
        .design_system_store
        .put_component(component.clone())
        .await
    {
        Ok(()) => (StatusCode::CREATED, Json(serde_json::json!(component))).into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": error.to_string() })),
        )
            .into_response(),
    }
}

/// `POST /api/uar/runs/{run_id}/a2ui/test-trigger`
///
/// Emits a real `ArtifactInputRequest` event onto the given run's SSE stream,
/// using the exact same [`RunManager::emit_to_run`] path a live agent tool
/// call uses — for testing/validating the A2UI round-trip on demand rather
/// than waiting for an agent to naturally request input.
async fn test_trigger_artifact(
    State(state): State<A2uiApiState>,
    Extension(user): Extension<UserContext>,
    Path(run_id): Path<String>,
    Json(payload): Json<TestTriggerPayload>,
) -> impl IntoResponse {
    let run = match surface_run(&state, &user, &run_id).await {
        Ok(run) => run,
        Err(response) => return response,
    };

    let artifact_id = uuid::Uuid::new_v4().to_string();
    let event = NormalizedEvent::ArtifactInputRequest {
        run_id: run.run_id.clone(),
        artifact: ArtifactPayload {
            artifact_id: artifact_id.clone(),
            artifact_type: payload.artifact_type,
            title: payload.title,
            content: payload.content,
            language: None,
            metadata: payload.metadata,
        },
    };

    state.run_manager.emit_to_run(&run.run_id, event).await;

    (
        StatusCode::OK,
        Json(TestTriggerAck {
            run_id: run.run_id,
            artifact_id,
            status: "triggered",
        }),
    )
        .into_response()
}

/// `POST /api/uar/runs/{run_id}/a2ui/surface-test-trigger` (Change 20).
///
/// Converts the given A2UI surface message into a
/// `NormalizedEvent::StatePatch` (via [`surface_message_to_state_patch`]),
/// publishes it to the durable replay backbone (so a subsequent
/// `GET .../surface-replay` call — or, once wired, a
/// `flint-realtime-fabric` late-joining subscriber — sees it), and emits it
/// onto the run's live SSE broadcast via the same [`RunManager::emit_to_run`]
/// path every other run event uses, so every currently-connected client
/// converges on it immediately.
async fn surface_test_trigger(
    State(state): State<A2uiApiState>,
    Extension(user): Extension<UserContext>,
    Path(run_id): Path<String>,
    Json(payload): Json<SurfaceTestTriggerPayload>,
) -> impl IntoResponse {
    let run = match surface_run(&state, &user, &run_id).await {
        Ok(run) => run,
        Err(response) => return response,
    };

    let kind = match payload.kind.as_str() {
        "createSurface" => A2uiWireKind::CreateSurface,
        "updateComponents" => A2uiWireKind::UpdateComponents,
        "updateDataModel" => A2uiWireKind::UpdateDataModel,
        "deleteSurface" => A2uiWireKind::DeleteSurface,
        other => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": format!(
                        "unknown kind '{other}' -- expected one of createSurface, updateComponents, updateDataModel, deleteSurface"
                    )
                })),
            )
                .into_response();
        }
    };

    let op = surface_message_to_state_patch(&payload.surface_id, kind, payload.payload);

    state.realtime_backbone.publish(&run.run_id, op.clone());

    state
        .run_manager
        .emit_to_run(
            &run.run_id,
            NormalizedEvent::StatePatch {
                run_id: run.run_id.clone(),
                patch: vec![op],
            },
        )
        .await;

    (
        StatusCode::OK,
        Json(SurfaceTestTriggerAck {
            run_id: run.run_id,
            surface_id: payload.surface_id,
            status: "triggered",
        }),
    )
        .into_response()
}

/// `GET /api/uar/runs/{run_id}/a2ui/surface-replay` (Change 20).
///
/// Returns every A2UI surface state-patch op published for this run so far,
/// in publish order — the "late-join reattach" read path: a client that
/// connects to the run's SSE stream after some surface updates already
/// happened calls this once to catch up, then relies on the live SSE
/// broadcast for anything published from that point on.
async fn surface_replay(
    State(state): State<A2uiApiState>,
    Extension(user): Extension<UserContext>,
    Path(run_id): Path<String>,
) -> impl IntoResponse {
    if state
        .run_manager
        .presentation_run_for_user(&user, &run_id)
        .await
        .is_none()
    {
        return StatusCode::NOT_FOUND.into_response();
    }
    let ops = state.realtime_backbone.replay(&run_id);
    (StatusCode::OK, Json(ops)).into_response()
}

// ── Router builders ───────────────────────────────────────────────────────────

/// Build the A2UI schema listing router (mounted at `/api/uar/a2ui`).
pub fn build_schema_router() -> Router<A2uiApiState> {
    Router::new()
        .route("/schemas", get(list_schemas))
        .route("/schemas/{schema_id}", get(get_schema))
        .route(
            "/custom-schemas",
            get(list_custom_schemas).post(create_custom_schema),
        )
        .route(
            "/custom-schemas/{schema_id}",
            get(get_custom_schema)
                .put(update_custom_schema)
                .delete(delete_custom_schema),
        )
        .route("/components/builtins", get(list_builtin_components))
        .route(
            "/components",
            get(list_library_components).post(promote_library_component),
        )
        .route(
            "/components/{id}",
            get(get_library_component)
                .put(update_library_component)
                .delete(delete_library_component),
        )
}

/// Build the run-adjacent A2UI router (merged into the `/api/uar` router).
///
/// This is separate from the schema router because it shares the path prefix
/// with the main runs API.
pub fn build_response_router() -> Router<A2uiApiState> {
    Router::new()
        .route(
            "/runs/{run_id}/artifact-response",
            post(submit_artifact_response),
        )
        .route(
            "/runs/{run_id}/a2ui/test-trigger",
            post(test_trigger_artifact),
        )
        .route(
            "/runs/{run_id}/a2ui/surface-test-trigger",
            post(surface_test_trigger),
        )
        .route(
            "/runs/{run_id}/a2ui/surface-replay",
            get(surface_replay),
        )
        .route("/runs/{run_id}/a2ui/messages", post(submit_messages))
        .route("/runs/{run_id}/a2ui/actions", post(submit_action))
}
