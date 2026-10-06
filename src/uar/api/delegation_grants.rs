//! Host-only issuance and revocation of bounded managed-local credentials.

use std::sync::Arc;

use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::{delete, post},
};
use serde_json::json;

use crate::uar::security::{
    claims::UserContext,
    delegation_grants::{DelegationGrantAuthority, IssueGrantRequest},
    sidecar_guard::HostAuthenticated,
};

mod openapi;
pub(crate) use openapi::extend_openapi;

/// This router is mounted only when a supervised sidecar guard exists.
pub fn build_router(authority: Arc<DelegationGrantAuthority>) -> Router<crate::AppState> {
    Router::new()
        .route("/api/uar/delegation-grants", post(issue))
        .route("/api/uar/delegation-grants/{id}", delete(revoke))
        .with_state(authority)
}

async fn issue(
    State(authority): State<Arc<DelegationGrantAuthority>>,
    host: Option<Extension<HostAuthenticated>>,
    user: Option<Extension<UserContext>>,
    Json(request): Json<IssueGrantRequest>,
) -> Response {
    if host.is_none() {
        return error(StatusCode::FORBIDDEN, "host_authentication_required");
    }
    let Some(Extension(user)) = user else {
        return error(StatusCode::UNAUTHORIZED, "principal_required");
    };
    match authority.issue(&user, request) {
        Ok(grant) => (
            StatusCode::CREATED,
            [(header::CACHE_CONTROL, "no-store")],
            Json(grant),
        )
            .into_response(),
        Err(code) => error(StatusCode::BAD_REQUEST, code),
    }
}

async fn revoke(
    State(authority): State<Arc<DelegationGrantAuthority>>,
    host: Option<Extension<HostAuthenticated>>,
    Path(id): Path<String>,
) -> Response {
    if host.is_none() {
        return error(StatusCode::FORBIDDEN, "host_authentication_required");
    }
    authority.revoke(&id);
    StatusCode::NO_CONTENT.into_response()
}

fn error(status: StatusCode, code: &'static str) -> Response {
    (status, Json(json!({"error": {"code": code}}))).into_response()
}
