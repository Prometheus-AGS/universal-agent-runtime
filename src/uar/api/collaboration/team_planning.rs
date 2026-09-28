//! Authenticated, workspace-scoped team planning and task ownership routes.

use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Extension, Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
};

use crate::uar::{
    domain::team_planning::{
        AssignTeamReviewerRequest, AssignTeamTaskRequest, CreateTeamRequest, CreateTeamTaskRequest,
        TransitionTeamTaskRequest,
    },
    security::claims::UserContext,
};

use super::{CollaborationApiState, error_response, owner_key, private_scope, result_response};

pub(super) fn build_router() -> Router<Arc<CollaborationApiState>> {
    Router::new()
        .route("/team-definitions", get(list_definitions))
        .route("/team-instances", get(list_teams).post(create_team))
        .route("/team-instances/{id}", get(get_team))
        .route(
            "/team-instances/{id}/tasks",
            get(list_tasks).post(create_task),
        )
        .route("/team-instances/{id}/tasks/{task_id}", get(get_task))
        .route(
            "/team-instances/{id}/tasks/{task_id}:claim",
            axum::routing::post(claim_task),
        )
        .route(
            "/team-instances/{id}/tasks/{task_id}:reassign",
            axum::routing::post(reassign_task),
        )
        .route(
            "/team-instances/{id}/tasks/{task_id}:reviewer",
            axum::routing::post(assign_reviewer),
        )
        .route(
            "/team-instances/{id}/tasks/{task_id}:state",
            axum::routing::post(transition_task),
        )
}

async fn list_definitions(
    State(state): State<Arc<CollaborationApiState>>,
    Extension(user): Extension<UserContext>,
) -> Response {
    if let Err(response) = owner_key(&user) {
        return response;
    }
    result_response(state.service.list_team_definitions().await)
}

async fn create_team(
    State(state): State<Arc<CollaborationApiState>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
    Json(request): Json<CreateTeamRequest>,
) -> Response {
    let (owner, workspace) = match private_scope(&user, &headers) {
        Ok(scope) => scope,
        Err(response) => return response,
    };
    match state
        .service
        .create_team_instance(&owner, &workspace, request)
        .await
    {
        Ok(team) => (StatusCode::CREATED, Json(team)).into_response(),
        Err(error) => error_response(error),
    }
}

async fn list_teams(
    State(state): State<Arc<CollaborationApiState>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
) -> Response {
    let (owner, workspace) = match private_scope(&user, &headers) {
        Ok(scope) => scope,
        Err(response) => return response,
    };
    result_response(state.service.list_team_instances(&owner, &workspace).await)
}

async fn get_team(
    State(state): State<Arc<CollaborationApiState>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    let (owner, workspace) = match private_scope(&user, &headers) {
        Ok(scope) => scope,
        Err(response) => return response,
    };
    result_response(
        state
            .service
            .get_team_instance(&owner, &workspace, &id)
            .await,
    )
}

async fn create_task(
    State(state): State<Arc<CollaborationApiState>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(request): Json<CreateTeamTaskRequest>,
) -> Response {
    let (owner, workspace) = match private_scope(&user, &headers) {
        Ok(scope) => scope,
        Err(response) => return response,
    };
    match state
        .service
        .create_team_task(&owner, &workspace, &id, request)
        .await
    {
        Ok(team) => (StatusCode::CREATED, Json(team)).into_response(),
        Err(error) => error_response(error),
    }
}

async fn list_tasks(
    State(state): State<Arc<CollaborationApiState>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    let (owner, workspace) = match private_scope(&user, &headers) {
        Ok(scope) => scope,
        Err(response) => return response,
    };
    match state
        .service
        .get_team_instance(&owner, &workspace, &id)
        .await
    {
        Ok(team) => Json(team.tasks).into_response(),
        Err(error) => error_response(error),
    }
}

async fn get_task(
    State(state): State<Arc<CollaborationApiState>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
    Path((id, task_id)): Path<(String, String)>,
) -> Response {
    let (owner, workspace) = match private_scope(&user, &headers) {
        Ok(scope) => scope,
        Err(response) => return response,
    };
    match state
        .service
        .get_team_instance(&owner, &workspace, &id)
        .await
    {
        Ok(team) => match team.tasks.into_iter().find(|task| task.id == task_id) {
            Some(task) => Json(task).into_response(),
            None => error_response(
                crate::uar::compiler::collaboration::CollaborationError::NotFound(task_id),
            ),
        },
        Err(error) => error_response(error),
    }
}

async fn claim_task(
    State(state): State<Arc<CollaborationApiState>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
    Path((id, task_id)): Path<(String, String)>,
    Json(request): Json<AssignTeamTaskRequest>,
) -> Response {
    let (owner, workspace) = match private_scope(&user, &headers) {
        Ok(scope) => scope,
        Err(response) => return response,
    };
    result_response(
        state
            .service
            .claim_team_task(&owner, &workspace, &id, &task_id, request)
            .await,
    )
}

async fn reassign_task(
    State(state): State<Arc<CollaborationApiState>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
    Path((id, task_id)): Path<(String, String)>,
    Json(request): Json<AssignTeamTaskRequest>,
) -> Response {
    let (owner, workspace) = match private_scope(&user, &headers) {
        Ok(scope) => scope,
        Err(response) => return response,
    };
    result_response(
        state
            .service
            .reassign_team_task(&owner, &workspace, &id, &task_id, request)
            .await,
    )
}

async fn assign_reviewer(
    State(state): State<Arc<CollaborationApiState>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
    Path((id, task_id)): Path<(String, String)>,
    Json(request): Json<AssignTeamReviewerRequest>,
) -> Response {
    let (owner, workspace) = match private_scope(&user, &headers) {
        Ok(scope) => scope,
        Err(response) => return response,
    };
    result_response(
        state
            .service
            .assign_team_reviewer(&owner, &workspace, &id, &task_id, request)
            .await,
    )
}

async fn transition_task(
    State(state): State<Arc<CollaborationApiState>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
    Path((id, task_id)): Path<(String, String)>,
    Json(request): Json<TransitionTeamTaskRequest>,
) -> Response {
    let (owner, workspace) = match private_scope(&user, &headers) {
        Ok(scope) => scope,
        Err(response) => return response,
    };
    result_response(
        state
            .service
            .transition_team_task(&owner, &workspace, &id, &task_id, request)
            .await,
    )
}
