use std::sync::Arc;

use axum::{
    Extension, Json, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use chrono::Utc;
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use tokio_stream::StreamExt;

use super::{ApiError, FullHarnessTaskAuthority, Reservation, TaskDiagnostic, TaskReceipt};
use crate::uar::{
    api::{
        routes::{
            CreateRunRequest, RunApiState, admit_run, is_terminal_stream_event,
            unrecoverable_stream_gap,
        },
    },
    runtime::actor::messages::ActorOwner,
    security::{claims::UserContext, sidecar_guard::HostAuthenticated},
};

#[derive(Clone)]
pub struct FullHarnessApiState {
    pub authority: Arc<FullHarnessTaskAuthority>,
    pub runs: Arc<RunApiState>,
    pub contexts: Arc<super::host_context::DelegatedHostContexts>,
}

pub fn build_router() -> Router<Arc<FullHarnessApiState>> {
    Router::new()
        .route("/capabilities", get(get_capabilities))
        .route("/delegated-host-contexts", post(super::host_context::register))
        .route("/delegated-host-contexts/{id}", axum::routing::delete(super::host_context::delete))
        .route("/delegated-host-contexts/{id}/runs/{run_id}", get(super::host_context::get_run))
        .route("/tasks", post(admit_task))
        .route("/admissions/{admission_id}", get(get_admission))
        .route("/tasks/{task_id}", get(get_task))
        .route("/tasks/{task_id}/stream", get(stream_task))
        .route("/tasks/{task_id}/tool-approval", post(tool_approval))
        .route("/tasks/{task_id}/cancel", post(cancel_task))
        .route("/tasks/{task_id}/detach", post(detach_task))
        .route("/tasks/{task_id}/steer", post(steer_task))
}

async fn get_capabilities(
    State(state): State<Arc<FullHarnessApiState>>,
    Extension(user): Extension<UserContext>,
) -> Result<Json<super::RuntimeDescriptor>, ApiError> {
    verified_owner(&user)?;
    Ok(Json(state.authority.runtime_descriptor()))
}

async fn admit_task(
    State(state): State<Arc<FullHarnessApiState>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
    host: Option<Extension<HostAuthenticated>>,
    delegated: Option<Extension<crate::uar::security::delegation_grants::DelegationAuthenticated>>,
    Json(mut body): Json<Value>,
) -> Result<(StatusCode, Json<TaskReceipt>), ApiError> {
    let owner = verified_owner(&user)?;
    let workspace_id = verified_workspace(&headers)?;
    let digest = canonical_digest(&body, &workspace_id)?;
    let object = body.as_object_mut().ok_or_else(|| {
        ApiError::bad_request("admission_invalid", "request must be a JSON object")
    })?;
    let admission_id = take_required_string(object, "admission_id")?;
    let native_task_id = take_required_string(object, "native_task_id")?;
    let context_id = object.remove("delegated_host_context_id").map(|value| {
        value.as_str().filter(|id| !id.trim().is_empty()).map(str::to_owned)
            .ok_or_else(|| ApiError::bad_request("delegated_host_context_invalid", "context reference must be a non-empty string"))
    }).transpose()?;
    let run_request: CreateRunRequest = serde_json::from_value(body)
        .map_err(|error| ApiError::bad_request("admission_invalid", error.to_string()))?;
    let reserved = state.authority.reserve(
        owner.clone(),
        workspace_id.clone(),
        admission_id,
        native_task_id,
        digest,
    )?;
    let receipt = match reserved {
        Reservation::Existing(status, receipt) if status.is_success() => {
            return Ok((status, Json(receipt)));
        }
        Reservation::Existing(status, receipt) => {
            return Err(ApiError::from_rejected_receipt(status, &receipt));
        }
        Reservation::New(receipt) => receipt,
    };
    let task_id = receipt.task_id.clone();
    let result = async {
        let context = match context_id {
            Some(id) => {
                if run_request.artifact.is_some() || run_request.agent_id.is_some()
                    || run_request.mcp_servers.is_some() || run_request.tool_admission.is_some()
                    || run_request.run_credentials.is_some() || run_request.working_directory.is_some()
                    || run_request.history.is_some()
                {
                    return Err(super::host_context::run_mismatch());
                }
                let context = state.contexts.for_admission(&id, delegated.as_ref().map(|value| &value.0),
                    &user, &workspace_id, run_request.deployment_binding_id.as_deref())
                    .map_err(super::host_context::run_error)?;
                state.authority.update(&task_id, |record| {
                    record.receipt.delegated_host_context = Some(context.receipt.clone());
                });
                state.contexts.bind_run(&context, &receipt);
                Some(context)
            }
            None => None,
        };
        admit_run(
            Arc::clone(&state.runs),
            user,
            headers,
            host.is_some(),
            run_request,
            Some(receipt.run_id.clone()),
            context,
        )
        .await
    }.await;
    match result {
        Ok(response) => {
            let _ = state.authority.capture_root(&task_id).await;
            let agent_id = state
                .authority
                .manager
                .get_run(&receipt.run_id)
                .await
                .map(|run| run.agent_id);
            state.authority.update(&task_id, |record| {
                record.receipt.state = "submitted".to_owned();
                record.receipt.agent_id = agent_id;
                record.receipt.effective_service_binding = response.effective_service_binding;
                record.receipt.diagnostics = response
                    .activation_failures
                    .into_iter()
                    .map(|failure| TaskDiagnostic {
                        code: serde_json::to_value(&failure)
                            .ok()
                            .and_then(|value| {
                                value.get("code").and_then(Value::as_str).map(str::to_owned)
                            })
                            .unwrap_or_else(|| "skill_activation_failed".to_owned()),
                        message: failure.to_string(),
                    })
                    .collect();
            });
            let admission_receipt = state.authority.freeze_admission_response(&task_id)?;
            state
                .authority
                .monitor(task_id.clone());
            Ok((StatusCode::ACCEPTED, Json(admission_receipt)))
        }
        Err(error) => {
            let status = error.status();
            let code = error.code().to_owned();
            let message = error.message().to_owned();
            state.authority.update(&task_id, |record| {
                let terminal_at = Utc::now();
                record.response_status = status;
                record.receipt.state = "rejected".to_owned();
                record.receipt.terminal_at = Some(terminal_at);
                record.receipt.expires_at = Some(
                    terminal_at
                        + chrono::Duration::seconds(
                            record.receipt.retention.terminal_ttl_seconds as i64,
                        ),
                );
                record.receipt.diagnostics = vec![TaskDiagnostic { code, message }];
            });
            let rejected = state.authority.owned(&owner, &workspace_id, &task_id)?;
            Err(ApiError::from_rejected_receipt(status, &rejected))
        }
    }
}

async fn get_admission(
    State(state): State<Arc<FullHarnessApiState>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<TaskReceipt>, ApiError> {
    let workspace_id = verified_workspace(&headers)?;
    Ok(Json(state.authority.admission(
        &verified_owner(&user)?,
        &workspace_id,
        &id,
    )?))
}
async fn get_task(
    State(state): State<Arc<FullHarnessApiState>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<TaskReceipt>, ApiError> {
    let workspace_id = verified_workspace(&headers)?;
    let owner = verified_owner(&user)?;
    state.authority.owned(&owner, &workspace_id, &id)?;
    let _ = state.authority.refresh_receipt(&id).await;
    Ok(Json(state.authority.owned(
        &owner,
        &workspace_id,
        &id,
    )?))
}

#[derive(Deserialize)]
struct StreamQuery {
    last_event_id: Option<u64>,
}
async fn stream_task(
    State(state): State<Arc<FullHarnessApiState>>,
    Extension(user): Extension<UserContext>,
    Path(id): Path<String>,
    Query(query): Query<StreamQuery>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let workspace_id = verified_workspace(&headers)?;
    let receipt = state
        .authority
        .owned(&verified_owner(&user)?, &workspace_id, &id)?;
    let last_id = query.last_event_id.or_else(|| {
        headers
            .get("last-event-id")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse().ok())
    });
    let mut replay = state
        .authority
        .manager
        .history_since(&receipt.run_id, None)
        .await
        .ok_or_else(|| {
            ApiError::gone(
                "task_unresolved",
                "native run stream is unavailable",
                Some(id.clone()),
                None,
            )
        })?;
    let run_terminal = replay.last().is_some_and(is_terminal_stream_event);
    replay.retain(|event| last_id.is_none_or(|cursor| event.id > cursor));
    if let Some(cursor) = last_id
        && replay
            .first()
            .is_some_and(|event| event.id > cursor.saturating_add(1))
    {
        replay = vec![unrecoverable_stream_gap(&receipt.run_id, cursor)];
    }
    let replay_terminal = run_terminal || replay.last().is_some_and(is_terminal_stream_event);
    if replay_terminal {
        return Ok(super::stream::build_response(tokio_stream::iter(replay), Arc::clone(&state.authority), id).into_response());
    }
    let mut last_seen = replay.last().map_or(last_id.unwrap_or(0), |event| event.id);
    let Some(mut receiver) = state.authority.manager.subscribe(&receipt.run_id).await else {
        return Err(ApiError::gone(
            "task_unresolved",
            "native run stream is unavailable",
            Some(receipt.task_id),
            None,
        ));
    };
    let manager = Arc::clone(&state.authority.manager);
    let run_id = receipt.run_id.clone();
    let live = async_stream::stream! {
        if !replay_terminal {
            loop {
                match receiver.recv().await {
                    Ok(event) if event.id <= last_seen => {}
                    Ok(event) if event.id == last_seen.saturating_add(1) => {
                        last_seen = event.id;
                        let terminal = is_terminal_stream_event(&event);
                        yield event;
                        if terminal { break; }
                    }
                    Ok(_) => { yield unrecoverable_stream_gap(&run_id, last_seen); break; }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                        let Some(recovered) = manager.history_since(&run_id, Some(last_seen)).await else { yield unrecoverable_stream_gap(&run_id, last_seen); break; };
                        let mut complete = true;
                        let mut terminal = false;
                        for event in recovered {
                            if event.id <= last_seen { continue; }
                            if event.id != last_seen.saturating_add(1) { complete = false; break; }
                            last_seen = event.id;
                            terminal = is_terminal_stream_event(&event);
                            yield event;
                            if terminal { break; }
                        }
                        if terminal { break; }
                        if !complete { yield unrecoverable_stream_gap(&run_id, last_seen); break; }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                }
            }
        }
    };
    Ok(super::stream::build_response(tokio_stream::iter(replay).chain(live), Arc::clone(&state.authority), id).into_response())
}

#[derive(Deserialize)]
struct ApprovalRequest {
    expected_revision: u64,
    approved: bool,
    approval_id: String,
}
async fn tool_approval(
    State(state): State<Arc<FullHarnessApiState>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<ApprovalRequest>,
) -> Result<Json<TaskReceipt>, ApiError> {
    let owner = verified_owner(&user)?;
    let workspace_id = verified_workspace(&headers)?;
    if body.approval_id.is_empty() {
        return Err(ApiError::bad_request(
            "approval_invalid",
            "approval_id must not be empty",
        ));
    }
    let receipt =
        state
            .authority
            .require_revision(&owner, &workspace_id, &id, body.expected_revision)?;
    state.contexts.coordinate_approval(&state, &user, &receipt, &body.approval_id, body.approved).await?;
    let resolved = state
        .authority
        .manager
        .resolve_approval_request(&receipt.run_id, Some(&body.approval_id), body.approved)
        .await;
    if !resolved {
        return Err(ApiError::conflict(
            "approval_unresolved",
            "no matching pending approval was resolved",
            Some(id),
            Some(receipt.admission_id),
        ));
    }
    state.authority.update(&id, |record| {
        record.receipt.diagnostics.push(TaskDiagnostic {
            code: "approval_forwarded".to_owned(),
            message: if body.approved {
                "tool approval was forwarded"
            } else {
                "tool rejection was forwarded"
            }
            .to_owned(),
        })
    });
    Ok(Json(state.authority.owned(&owner, &workspace_id, &id)?))
}

#[derive(Deserialize)]
struct MutationRequest {
    expected_revision: u64,
}

async fn cancel_task(
    State(state): State<Arc<FullHarnessApiState>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<MutationRequest>,
) -> Result<Json<TaskReceipt>, ApiError> {
    let owner = verified_owner(&user)?;
    let workspace_id = verified_workspace(&headers)?;
    Ok(Json(
        state
            .authority
            .cancel_at_revision(&owner, &workspace_id, &id, body.expected_revision)
            .await?,
    ))
}

#[derive(Deserialize)]
struct DetachRequest {
    expected_revision: u64,
    observer_id: String,
}

async fn detach_task(
    State(state): State<Arc<FullHarnessApiState>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<DetachRequest>,
) -> Result<Json<TaskReceipt>, ApiError> {
    let owner = verified_owner(&user)?;
    let workspace_id = verified_workspace(&headers)?;
    Ok(Json(state.authority.detach_at_revision(
        &owner,
        &workspace_id,
        &id,
        body.expected_revision,
        &body.observer_id,
    )?))
}
async fn steer_task(
    State(state): State<Arc<FullHarnessApiState>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Response, ApiError> {
    let workspace_id = verified_workspace(&headers)?;
    state
        .authority
        .owned(&verified_owner(&user)?, &workspace_id, &id)?;
    Err(ApiError::unprocessable(
        "capability_unsupported",
        "steer is unsupported by the process-ephemeral full-harness profile",
        Some(id),
    ))
}

fn verified_owner(user: &UserContext) -> Result<ActorOwner, ApiError> {
    ActorOwner::from_verified_context(user)
        .map_err(|_| ApiError::unauthorized("principal_invalid", "verified user context required"))
}

pub(super) fn verified_workspace(headers: &HeaderMap) -> Result<String, ApiError> {
    headers
        .get("x-uar-workspace-id")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| {
            ApiError::bad_request(
                "workspace_required",
                "x-uar-workspace-id must identify the authenticated workspace",
            )
        })
}
fn take_required_string(
    object: &mut serde_json::Map<String, Value>,
    field: &'static str,
) -> Result<String, ApiError> {
    object
        .remove(field)
        .and_then(|value| value.as_str().map(str::to_owned))
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| {
            ApiError::bad_request(
                "admission_invalid",
                format!("{field} must be a non-empty string"),
            )
        })
}
fn canonical_digest(value: &Value, workspace_id: &str) -> Result<String, ApiError> {
    fn sort(value: &Value) -> Value {
        match value {
            Value::Object(map) => {
                let mut entries = map.iter().collect::<Vec<_>>();
                entries.sort_by(|(a, _), (b, _)| a.cmp(b));
                Value::Object(
                    entries
                        .into_iter()
                        .map(|(key, value)| (key.clone(), sort(value)))
                        .collect(),
                )
            }
            Value::Array(values) => Value::Array(values.iter().map(sort).collect()),
            other => other.clone(),
        }
    }
    let bytes = serde_json::to_vec(&serde_json::json!({
        "request": sort(value),
        "workspace_id": workspace_id,
    }))
    .map_err(|error| ApiError::bad_request("admission_invalid", error.to_string()))?;
    let digest = Sha256::digest(bytes);
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write as _;
        write!(&mut hex, "{byte:02x}").expect("writing to a String cannot fail");
    }
    Ok(hex)
}
