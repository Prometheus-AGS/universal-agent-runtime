//! Authenticated durable schedule administration; no caller-supplied owner.
use std::sync::Arc;
use axum::{Json, Router, extract::{Extension, Path, State}, http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response}, routing::{get, post}};
use serde::Deserialize;
use serde_json::json;
use crate::uar::{domain::schedules::ScheduleRequest, runtime::{actor::messages::ActorOwner,
    schedules::{ScheduleController, ScheduleError}}, security::claims::UserContext};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Revision { expected_revision: u64 }
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Edit { expected_revision: u64, schedule: ScheduleRequest }

pub fn build_router() -> Router<Arc<ScheduleController>> {
    Router::new().route("/", get(list).post(create))
        .route("/{id}", get(show).put(edit))
        .route("/{id}/{operation}", post(control))
}

fn scope(user: &UserContext, headers: &HeaderMap) -> Result<(ActorOwner, String), ScheduleError> {
    let owner = ActorOwner::from_verified_context(user).map_err(|_| ScheduleError::Denied)?;
    if owner.credential_principal().is_none() { return Err(ScheduleError::Denied); }
    let workspace = headers.get("x-uar-workspace-id").and_then(|v| v.to_str().ok()).map(str::trim)
        .filter(|v| !v.is_empty()).ok_or(ScheduleError::Invalid("workspace header is required"))?;
    Ok((owner, workspace.to_owned()))
}

pub(crate) fn error(error: ScheduleError) -> Response {
    let status = match error { ScheduleError::Invalid(_) => StatusCode::BAD_REQUEST,
        ScheduleError::Conflict(_) => StatusCode::CONFLICT, ScheduleError::NotFound => StatusCode::NOT_FOUND,
        ScheduleError::Denied => StatusCode::FORBIDDEN, ScheduleError::Unsupported | ScheduleError::CapabilityUnsupported(_) | ScheduleError::Storage => StatusCode::SERVICE_UNAVAILABLE };
    match error.capability() {
        Some(capability) => (status, Json(crate::uar::persistence::capabilities::refusal_body(capability))).into_response(),
        None => (status, Json(json!({"error":{"code":error.code()}}))).into_response(),
    }
}

async fn create(State(controller): State<Arc<ScheduleController>>, Extension(user): Extension<UserContext>,
    headers: HeaderMap, Json(request): Json<ScheduleRequest>) -> Response {
    let (owner, workspace) = match scope(&user, &headers) { Ok(v) => v, Err(e) => return error(e) };
    match controller.create(&owner, &workspace, request).await {
        Ok(v) => (StatusCode::CREATED, Json(v)).into_response(), Err(e) => error(e) }
}
async fn list(State(controller): State<Arc<ScheduleController>>, Extension(user): Extension<UserContext>, headers: HeaderMap) -> Response {
    let (owner, workspace) = match scope(&user, &headers) { Ok(v) => v, Err(e) => return error(e) };
    match controller.list(&owner, &workspace).await { Ok(v) => Json(v).into_response(), Err(e) => error(e) }
}
async fn show(State(controller): State<Arc<ScheduleController>>, Extension(user): Extension<UserContext>,
    headers: HeaderMap, Path(id): Path<String>) -> Response {
    let (owner, workspace) = match scope(&user, &headers) { Ok(v) => v, Err(e) => return error(e) };
    match controller.get(&owner, &workspace, &id).await { Ok(v) => Json(v).into_response(), Err(e) => error(e) }
}
async fn edit(State(controller): State<Arc<ScheduleController>>, Extension(user): Extension<UserContext>,
    headers: HeaderMap, Path(id): Path<String>, Json(request): Json<Edit>) -> Response {
    let (owner, workspace) = match scope(&user, &headers) { Ok(v) => v, Err(e) => return error(e) };
    match controller.revise(&owner, &workspace, &id, request.expected_revision, request.schedule).await {
        Ok(v) => Json(v).into_response(), Err(e) => error(e) }
}
async fn control(State(controller): State<Arc<ScheduleController>>, Extension(user): Extension<UserContext>,
    headers: HeaderMap, Path((id, operation)): Path<(String, String)>, Json(request): Json<Revision>) -> Response {
    let (owner, workspace) = match scope(&user, &headers) { Ok(v) => v, Err(e) => return error(e) };
    match controller.control(&owner, &workspace, &id, request.expected_revision, &operation).await {
        Ok(v) => Json(v).into_response(), Err(e) => error(e) }
}
