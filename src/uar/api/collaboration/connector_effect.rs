//! Scoped connector administration and host-only effect dispatch.
use super::{CollaborationApiState, error_response, private_scope, result_response};
use crate::uar::{
    domain::connector_effect::{
        ConnectorBindingRequest, ConnectorEffectRequest, ConnectorOutcomeRequest,
    },
    security::{claims::UserContext, sidecar_guard::HostAuthenticated},
};
use axum::{
    Json, Router,
    extract::{Extension, Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct DispatchRequest {
    credential_ref: String,
}

pub(super) fn build_router() -> Router<Arc<CollaborationApiState>> {
    Router::new()
        .route("/connector-bindings", get(bindings).post(install_binding))
        .route("/connector-effects", get(effects).post(prepare))
        .route("/connector-effects/{id}", get(inspect))
        .route("/connector-effects/{id}/dispatch", post(dispatch))
        .route("/connector-effects/{id}/outcome", post(outcome))
        .route("/connector-effects/{id}/reconcile", post(reconcile))
}

fn require_host(marker: Option<Extension<HostAuthenticated>>) -> Result<(), Response> {
    if marker.is_none() {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            "CONNECTOR_CREDENTIAL_BROKER_UNAVAILABLE",
        )
            .into_response());
    }
    Ok(())
}

async fn bindings(
    State(state): State<Arc<CollaborationApiState>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
) -> Response {
    let (owner, workspace) = match private_scope(&user, &headers) {
        Ok(v) => v,
        Err(r) => return r,
    };
    result_response(
        state
            .service
            .list_connector_bindings(&owner, &workspace)
            .await,
    )
}
async fn install_binding(
    State(state): State<Arc<CollaborationApiState>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
    marker: Option<Extension<HostAuthenticated>>,
    Json(request): Json<ConnectorBindingRequest>,
) -> Response {
    let (owner, workspace) = match private_scope(&user, &headers) {
        Ok(v) => v,
        Err(r) => return r,
    };
    if let Err(r) = require_host(marker) {
        return r;
    }
    result_response(
        state
            .service
            .install_connector_binding(&owner, &workspace, request)
            .await,
    )
}
async fn effects(
    State(state): State<Arc<CollaborationApiState>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
) -> Response {
    let (owner, workspace) = match private_scope(&user, &headers) {
        Ok(v) => v,
        Err(r) => return r,
    };
    result_response(
        state
            .service
            .list_connector_effects(&owner, &workspace)
            .await,
    )
}
async fn prepare(
    State(state): State<Arc<CollaborationApiState>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
    marker: Option<Extension<HostAuthenticated>>,
    Json(request): Json<ConnectorEffectRequest>,
) -> Response {
    let (owner, workspace) = match private_scope(&user, &headers) {
        Ok(v) => v,
        Err(r) => return r,
    };
    if let Err(r) = require_host(marker) {
        return r;
    }
    result_response(
        state
            .service
            .prepare_connector_effect(&owner, &workspace, request)
            .await,
    )
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
            .get_connector_effect(&owner, &workspace, &id)
            .await,
    )
}
async fn dispatch(
    State(state): State<Arc<CollaborationApiState>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
    marker: Option<Extension<HostAuthenticated>>,
    Path(id): Path<String>,
    Json(request): Json<DispatchRequest>,
) -> Response {
    let (owner, workspace) = match private_scope(&user, &headers) {
        Ok(v) => v,
        Err(r) => return r,
    };
    if let Err(r) = require_host(marker) {
        return r;
    }
    result_response(
        state
            .service
            .lease_connector_effect(&owner, &workspace, &id, &request.credential_ref)
            .await,
    )
}
async fn outcome(
    State(state): State<Arc<CollaborationApiState>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
    marker: Option<Extension<HostAuthenticated>>,
    Path(id): Path<String>,
    Json(request): Json<ConnectorOutcomeRequest>,
) -> Response {
    let (owner, workspace) = match private_scope(&user, &headers) {
        Ok(v) => v,
        Err(r) => return r,
    };
    if let Err(r) = require_host(marker) {
        return r;
    }
    result_response(
        state
            .service
            .record_connector_outcome(&owner, &workspace, &id, request)
            .await,
    )
}
async fn reconcile(
    State(state): State<Arc<CollaborationApiState>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
    marker: Option<Extension<HostAuthenticated>>,
    Path(id): Path<String>,
    Json(request): Json<ConnectorOutcomeRequest>,
) -> Response {
    let (owner, workspace) = match private_scope(&user, &headers) {
        Ok(v) => v,
        Err(r) => return r,
    };
    if let Err(r) = require_host(marker) {
        return r;
    }
    result_response(
        state
            .service
            .reconcile_connector_effect(&owner, &workspace, &id, request)
            .await,
    )
}
