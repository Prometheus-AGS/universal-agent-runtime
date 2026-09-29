//! Fixed authenticated owner/workspace team execution administration contract.
use super::{CollaborationApiState, error_response, private_scope, result_response};
use crate::uar::{
    domain::team_execution::{AdmitTeamTaskRequest, TeamControlRequest},
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
        .route("/team-instances/{id}/tasks/{task}/admit", post(admit))
        .route("/team-instances/{id}/execution", get(execution))
        .route(
            "/team-instances/{id}/attempts/{attempt}/cancel",
            post(cancel),
        )
        .route("/team-instances/{id}/recover", post(recover))
}

async fn admit(
    State(state): State<Arc<CollaborationApiState>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
    Path((team, task)): Path<(String, String)>,
    Json(request): Json<AdmitTeamTaskRequest>,
) -> Response {
    let (owner, workspace) = match private_scope(&user, &headers) {
        Ok(scope) => scope,
        Err(response) => return response,
    };
    let verified = match ActorOwner::from_verified_context(&user) {
        Ok(owner) => owner,
        Err(_) => return StatusCode::UNAUTHORIZED.into_response(),
    };
    if !state.runtime.available() {
        return error_response(
            crate::uar::compiler::collaboration::CollaborationError::Conflict(
                "Durable team execution is unavailable for this runtime storage".into(),
            ),
        );
    }
    match state
        .service
        .admit_team_task(&owner, &workspace, &team, &task, request)
        .await
    {
        Ok(attempt) => result_response(state.runtime.dispatch(verified, attempt).await),
        Err(error) => error_response(error),
    }
}

async fn execution(
    State(state): State<Arc<CollaborationApiState>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
    Path(team): Path<String>,
) -> Response {
    let (owner, workspace) = match private_scope(&user, &headers) {
        Ok(scope) => scope,
        Err(response) => return response,
    };
    result_response(
        state
            .service
            .team_execution_summary(&owner, &workspace, &team)
            .await,
    )
}

async fn cancel(
    State(state): State<Arc<CollaborationApiState>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
    Path((team, attempt)): Path<(String, String)>,
    Json(request): Json<TeamControlRequest>,
) -> Response {
    let (_, workspace) = match private_scope(&user, &headers) {
        Ok(scope) => scope,
        Err(response) => return response,
    };
    let owner = match ActorOwner::from_verified_context(&user) {
        Ok(owner) => owner,
        Err(_) => return StatusCode::UNAUTHORIZED.into_response(),
    };
    result_response(
        state
            .runtime
            .cancel(&owner, &workspace, &team, &attempt, request)
            .await,
    )
}

async fn recover(
    State(state): State<Arc<CollaborationApiState>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
    Path(team): Path<String>,
    Json(request): Json<TeamControlRequest>,
) -> Response {
    let (_, workspace) = match private_scope(&user, &headers) {
        Ok(scope) => scope,
        Err(response) => return response,
    };
    let owner = match ActorOwner::from_verified_context(&user) {
        Ok(owner) => owner,
        Err(_) => return StatusCode::UNAUTHORIZED.into_response(),
    };
    result_response(
        state
            .runtime
            .recover(owner, &workspace, &team, request)
            .await,
    )
}
