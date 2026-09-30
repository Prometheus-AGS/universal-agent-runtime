//! Authenticated owner/workspace artifact views and membership control.

use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Extension, Path, State},
    http::HeaderMap,
    response::Response,
    routing::{get, post},
};

use crate::uar::{domain::team_execution::TeamControlRequest, security::claims::UserContext};

use super::{CollaborationApiState, private_scope, result_response};

pub(super) fn build_router() -> Router<Arc<CollaborationApiState>> {
    Router::new()
        .route("/team-instances/{id}/artifacts", get(list_artifacts))
        .route(
            "/team-instances/{id}/members/{member_id}/revoke",
            post(revoke_member),
        )
}

async fn list_artifacts(
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
            .list_team_artifacts(&owner, &workspace, &id)
            .await,
    )
}

async fn revoke_member(
    State(state): State<Arc<CollaborationApiState>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
    Path((id, member_id)): Path<(String, String)>,
    Json(request): Json<TeamControlRequest>,
) -> Response {
    let (owner, workspace) = match private_scope(&user, &headers) {
        Ok(scope) => scope,
        Err(response) => return response,
    };
    let result = state
        .service
        .revoke_team_member(&owner, &workspace, &id, &member_id, request)
        .await;
    if result.is_ok() {
        state
            .runtime
            .revoke_member(&owner, &workspace, &id, &member_id)
            .await;
    }
    result_response(result)
}
