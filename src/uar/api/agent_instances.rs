//! Authenticated administration of durable ordinary-agent instances.

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
    persistence::agent_instances::{AgentInstanceLimits, InstanceActivationProfile},
    runtime::{
        actor::messages::ActorOwner,
        instance::{AgentInstanceController, AgentInstanceError, EffectDisposition},
    },
    security::claims::UserContext,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CreateInstanceRequest {
    deployment_binding_id: String,
    profile: InstanceActivationProfile,
    #[serde(default)]
    limits: AgentInstanceLimits,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SubmitTurnRequest {
    command_id: String,
    prompt: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct LifecycleRequest {
    command_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ReconcileInstanceRequest {
    target_command_id: String,
    target_attempt_id: String,
    effect_disposition: EffectDisposition,
    receipt_id: String,
}

#[derive(Debug, Default, Deserialize)]
struct EventCursor {
    after: Option<u64>,
}

pub fn build_router() -> Router<Arc<AgentInstanceController>> {
    Router::new()
        .route("/", get(list_instances).post(create_instance))
        .route("/{id}", get(get_instance))
        .route("/{id}/events", get(get_events))
        .route("/{id}/turns", post(submit_turn))
        .route("/{id}/activate", post(activate_instance))
        .route("/{id}/passivate", post(passivate_instance))
        .route("/{id}/drain", post(drain_instance))
        .route("/{id}/disable", post(disable_instance))
        .route("/{id}/restart", post(restart_instance))
        .route("/{id}/cancel", post(cancel_instance))
        .route("/{id}/reconcile", post(reconcile_instance))
}

async fn create_instance(
    State(controller): State<Arc<AgentInstanceController>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
    Json(request): Json<CreateInstanceRequest>,
) -> Response {
    let (owner, workspace) = match scope(&user, &headers) {
        Ok(scope) => scope,
        Err(response) => return response,
    };
    match controller
        .create(
            &owner,
            &workspace,
            &request.deployment_binding_id,
            request.profile,
            request.limits,
        )
        .await
    {
        Ok(view) => (StatusCode::CREATED, Json(view)).into_response(),
        Err(error) => error_response(error),
    }
}

async fn list_instances(
    State(controller): State<Arc<AgentInstanceController>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
) -> Response {
    let (owner, workspace) = match scope(&user, &headers) {
        Ok(scope) => scope,
        Err(response) => return response,
    };
    match controller.list(&owner, &workspace).await {
        Ok(views) => Json(views).into_response(),
        Err(error) => error_response(error),
    }
}

async fn get_instance(
    State(controller): State<Arc<AgentInstanceController>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    let (owner, workspace) = match scope(&user, &headers) {
        Ok(scope) => scope,
        Err(response) => return response,
    };
    match controller.get(&owner, &workspace, &id).await {
        Ok(view) => Json(view).into_response(),
        Err(error) => error_response(error),
    }
}

async fn get_events(
    State(controller): State<Arc<AgentInstanceController>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Query(cursor): Query<EventCursor>,
) -> Response {
    let (owner, workspace) = match scope(&user, &headers) {
        Ok(scope) => scope,
        Err(response) => return response,
    };
    let view = match controller.get(&owner, &workspace, &id).await {
        Ok(view) => view,
        Err(error) => return error_response(error),
    };
    let gap = cursor.after.is_some_and(|after| {
        after >= view.next_event_sequence
            || view
                .events
                .first()
                .is_some_and(|event| event.sequence > after.saturating_add(1))
    });
    if gap {
        return Json(json!({
            "gap": true,
            "snapshot": view,
            "events": [],
        }))
        .into_response();
    }
    let events = view
        .events
        .into_iter()
        .filter(|event| cursor.after.is_none_or(|after| event.sequence > after))
        .collect::<Vec<_>>();
    Json(json!({
        "gap": false,
        "nextCursor": events.last().map(|event| event.sequence).or(cursor.after),
        "events": events,
    }))
    .into_response()
}

async fn submit_turn(
    State(controller): State<Arc<AgentInstanceController>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(request): Json<SubmitTurnRequest>,
) -> Response {
    let (owner, workspace) = match scope(&user, &headers) {
        Ok(scope) => scope,
        Err(response) => return response,
    };
    match controller
        .submit(
            &owner,
            &workspace,
            &id,
            &request.command_id,
            &request.prompt,
        )
        .await
    {
        Ok(view) => (StatusCode::ACCEPTED, Json(view)).into_response(),
        Err(error) => error_response(error),
    }
}

macro_rules! lifecycle_handler {
    ($name:ident, $method:ident) => {
        async fn $name(
            State(controller): State<Arc<AgentInstanceController>>,
            Extension(user): Extension<UserContext>,
            headers: HeaderMap,
            Path(id): Path<String>,
            Json(request): Json<LifecycleRequest>,
        ) -> Response {
            let (owner, workspace) = match scope(&user, &headers) {
                Ok(scope) => scope,
                Err(response) => return response,
            };
            match controller
                .$method(&owner, &workspace, &id, &request.command_id)
                .await
            {
                Ok(view) => Json(view).into_response(),
                Err(error) => error_response(error),
            }
        }
    };
}

lifecycle_handler!(activate_instance, activate);
lifecycle_handler!(passivate_instance, passivate);
lifecycle_handler!(drain_instance, drain);
lifecycle_handler!(disable_instance, disable);
lifecycle_handler!(restart_instance, restart);
lifecycle_handler!(cancel_instance, cancel);

async fn reconcile_instance(
    State(controller): State<Arc<AgentInstanceController>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(request): Json<ReconcileInstanceRequest>,
) -> Response {
    let (owner, workspace) = match scope(&user, &headers) {
        Ok(scope) => scope,
        Err(response) => return response,
    };
    if !user.claims.roles.as_ref().is_some_and(|roles| {
        roles
            .iter()
            .any(|role| matches!(role.as_str(), "operator" | "admin" | "host-session"))
    }) {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"error": {"code": "reconciliation_operator_required"}})),
        )
            .into_response();
    }
    match controller
        .reconcile(
            &owner,
            &workspace,
            &id,
            &request.target_command_id,
            &request.target_attempt_id,
            request.effect_disposition,
            &request.receipt_id,
        )
        .await
    {
        Ok(view) => Json(view).into_response(),
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

fn error_response(error: AgentInstanceError) -> Response {
    let status =
        StatusCode::from_u16(error.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let code = error.code();
    (status, Json(json!({"error": {"code": code}}))).into_response()
}
