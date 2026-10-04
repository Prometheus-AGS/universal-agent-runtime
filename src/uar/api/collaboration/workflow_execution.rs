//! Fixed authenticated workflow routes; no caller-supplied owner or workspace authority.
use super::{CollaborationApiState, error_response, private_scope, result_response};
use crate::uar::{
    compiler::collaboration::CollaborationError,
    domain::workflow_execution::{
        StartWorkflowRequest, WorkflowControlRequest, WorkflowDecisionRequest,
    },
    runtime::actor::messages::ActorOwner,
    security::claims::UserContext,
};
use axum::{
    Json, Router,
    extract::{Extension, Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use std::sync::Arc;

pub(super) fn build_router() -> Router<Arc<CollaborationApiState>> {
    Router::new()
        .route("/workflow-definitions", get(definitions))
        .route("/workflow-runs", get(list).post(start))
        .route("/workflow-runs/{id}", get(inspect))
        .route("/workflow-runs/{id}/decide", post(decide))
        .route("/workflow-runs/{id}/cancel", post(cancel))
        .route("/workflow-runs/{id}/recover", post(recover))
}
async fn available(state: &CollaborationApiState) -> Result<(), Response> {
    if !state.runtime.available()
        || !super::super::capabilities::workflow_execution_enabled()
        || !state
            .service
            .execution_ownership_view()
            .await
            .is_ok_and(|v| v.owns_execution)
    {
        return Err(error_response(CollaborationError::Conflict(
            "WORKFLOW_CAPABILITY_UNSUPPORTED".into(),
        )));
    }
    Ok(())
}
async fn definitions(
    State(state): State<Arc<CollaborationApiState>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
) -> Response {
    if let Err(response) = private_scope(&user, &headers) {
        return response;
    }
    result_response(state.service.list_workflow_definitions().await)
}
async fn list(
    State(state): State<Arc<CollaborationApiState>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
) -> Response {
    let (owner, workspace) = match private_scope(&user, &headers) {
        Ok(v) => v,
        Err(r) => return r,
    };
    result_response(state.service.list_workflow_runs(&owner, &workspace).await)
}
async fn inspect(
    State(state): State<Arc<CollaborationApiState>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    let (owner, workspace) = match private_scope(&user, &headers) {
        Ok(v) => v,
        Err(r) => return r,
    };
    result_response(
        state
            .service
            .get_workflow_run(&owner, &workspace, &id)
            .await,
    )
}
async fn start(
    State(state): State<Arc<CollaborationApiState>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
    Json(request): Json<StartWorkflowRequest>,
) -> Response {
    let (owner, workspace) = match private_scope(&user, &headers) {
        Ok(v) => v,
        Err(r) => return r,
    };
    let verified = match ActorOwner::from_verified_context(&user) {
        Ok(v) => v,
        Err(_) => return StatusCode::UNAUTHORIZED.into_response(),
    };
    if let Err(response) = available(&state).await {
        return response;
    }
    match state
        .service
        .start_workflow(&owner, &workspace, request)
        .await
    {
        Ok(run) => {
            state.runtime.activate_workflows(verified).await;
            result_response(
                state
                    .service
                    .get_workflow_run(&owner, &workspace, &run.id)
                    .await,
            )
        }
        Err(error) => error_response(error),
    }
}
async fn decide(
    State(state): State<Arc<CollaborationApiState>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(request): Json<WorkflowDecisionRequest>,
) -> Response {
    let (owner, workspace) = match private_scope(&user, &headers) {
        Ok(v) => v,
        Err(r) => return r,
    };
    if let Err(response) = available(&state).await {
        return response;
    }
    result_response(
        state
            .service
            .decide_workflow(&owner, &workspace, &id, request)
            .await,
    )
}
async fn cancel(
    State(state): State<Arc<CollaborationApiState>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(request): Json<WorkflowControlRequest>,
) -> Response {
    control(state, user, headers, id, request, "cancel").await
}
async fn recover(
    State(state): State<Arc<CollaborationApiState>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(request): Json<WorkflowControlRequest>,
) -> Response {
    control(state, user, headers, id, request, "recover").await
}
async fn control(
    state: Arc<CollaborationApiState>,
    user: UserContext,
    headers: HeaderMap,
    id: String,
    request: WorkflowControlRequest,
    operation: &str,
) -> Response {
    let (_, workspace) = match private_scope(&user, &headers) {
        Ok(v) => v,
        Err(r) => return r,
    };
    let owner = match ActorOwner::from_verified_context(&user) {
        Ok(v) => v,
        Err(_) => return StatusCode::UNAUTHORIZED.into_response(),
    };
    if let Err(response) = available(&state).await {
        return response;
    }
    result_response(
        state
            .runtime
            .control_workflow(owner, &workspace, &id, operation, request)
            .await,
    )
}
