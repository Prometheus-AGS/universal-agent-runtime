//! Fixed authenticated owner/workspace team execution administration contract.
use super::{CollaborationApiState, error_response, private_scope, result_response};
use crate::uar::{
    domain::team_execution::{
        AdmitTeamTaskRequest, DispatchTeamAttemptRequest, TeamControlRequest,
        TeamExecutionReclaimRequest,
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
        .route("/team-instances/{id}/host-context", post(host_context))
        .route("/team-instances/{id}/tasks/{task}/admit", post(admit))
        .route(
            "/team-instances/{id}/tasks/{task}/admit-queued",
            post(admit_queued),
        )
        .route(
            "/team-instances/{id}/attempts/{attempt}/dispatch",
            post(dispatch),
        )
        .route("/team-instances/{id}/execution", get(execution))
        .route(
            "/team-instances/{id}/attempts/{attempt}/context",
            get(context),
        )
        .route("/team-instances/{id}/peer-messages", get(peer_messages))
        .route(
            "/team-instances/{id}/attempts/{attempt}/cancel",
            post(cancel),
        )
        .route("/team-instances/{id}/recover", post(recover))
        .route("/execution-owner", get(ownership))
        .route("/execution-owner/reclaim", post(reclaim))
        .route("/execution-owner/quiesce", post(quiesce))
}

async fn host_context(
    State(state): State<Arc<CollaborationApiState>>,
    Extension(user): Extension<UserContext>,
    host: Option<Extension<crate::uar::security::sidecar_guard::HostAuthenticated>>,
    headers: HeaderMap,
    Path(team): Path<String>,
    Json(input): Json<crate::uar::runtime::team_execution::host::TeamHostContextInput>,
) -> Response {
    if host.is_none() {
        return StatusCode::FORBIDDEN.into_response();
    }
    let (_, workspace) = match private_scope(&user, &headers) {
        Ok(scope) => scope,
        Err(response) => return response,
    };
    result_response(
        state
            .runtime
            .attach_host_context(&user, &workspace, &team, input)
            .await,
    )
}

async fn admit_queued(
    State(state): State<Arc<CollaborationApiState>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
    Path((team, task)): Path<(String, String)>,
    Json(request): Json<AdmitTeamTaskRequest>,
) -> Response {
    if let Err(response) = privileged(&state, &user, &headers) {
        return response;
    }
    let (owner, workspace) = match private_scope(&user, &headers) {
        Ok(v) => v,
        Err(r) => return r,
    };
    if !state.runtime.available() {
        return error_response(
            crate::uar::compiler::collaboration::CollaborationError::Conflict(
                "TEAM_CAPABILITY_UNSUPPORTED".into(),
            ),
        );
    }
    result_response(
        state
            .service
            .admit_team_task(&owner, &workspace, &team, &task, request)
            .await,
    )
}

async fn dispatch(
    State(state): State<Arc<CollaborationApiState>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
    Path((team, attempt)): Path<(String, String)>,
    Json(request): Json<DispatchTeamAttemptRequest>,
) -> Response {
    if let Err(response) = privileged(&state, &user, &headers) {
        return response;
    }
    let (owner, workspace) = match private_scope(&user, &headers) {
        Ok(v) => v,
        Err(r) => return r,
    };
    let verified = match ActorOwner::from_verified_context(&user) {
        Ok(v) => v,
        Err(_) => return StatusCode::UNAUTHORIZED.into_response(),
    };
    match state
        .service
        .record_team_dispatch_request(&owner, &workspace, &team, &attempt, request)
        .await
    {
        Ok(attempt) => result_response(state.runtime.dispatch(verified, attempt).await),
        Err(error) => error_response(error),
    }
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

async fn ownership(
    State(state): State<Arc<CollaborationApiState>>,
    Extension(user): Extension<UserContext>,
) -> Response {
    if super::owner_key(&user).is_err() {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    result_response(state.service.execution_ownership_view().await)
}
fn privileged(
    state: &CollaborationApiState,
    user: &UserContext,
    headers: &HeaderMap,
) -> Result<String, Response> {
    let actor = super::owner_key(user)?;
    let supplied = headers.get("x-uar-admin-key").and_then(|v| v.to_str().ok());
    if user.claims.uar_instance_id.is_some()
        || !crate::config::secret_value_matches(&state.admin_key, supplied)
    {
        return Err((StatusCode::FORBIDDEN, Json(serde_json::json!({"error":{"code":"TEAM_RECLAIM_UNAUTHORIZED","messageKey":"collaboration.error.authenticatedOwnerRequired"}}))).into_response());
    }
    Ok(actor)
}
async fn reclaim(
    State(state): State<Arc<CollaborationApiState>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
    Json(request): Json<TeamExecutionReclaimRequest>,
) -> Response {
    let actor = match privileged(&state, &user, &headers) {
        Ok(v) => v,
        Err(r) => return r,
    };
    let decision = format!("admin-key-{}", uuid::Uuid::new_v4());
    result_response(
        state
            .service
            .reclaim_execution_owner(&actor, &decision, request)
            .await,
    )
}
async fn quiesce(
    State(state): State<Arc<CollaborationApiState>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
) -> Response {
    if let Err(response) = privileged(&state, &user, &headers) {
        return response;
    }
    match state.runtime.quiesce().await {
        Ok(evidence) => match state.service.execution_ownership_view().await {
            Ok(view) => {
                Json(serde_json::json!({"fencingEvidenceRef":evidence.id,"claim":view.claim}))
                    .into_response()
            }
            Err(e) => error_response(e),
        },
        Err(_) => error_response(
            crate::uar::compiler::collaboration::CollaborationError::Conflict(
                "TEAM_EFFECTS_UNCERTAIN".into(),
            ),
        ),
    }
}

async fn context(
    State(state): State<Arc<CollaborationApiState>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
    Path((team, attempt)): Path<(String, String)>,
) -> Response {
    let (owner, workspace) = match private_scope(&user, &headers) {
        Ok(v) => v,
        Err(r) => return r,
    };
    result_response(
        state
            .service
            .read_team_context(&owner, &workspace, &team, &attempt)
            .await,
    )
}
async fn peer_messages(
    State(state): State<Arc<CollaborationApiState>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
    Path(team): Path<String>,
) -> Response {
    let (owner, workspace) = match private_scope(&user, &headers) {
        Ok(v) => v,
        Err(r) => return r,
    };
    result_response(
        state
            .service
            .read_team_peer_messages(&owner, &workspace, &team)
            .await,
    )
}
