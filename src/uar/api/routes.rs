use crate::uar::{
    api::sse::{build_agui_replay_snapshot, build_sse_response},
    domain::artifact::AgentArtifact,
    runtime::{
        checkpoint::Checkpoint,
        manager::{RunManager, StreamEvent},
    },
    security::claims::UserContext,
};
use axum::{
    Extension, Json, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio_stream::StreamExt;

pub fn build_router() -> Router<Arc<RunManager>> {
    Router::new()
        .route("/runs", post(create_run))
        .route("/runs/{id}/stream", get(stream_run))
        .route("/runs/{run_id}/tool-approval", post(api_tool_approval))
        .route("/runs/{run_id}/cancel", post(api_cancel_run))
        .route(
            "/sessions/{session_id}/cancel",
            post(api_cancel_session_run),
        )
        .route("/runs/{run_id}/checkpoints", get(list_checkpoints))
        .route("/runs/{run_id}/resume", post(resume_run))
        .route(
            "/runs/{run_id}/resume/{checkpoint_id}",
            post(resume_run_from_checkpoint),
        )
        .route("/resolve-model", get(resolve_model))
}

#[derive(Deserialize)]
struct CreateRunRequest {
    artifact: AgentArtifact,
    input: String,
    session_id: Option<String>,
    #[serde(default)]
    run_credentials: Option<Vec<crate::uar::runtime::turn::host::RunCredentialInput>>,
    #[serde(default)]
    mcp_servers: Option<Vec<crate::uar::runtime::turn::host::RunMcpServerInput>>,
    #[serde(default)]
    working_directory: Option<std::path::PathBuf>,
    #[serde(default)]
    reasoning_effort: Option<String>,
    #[serde(default)]
    history: Option<crate::uar::runtime::turn::host::HostHistoryInput>,
    #[serde(default)]
    skill_attachments: Vec<String>,
    #[serde(flatten)]
    presentation_negotiation: crate::uar::a2ui::presentation_selection::PresentationNegotiation,
}

#[derive(serde::Serialize)]
struct CreateRunResponse {
    run_id: String,
    stream_url: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    activation_failures: Vec<crate::uar::runtime::skills::activation::ActivationFailure>,
    history: crate::uar::runtime::turn::host::HistorySeedStatus,
    seeded_messages: usize,
}

#[derive(Debug)]
pub(crate) struct RunApiError {
    status: StatusCode,
    code: &'static str,
    message: &'static str,
}

impl From<crate::uar::runtime::turn::host::HostInputError> for RunApiError {
    fn from(error: crate::uar::runtime::turn::host::HostInputError) -> Self {
        Self {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            code: error.code,
            message: error.message,
        }
    }
}

impl IntoResponse for RunApiError {
    fn into_response(self) -> axum::response::Response {
        (
            self.status,
            Json(serde_json::json!({ "code": self.code, "error": self.message })),
        )
            .into_response()
    }
}

fn canonical_working_directory(
    requested: Option<std::path::PathBuf>,
) -> Result<Option<std::path::PathBuf>, RunApiError> {
    let Some(requested) = requested else {
        return Ok(None);
    };
    if !requested.is_absolute() {
        return Err(crate::uar::runtime::turn::host::HostInputError::new(
            "working_directory_invalid",
            "working_directory must name an absolute non-root directory",
        )
        .into());
    }
    let canonical = std::fs::canonicalize(requested).map_err(|_| {
        RunApiError::from(crate::uar::runtime::turn::host::HostInputError::new(
            "working_directory_invalid",
            "working_directory must name an existing directory",
        ))
    })?;
    if !canonical.is_dir() || canonical.parent().is_none() {
        return Err(crate::uar::runtime::turn::host::HostInputError::new(
            "working_directory_invalid",
            "working_directory must name an absolute non-root directory",
        )
        .into());
    }
    Ok(Some(canonical))
}

pub(crate) fn attach_host_resources(
    request: &mut crate::uar::runtime::turn::RunExecutionRequest,
    run_credentials: Option<Vec<crate::uar::runtime::turn::host::RunCredentialInput>>,
    mcp_servers: Option<Vec<crate::uar::runtime::turn::host::RunMcpServerInput>>,
    working_directory: Option<std::path::PathBuf>,
    reasoning_effort: Option<String>,
    history: Option<crate::uar::runtime::turn::host::HostHistoryInput>,
) -> Result<(), RunApiError> {
    if working_directory.is_some() {
        request.working_directory = canonical_working_directory(working_directory)?;
    }
    if let Some(reasoning_effort) = reasoning_effort {
        request.reasoning_effort = Some(match reasoning_effort.as_str() {
            "none" => crate::config::ReasoningEffort::None,
            "low" => crate::config::ReasoningEffort::Low,
            "medium" => crate::config::ReasoningEffort::Medium,
            "high" => crate::config::ReasoningEffort::High,
            "max" => crate::config::ReasoningEffort::Max,
            _ => {
                return Err(crate::uar::runtime::turn::host::HostInputError::new(
                    "reasoning_effort_invalid",
                    "reasoning_effort must be none, low, medium, high, or max",
                )
                .into());
            }
        });
    }
    if let Some(history) = history {
        request.host_history = Some(history.validate(request.session_id.as_deref())?);
    }
    if let Some(credentials) = run_credentials {
        let credentials =
            crate::uar::runtime::turn::host::RunCredentials::from_inputs(credentials)?;
        if !credentials.contains(&request.artifact.policy.provider.default.provider) {
            return Err(crate::uar::runtime::turn::host::HostInputError::new(
                "run_credential_provider_unavailable",
                "agent default provider has no run credential",
            )
            .into());
        }
        if request
            .artifact
            .policy
            .provider
            .fallbacks
            .iter()
            .any(|fallback| !credentials.contains(&fallback.provider))
        {
            return Err(crate::uar::runtime::turn::host::HostInputError::new(
                "run_credential_provider_unavailable",
                "agent fallback provider has no run credential",
            )
            .into());
        }
        request.host_resources_marker.credential_providers =
            credentials.provider_ids().into_iter().collect();
        request.host_secret_scrubber.extend(credentials.scrubber());
        request.run_credentials = Some(credentials);
    }
    if let Some(servers) = mcp_servers {
        let servers = crate::uar::runtime::turn::host::RunMcpServers::from_inputs(servers)?;
        let server_names = servers.names();
        if let Some(value) = request.artifact.extensions.get("mcp_servers")
            && !value.is_null()
        {
            let declared = serde_json::from_value::<crate::uar::compiler::ir::McpServersSection>(
                value.clone(),
            )
            .map_err(|_| {
                RunApiError::from(crate::uar::runtime::turn::host::HostInputError::new(
                    "mcp_server_not_run_scoped",
                    "agent MCP selection is invalid",
                ))
            })?;
            if declared
                .servers
                .iter()
                .any(|server| !server_names.contains(&server.id))
            {
                return Err(crate::uar::runtime::turn::host::HostInputError::new(
                    "mcp_server_not_run_scoped",
                    "agent selects an MCP server outside this run",
                )
                .into());
            }
        }
        let owner = request.verified_owner.clone().ok_or_else(|| RunApiError {
            status: StatusCode::UNAUTHORIZED,
            code: "run_mcp_server_invalid",
            message: "run-scoped MCP requires a verified principal",
        })?;
        let cwd = request
            .working_directory
            .clone()
            .or_else(|| std::env::current_dir().ok())
            .ok_or_else(|| {
                RunApiError::from(crate::uar::runtime::turn::host::HostInputError::new(
                    "working_directory_invalid",
                    "working directory is unavailable",
                ))
            })?;
        request.host_resources_marker.mcp_servers = server_names.into_iter().collect();
        request.host_secret_scrubber.extend(servers.scrubber());
        request.mcp_resources = Some(servers.resources(owner, cwd)?);
    }
    request.host_resources_marker.artifact_inline = true;
    Ok(())
}

pub(crate) fn require_matching_host_resources(
    marker: &crate::uar::runtime::turn::host::HostResourcesMarker,
    request: &crate::uar::runtime::turn::RunExecutionRequest,
) -> Result<(), RunApiError> {
    if marker.artifact_inline != request.host_resources_marker.artifact_inline {
        return Err(crate::uar::runtime::turn::host::HostInputError::new(
            "run_artifact_required",
            "continuation must reattach the source run artifact",
        )
        .into());
    }
    if marker.credential_providers != request.host_resources_marker.credential_providers {
        return Err(crate::uar::runtime::turn::host::HostInputError::new(
            "run_credential_required",
            "resume must reattach the source run provider credentials",
        )
        .into());
    }
    if marker.mcp_servers != request.host_resources_marker.mcp_servers {
        return Err(crate::uar::runtime::turn::host::HostInputError::new(
            "run_mcp_servers_required",
            "resume must reattach the source run MCP servers",
        )
        .into());
    }
    Ok(())
}

fn inherit_host_context(
    source: &crate::uar::domain::runs::Run,
    request: &mut crate::uar::runtime::turn::RunExecutionRequest,
) {
    let Some(context) = source.context.get("host_context") else {
        return;
    };
    request.working_directory = context
        .get("working_directory")
        .and_then(serde_json::Value::as_str)
        .map(std::path::PathBuf::from);
    request.reasoning_effort = context
        .get("reasoning_effort")
        .cloned()
        .and_then(|value| serde_json::from_value(value).ok());
}

#[derive(Deserialize)]
struct StreamParams {
    last_event_id: Option<u64>,
    stream_mode: Option<String>,
}

async fn create_run(
    State(manager): State<Arc<RunManager>>,
    Extension(user): Extension<UserContext>,
    Json(req): Json<CreateRunRequest>,
) -> Result<Json<CreateRunResponse>, RunApiError> {
    let mut request = crate::uar::runtime::turn::RunExecutionRequest::new(req.artifact, req.input)
        .with_user_context(&user)
        .map_err(|_| RunApiError {
            status: StatusCode::UNAUTHORIZED,
            code: "principal_invalid",
            message: "run principal is invalid",
        })?;
    request.session_id = req.session_id;
    request.skill_attachments = req.skill_attachments;
    request.presentation_negotiation = req.presentation_negotiation;
    attach_host_resources(
        &mut request,
        req.run_credentials,
        req.mcp_servers,
        req.working_directory,
        req.reasoning_effort,
        req.history,
    )?;
    let run_id = manager.execute_request(request).await;
    let run_context = manager
        .get_run(&run_id)
        .await
        .map(|run| run.context)
        .unwrap_or_default();
    let activation_failures = run_context
        .get("activation_failures")
        .cloned()
        .and_then(|value| serde_json::from_value(value).ok())
        .unwrap_or_default();
    let history = run_context
        .get("history")
        .cloned()
        .and_then(|value| serde_json::from_value(value).ok())
        .unwrap_or(crate::uar::runtime::turn::host::HistorySeedStatus::None);
    let seeded_messages = run_context
        .get("seeded_messages")
        .and_then(serde_json::Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or_default();
    Ok(Json(CreateRunResponse {
        run_id: run_id.clone(),
        stream_url: format!("/api/uar/runs/{run_id}/stream"),
        activation_failures,
        history,
        seeded_messages,
    }))
}

async fn stream_run(
    State(manager): State<Arc<RunManager>>,
    Extension(user): Extension<UserContext>,
    Path(run_id): Path<String>,
    Query(params): Query<StreamParams>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if manager.get_run_for_context(&user, &run_id).await.is_none() {
        return StatusCode::NOT_FOUND.into_response();
    }
    let Some(rx) = manager.subscribe(&run_id).await else {
        return axum::http::StatusCode::NOT_FOUND.into_response();
    };

    let last_event_id = params.last_event_id.or_else(|| {
        headers
            .get("last-event-id")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse::<u64>().ok())
    });

    let agui_spec = params.stream_mode.as_deref() == Some("agui_spec");
    let (replay, replay_max_id, replay_snapshot) = if agui_spec {
        let full_history = manager
            .history_since(&run_id, None)
            .await
            .unwrap_or_default();
        let cursor =
            last_event_id.unwrap_or_else(|| full_history.last().map_or(0, |event| event.id));
        let mut replay = full_history
            .iter()
            .filter(|event| event.id > cursor)
            .cloned()
            .collect::<Vec<_>>();
        if replay
            .first()
            .is_some_and(|event| event.id > cursor.saturating_add(1))
        {
            replay = vec![unrecoverable_stream_gap(&run_id, cursor)];
        }
        let replay_max_id = replay.last().map_or(cursor, |event| event.id);
        let snapshot = build_agui_replay_snapshot(&run_id, &full_history, cursor);
        (replay, replay_max_id, Some(snapshot))
    } else {
        let replay = manager
            .history_since(&run_id, last_event_id)
            .await
            .unwrap_or_default();
        let replay_max_id = replay.last().map_or(0, |event| event.id);
        (replay, replay_max_id, None)
    };

    let live_manager = Arc::clone(&manager);
    let live_run_id = run_id.clone();
    let live_stream = async_stream::stream! {
        let mut rx = rx;
        let mut last_id = replay_max_id;
        loop {
            match rx.recv().await {
                Ok(event) if event.id <= last_id => {}
                Ok(event) if event.id == last_id.saturating_add(1) => {
                    last_id = event.id;
                    yield event;
                }
                Ok(event) => {
                    yield unrecoverable_stream_gap(&live_run_id, last_id);
                    break;
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                    let Some(recovered) = live_manager.history_since(&live_run_id, Some(last_id)).await else {
                        yield unrecoverable_stream_gap(&live_run_id, last_id);
                        break;
                    };
                    let mut complete = true;
                    for event in recovered {
                        if event.id <= last_id {
                            continue;
                        }
                        if event.id != last_id.saturating_add(1) {
                            complete = false;
                            break;
                        }
                        last_id = event.id;
                        yield event;
                    }
                    if !complete {
                        yield unrecoverable_stream_gap(&live_run_id, last_id);
                        break;
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            }
        }
    };

    // Last-subscriber-drop guard: tied to the stream's lifetime so that when the
    // client disconnects (stream dropped), the run is cancelled iff no other
    // subscriber remains after a short grace period.
    let disconnect_guard =
        crate::uar::runtime::manager::RunDisconnectGuard::new(Arc::clone(&manager), run_id.clone());
    let stream = tokio_stream::iter(replay)
        .chain(live_stream)
        .map(move |event| {
            let _ = &disconnect_guard;
            event
        });
    let stream = async_stream::stream! {
        tokio::pin!(stream);
        while let Some(event) = stream.next().await {
            let terminal = matches!(
                &event.event,
                crate::uar::domain::events::NormalizedEvent::RunDone { .. }
                    | crate::uar::domain::events::NormalizedEvent::RunDoneWithUsage { .. }
                    | crate::uar::domain::events::NormalizedEvent::Cancelled { .. }
                    | crate::uar::domain::events::NormalizedEvent::Error { .. }
            );
            yield event;
            if terminal {
                break;
            }
        }
    };

    build_sse_response(stream, agui_spec, replay_snapshot).into_response()
}

fn unrecoverable_stream_gap(run_id: &str, last_id: u64) -> StreamEvent {
    StreamEvent {
        id: last_id.saturating_add(1),
        event: crate::uar::domain::events::NormalizedEvent::Error {
            run_id: run_id.to_owned(),
            code: "STREAM_GAP".to_owned(),
            message: "Run stream lost events that are no longer available for replay".to_owned(),
        },
    }
}

#[derive(Deserialize)]
struct ToolApprovalRequest {
    approved: bool,
    #[serde(default)]
    approval_id: Option<String>,
}

/// POST /api/uar/runs/{run_id}/tool-approval
///
/// Submit an approval or rejection decision for a pending tool call.
/// Returns 200 OK if the decision was delivered, 404 if no pending approval exists.
async fn api_tool_approval(
    State(manager): State<Arc<RunManager>>,
    Extension(user): Extension<UserContext>,
    Path(run_id): Path<String>,
    Json(body): Json<ToolApprovalRequest>,
) -> impl IntoResponse {
    if manager.get_run_for_context(&user, &run_id).await.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "resolved": false })),
        );
    }
    if manager
        .resolve_approval_request(&run_id, body.approval_id.as_deref(), body.approved)
        .await
    {
        (
            StatusCode::OK,
            Json(serde_json::json!({
                "resolved": true,
                "decision": if body.approved { "allow" } else { "deny" }
            })),
        )
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "resolved": false })),
        )
    }
}

/// POST /api/uar/runs/{run_id}/cancel
///
/// Request cancellation of an in-flight run. Idempotent: always responds 200
/// with `{ "cancelled": <bool> }` — `true` if a live run was found and
/// cancelled, `false` for an unknown or already-terminal run (no error, no
/// duplicate terminal event).
async fn api_cancel_run(
    State(manager): State<Arc<RunManager>>,
    Extension(user): Extension<UserContext>,
    Path(run_id): Path<String>,
) -> impl IntoResponse {
    if manager.get_run_for_context(&user, &run_id).await.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "cancelled": false })),
        )
            .into_response();
    }
    let cancelled = manager.cancel_run_for_context(&user, &run_id).await;
    Json(serde_json::json!({ "cancelled": cancelled })).into_response()
}

/// POST /api/uar/sessions/{session_id}/cancel
///
/// Cancel the active run projected through a stable conversation session id.
async fn api_cancel_session_run(
    State(manager): State<Arc<RunManager>>,
    Extension(user): Extension<UserContext>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    let cancelled = manager
        .cancel_session_run_for_context(&user, &session_id)
        .await;
    Json(serde_json::json!({ "cancelled": cancelled }))
}

// ── Checkpoint endpoints ──────────────────────────────────────────────────────

#[derive(Serialize)]
struct CheckpointListResponse {
    run_id: String,
    checkpoints: Vec<Checkpoint>,
}

/// GET /api/uar/runs/{run_id}/checkpoints
///
/// List all persisted checkpoints for a run, ordered by creation time.
/// Returns 503 if no persistence layer is configured.
async fn list_checkpoints(
    State(manager): State<Arc<RunManager>>,
    Extension(user): Extension<UserContext>,
    Path(run_id): Path<String>,
) -> impl IntoResponse {
    if manager.get_run_for_context(&user, &run_id).await.is_none() {
        return StatusCode::NOT_FOUND.into_response();
    }
    let Some(db) = &manager.persistence else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "persistence not configured"})),
        )
            .into_response();
    };

    match db.list_checkpoints(&run_id).await {
        Ok(checkpoints) => Json(CheckpointListResponse {
            run_id,
            checkpoints,
        })
        .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
struct ResumeRequest {
    artifact: AgentArtifact,
    /// Optional new input message; defaults to restoring the last checkpoint state.
    input: Option<String>,
    session_id: Option<String>,
    #[serde(default)]
    run_credentials: Option<Vec<crate::uar::runtime::turn::host::RunCredentialInput>>,
    #[serde(default)]
    mcp_servers: Option<Vec<crate::uar::runtime::turn::host::RunMcpServerInput>>,
    #[serde(default)]
    working_directory: Option<std::path::PathBuf>,
    #[serde(default)]
    reasoning_effort: Option<String>,
    #[serde(default)]
    history: Option<crate::uar::runtime::turn::host::HostHistoryInput>,
    #[serde(flatten)]
    presentation_negotiation: crate::uar::a2ui::presentation_selection::PresentationNegotiation,
}

/// POST /api/uar/runs/{run_id}/resume
///
/// Resume a run from its latest checkpoint (if any), or start fresh.
async fn resume_run(
    State(manager): State<Arc<RunManager>>,
    Extension(user): Extension<UserContext>,
    Path(run_id): Path<String>,
    Json(req): Json<ResumeRequest>,
) -> impl IntoResponse {
    let Some(source_run) = manager.get_run_for_context(&user, &run_id).await else {
        return StatusCode::NOT_FOUND.into_response();
    };
    if req.history.is_some() {
        return RunApiError::from(crate::uar::runtime::turn::host::HostInputError::new(
            "history_invalid",
            "resume uses the source session history",
        ))
        .into_response();
    }
    let source_marker = source_run
        .context
        .get("host_resources")
        .cloned()
        .and_then(|value| serde_json::from_value(value).ok())
        .unwrap_or_default();
    if req.artifact.id != source_run.agent_id {
        return RunApiError::from(crate::uar::runtime::turn::host::HostInputError::new(
            "run_artifact_mismatch",
            "resume artifact does not match the source run",
        ))
        .into_response();
    }
    let input = req.input.unwrap_or_else(|| {
        // No explicit input — use a standard resume message.
        format!("Resuming run {run_id}")
    });

    let mut request = match crate::uar::runtime::turn::RunExecutionRequest::new(req.artifact, input)
        .with_user_context(&user)
    {
        Ok(request) => request,
        Err(_) => return StatusCode::UNAUTHORIZED.into_response(),
    };
    request.session_id = req
        .session_id
        .or_else(|| source_run.conversation_id.clone());
    request.presentation_negotiation = req.presentation_negotiation;
    inherit_host_context(&source_run, &mut request);
    if let Err(error) = attach_host_resources(
        &mut request,
        req.run_credentials,
        req.mcp_servers,
        req.working_directory,
        req.reasoning_effort,
        None,
    )
    .and_then(|()| require_matching_host_resources(&source_marker, &request))
    {
        return error.into_response();
    }
    let new_run_id = manager.execute_request(request).await;

    Json(serde_json::json!({
        "resumed_from_run_id": run_id,
        "run_id": new_run_id,
        "stream_url": format!("/api/uar/runs/{new_run_id}/stream"),
    }))
    .into_response()
}

/// POST /api/uar/runs/{run_id}/resume/{checkpoint_id}
///
/// Resume a run from a specific named checkpoint.
/// The checkpoint's saved state is injected as context into the new run.
async fn resume_run_from_checkpoint(
    State(manager): State<Arc<RunManager>>,
    Extension(user): Extension<UserContext>,
    Path((run_id, checkpoint_id)): Path<(String, String)>,
    Json(req): Json<ResumeRequest>,
) -> impl IntoResponse {
    let Some(source_run) = manager.get_run_for_context(&user, &run_id).await else {
        return StatusCode::NOT_FOUND.into_response();
    };
    if req.history.is_some() {
        return RunApiError::from(crate::uar::runtime::turn::host::HostInputError::new(
            "history_invalid",
            "checkpoint resume uses protected checkpoint history",
        ))
        .into_response();
    }
    let source_marker = source_run
        .context
        .get("host_resources")
        .cloned()
        .and_then(|value| serde_json::from_value(value).ok())
        .unwrap_or_default();
    if req.artifact.id != source_run.agent_id {
        return RunApiError::from(crate::uar::runtime::turn::host::HostInputError::new(
            "run_artifact_mismatch",
            "checkpoint resume artifact does not match the source run",
        ))
        .into_response();
    }
    let Some(db) = &manager.persistence else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "persistence not configured"})),
        )
            .into_response();
    };

    let checkpoint = match db.load_checkpoint(&checkpoint_id).await {
        Ok(Some(cp)) if cp.run_id == run_id => cp,
        Ok(Some(_)) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "checkpoint not found"})),
            )
                .into_response();
        }
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "checkpoint not found"})),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response();
        }
    };

    // Seed the new run with what the checkpoint actually recorded. A resume
    // that starts from a prose sentence is not a resume: it discards the
    // conversation the checkpoint exists to preserve.
    let restored = match checkpoint.try_restore_state() {
        Ok(state) => state,
        Err(e) => {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response();
        }
    };
    let history = match crate::uar::runtime::checkpoint::history_from_checkpoint(&checkpoint) {
        Ok(messages) => messages,
        Err(e) => {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response();
        }
    };
    let restored_state_keys = restored.data.len();
    let restored_messages = history.len();
    let checkpoint_authorization_digest = checkpoint
        .protection
        .as_ref()
        .and_then(|protection| protection.authorization_sha256.clone());

    let mut request = match crate::uar::runtime::turn::RunExecutionRequest::new(
        req.artifact,
        req.input.clone().unwrap_or_default(),
    )
    .with_user_context(&user)
    {
        Ok(request) => request,
        Err(_) => return StatusCode::UNAUTHORIZED.into_response(),
    };
    request.input = req.input;
    request.session_id = req
        .session_id
        .or_else(|| source_run.conversation_id.clone());
    request.checkpoint_resume = Some(crate::uar::runtime::turn::CheckpointResume {
        state: restored,
        history,
        authorization_digest: checkpoint_authorization_digest
            .expect("validated current checkpoint has authorization binding"),
    });
    request.presentation_negotiation = req.presentation_negotiation;
    inherit_host_context(&source_run, &mut request);
    if let Err(error) = attach_host_resources(
        &mut request,
        req.run_credentials,
        req.mcp_servers,
        req.working_directory,
        req.reasoning_effort,
        None,
    )
    .and_then(|()| require_matching_host_resources(&source_marker, &request))
    {
        return error.into_response();
    }
    let new_run_id = manager.execute_request(request).await;

    Json(serde_json::json!({
        "resumed_from_run_id": run_id,
        "checkpoint_id": checkpoint_id,
        "checkpoint_node_id": checkpoint.node_id,
        "checkpoint_iteration": checkpoint.iteration,
        "restored_messages": restored_messages,
        "restored_state_keys": restored_state_keys,
        "run_id": new_run_id,
        "stream_url": format!("/api/uar/runs/{new_run_id}/stream"),
    }))
    .into_response()
}

/// GET /api/uar/resolve-model
///
/// Returns the resolved default model configuration (provider + model) or an error
/// if no model is available. Used by the frontend to guard chat before starting a run.
async fn resolve_model(State(manager): State<Arc<RunManager>>) -> impl IntoResponse {
    let model = manager.resolve_default_model().await;
    match model {
        Some((provider_id, model_id)) => Json(serde_json::json!({
            "ok": true,
            "provider_id": provider_id,
            "model_id": model_id,
        }))
        .into_response(),
        None => (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({
                "ok": false,
                "error": "No model configured. Add a provider and set a default model in Settings → Providers.",
            })),
        )
            .into_response(),
    }
}
