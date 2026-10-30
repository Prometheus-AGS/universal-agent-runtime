//! Installed assistant operations. Discovery is descriptive, never authority.
use std::{collections::BTreeSet, sync::Arc};
use axum::{Json, Router, extract::{Extension, Path, State}, http::StatusCode,
    response::{IntoResponse, Response}, routing::get};
use serde::Serialize;
use serde_json::json;
use crate::{AppState, uar::{runtime::{actor::messages::ActorOwner,
    schedules::ScheduleController, followups::FollowupController}, security::claims::UserContext}};

#[derive(Clone)]
pub struct AssistantApiState {
    pub app: AppState,
    pub schedules: Arc<ScheduleController>,
    pub followups: Arc<FollowupController>,
    pub supervisor_registered: bool,
}

impl std::fmt::Debug for AssistantApiState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AssistantApiState").finish_non_exhaustive()
    }
}

pub fn router(state: AssistantApiState) -> Router<AppState> {
    Router::new()
        .route("/api/uar/assistant-capabilities", get(discover))
        .route("/api/uar/runs/{run_id}/tool-outcomes", get(outcomes))
        .with_state(state)
}

#[derive(Serialize)]
struct EntryPoint {
    transport: &'static str,
    available: bool,
    reason: Option<&'static str>,
    method: Option<&'static str>,
    target: Option<String>,
}
#[derive(Serialize)]
struct Operation {
    id: &'static str,
    revision: u32,
    authority: &'static str,
    persistence: &'static str,
    entry_points: Vec<EntryPoint>,
}

fn entry(transport: &'static str, available: bool, reason: &'static str,
    method: Option<&'static str>, target: Option<String>) -> EntryPoint {
    EntryPoint { transport, available, reason: (!available).then_some(reason), method, target }
}

fn operation(id: &'static str, method: &'static str, path: &'static str,
    supported: bool, reason: &'static str, native: Option<&str>, mcp: Option<String>,
    native_names: &BTreeSet<String>, authority: &'static str) -> Operation {
    let native_registered = native.is_some_and(|name| native_names.contains(name));
    Operation { id, revision: 1, authority, persistence: if supported { "durable" } else { "unavailable" },
        entry_points: vec![
            entry("http", supported, reason, Some(method), Some(path.into())),
            entry("native_tool", supported && native_registered,
                if supported { "tool_not_registered" } else { reason }, None, native.map(str::to_owned)),
            entry("mcp", supported && mcp.is_some(),
                if supported { "tool_not_registered" } else { reason }, Some("tools/call"), mcp),
            entry("direct_embedded_runtime", false, "host_assistant_contract_not_exposed", None, None),
        ] }
}

fn owner(user: &UserContext) -> Result<ActorOwner, Response> {
    let owner = ActorOwner::from_verified_context(user).map_err(|_| StatusCode::FORBIDDEN.into_response())?;
    if owner.credential_principal().is_none() { return Err(StatusCode::FORBIDDEN.into_response()); }
    Ok(owner)
}

async fn discover(State(state): State<AssistantApiState>, Extension(user): Extension<UserContext>) -> Response {
    if let Err(error) = owner(&user) { return error; }
    let app = &state.app;
    let mut names: BTreeSet<String> = app.native_skill_registry.descriptors().await.iter()
        .map(|descriptor| descriptor.provider_name.clone()).collect();
    names.extend(app.mcp.descriptors().iter().map(|descriptor| descriptor.provider_name.clone()));
    let memory = app.memory_service.as_ref();
    let memory_mcp = memory.map(|service| service.config().mcp_http_path.as_str());
    let mut operations = Vec::new();
    for (id, method, path, native, mcp) in [
        ("memory.policy.read", "GET", "/api/memory/policy", Some("memory_capture_policy"), Some("memory_capture_policy")),
        ("memory.policy.update", "PUT", "/api/memory/policy", None, Some("memory_set_capture_policy")),
        ("memory.export", "GET", "/api/memory/records", None, None),
        ("memory.read", "GET", "/api/memory/records/{id}", None, Some("memory_get")),
        ("memory.update", "PATCH", "/api/memory/records/{id}", Some("memory_update_by_id"), Some("memory_update")),
        ("memory.delete", "DELETE", "/api/memory/records/{id}", Some("memory_delete_by_id"), Some("memory_delete")),
        ("memory.history", "GET", "/api/memory/records/{id}/history", Some("memory_history"), Some("memory_history")),
        ("memory.control", "GET", "/api/memory/records/{id}/control", None, None),
    ] {
        operations.push(operation(id, method, path, memory.is_some(), "memory_disabled", native,
            memory_mcp.zip(mcp).map(|(endpoint, tool)| format!("{endpoint}#{tool}")), &names, "verified_owner"));
    }
    for (id, method, path, native) in [
        ("schedule.create", "POST", "/api/uar/schedules/", "schedule_create"),
        ("schedule.list", "GET", "/api/uar/schedules/", "schedule_list"),
        ("schedule.read", "GET", "/api/uar/schedules/{id}", "schedule_get"),
        ("schedule.update", "PUT", "/api/uar/schedules/{id}", "schedule_update"),
        ("schedule.pause", "POST", "/api/uar/schedules/{id}/pause", "schedule_pause"),
        ("schedule.resume", "POST", "/api/uar/schedules/{id}/resume", "schedule_resume"),
        ("schedule.cancel", "POST", "/api/uar/schedules/{id}/cancel", "schedule_cancel"),
    ] {
        operations.push(operation(id, method, path, state.schedules.supported(), "capability_unsupported",
            Some(native), None, &names, "verified_owner_and_workspace"));
    }
    let db = app.persistence.as_ref();
    let approvals = db.is_some_and(|db| db.supports_durable_approvals());
    let checkpoints = db.is_some_and(|db| db.supports_durable_checkpoint_resume());
    for (id, method, path, available, reason, authority) in [
        ("tool.outcomes", "GET", "/api/uar/runs/{run_id}/tool-outcomes", durable_outcomes(app), "durable_tool_outcomes_unsupported", "verified_owner"),
        ("approval.read", "GET", "/api/uar/runs/{run_id}/tool-approval", approvals, "durable_approvals_unsupported", "verified_owner_and_workspace"),
        ("approval.decide", "POST", "/api/uar/runs/{run_id}/tool-approval", approvals, "durable_approvals_unsupported", "verified_owner_and_current_admission"),
        ("checkpoint.list", "GET", "/api/uar/runs/{run_id}/checkpoints", checkpoints, "checkpoint_resume_unsupported", "verified_owner"),
        ("checkpoint.resume", "POST", "/api/uar/runs/{run_id}/resume/{checkpoint_id}", checkpoints, "checkpoint_resume_unsupported", "verified_owner_and_current_admission"),
        ("recovery.challenge", "POST", "/api/uar/recovery/supervisor/challenges", state.supervisor_registered, "qualified_supervisor_unavailable", "trusted_host"),
        ("recovery.exclude", "POST", "/api/uar/recovery/supervisor/challenges/{id}/exclude", state.supervisor_registered, "qualified_supervisor_unavailable", "trusted_host"),
    ] { operations.push(operation(id, method, path, available, reason, None, None, &names, authority)); }
    // Follow-up entries are supplied by the controller that owns the same routes.
    operations.extend(followup_operations(&state, &names));
    let owns_execution = app.collaboration_catalog.execution_ownership_view().await
        .is_ok_and(|view| view.owns_execution);
    let backend = app.collaboration_catalog.storage_descriptor().backend;
    let profile = match backend { "surrealkv" => "local_surreal_kv", "surrealdb" => "networked_surreal_db",
        "postgresql" => "postgres", _ => "non_durable" };
    Json(json!({"schema_version":1,"backend":backend,"deployment_profile":profile,
        "entry_point":if app.sidecar_mode {"supervised_sidecar_http"} else {"server_http"},
        "execution_owner":owns_execution,"operations":operations,
        "availability_semantics":"Installed operation support; current owner, binding, consent, budget, approval and storage health are checked again on invocation. This is not a release certification.",
        "external_followup_connectors":{"available":false,"reason":"not_implemented"},
        "native_recovery":"Requires a confirmed same-host reboot unless an implemented qualified mechanism proves descendant exclusion. Exclusion never settles unknown external effects."})).into_response()
}

fn durable_outcomes(app: &AppState) -> bool {
    app.persistence.as_ref().is_some_and(|db| db.supports_durable_tool_receipts())
}

fn followup_operations(state: &AssistantApiState, names: &BTreeSet<String>) -> Vec<Operation> {
    [
        ("followup.create", "POST", "/api/uar/followups/", "followup_create"),
        ("followup.list", "GET", "/api/uar/followups/", "followup_list"),
        ("followup.read", "GET", "/api/uar/followups/{id}", "followup_get"),
        ("followup.update", "PUT", "/api/uar/followups/{id}", "followup_update"),
        ("followup.enable", "POST", "/api/uar/followups/{id}/enable", "followup_enable"),
        ("followup.disable", "POST", "/api/uar/followups/{id}/disable", "followup_disable"),
        ("followup.revoke", "POST", "/api/uar/followups/{id}/revoke", "followup_revoke"),
        ("followup.inbox", "GET", "/api/uar/followups/inbox/{conversation}", "followup_inbox"),
        ("followup.acknowledge", "POST", "/api/uar/followups/inbox/{conversation}/{item}/acknowledge", "followup_acknowledge"),
    ].into_iter().map(|(id, method, path, native)| operation(id, method, path,
        state.followups.supported(), "capability_unsupported", Some(native), None, names,
        "verified_owner_and_workspace")).collect()
}

async fn outcomes(State(state): State<AssistantApiState>, Extension(user): Extension<UserContext>,
    Path(run_id): Path<String>) -> Response {
    let owner = match owner(&user) { Ok(owner) => owner, Err(error) => return error };
    if !durable_outcomes(&state.app) {
        return (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"code":"durable_tool_outcomes_unsupported"}))).into_response();
    }
    let Some(db) = &state.app.persistence else { return StatusCode::SERVICE_UNAVAILABLE.into_response(); };
    let Some(principal) = owner.credential_principal() else { return StatusCode::FORBIDDEN.into_response(); };
    let principal_key = principal.storage_key();
    let exact_live_owner = state.app.run_manager.get_run_for_context(&user, &run_id).await.is_some();
    match db.list_canonical_tool_receipts(owner.user_id(), &run_id).await {
        Ok(receipts) => Json(json!({"run_id":run_id,"receipts":receipts.into_iter()
            .filter(|receipt| match &receipt.principal_key {
                Some(key) => key == &principal_key,
                None => exact_live_owner,
            }).map(|receipt| json!({
            "call_id":receipt.call_id,"sequence":receipt.sequence,"tool":receipt.tool,
            "outcome":receipt.outcome.unwrap_or(crate::mcp::outcome::ToolOutcome::OutcomeUnknown),
            "legacy_uncertain":receipt.outcome.is_none(),"completeness":receipt.completeness
        })).collect::<Vec<_>>()})).into_response(),
        Err(_) => (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"code":"tool_outcomes_storage_unavailable"}))).into_response(),
    }
}
