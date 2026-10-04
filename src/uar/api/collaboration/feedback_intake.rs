//! Owner-scoped feedback administration. Mutations require the trusted host boundary.
use super::{CollaborationApiState, private_scope, result_response};
use crate::uar::{
    domain::feedback_intake::{
        AdmitFeedbackImplementationRequest, AttachFeedbackReviewRequest,
        AuthorizeFeedbackIssueRequest, FeedbackPolicyRequest, LinkFeedbackWorkflowRequest,
        ObserveFeedbackRequest,
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
struct FinalizeRequest {
    expected_revision: u64,
}

pub(super) fn build_router() -> Router<Arc<CollaborationApiState>> {
    Router::new()
        .route("/feedback-intakes", get(list).post(observe))
        .route("/feedback-intakes/{id}", get(inspect))
        .route("/feedback-intakes/{id}/workflow", post(link))
        .route("/feedback-intakes/{id}/finalize", post(finalize))
        .route("/feedback-intakes/{id}/review", post(review))
        .route("/feedback-intakes/{id}/issue-approval", post(issue))
        .route(
            "/feedback-intakes/{id}/implementation",
            post(implementation),
        )
        .route("/feedback-policies", get(policies).post(policy))
}

fn require_host(marker: Option<Extension<HostAuthenticated>>) -> Result<(), Response> {
    if marker.is_none() {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            "FEEDBACK_TRUSTED_HOST_UNAVAILABLE",
        )
            .into_response());
    }
    Ok(())
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
    result_response(
        state
            .service
            .list_feedback_intakes(&owner, &workspace)
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
            .get_feedback_intake(&owner, &workspace, &id)
            .await,
    )
}
async fn observe(
    State(state): State<Arc<CollaborationApiState>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
    marker: Option<Extension<HostAuthenticated>>,
    Json(request): Json<ObserveFeedbackRequest>,
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
            .observe_feedback(&owner, &workspace, request)
            .await,
    )
}
async fn link(
    State(state): State<Arc<CollaborationApiState>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
    marker: Option<Extension<HostAuthenticated>>,
    Path(id): Path<String>,
    Json(request): Json<LinkFeedbackWorkflowRequest>,
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
            .link_feedback_workflow(&owner, &workspace, &id, request)
            .await,
    )
}
async fn finalize(
    State(state): State<Arc<CollaborationApiState>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
    marker: Option<Extension<HostAuthenticated>>,
    Path(id): Path<String>,
    Json(request): Json<FinalizeRequest>,
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
            .finalize_feedback(&owner, &workspace, &id, request.expected_revision)
            .await,
    )
}
async fn review(
    State(state): State<Arc<CollaborationApiState>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
    marker: Option<Extension<HostAuthenticated>>,
    Path(id): Path<String>,
    Json(request): Json<AttachFeedbackReviewRequest>,
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
            .attach_feedback_review(&owner, &workspace, &id, request)
            .await,
    )
}
async fn issue(
    State(state): State<Arc<CollaborationApiState>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
    marker: Option<Extension<HostAuthenticated>>,
    Path(id): Path<String>,
    Json(request): Json<AuthorizeFeedbackIssueRequest>,
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
            .authorize_feedback_issue(&owner, &workspace, &id, request)
            .await,
    )
}
async fn implementation(
    State(state): State<Arc<CollaborationApiState>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
    marker: Option<Extension<HostAuthenticated>>,
    Path(id): Path<String>,
    Json(request): Json<AdmitFeedbackImplementationRequest>,
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
            .admit_feedback_implementation(&owner, &workspace, &id, &owner, request)
            .await,
    )
}
async fn policies(
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
            .list_feedback_policies(&owner, &workspace)
            .await,
    )
}
async fn policy(
    State(state): State<Arc<CollaborationApiState>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
    marker: Option<Extension<HostAuthenticated>>,
    Json(request): Json<FeedbackPolicyRequest>,
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
            .install_feedback_policy(&owner, &workspace, request)
            .await,
    )
}
