use crate::uar::security::credential_capture::AuthenticatedCredentialCapture;
use crate::uar::{
    api::sse::{build_agui_replay_snapshot, build_sse_response},
    domain::artifact::AgentArtifact,
    runtime::{
        checkpoint::Checkpoint,
        manager::{RunManager, StreamEvent},
    },
    security::{claims::UserContext, sidecar_guard::HostAuthenticated},
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

#[derive(Debug, Clone)]
pub struct RunApiState {
    pub manager: Arc<RunManager>,
    pub collaboration_catalog:
        Arc<crate::uar::compiler::collaboration::CollaborationCatalogService>,
    pub service_instance: Arc<crate::uar::service_instance::ServiceInstanceAuthority>,
}

#[derive(Clone)]
struct RunManagerState(Arc<RunManager>);

impl axum::extract::FromRef<Arc<RunApiState>> for RunManagerState {
    fn from_ref(state: &Arc<RunApiState>) -> Self {
        Self(Arc::clone(&state.manager))
    }
}

pub fn build_router() -> Router<Arc<RunApiState>> {
    Router::new()
        .route("/runs", get(list_runs).post(create_run))
        .route("/runs/{id}", get(read_run))
        .route("/runs/{id}/stream", get(stream_run))
        .route("/runs/{id}/events", get(super::run_events::snapshot))
        .route("/runs/{run_id}/tool-approval", get(api_approval_records).post(api_tool_approval))
        .route(
            "/runs/{run_id}/tool-approval/pending",
            get(api_pending_tool_approval),
        )
        .route(
            "/runs/{run_id}/tool-admission-evidence",
            get(api_tool_admission_evidence),
        )
        .route("/runs/{run_id}/cancel", post(api_cancel_run))
        .route(
            "/runs/{run_id}/mcp-grants/{server}/revoke",
            post(api_revoke_run_mcp_grant),
        )
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

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct CreateRunRequest {
    #[serde(default)]
    pub(crate) artifact: Option<AgentArtifact>,
    #[serde(default)]
    pub(crate) agent_id: Option<String>,
    #[serde(default)]
    pub(crate) deployment_binding_id: Option<String>,
    #[serde(default)]
    pub(crate) service_placement: Option<crate::uar::service_instance::ServicePlacementExpectation>,
    pub(crate) input: String,
    pub(crate) session_id: Option<String>,
    #[serde(default)]
    pub(crate) run_credentials: Option<Vec<crate::uar::runtime::turn::host::RunCredentialInput>>,
    #[serde(default)]
    pub(crate) mcp_servers: Option<Vec<crate::uar::runtime::turn::host::RunMcpServerInput>>,
    #[serde(default)]
    pub(crate) tool_admission: Option<crate::uar::runtime::tool_admission::RunToolAdmissionInput>,
    #[serde(default)]
    pub(crate) working_directory: Option<std::path::PathBuf>,
    #[serde(default)]
    pub(crate) reasoning_effort: Option<String>,
    #[serde(default)]
    pub(crate) history: Option<crate::uar::runtime::turn::host::HostHistoryInput>,
    #[serde(default)]
    pub(crate) skill_attachments: Vec<String>,
    #[serde(flatten)]
    pub(crate) presentation_negotiation:
        crate::uar::a2ui::presentation_selection::PresentationNegotiation,
}

#[derive(Clone, serde::Serialize)]
pub(crate) struct CreateRunResponse {
    pub(crate) run_id: String,
    pub(crate) stream_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) effective_service_binding:
        Option<crate::uar::service_instance::EffectiveServiceBinding>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) activation_failures: Vec<crate::uar::runtime::skills::activation::ActivationFailure>,
    pub(crate) history: crate::uar::runtime::turn::host::HistorySeedStatus,
    pub(crate) seeded_messages: usize,
}

#[derive(Serialize)]
struct RunInspection {
    run_id: String,
    agent_id: String,
    conversation_id: Option<String>,
    status: crate::uar::domain::runs::RunStatus,
    agent_revision: Option<String>,
    effective_model: Option<serde_json::Value>,
    effective_run_policy: Option<serde_json::Value>,
    presentation_selection: Option<serde_json::Value>,
    host_resources: Option<serde_json::Value>,
    effective_service_binding: Option<serde_json::Value>,
}

impl From<crate::uar::domain::runs::Run> for RunInspection {
    fn from(run: crate::uar::domain::runs::Run) -> Self {
        Self {
            run_id: run.run_id,
            agent_id: run.agent_id,
            conversation_id: run.conversation_id,
            status: run.status,
            agent_revision: run
                .context
                .pointer("/agent_snapshot/revision")
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned),
            effective_model: run.context.pointer("/effective_run_policy/model").cloned(),
            effective_run_policy: run.context.get("effective_run_policy").cloned(),
            presentation_selection: run.context.get("presentation_selection").cloned(),
            host_resources: run.context.get("host_resources").cloned(),
            effective_service_binding: run
                .context
                .get("effective_service_binding")
                .filter(|value| !value.is_null())
                .cloned(),
        }
    }
}

async fn list_runs(
    State(RunManagerState(manager)): State<RunManagerState>,
    Extension(user): Extension<UserContext>,
) -> Json<Vec<RunInspection>> {
    Json(
        manager
            .list_runs_for_context(&user)
            .await
            .into_iter()
            .map(RunInspection::from)
            .collect(),
    )
}

async fn read_run(
    State(RunManagerState(manager)): State<RunManagerState>,
    Extension(user): Extension<UserContext>,
    Path(run_id): Path<String>,
) -> Result<Json<RunInspection>, StatusCode> {
    manager
        .get_run_for_context(&user, &run_id)
        .await
        .map(RunInspection::from)
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

#[derive(Debug)]
pub(crate) struct RunApiError {
    status: StatusCode,
    code: &'static str,
    message: String,
}

impl RunApiError {
    pub(crate) fn delegated_context(status: StatusCode, code: &'static str, message: String) -> Self {
        Self { status, code, message }
    }
    pub(crate) fn status(&self) -> StatusCode {
        self.status
    }

    pub(crate) fn code(&self) -> &'static str {
        self.code
    }

    pub(crate) fn message(&self) -> &str {
        &self.message
    }
}

impl From<crate::uar::runtime::turn::host::HostInputError> for RunApiError {
    fn from(error: crate::uar::runtime::turn::host::HostInputError) -> Self {
        Self {
            status: if error.code == "run_mcp_grant_authentication_required" {
                StatusCode::UNAUTHORIZED
            } else {
                StatusCode::UNPROCESSABLE_ENTITY
            },
            code: error.code,
            message: error.message.to_string(),
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

pub(crate) async fn attach_host_resources(
    manager: &RunManager,
    user: &UserContext,
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
        let server_names = servers
            .iter()
            .map(|server| server.name.trim().to_owned())
            .collect::<std::collections::BTreeSet<_>>();
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
            message: "run-scoped MCP requires a verified principal".to_string(),
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
        let (servers, resources) = manager
            .admit_run_mcp_servers(servers, user, owner, cwd)
            .await?;
        request.host_resources_marker.mcp_servers = server_names.into_iter().collect();
        request.host_resources_marker.mcp_grants = servers.grant_markers();
        request.host_secret_scrubber.extend(servers.scrubber());
        request.mcp_resources = Some(resources);
    }
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
    let renewed = &request.host_resources_marker.mcp_grants;
    if marker.mcp_grants.len() != renewed.len()
        || marker.mcp_grants.iter().any(|source| {
            renewed
                .iter()
                .find(|candidate| candidate.server == source.server)
                .is_none_or(|candidate| !source.accepts_renewal(candidate))
        })
    {
        return Err(crate::uar::runtime::turn::host::HostInputError::new(
            "run_mcp_grant_authentication_required",
            "resume requires a same-owner MCP grant for the original destination and scope",
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
    State(state): State<Arc<RunApiState>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
    host_authenticated: Option<Extension<HostAuthenticated>>,
    credential_capture: Option<Extension<AuthenticatedCredentialCapture>>,
    Json(req): Json<CreateRunRequest>,
) -> Result<Json<CreateRunResponse>, RunApiError> {
    admit_run(
        state,
        user,
        headers,
        host_authenticated.is_some(),
        req,
        None,
        None,
        credential_capture.map(|Extension(capture)| capture),
    )
    .await
    .map(Json)
}

pub(crate) async fn admit_run(
    state: Arc<RunApiState>,
    user: UserContext,
    headers: HeaderMap,
    host_authenticated: bool,
    req: CreateRunRequest,
    reserved_run_id: Option<String>,
    delegated_host_context: Option<Arc<super::full_harness::host_context::DelegatedHostContext>>,
    credential_capture: Option<AuthenticatedCredentialCapture>,
) -> Result<CreateRunResponse, RunApiError> {
    let CreateRunRequest {
        artifact,
        agent_id,
        deployment_binding_id,
        service_placement,
        input,
        session_id,
        run_credentials,
        mcp_servers,
        tool_admission,
        working_directory,
        reasoning_effort,
        history,
        skill_attachments,
        presentation_negotiation,
    } = req;
    let admitted_service_binding = if let Some(expectation) = service_placement.as_ref() {
        if expectation.binding_id.is_some()
            && expectation.binding_id.as_deref() != deployment_binding_id.as_deref()
        {
            return Err(RunApiError {
                status: StatusCode::CONFLICT,
                code: "service_binding_mismatch",
                message: "service placement bindingId does not match deployment_binding_id"
                    .to_owned(),
            });
        }
        let response = state.service_instance.evaluate(
            expectation,
            crate::uar::service_instance::PlacementIntent::New,
        );
        if !response.compatible {
            return Err(service_placement_error(response));
        }
        response.effective_binding
    } else {
        None
    };
    let bound_selector = deployment_binding_id.is_some();
    let mut request = if let Some(binding_id) = deployment_binding_id {
        if agent_id.is_some() || artifact.is_some() {
            return Err(selector_ambiguous());
        }
        let owner_id =
            super::user_settings::principal_storage_key(&user).ok_or_else(|| RunApiError {
                status: StatusCode::UNAUTHORIZED,
                code: "principal_invalid",
                message: "deployment binding requires a verified principal".to_string(),
            })?;
        let workspace_id = headers
            .get("x-uar-workspace-id")
            .and_then(|value| value.to_str().ok())
            .filter(|value| !value.trim().is_empty())
            .map(str::to_owned)
            .ok_or_else(|| RunApiError {
                status: StatusCode::BAD_REQUEST,
                code: "workspace_required",
                message: "deployment binding requires x-uar-workspace-id".to_string(),
            })?;
        let bound = state
            .collaboration_catalog
            .resolve_bound_agent_run(&owner_id, &workspace_id, &binding_id)
            .await
            .map_err(collaboration_run_error)?;
        crate::uar::runtime::turn::RunExecutionRequest::from_bound_agent(
            bound,
            input,
            owner_id,
            workspace_id,
            Arc::clone(&state.collaboration_catalog),
        )
        .with_user_context(&user)
        .map_err(|_| principal_invalid())?
    } else {
        let (artifact, artifact_inline) =
            resolve_run_agent(&state.manager, agent_id, artifact).await?;
        let mut request = crate::uar::runtime::turn::RunExecutionRequest::new(artifact, input)
            .with_user_context(&user)
            .map_err(|_| principal_invalid())?;
        request.host_resources_marker.artifact_inline = artifact_inline;
        request
    };
    request = request.with_credential_capture(credential_capture);
    let response_service_binding = admitted_service_binding.clone();
    if let Some(admitted) = admitted_service_binding {
        if let Some(bound) = request.service_binding.as_ref() {
            if (admitted.binding_id.is_some() && admitted.binding_id != bound.binding_id)
                || (admitted.binding_revision.is_some()
                    && admitted.binding_revision != bound.binding_revision)
                || (admitted.credential_ref.is_some()
                    && admitted.credential_ref != bound.credential_ref)
            {
                return Err(RunApiError {
                    status: StatusCode::CONFLICT,
                    code: "service_binding_mismatch",
                    message: "service placement does not match the effective deployment binding"
                        .to_owned(),
                });
            }
        } else {
            request.service_binding = Some(admitted);
        }
    }
    request.session_id = session_id;
    request.skill_attachments = skill_attachments;
    request.presentation_negotiation = presentation_negotiation;
    if let Some(context) = delegated_host_context {
        let run_id = reserved_run_id.as_deref().ok_or_else(super::full_harness::host_context::run_mismatch)?;
        context.attach(&state, &mut request, run_id).await
            .map_err(super::full_harness::host_context::run_error)?;
    }
    if let Some(input) = tool_admission {
        if !host_authenticated {
            return Err(RunApiError {
                status: StatusCode::FORBIDDEN,
                code: "tool_admission_host_authentication_required",
                message: "paired host tool admission requires authenticated sidecar authority"
                    .to_string(),
            });
        }
        let adapter =
            crate::uar::runtime::tool_admission::HttpHostToolAdmissionPort::from_input(input)
                .map_err(|_| RunApiError {
                    status: StatusCode::UNPROCESSABLE_ENTITY,
                    code: "tool_admission_invalid",
                    message: "paired host tool admission is invalid or incompatible".to_string(),
                })?;
        request.host_tool_admission = Some(Arc::new(adapter));
    }
    attach_host_resources(
        &state.manager,
        &user,
        &mut request,
        run_credentials,
        mcp_servers,
        working_directory,
        reasoning_effort,
        history,
    )
    .await?;
    let run_id = if bound_selector {
        if let Some(run_id) = reserved_run_id {
            state
                .manager
                .execute_bound_request_with_run_id(request, run_id)
                .await
                .map_err(collaboration_run_error)?
        } else {
            state
                .manager
                .execute_bound_request(request)
                .await
                .map_err(collaboration_run_error)?
        }
    } else if let Some(run_id) = reserved_run_id {
        state
            .manager
            .execute_request_with_run_id(request, run_id)
            .await
    } else {
        state.manager.execute_request(request).await
    };
    let run_context = state
        .manager
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
    let effective_service_binding = response_service_binding.or_else(|| {
        run_context
            .get("effective_service_binding")
            .cloned()
            .and_then(|value| serde_json::from_value(value).ok())
    });
    Ok(CreateRunResponse {
        run_id: run_id.clone(),
        stream_url: format!("/api/uar/runs/{run_id}/stream"),
        effective_service_binding,
        activation_failures,
        history,
        seeded_messages,
    })
}

fn service_placement_error(
    response: crate::uar::service_instance::CompatibilityResponse,
) -> RunApiError {
    RunApiError {
        status: StatusCode::CONFLICT,
        code: if response
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "placement.migration-unsupported")
        {
            "service_migration_unsupported"
        } else {
            "service_instance_incompatible"
        },
        message: response
            .diagnostics
            .into_iter()
            .map(|diagnostic| diagnostic.message)
            .collect::<Vec<_>>()
            .join("; "),
    }
}

fn selector_ambiguous() -> RunApiError {
    RunApiError {
        status: StatusCode::UNPROCESSABLE_ENTITY,
        code: "run_agent_selector_ambiguous",
        message: "provide exactly one of agent_id, artifact, or deployment_binding_id".to_string(),
    }
}

fn principal_invalid() -> RunApiError {
    RunApiError {
        status: StatusCode::UNAUTHORIZED,
        code: "principal_invalid",
        message: "run principal is invalid".to_string(),
    }
}

fn collaboration_run_error(
    error: crate::uar::compiler::collaboration::CollaborationError,
) -> RunApiError {
    use crate::uar::compiler::collaboration::CollaborationError;
    match error {
        CollaborationError::Invalid(message) => RunApiError {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            code: "run_binding_invalid",
            message,
        },
        CollaborationError::NotFound(message) => RunApiError {
            status: StatusCode::NOT_FOUND,
            code: "run_binding_not_found",
            message,
        },
        CollaborationError::Conflict(message) => RunApiError {
            status: StatusCode::CONFLICT,
            code: "run_binding_conflict",
            message,
        },
        CollaborationError::Storage(message) => {
            tracing::error!(%message, "collaboration catalog unavailable during run admission");
            RunApiError {
                status: StatusCode::SERVICE_UNAVAILABLE,
                code: "run_binding_unavailable",
                message: "collaboration catalog is unavailable".to_string(),
            }
        }
    }
}

async fn resolve_run_agent(
    manager: &RunManager,
    agent_id: Option<String>,
    artifact: Option<AgentArtifact>,
) -> Result<(AgentArtifact, bool), RunApiError> {
    match (agent_id, artifact) {
        (Some(_), Some(_)) => Err(selector_ambiguous()),
        (None, None) => Err(RunApiError {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            code: "run_agent_selector_required",
            message: "agent_id, artifact, or deployment_binding_id is required".to_string(),
        }),
        (None, Some(artifact)) => {
            crate::uar::domain::agent_store::validate_agent(&artifact).map_err(|error| {
                RunApiError {
                    status: StatusCode::UNPROCESSABLE_ENTITY,
                    code: "run_artifact_invalid",
                    message: error.to_string(),
                }
            })?;
            Ok((artifact.with_catalog_metadata("inline"), true))
        }
        (Some(agent_id), None) => manager
            .resolve_registered_agent(&agent_id)
            .await
            .map(|artifact| (artifact, false))
            .map_err(|error| match error {
                crate::uar::domain::agent_store::AgentStoreError::NotFound(id) => RunApiError {
                    status: StatusCode::NOT_FOUND,
                    code: "run_agent_not_found",
                    message: format!("agent '{id}' is not registered"),
                },
                crate::uar::domain::agent_store::AgentStoreError::Invalid(message) => RunApiError {
                    status: StatusCode::UNPROCESSABLE_ENTITY,
                    code: "run_agent_invalid",
                    message,
                },
                other => {
                    tracing::error!(%other, "Agent catalog resolution failed");
                    RunApiError {
                        status: StatusCode::SERVICE_UNAVAILABLE,
                        code: "run_agent_catalog_unavailable",
                        message: "agent catalog is unavailable".to_string(),
                    }
                }
            }),
    }
}

async fn stream_run(
    State(RunManagerState(manager)): State<RunManagerState>,
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
                Ok(_event) => {
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

    // GET /runs/{id}/stream is an observer of an admitted run. Dropping the
    // subscription never owns cancellation; use the explicit cancel endpoint.
    let stream = tokio_stream::iter(replay).chain(live_stream);
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

pub(crate) fn unrecoverable_stream_gap(run_id: &str, last_id: u64) -> StreamEvent {
    StreamEvent {
        id: last_id.saturating_add(1),
        event: crate::uar::domain::events::NormalizedEvent::Error {
            run_id: run_id.to_owned(),
            code: "STREAM_GAP".to_owned(),
            message: "Run stream lost events that are no longer available for replay".to_owned(),
        },
    }
}

pub(crate) fn is_terminal_stream_event(event: &StreamEvent) -> bool {
    matches!(
        &event.event,
        crate::uar::domain::events::NormalizedEvent::RunDone { .. }
            | crate::uar::domain::events::NormalizedEvent::RunDoneWithUsage { .. }
            | crate::uar::domain::events::NormalizedEvent::Cancelled { .. }
            | crate::uar::domain::events::NormalizedEvent::Error { .. }
    )
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
    State(RunManagerState(manager)): State<RunManagerState>,
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
    let Some(approval_id) = body.approval_id.as_deref().filter(|id| !id.trim().is_empty()) else {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
            "resolved": false, "error": "approval_id_required",
            "message": "Submit the approval_id from the originating approval event or pending snapshot"
        })));
    };
    match manager.resolve_approval_record(&run_id, Some(approval_id), body.approved).await {
        Ok(Some((record, delivered))) => (StatusCode::OK, Json(serde_json::json!({
            "resolved": delivered, "decision": if body.approved { "allow" } else { "deny" },
            "record": crate::uar::persistence::approval_decisions::ApprovalRecordView { record, resolvable: false },
        }))),
        Ok(None) => (StatusCode::NOT_FOUND, Json(serde_json::json!({ "resolved": false }))),
        Err(_) => (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({
            "resolved": false, "code": "approval_persistence_unavailable",
        }))),
    }
}

/// Owner-scoped history remains readable after the live run leaves memory.
async fn api_approval_records(
    State(RunManagerState(manager)): State<RunManagerState>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
    Path(run_id): Path<String>,
) -> impl IntoResponse {
    let workspace = headers.get("x-uar-workspace-id").and_then(|value| value.to_str().ok());
    match manager.approval_records_for_context(&user, &run_id, workspace).await {
        Ok((durable, records)) => {
            (StatusCode::OK, Json(serde_json::json!({ "version": 1, "runId": run_id, "durable": durable, "records": records })))
        }
        Err(_) => (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({ "code": "approval_history_unavailable" }))),
    }
}

/// GET /api/uar/runs/{run_id}/tool-approval/pending
///
/// Replays the owner-scoped live waiter with its stable approval identity and
/// original stream cursor. It never creates a new waiter.
async fn api_pending_tool_approval(
    State(RunManagerState(manager)): State<RunManagerState>,
    Extension(user): Extension<UserContext>,
    Path(run_id): Path<String>,
) -> impl IntoResponse {
    if manager.get_run_for_context(&user, &run_id).await.is_none() {
        return Json(serde_json::json!({
            "version": 1,
            "runId": run_id,
            "pending": null,
        }));
    }
    let pending = manager
        .pending_approval_for_user(&user.user_id, &run_id)
        .await;
    Json(serde_json::json!({
        "version": 1,
        "runId": run_id,
        "pending": pending,
    }))
}

/// GET /api/uar/runs/{run_id}/tool-admission-evidence
///
/// Returns sanitized append-only lifecycle evidence. The owner filter is
/// applied in storage before the requested run tree is selected.
async fn api_tool_admission_evidence(
    State(RunManagerState(manager)): State<RunManagerState>,
    Extension(user): Extension<UserContext>,
    Path(run_id): Path<String>,
) -> impl IntoResponse {
    match manager
        .tool_admission_evidence_for_user(&user.user_id, &run_id)
        .await
    {
        Ok(records) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "version": 1,
                "runId": run_id,
                "records": records,
            })),
        ),
        Err(_) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "version": 1,
                "runId": run_id,
                "code": "tool_admission_evidence_unavailable",
                "error": "Tool admission evidence could not be read; check the configured persistence service",
            })),
        ),
    }
}

/// POST /api/uar/runs/{run_id}/cancel
///
/// Request cancellation of an in-flight run. Idempotent: always responds 200
/// with `{ "cancelled": <bool> }` — `true` if a live run was found and
/// cancelled, `false` for an unknown or already-terminal run (no error, no
/// duplicate terminal event).
async fn api_cancel_run(
    State(RunManagerState(manager)): State<RunManagerState>,
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

/// POST /api/uar/runs/{run_id}/mcp-grants/{server}/revoke
///
/// Revoke an admitted downstream credential and cancel its run. A replacement
/// is accepted only through the ordinary authenticated resume boundary.
async fn api_revoke_run_mcp_grant(
    State(RunManagerState(manager)): State<RunManagerState>,
    Extension(user): Extension<UserContext>,
    Path((run_id, server)): Path<(String, String)>,
) -> impl IntoResponse {
    if manager.get_run_for_context(&user, &run_id).await.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "revoked": false })),
        )
            .into_response();
    }
    let revoked = manager
        .revoke_run_mcp_grant_for_context(&user, &run_id, &server)
        .await;
    Json(serde_json::json!({
        "run_id": run_id,
        "server": server,
        "revoked": revoked,
        "renewal": if revoked { "resume_required" } else { "not_applicable" },
    }))
    .into_response()
}

/// POST /api/uar/sessions/{session_id}/cancel
///
/// Cancel the active run projected through a stable conversation session id.
async fn api_cancel_session_run(
    State(RunManagerState(manager)): State<RunManagerState>,
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
    State(RunManagerState(manager)): State<RunManagerState>,
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
    #[serde(default)]
    artifact: Option<AgentArtifact>,
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
    #[serde(default)]
    service_placement: Option<crate::uar::service_instance::ServicePlacementExpectation>,
    #[serde(flatten)]
    presentation_negotiation: crate::uar::a2ui::presentation_selection::PresentationNegotiation,
}

fn resume_artifact(
    source_run: &crate::uar::domain::runs::Run,
    supplied: Option<AgentArtifact>,
) -> Result<AgentArtifact, RunApiError> {
    let snapshot =
        crate::uar::domain::artifact::AgentArtifactSnapshot::from_run_context(&source_run.context)
            .map_err(|message| RunApiError {
                status: StatusCode::CONFLICT,
                code: "run_artifact_snapshot_unavailable",
                message: message.to_string(),
            })?;
    if supplied.is_some_and(|artifact| {
        artifact.definition_revision() != snapshot.artifact.definition_revision()
    }) {
        return Err(RunApiError {
            status: StatusCode::CONFLICT,
            code: "run_artifact_mismatch",
            message: "resume artifact does not match the source run snapshot".to_string(),
        });
    }
    Ok(snapshot.artifact)
}

fn resume_service_binding(
    authority: &crate::uar::service_instance::ServiceInstanceAuthority,
    source_run: &crate::uar::domain::runs::Run,
    expectation: Option<&crate::uar::service_instance::ServicePlacementExpectation>,
) -> Result<Option<crate::uar::service_instance::EffectiveServiceBinding>, RunApiError> {
    let source = source_run
        .context
        .get("effective_service_binding")
        .filter(|value| !value.is_null())
        .cloned()
        .map(serde_json::from_value)
        .transpose()
        .map_err(|_| RunApiError {
            status: StatusCode::CONFLICT,
            code: "service_binding_invalid",
            message: "source run service binding cannot be read".to_owned(),
        })?;
    let Some(mut source) = source else {
        if expectation.is_some() {
            return Err(RunApiError {
                status: StatusCode::CONFLICT,
                code: "service_binding_unavailable",
                message: "source run has no effective service binding to reattach".to_owned(),
            });
        }
        return Ok(None);
    };
    authority
        .revalidate(&source)
        .map_err(service_placement_error)?;
    if let Some(expectation) = expectation {
        let response = authority.evaluate(
            expectation,
            crate::uar::service_instance::PlacementIntent::Reattach,
        );
        if !response.compatible {
            return Err(service_placement_error(response));
        }
        let admitted = response.effective_binding.ok_or_else(|| RunApiError {
            status: StatusCode::CONFLICT,
            code: "service_instance_incompatible",
            message: "service placement produced no effective binding".to_owned(),
        })?;
        if admitted.instance_id != source.instance_id
            || admitted.profile != source.profile
            || admitted.endpoints != source.endpoints
            || (admitted.binding_id.is_some() && admitted.binding_id != source.binding_id)
            || (admitted.binding_revision.is_some()
                && admitted.binding_revision != source.binding_revision)
            || (admitted.credential_ref.is_some()
                && admitted.credential_ref != source.credential_ref)
        {
            return Err(RunApiError {
                status: StatusCode::CONFLICT,
                code: "service_reattachment_mismatch",
                message: "requested reattachment does not match the source run binding".to_owned(),
            });
        }
    }
    source.intent = crate::uar::service_instance::PlacementIntent::Reattach;
    Ok(Some(source))
}

async fn resume_execution_request(
    state: &RunApiState,
    user: &UserContext,
    source_run: &crate::uar::domain::runs::Run,
    artifact: AgentArtifact,
    input: String,
) -> Result<(crate::uar::runtime::turn::RunExecutionRequest, bool), RunApiError> {
    let Some(binding) = source_run
        .context
        .get("effective_collaboration_binding")
        .filter(|value| !value.is_null())
    else {
        let request = crate::uar::runtime::turn::RunExecutionRequest::new(artifact, input)
            .with_user_context(user)
            .map_err(|_| principal_invalid())?;
        return Ok((request, false));
    };
    let owner_id = binding
        .get("ownerId")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| RunApiError {
            status: StatusCode::CONFLICT,
            code: "run_binding_invalid",
            message: "source run collaboration owner is unavailable".to_owned(),
        })?;
    let workspace_id = binding
        .get("workspaceId")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| RunApiError {
            status: StatusCode::CONFLICT,
            code: "run_binding_invalid",
            message: "source run collaboration workspace is unavailable".to_owned(),
        })?;
    let binding_id = binding
        .get("bindingId")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| RunApiError {
            status: StatusCode::CONFLICT,
            code: "run_binding_invalid",
            message: "source run collaboration binding is unavailable".to_owned(),
        })?;
    let binding_revision = binding
        .get("bindingRevision")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| RunApiError {
            status: StatusCode::CONFLICT,
            code: "run_binding_invalid",
            message: "source run collaboration binding revision is unavailable".to_owned(),
        })?;
    let current_owner =
        super::user_settings::principal_storage_key(user).ok_or_else(|| RunApiError {
            status: StatusCode::UNAUTHORIZED,
            code: "principal_invalid",
            message: "deployment binding requires a verified principal".to_owned(),
        })?;
    if current_owner != owner_id {
        return Err(RunApiError {
            status: StatusCode::FORBIDDEN,
            code: "run_binding_owner_mismatch",
            message: "source run deployment binding belongs to another principal".to_owned(),
        });
    }
    let bound = state
        .collaboration_catalog
        .resolve_bound_agent_run(owner_id, workspace_id, binding_id)
        .await
        .map_err(collaboration_run_error)?;
    if bound.binding.revision != binding_revision
        || bound.artifact.definition_revision() != artifact.definition_revision()
    {
        return Err(RunApiError {
            status: StatusCode::CONFLICT,
            code: "run_binding_changed",
            message: "deployment binding no longer resolves the source run definition".to_owned(),
        });
    }
    let request = crate::uar::runtime::turn::RunExecutionRequest::from_bound_agent(
        bound,
        input,
        owner_id.to_owned(),
        workspace_id.to_owned(),
        Arc::clone(&state.collaboration_catalog),
    )
    .with_user_context(user)
    .map_err(|_| principal_invalid())?;
    Ok((request, true))
}

async fn execute_resumed_request(
    manager: &RunManager,
    request: crate::uar::runtime::turn::RunExecutionRequest,
    bound: bool,
) -> Result<String, RunApiError> {
    if bound {
        manager
            .execute_bound_request(request)
            .await
            .map_err(collaboration_run_error)
    } else {
        Ok(manager.execute_request(request).await)
    }
}

/// POST /api/uar/runs/{run_id}/resume
///
/// Resume a run from its latest checkpoint (if any), or start fresh.
async fn resume_run(
    State(state): State<Arc<RunApiState>>,
    Extension(user): Extension<UserContext>,
    credential_capture: Option<Extension<AuthenticatedCredentialCapture>>,
    Path(run_id): Path<String>,
    Json(req): Json<ResumeRequest>,
) -> impl IntoResponse {
    let manager = Arc::clone(&state.manager);
    let Some(source_run) = manager.get_run_for_context(&user, &run_id).await else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let service_binding = match resume_service_binding(
        &state.service_instance,
        &source_run,
        req.service_placement.as_ref(),
    ) {
        Ok(binding) => binding,
        Err(error) => return error.into_response(),
    };
    if req.history.is_some() {
        return RunApiError::from(crate::uar::runtime::turn::host::HostInputError::new(
            "history_invalid",
            "resume uses the source session history",
        ))
        .into_response();
    }
    let source_marker: crate::uar::runtime::turn::host::HostResourcesMarker = source_run
        .context
        .get("host_resources")
        .cloned()
        .and_then(|value| serde_json::from_value(value).ok())
        .unwrap_or_default();
    let artifact = match resume_artifact(&source_run, req.artifact) {
        Ok(artifact) => artifact,
        Err(error) => return error.into_response(),
    };
    let input = req.input.unwrap_or_else(|| {
        // No explicit input — use a standard resume message.
        format!("Resuming run {run_id}")
    });

    let (mut request, bound) =
        match resume_execution_request(&state, &user, &source_run, artifact, input).await {
            Ok(result) => result,
            Err(error) => return error.into_response(),
        };
    request = request.with_credential_capture(credential_capture.map(|Extension(capture)| capture));
    request.session_id = req
        .session_id
        .or_else(|| source_run.conversation_id.clone());
    request.host_resources_marker.artifact_inline = source_marker.artifact_inline;
    request.presentation_negotiation = req.presentation_negotiation;
    let response_service_binding = service_binding.clone();
    request.service_binding = service_binding;
    inherit_host_context(&source_run, &mut request);
    if let Err(error) = attach_host_resources(
        &manager,
        &user,
        &mut request,
        req.run_credentials,
        req.mcp_servers,
        req.working_directory,
        req.reasoning_effort,
        None,
    )
    .await
    {
        return error.into_response();
    }
    if let Err(error) = require_matching_host_resources(&source_marker, &request) {
        return error.into_response();
    }
    let new_run_id = match execute_resumed_request(&manager, request, bound).await {
        Ok(run_id) => run_id,
        Err(error) => return error.into_response(),
    };

    Json(serde_json::json!({
        "resumed_from_run_id": run_id,
        "run_id": new_run_id,
        "stream_url": format!("/api/uar/runs/{new_run_id}/stream"),
        "effective_service_binding": response_service_binding,
    }))
    .into_response()
}

/// POST /api/uar/runs/{run_id}/resume/{checkpoint_id}
///
/// Resume a run from a specific named checkpoint.
/// The checkpoint's saved state is injected as context into the new run.
async fn resume_run_from_checkpoint(
    State(state): State<Arc<RunApiState>>,
    Extension(user): Extension<UserContext>,
    credential_capture: Option<Extension<AuthenticatedCredentialCapture>>,
    Path((run_id, checkpoint_id)): Path<(String, String)>,
    Json(req): Json<ResumeRequest>,
) -> impl IntoResponse {
    let manager = Arc::clone(&state.manager);
    let Some(source_run) = manager.get_run_for_context(&user, &run_id).await else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let service_binding = match resume_service_binding(
        &state.service_instance,
        &source_run,
        req.service_placement.as_ref(),
    ) {
        Ok(binding) => binding,
        Err(error) => return error.into_response(),
    };
    if req.history.is_some() {
        return RunApiError::from(crate::uar::runtime::turn::host::HostInputError::new(
            "history_invalid",
            "checkpoint resume uses protected checkpoint history",
        ))
        .into_response();
    }
    let source_marker: crate::uar::runtime::turn::host::HostResourcesMarker = source_run
        .context
        .get("host_resources")
        .cloned()
        .and_then(|value| serde_json::from_value(value).ok())
        .unwrap_or_default();
    let artifact = match resume_artifact(&source_run, req.artifact) {
        Ok(artifact) => artifact,
        Err(error) => return error.into_response(),
    };
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

    let (mut request, bound) = match resume_execution_request(
        &state,
        &user,
        &source_run,
        artifact,
        req.input.clone().unwrap_or_default(),
    )
    .await
    {
        Ok(result) => result,
        Err(error) => return error.into_response(),
    };
    request = request.with_credential_capture(credential_capture.map(|Extension(capture)| capture));
    request.input = req.input;
    request.session_id = req
        .session_id
        .or_else(|| source_run.conversation_id.clone());
    request.host_resources_marker.artifact_inline = source_marker.artifact_inline;
    request.checkpoint_resume = Some(crate::uar::runtime::turn::CheckpointResume {
        state: restored,
        history,
        authorization_digest: checkpoint_authorization_digest
            .expect("validated current checkpoint has authorization binding"),
    });
    request.presentation_negotiation = req.presentation_negotiation;
    let response_service_binding = service_binding.clone();
    request.service_binding = service_binding;
    inherit_host_context(&source_run, &mut request);
    if let Err(error) = attach_host_resources(
        &manager,
        &user,
        &mut request,
        req.run_credentials,
        req.mcp_servers,
        req.working_directory,
        req.reasoning_effort,
        None,
    )
    .await
    {
        return error.into_response();
    }
    if let Err(error) = require_matching_host_resources(&source_marker, &request) {
        return error.into_response();
    }
    let new_run_id = match execute_resumed_request(&manager, request, bound).await {
        Ok(run_id) => run_id,
        Err(error) => return error.into_response(),
    };

    Json(serde_json::json!({
        "resumed_from_run_id": run_id,
        "checkpoint_id": checkpoint_id,
        "checkpoint_node_id": checkpoint.node_id,
        "checkpoint_iteration": checkpoint.iteration,
        "restored_messages": restored_messages,
        "restored_state_keys": restored_state_keys,
        "run_id": new_run_id,
        "stream_url": format!("/api/uar/runs/{new_run_id}/stream"),
        "effective_service_binding": response_service_binding,
    }))
    .into_response()
}

/// GET /api/uar/resolve-model
///
/// Returns the resolved default model configuration (provider + model) or an error
/// if no model is available. Used by the frontend to guard chat before starting a run.
async fn resolve_model(
    State(RunManagerState(manager)): State<RunManagerState>,
) -> impl IntoResponse {
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
