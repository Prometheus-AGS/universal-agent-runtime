//! Authenticated I1 collaboration package and deployment-binding resources.

mod team_mailbox;
mod team_planning;

use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Extension, Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::Serialize;
use serde_json::json;

use crate::uar::compiler::collaboration::{
    CollaborationCatalogService, CollaborationError, GrantCommandRequest, PackageExportRequest,
};
use crate::uar::domain::collaboration::{BindingCommandRequest, PackageSourceRequest};
use crate::uar::security::claims::UserContext;

#[derive(Debug, Clone)]
pub struct CollaborationApiState {
    pub service: Arc<CollaborationCatalogService>,
    pub service_instance: Arc<crate::uar::service_instance::ServiceInstanceAuthority>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CollaborationCapabilitiesResponse {
    #[serde(flatten)]
    runtime: super::capabilities::CapabilitiesResponse,
    binding_owner_id: String,
}

/// The versioned collaboration administration contract consumed by host adapters.
pub fn build_router() -> Router<Arc<CollaborationApiState>> {
    Router::new()
        .merge(team_planning::build_router())
        .merge(team_mailbox::build_router())
        .route("/capabilities", get(collaboration_capabilities))
        .route("/packages:preflight", post(preflight_package))
        .route("/packages:install", post(install_package))
        .route("/packages", get(list_packages))
        .route("/packages:export", post(export_package))
        .route(
            "/packages/{id}/versions/{version}",
            get(get_package_version),
        )
        .route("/deployment-bindings:preflight", post(preflight_binding))
        .route(
            "/deployment-bindings",
            get(list_bindings).post(install_binding),
        )
        .route("/deployment-bindings/{id}", get(get_binding))
        .route(
            "/deployment-bindings/{id}/effective-receipt",
            get(get_effective_binding_receipt),
        )
        .route(
            "/deployment-bindings/{id}/template:export",
            post(export_binding_template),
        )
        .route(
            "/representation-grants",
            get(list_representation_grants).post(install_representation_grant),
        )
        .route("/representation-grants/{id}", get(get_representation_grant))
        .route(
            "/representation-grants/{id}/revisions/{revision}",
            get(get_representation_grant_revision),
        )
        .route(
            "/representation-grants/{id}/history",
            get(list_representation_grant_history),
        )
}

async fn collaboration_capabilities(
    State(state): State<Arc<CollaborationApiState>>,
    Extension(user): Extension<UserContext>,
) -> Response {
    let binding_owner_id = match owner_key(&user) {
        Ok(owner) => owner,
        Err(response) => return response,
    };
    let runtime = super::capabilities::capabilities_response(&state.service_instance);
    Json(CollaborationCapabilitiesResponse {
        runtime,
        binding_owner_id,
    })
    .into_response()
}

async fn list_packages(
    State(state): State<Arc<CollaborationApiState>>,
    Extension(user): Extension<UserContext>,
) -> Response {
    if owner_key(&user).is_err() {
        return unauthorized();
    }
    result_response(state.service.list_packages().await)
}

async fn export_package(
    State(state): State<Arc<CollaborationApiState>>,
    Extension(user): Extension<UserContext>,
    Json(request): Json<PackageExportRequest>,
) -> Response {
    if owner_key(&user).is_err() {
        return unauthorized();
    }
    result_response(state.service.export_package(&request).await)
}

async fn preflight_package(
    State(state): State<Arc<CollaborationApiState>>,
    Extension(user): Extension<UserContext>,
    Json(request): Json<PackageSourceRequest>,
) -> Response {
    if owner_key(&user).is_err() {
        return unauthorized();
    }
    match state.service.preflight_package(&request).await {
        Ok(response) => Json(response).into_response(),
        Err(error) => error_response(error),
    }
}

async fn install_package(
    State(state): State<Arc<CollaborationApiState>>,
    Extension(user): Extension<UserContext>,
    Json(request): Json<PackageSourceRequest>,
) -> Response {
    let owner = match owner_key(&user) {
        Ok(owner) => owner,
        Err(response) => return response,
    };
    match state.service.install_package(&owner, request).await {
        Ok(response) => (StatusCode::CREATED, Json(response)).into_response(),
        Err(error) => error_response(error),
    }
}

async fn get_package_version(
    State(state): State<Arc<CollaborationApiState>>,
    Extension(user): Extension<UserContext>,
    Path((id, version)): Path<(String, String)>,
) -> Response {
    if owner_key(&user).is_err() {
        return unauthorized();
    }
    match state.service.get_package_version(&id, &version).await {
        Ok(package) => Json(package).into_response(),
        Err(error) => error_response(error),
    }
}

async fn preflight_binding(
    State(state): State<Arc<CollaborationApiState>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
    Json(request): Json<BindingCommandRequest>,
) -> Response {
    let owner = match owner_key(&user) {
        Ok(owner) => owner,
        Err(response) => return response,
    };
    let workspace = match workspace_header(&headers) {
        Ok(workspace) => workspace,
        Err(response) => return response,
    };
    match state
        .service
        .preflight_binding(&owner, &workspace, &request)
        .await
    {
        Ok(response) => Json(response).into_response(),
        Err(error) => error_response(error),
    }
}

async fn install_binding(
    State(state): State<Arc<CollaborationApiState>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
    Json(request): Json<BindingCommandRequest>,
) -> Response {
    let owner = match owner_key(&user) {
        Ok(owner) => owner,
        Err(response) => return response,
    };
    let workspace = match workspace_header(&headers) {
        Ok(workspace) => workspace,
        Err(response) => return response,
    };
    match state
        .service
        .install_binding(&owner, &workspace, request)
        .await
    {
        Ok(response) => (StatusCode::CREATED, Json(response)).into_response(),
        Err(error) => error_response(error),
    }
}

async fn get_binding(
    State(state): State<Arc<CollaborationApiState>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    let owner = match owner_key(&user) {
        Ok(owner) => owner,
        Err(response) => return response,
    };
    let workspace = match workspace_header(&headers) {
        Ok(workspace) => workspace,
        Err(response) => return response,
    };
    match state.service.get_binding(&owner, &workspace, &id).await {
        Ok(binding) => Json(binding).into_response(),
        Err(error) => error_response(error),
    }
}

async fn list_bindings(
    State(state): State<Arc<CollaborationApiState>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
) -> Response {
    let (owner, workspace) = match private_scope(&user, &headers) {
        Ok(scope) => scope,
        Err(response) => return response,
    };
    result_response(state.service.list_bindings(&owner, &workspace).await)
}

async fn get_effective_binding_receipt(
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
            .get_effective_binding_receipt(&owner, &workspace, &id)
            .await,
    )
}

async fn export_binding_template(
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
            .export_binding_template(&owner, &workspace, &id)
            .await,
    )
}

async fn install_representation_grant(
    State(state): State<Arc<CollaborationApiState>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
    Json(request): Json<GrantCommandRequest>,
) -> Response {
    let (owner, workspace) = match private_scope(&user, &headers) {
        Ok(scope) => scope,
        Err(response) => return response,
    };
    match state
        .service
        .install_representation_grant(&owner, &workspace, request)
        .await
    {
        Ok(response) => (StatusCode::CREATED, Json(response)).into_response(),
        Err(error) => error_response(error),
    }
}

async fn list_representation_grants(
    State(state): State<Arc<CollaborationApiState>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
) -> Response {
    let (owner, workspace) = match private_scope(&user, &headers) {
        Ok(scope) => scope,
        Err(response) => return response,
    };
    result_response(
        state
            .service
            .list_representation_grants(&owner, &workspace)
            .await,
    )
}

async fn get_representation_grant(
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
            .get_representation_grant(&owner, &workspace, &id)
            .await,
    )
}

async fn get_representation_grant_revision(
    State(state): State<Arc<CollaborationApiState>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
    Path((id, revision)): Path<(String, u64)>,
) -> Response {
    let (owner, workspace) = match private_scope(&user, &headers) {
        Ok(scope) => scope,
        Err(response) => return response,
    };
    result_response(
        state
            .service
            .get_representation_grant_revision(&owner, &workspace, &id, revision)
            .await,
    )
}

async fn list_representation_grant_history(
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
            .list_representation_grant_history(&owner, &workspace, &id)
            .await,
    )
}

fn private_scope(user: &UserContext, headers: &HeaderMap) -> Result<(String, String), Response> {
    Ok((owner_key(user)?, workspace_header(headers)?))
}

fn result_response<T: Serialize>(result: Result<T, CollaborationError>) -> Response {
    match result {
        Ok(value) => Json(value).into_response(),
        Err(error) => error_response(error),
    }
}

fn owner_key(user: &UserContext) -> Result<String, Response> {
    super::user_settings::principal_storage_key(user).ok_or_else(unauthorized)
}

fn workspace_header(headers: &HeaderMap) -> Result<String, Response> {
    headers
        .get("x-uar-workspace-id")
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.trim().is_empty())
        .map(str::to_owned)
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": {
                        "code": "workspace_required",
                        "messageKey": "collaboration.error.workspaceRequired"
                    }
                })),
            )
                .into_response()
        })
}

fn unauthorized() -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(json!({
            "error": {
                "code": "authenticated_owner_required",
                "messageKey": "collaboration.error.authenticatedOwnerRequired"
            }
        })),
    )
        .into_response()
}

fn error_response(error: CollaborationError) -> Response {
    let (status, code, message_key, detail) = match error {
        CollaborationError::Invalid(detail) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            "collaboration_invalid",
            "collaboration.error.invalid",
            Some(detail),
        ),
        CollaborationError::NotFound(detail) => (
            StatusCode::NOT_FOUND,
            "collaboration_not_found",
            "collaboration.error.notFound",
            Some(detail),
        ),
        CollaborationError::Conflict(detail) => (
            StatusCode::CONFLICT,
            "collaboration_revision_conflict",
            "collaboration.error.revisionConflict",
            Some(detail),
        ),
        CollaborationError::Storage(error) => {
            tracing::error!(%error, "collaboration catalog persistence failed");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                "collaboration_storage_unavailable",
                "collaboration.error.storageUnavailable",
                None,
            )
        }
    };
    (
        status,
        Json(json!({
            "error": {
                "code": code,
                "messageKey": message_key,
                "detail": detail
            }
        })),
    )
        .into_response()
}
