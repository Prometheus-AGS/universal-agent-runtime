//! Private paired-host resources for an exact scoped delegation. No private
//! input is serializable or retained in a run/task persistence record.

use std::{collections::HashMap, path::PathBuf, sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}}};
use axum::{Extension, Json, extract::{Path, State}, http::{HeaderMap, StatusCode}};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{ApiError, FullHarnessApiState, TaskReceipt};
use crate::uar::{
    api::routes::attach_host_resources,
    domain::collaboration::{ImmutableDefinitionRef, PrivateRevisionRef},
    runtime::{actor::messages::ActorOwner, turn::{RunExecutionRequest, host::RunMcpServerInput}, tool_admission::{HttpHostToolAdmissionPort, RunToolAdmissionInput}},
    security::{claims::UserContext, delegation_grants::{DelegationAuthenticated, DelegationGrantAuthority}, sidecar_guard::HostAuthenticated},
};

#[path = "host_context/approval.rs"]
mod approval;
#[path = "host_context/port.rs"]
mod port;

/// Safe identity only; resource URLs, headers and grants never cross this DTO.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DelegatedHostContextReceipt {
    pub context_id: String,
    pub grant_id: String,
    pub workspace_id: String,
    pub runtime_epoch: String,
    pub definition: ImmutableDefinitionRef,
    pub binding: PrivateRevisionRef,
    pub agent_id: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Registration {
    grant_id: String,
    workspace_id: String,
    runtime_epoch: String,
    deployment_binding_id: String,
    definition: ImmutableDefinitionRef,
    working_directory: PathBuf,
    mcp_servers: Vec<RunMcpServerInput>,
    tool_admission: RunToolAdmissionInput,
}

pub(crate) struct DelegatedHostContext {
    pub(crate) receipt: DelegatedHostContextReceipt,
    grants: Arc<DelegationGrantAuthority>,
    principal: UserContext,
    active: AtomicBool,
    working_directory: PathBuf,
    mcp_servers: Vec<RunMcpServerInput>,
    admission_input: RunToolAdmissionInput,
    admission: Arc<HttpHostToolAdmissionPort>,
}

impl std::fmt::Debug for DelegatedHostContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("DelegatedHostContext([private])")
    }
}

impl DelegatedHostContext {
    pub(crate) fn require_live(&self) -> Result<(), ApiError> {
        if !self.active.load(Ordering::Acquire)
            || self.grants.full_harness_grant(&self.receipt.grant_id,
                &self.receipt.workspace_id, &self.receipt.runtime_epoch).is_none()
        {
            return Err(unavailable());
        }
        Ok(())
    }

    pub(crate) async fn attach(
        self: &Arc<Self>,
        state: &crate::uar::api::routes::RunApiState,
        request: &mut RunExecutionRequest,
        run_id: &str,
    ) -> Result<(), ApiError> {
        self.require_live()?;
        let bound = request.collaboration_binding.as_ref().ok_or_else(mismatch)?;
        if bound.workspace_id != self.receipt.workspace_id
            || bound.receipt.package != self.receipt.definition
            || bound.receipt.binding_ref != self.receipt.binding
            || request.artifact.id != self.receipt.agent_id
            || request.verified_owner.as_ref() != Some(&verified_owner(&self.principal)?)
        {
            return Err(mismatch());
        }
        attach_host_resources(&state.manager, &self.principal, request, None,
            Some(self.mcp_servers.clone()), Some(self.working_directory.clone()), None, None)
            .await.map_err(|_| invalid())?;
        request.host_tool_admission = Some(Arc::new(port::DelegatedAdmissionPort::new(
            Arc::clone(self), run_id.to_owned(),
        )));
        self.require_live()
    }
}

/// Process-local registry shared only by the host route and full-harness adapter.
pub struct DelegatedHostContexts {
    grants: Arc<DelegationGrantAuthority>,
    contexts: Mutex<HashMap<String, Arc<DelegatedHostContext>>>,
    runs: Mutex<HashMap<(String, String), String>>,
}

impl DelegatedHostContexts {
    pub fn new(grants: Arc<DelegationGrantAuthority>) -> Self {
        Self { grants, contexts: Mutex::new(HashMap::new()), runs: Mutex::new(HashMap::new()) }
    }

    fn context(&self, id: &str) -> Result<Arc<DelegatedHostContext>, ApiError> {
        let mut contexts = self.contexts.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        contexts.retain(|_, context| context.require_live().is_ok());
        let context = contexts.get(id).cloned().ok_or_else(unavailable)?;
        Ok(context)
    }

    pub(crate) fn for_admission(
        &self, id: &str, delegated: Option<&DelegationAuthenticated>, user: &UserContext,
        workspace: &str, binding_id: Option<&str>,
    ) -> Result<Arc<DelegatedHostContext>, ApiError> {
        let context = self.context(id)?;
        let delegated = delegated.ok_or_else(mismatch)?;
        if delegated.1 != context.receipt.grant_id
            || workspace != context.receipt.workspace_id
            || binding_id != Some(context.receipt.binding.id.as_str())
            || verified_owner(user)? != verified_owner(&context.principal)?
        {
            return Err(mismatch());
        }
        Ok(context)
    }

    pub(crate) fn bind_run(&self, context: &DelegatedHostContext, receipt: &TaskReceipt) {
        self.runs.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert((context.receipt.context_id.clone(), receipt.run_id.clone()), receipt.task_id.clone());
    }

    pub(crate) async fn coordinate_approval(
        &self, state: &FullHarnessApiState, user: &UserContext, receipt: &TaskReceipt,
        approval_id: &str, approved: bool,
    ) -> Result<(), ApiError> {
        let Some(identity) = &receipt.delegated_host_context else { return Ok(()); };
        let context = self.context(&identity.context_id)?;
        if identity != &context.receipt || verified_owner(user)? != verified_owner(&context.principal)? {
            return Err(mismatch());
        }
        approval::coordinate(&context, state, user, receipt, approval_id, approved).await
    }
}

pub(super) async fn register(
    State(state): State<Arc<FullHarnessApiState>>,
    host: Option<Extension<HostAuthenticated>>,
    Extension(user): Extension<UserContext>,
    Json(body): Json<Registration>,
) -> Result<(StatusCode, [(axum::http::HeaderName, &'static str); 1], Json<DelegatedHostContextReceipt>), ApiError> {
    require_host(host)?;
    let grant = state.contexts.grants.full_harness_grant(&body.grant_id,
        &body.workspace_id, &body.runtime_epoch).ok_or_else(unavailable)?;
    if verified_owner(&user)? != verified_owner(&grant.principal)? { return Err(mismatch()); }
    let owner = super::super::user_settings::principal_storage_key(&grant.principal)
        .ok_or_else(mismatch)?;
    let bound = state.runs.collaboration_catalog.resolve_bound_agent_run(&owner,
        &body.workspace_id, &body.deployment_binding_id).await.map_err(|error| {
            use crate::uar::compiler::collaboration::CollaborationError;
            match error {
                CollaborationError::Invalid(message) if message == "ordinary binding requires exactly one AgentDefinition entrypoint"
                    || message == "bound definition has no ordinary-agent projection" =>
                    ApiError::unprocessable("delegated_host_binding_not_agent", "binding has no supported ordinary agent entrypoint", None),
                _ => ApiError::conflict("delegated_host_binding_unavailable", "private binding is not currently admitted", None, None),
            }
        })?;
    if bound.binding.package != body.definition { return Err(mismatch()); }
    let receipt = DelegatedHostContextReceipt {
        context_id: uuid::Uuid::new_v4().to_string(), grant_id: body.grant_id,
        workspace_id: body.workspace_id.clone(), runtime_epoch: body.runtime_epoch,
        definition: bound.binding.package.clone(), binding: bound.effective_binding_receipt.binding_ref.clone(),
        agent_id: bound.artifact.id.clone(), expires_at: grant.expires_at,
    };
    let mut probe = RunExecutionRequest::from_bound_agent(bound, String::new(), owner,
        body.workspace_id, Arc::clone(&state.runs.collaboration_catalog))
        .with_user_context(&grant.principal).map_err(|_| mismatch())?;
    attach_host_resources(&state.runs.manager, &grant.principal, &mut probe, None,
        Some(body.mcp_servers.clone()), Some(body.working_directory), None, None)
        .await.map_err(|_| invalid())?;
    let working_directory = probe.working_directory.ok_or_else(invalid)?;
    let admission = Arc::new(HttpHostToolAdmissionPort::from_input(body.tool_admission.clone()).map_err(|_| invalid())?);
    let context = Arc::new(DelegatedHostContext {
        receipt: receipt.clone(), grants: Arc::clone(&state.contexts.grants), principal: grant.principal,
        active: AtomicBool::new(true), working_directory, mcp_servers: body.mcp_servers,
        admission_input: body.tool_admission, admission,
    });
    context.require_live()?;
    let mut contexts = state.contexts.contexts.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    contexts.retain(|_, context| context.require_live().is_ok());
    contexts.insert(receipt.context_id.clone(), context);
    Ok((StatusCode::CREATED, [(axum::http::header::CACHE_CONTROL, "no-store")], Json(receipt)))
}

pub(super) async fn delete(
    State(state): State<Arc<FullHarnessApiState>>,
    host: Option<Extension<HostAuthenticated>>, Extension(user): Extension<UserContext>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    require_host(host)?;
    let mut contexts = state.contexts.contexts.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(context) = contexts.get(&id) {
        if verified_owner(&user)? != verified_owner(&context.principal)? { return Err(mismatch()); }
        context.active.store(false, Ordering::Release);
        contexts.remove(&id);
    }
    state.contexts.runs.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
        .retain(|(context_id, _), _| context_id != &id);
    Ok(StatusCode::NO_CONTENT)
}

pub(super) async fn get_run(
    State(state): State<Arc<FullHarnessApiState>>,
    host: Option<Extension<HostAuthenticated>>, Extension(user): Extension<UserContext>,
    Path((id, run_id)): Path<(String, String)>, headers: HeaderMap,
) -> Result<Json<TaskReceipt>, ApiError> {
    require_host(host)?;
    let context = state.contexts.context(&id)?;
    let owner = verified_owner(&user)?;
    if owner != verified_owner(&context.principal)?
        || super::handlers::verified_workspace(&headers)? != context.receipt.workspace_id
    { return Err(mismatch()); }
    let task = state.contexts.runs.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
        .get(&(id, run_id.clone())).cloned().ok_or_else(unavailable)?;
    let receipt = state.authority.owned(&owner, &context.receipt.workspace_id, &task)?;
    if receipt.run_id != run_id || receipt.delegated_host_context.as_ref() != Some(&context.receipt) {
        return Err(mismatch());
    }
    Ok(Json(receipt))
}

fn require_host(host: Option<Extension<HostAuthenticated>>) -> Result<(), ApiError> {
    host.ok_or_else(|| ApiError::unauthorized("host_authentication_required", "paired host authentication required"))?;
    Ok(())
}
fn verified_owner(user: &UserContext) -> Result<ActorOwner, ApiError> {
    ActorOwner::from_verified_context(user).map_err(|_| mismatch())
}
fn invalid() -> ApiError {
    ApiError::unprocessable("delegated_host_context_invalid", "private host resources are invalid", None)
}
fn mismatch() -> ApiError {
    ApiError::conflict("delegated_host_context_mismatch", "original grant, workspace, definition and binding must match", None, None)
}
fn unavailable() -> ApiError {
    ApiError::gone("delegated_host_context_unavailable", "original host context or grant expired, was revoked, or is unavailable; replacement does not renew an original run", None, None)
}

pub(crate) fn run_error(error: ApiError) -> super::super::routes::RunApiError {
    let (status, code) = match error.code() {
        "delegated_host_context_unavailable" => (StatusCode::GONE, "delegated_host_context_unavailable"),
        "delegated_host_context_invalid" => (StatusCode::UNPROCESSABLE_ENTITY, "delegated_host_context_invalid"),
        _ => (StatusCode::CONFLICT, "delegated_host_context_mismatch"),
    };
    super::super::routes::RunApiError::delegated_context(status, code, error.to_string())
}
pub(crate) fn run_mismatch() -> super::super::routes::RunApiError { run_error(mismatch()) }
