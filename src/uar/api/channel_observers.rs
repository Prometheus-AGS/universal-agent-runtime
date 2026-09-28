//! Authenticated channel-source endpoints. They do not extend the C07 API.

use std::sync::Arc;

use axum::{Json, Router, extract::{Extension, Path, State}, http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response}, routing::{get, post}};
use serde::Deserialize;
use serde_json::json;

use crate::uar::{
    persistence::channel_observers::ChannelSourceScope,
    runtime::{actor::messages::ActorOwner,
        observer::{ChannelDeliveryInput, ChannelObserverController, ChannelObserverError, FabricRoutedObserver}},
    security::claims::UserContext,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CreateSubscription {
    observer_instance_id: String,
    source: ChannelSourceScope,
    grant_issuer: String,
    grant_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StatusChange { expected_revision: u64 }

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct HandlerTurn { channel: ChannelDeliveryInput, prompt: String }

pub fn build_router() -> Router<Arc<ChannelObserverController>> {
    Router::new()
        .route("/capabilities", get(capabilities))
        .route("/subscriptions", get(list).post(create))
        .route("/subscriptions/{id}/pause", post(pause))
        .route("/subscriptions/{id}/resume", post(resume))
        .route("/subscriptions/{id}/revoke", post(revoke))
        .route("/subscriptions/{id}/deliveries", post(deliver))
        .route("/subscriptions/{id}/deliveries/{delivery}/acknowledge", post(acknowledge))
        .route("/handler-turns", post(handler_turn))
}

async fn capabilities(State(controller): State<Arc<ChannelObserverController>>) -> Response {
    Json(controller.capabilities()).into_response()
}

async fn create(State(controller): State<Arc<ChannelObserverController>>,
    Extension(user): Extension<UserContext>, headers: HeaderMap,
    Json(request): Json<CreateSubscription>) -> Response {
    let (owner, workspace) = match scope(&user, &headers) { Ok(value) => value, Err(error) => return error };
    match controller.create_subscription(&owner, &workspace, &request.observer_instance_id,
        request.source, &request.grant_issuer, &request.grant_id).await {
        Ok(record) => (StatusCode::CREATED, Json(record)).into_response(),
        Err(error) => error_response(error),
    }
}

async fn list(State(controller): State<Arc<ChannelObserverController>>,
    Extension(user): Extension<UserContext>, headers: HeaderMap) -> Response {
    let (owner, workspace) = match scope(&user, &headers) { Ok(value) => value, Err(error) => return error };
    match controller.list(&owner, &workspace).await {
        Ok(records) => Json(records).into_response(), Err(error) => error_response(error),
    }
}

async fn pause(State(controller): State<Arc<ChannelObserverController>>,
    Extension(user): Extension<UserContext>, headers: HeaderMap, Path(id): Path<String>,
    Json(request): Json<StatusChange>) -> Response {
    change(controller, user, headers, id, request.expected_revision, Some(true), false).await
}

async fn resume(State(controller): State<Arc<ChannelObserverController>>,
    Extension(user): Extension<UserContext>, headers: HeaderMap, Path(id): Path<String>,
    Json(request): Json<StatusChange>) -> Response {
    change(controller, user, headers, id, request.expected_revision, Some(false), false).await
}

async fn revoke(State(controller): State<Arc<ChannelObserverController>>,
    Extension(user): Extension<UserContext>, headers: HeaderMap, Path(id): Path<String>,
    Json(request): Json<StatusChange>) -> Response {
    change(controller, user, headers, id, request.expected_revision, None, true).await
}

async fn change(controller: Arc<ChannelObserverController>, user: UserContext,
    headers: HeaderMap, id: String, revision: u64, paused: Option<bool>, revoke: bool) -> Response {
    let (owner, workspace) = match scope(&user, &headers) { Ok(value) => value, Err(error) => return error };
    match controller.set_status(&owner, &workspace, &id, revision, paused, revoke).await {
        Ok(record) => Json(record).into_response(), Err(error) => error_response(error),
    }
}

async fn deliver(State(controller): State<Arc<ChannelObserverController>>,
    Extension(user): Extension<UserContext>, headers: HeaderMap, Path(id): Path<String>,
    Json(request): Json<FabricRoutedObserver>) -> Response {
    if !trusted_channel_source(&user) { return forbidden(); }
    let (owner, workspace) = match scope(&user, &headers) { Ok(value) => value, Err(error) => return error };
    let input = match request.into_channel_input(&id) {
        Ok(value) => value,
        Err(error) => return error_response(error),
    };
    match controller.deliver(&owner, &workspace, &id, input).await {
        Ok(record) => (StatusCode::ACCEPTED, Json(record)).into_response(),
        Err(error) => error_response(error),
    }
}

async fn acknowledge(State(controller): State<Arc<ChannelObserverController>>,
    Extension(user): Extension<UserContext>, headers: HeaderMap,
    Path((id, delivery)): Path<(String, String)>, Json(request): Json<StatusChange>) -> Response {
    let (owner, workspace) = match scope(&user, &headers) { Ok(value) => value, Err(error) => return error };
    match controller.acknowledge(&owner, &workspace, &id, &delivery, request.expected_revision).await {
        Ok(record) => Json(record).into_response(), Err(error) => error_response(error),
    }
}

async fn handler_turn(State(controller): State<Arc<ChannelObserverController>>,
    Extension(user): Extension<UserContext>, headers: HeaderMap,
    Json(request): Json<HandlerTurn>) -> Response {
    if !trusted_channel_source(&user) { return forbidden(); }
    let (owner, workspace) = match scope(&user, &headers) { Ok(value) => value, Err(error) => return error };
    match controller.execute_selected_handler(&owner, &workspace, request.channel, &request.prompt).await {
        Ok(view) => (StatusCode::ACCEPTED, Json(view)).into_response(),
        Err(error) => error_response(error),
    }
}

fn scope(user: &UserContext, headers: &HeaderMap) -> Result<(ActorOwner, String), Response> {
    let owner = ActorOwner::from_verified_context(user).map_err(|_| (
        StatusCode::UNAUTHORIZED, Json(json!({"error":{"code":"authenticated_owner_required"}})),
    ).into_response())?;
    let workspace = headers.get("x-uar-workspace-id").and_then(|value| value.to_str().ok())
        .map(str::trim).filter(|value| !value.is_empty())
        .ok_or_else(|| (StatusCode::BAD_REQUEST,
            Json(json!({"error":{"code":"workspace_required"}}))).into_response())?;
    Ok((owner, workspace.to_owned()))
}

fn trusted_channel_source(user: &UserContext) -> bool {
    user.claims.roles.as_ref().is_some_and(|roles| roles.iter().any(|role|
        matches!(role.as_str(), "host-session" | "operator" | "admin")))
}

fn forbidden() -> Response {
    (StatusCode::FORBIDDEN, Json(json!({"error":{"code":"channel_source_role_required"}}))).into_response()
}

fn error_response(error: ChannelObserverError) -> Response {
    let status = StatusCode::from_u16(error.status()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    (status, Json(json!({"error":{"code":error.code(),"message":error.to_string()}}))).into_response()
}
