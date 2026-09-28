//! Authenticated local observer administration over durable C06 occurrences.

use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Extension, Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::Deserialize;
use serde_json::json;

use crate::uar::{
    persistence::observers::ObserverLimits,
    runtime::{
        actor::messages::ActorOwner,
        observer::{ObserverController, ObserverError},
    },
    security::claims::UserContext,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CreateObserverRequest {
    observer_instance_id: String,
    source_instance_ids: Vec<String>,
    conversation_ids: Option<Vec<String>>,
    #[serde(default)]
    limits: ObserverLimits,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RevisionRequest {
    expected_revision: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AcknowledgeGapRequest {
    expected_revision: u64,
    source_instance_id: String,
    missing_from: u64,
    missing_through: u64,
}

pub fn build_router() -> Router<Arc<ObserverController>> {
    Router::new()
        .route("/", get(list_observers).post(create_observer))
        .route("/{id}", get(get_observer).delete(revoke_observer))
        .route("/{id}/pause", post(pause_observer))
        .route("/{id}/resume", post(resume_observer))
        .route("/{id}/gaps/acknowledge", post(acknowledge_gap))
}

async fn create_observer(
    State(controller): State<Arc<ObserverController>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
    Json(request): Json<CreateObserverRequest>,
) -> Response {
    let (owner, workspace) = match scope(&user, &headers) {
        Ok(scope) => scope,
        Err(response) => return response,
    };
    match controller
        .create(
            &owner,
            &workspace,
            &request.observer_instance_id,
            request.source_instance_ids,
            request.conversation_ids,
            request.limits,
        )
        .await
    {
        Ok(status) => (StatusCode::CREATED, Json(status)).into_response(),
        Err(error) => error_response(error),
    }
}

async fn list_observers(
    State(controller): State<Arc<ObserverController>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
) -> Response {
    let (owner, workspace) = match scope(&user, &headers) {
        Ok(scope) => scope,
        Err(response) => return response,
    };
    match controller.list(&owner, &workspace).await {
        Ok(statuses) => Json(statuses).into_response(),
        Err(error) => error_response(error),
    }
}

async fn get_observer(
    State(controller): State<Arc<ObserverController>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    let (owner, workspace) = match scope(&user, &headers) {
        Ok(scope) => scope,
        Err(response) => return response,
    };
    match controller.get(&owner, &workspace, &id).await {
        Ok(status) => Json(status).into_response(),
        Err(error) => error_response(error),
    }
}

async fn pause_observer(
    State(controller): State<Arc<ObserverController>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(request): Json<RevisionRequest>,
) -> Response {
    set_paused(
        controller,
        user,
        headers,
        id,
        request.expected_revision,
        true,
    )
    .await
}

async fn resume_observer(
    State(controller): State<Arc<ObserverController>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(request): Json<RevisionRequest>,
) -> Response {
    set_paused(
        controller,
        user,
        headers,
        id,
        request.expected_revision,
        false,
    )
    .await
}

async fn set_paused(
    controller: Arc<ObserverController>,
    user: UserContext,
    headers: HeaderMap,
    id: String,
    expected_revision: u64,
    paused: bool,
) -> Response {
    let (owner, workspace) = match scope(&user, &headers) {
        Ok(scope) => scope,
        Err(response) => return response,
    };
    match controller
        .set_paused(&owner, &workspace, &id, expected_revision, paused)
        .await
    {
        Ok(status) => Json(status).into_response(),
        Err(error) => error_response(error),
    }
}

async fn revoke_observer(
    State(controller): State<Arc<ObserverController>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Query(request): Query<RevisionRequest>,
) -> Response {
    let (owner, workspace) = match scope(&user, &headers) {
        Ok(scope) => scope,
        Err(response) => return response,
    };
    match controller
        .revoke(&owner, &workspace, &id, request.expected_revision)
        .await
    {
        Ok(status) => Json(status).into_response(),
        Err(error) => error_response(error),
    }
}

async fn acknowledge_gap(
    State(controller): State<Arc<ObserverController>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(request): Json<AcknowledgeGapRequest>,
) -> Response {
    if !operator(&user) {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"error": {"code": "observer_operator_required"}})),
        )
            .into_response();
    }
    let (owner, workspace) = match scope(&user, &headers) {
        Ok(scope) => scope,
        Err(response) => return response,
    };
    match controller
        .acknowledge_gap(
            &owner,
            &workspace,
            &id,
            request.expected_revision,
            &request.source_instance_id,
            request.missing_from,
            request.missing_through,
        )
        .await
    {
        Ok(status) => Json(status).into_response(),
        Err(error) => error_response(error),
    }
}

fn scope(user: &UserContext, headers: &HeaderMap) -> Result<(ActorOwner, String), Response> {
    let owner = ActorOwner::from_verified_context(user).map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": {"code": "authenticated_owner_required"}})),
        )
            .into_response()
    })?;
    let workspace = headers
        .get("x-uar-workspace-id")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": {"code": "workspace_required"}})),
            )
                .into_response()
        })?;
    Ok((owner, workspace.to_owned()))
}

fn operator(user: &UserContext) -> bool {
    user.claims.roles.as_ref().is_some_and(|roles| {
        roles
            .iter()
            .any(|role| matches!(role.as_str(), "operator" | "admin" | "host-session"))
    })
}

fn error_response(error: ObserverError) -> Response {
    let status =
        StatusCode::from_u16(error.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    (status, Json(json!({"error": {"code": error.code()}}))).into_response()
}
