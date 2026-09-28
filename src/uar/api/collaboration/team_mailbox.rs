//! Authenticated, workspace-scoped team inbox routes.

use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Extension, Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
};
use serde::Serialize;

use crate::uar::{
    domain::team_mailbox::{EnqueueTeamMessageRequest, TeamInboxMessage},
    security::claims::UserContext,
};

use super::{CollaborationApiState, error_response, private_scope, result_response};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TeamInboxListResponse {
    messages: Vec<TeamInboxMessage>,
}

pub(super) fn build_router() -> Router<Arc<CollaborationApiState>> {
    Router::new()
        .route(
            "/team-instances/{id}/messages",
            get(list_messages).post(enqueue_message),
        )
        .route(
            "/team-instances/{id}/messages/{message_id}",
            get(get_message),
        )
}

async fn enqueue_message(
    State(state): State<Arc<CollaborationApiState>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
    Path(team_id): Path<String>,
    Json(request): Json<EnqueueTeamMessageRequest>,
) -> Response {
    let (owner, workspace) = match private_scope(&user, &headers) {
        Ok(scope) => scope,
        Err(response) => return response,
    };
    match state
        .service
        .enqueue_team_message(&owner, &workspace, &team_id, request)
        .await
    {
        Ok(message) => (StatusCode::CREATED, Json(message)).into_response(),
        Err(error) => error_response(error),
    }
}

async fn list_messages(
    State(state): State<Arc<CollaborationApiState>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
    Path(team_id): Path<String>,
) -> Response {
    let (owner, workspace) = match private_scope(&user, &headers) {
        Ok(scope) => scope,
        Err(response) => return response,
    };
    match state
        .service
        .list_team_messages(&owner, &workspace, &team_id)
        .await
    {
        Ok(messages) => Json(TeamInboxListResponse { messages }).into_response(),
        Err(error) => error_response(error),
    }
}

async fn get_message(
    State(state): State<Arc<CollaborationApiState>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
    Path((team_id, message_id)): Path<(String, String)>,
) -> Response {
    let (owner, workspace) = match private_scope(&user, &headers) {
        Ok(scope) => scope,
        Err(response) => return response,
    };
    result_response(
        state
            .service
            .get_team_message(&owner, &workspace, &team_id, &message_id)
            .await,
    )
}
