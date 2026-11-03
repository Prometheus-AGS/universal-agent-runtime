//! Host-mediated media commands. Untrusted observations never enter these routes.

use super::{CollaborationApiState, private_scope};
use crate::uar::{
    compiler::collaboration::CollaborationError,
    domain::media_effect::{
        ConfirmMediaGrantRequest, DelegateMediaGrantRequest, DispatchMediaEffectRequest,
        MediaAuthorityScope, MediaCommandRequest, MediaOutcomeRequest, PrepareMediaEffectRequest,
    },
    security::claims::UserContext,
};
use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, FromRequest, FromRequestParts, Path, Request, State},
    http::{StatusCode, header, request::Parts},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::json;
use std::sync::Arc;

pub(super) fn build_router() -> Router<Arc<CollaborationApiState>> {
    Router::new()
        .route("/media-grants", post(confirm))
        .route("/media-grants/{id}/revoke", post(revoke))
        .route("/media-delegations", post(delegate))
        .route("/media-effects", post(prepare))
        .route("/media-effects/{id}", get(inspect))
        .route("/media-effects/{id}/dispatch", post(dispatch))
        .route("/media-effects/{id}/cancel", post(cancel))
        .route("/media-effects/{id}/outcome", post(outcome))
        .route("/media-effects/{id}/reconcile", post(reconcile))
        // Allow JSON escaping of the connector's 64 KiB transient content limit.
        .layer(DefaultBodyLimit::max(512 * 1024))
}

struct MediaHost(MediaAuthorityScope);

impl<S: Send + Sync> FromRequestParts<S> for MediaHost {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let Some(user) = parts.extensions.get::<UserContext>() else {
            return Err(error(StatusCode::UNAUTHORIZED, "MEDIA_HOST_REQUIRED"));
        };
        let Some(host_id) = user.trusted_host_id() else {
            return Err(error(StatusCode::FORBIDDEN, "MEDIA_HOST_REQUIRED"));
        };
        let (owner_id, workspace_id) = private_scope(user, &parts.headers)?;
        Ok(Self(MediaAuthorityScope {
            owner_id,
            workspace_id,
            tenant_id: user.tenant_id.as_ref().map(|tenant| tenant.as_str().to_owned()),
            host_id: host_id.to_owned(),
        }))
    }
}

struct MediaBody<T>(T);

impl<S: Send + Sync, T: DeserializeOwned> FromRequest<S> for MediaBody<T> {
    type Rejection = Response;

    async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
        Json::<T>::from_request(request, state)
            .await
            .map(|Json(body)| Self(body))
            .map_err(|rejection| error(rejection.status(), "MEDIA_REQUEST_INVALID"))
    }
}

fn error(status: StatusCode, code: &str) -> Response {
    (status, Json(json!({ "error": { "code": code } }))).into_response()
}

fn respond<T: Serialize>(result: Result<T, CollaborationError>) -> Response {
    let mut response = match result {
        Ok(value) => Json(value).into_response(),
        Err(CollaborationError::Invalid(_)) => {
            error(StatusCode::UNPROCESSABLE_ENTITY, "MEDIA_REQUEST_INVALID")
        }
        Err(CollaborationError::NotFound(_)) => {
            error(StatusCode::NOT_FOUND, "MEDIA_RESOURCE_NOT_FOUND")
        }
        Err(CollaborationError::Conflict(_)) => {
            error(StatusCode::CONFLICT, "MEDIA_COMMAND_REFUSED")
        }
        Err(CollaborationError::Storage(_)) => {
            error(StatusCode::SERVICE_UNAVAILABLE, "MEDIA_STORAGE_UNAVAILABLE")
        }
    };
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        axum::http::HeaderValue::from_static("no-store"),
    );
    response
}

async fn inspect(
    State(state): State<Arc<CollaborationApiState>>,
    MediaHost(scope): MediaHost,
    Path(id): Path<String>,
) -> Response {
    respond(state.service.get_media_effect(&scope, &id).await)
}

async fn confirm(
    State(state): State<Arc<CollaborationApiState>>,
    MediaHost(scope): MediaHost,
    MediaBody(request): MediaBody<ConfirmMediaGrantRequest>,
) -> Response {
    respond(state.service.confirm_media_grant(&scope, request).await)
}

async fn revoke(
    State(state): State<Arc<CollaborationApiState>>,
    MediaHost(scope): MediaHost,
    Path(id): Path<String>,
    MediaBody(request): MediaBody<MediaCommandRequest>,
) -> Response {
    respond(state.service.revoke_media_grant(&scope, &id, request).await)
}

async fn delegate(
    State(state): State<Arc<CollaborationApiState>>,
    MediaHost(scope): MediaHost,
    MediaBody(request): MediaBody<DelegateMediaGrantRequest>,
) -> Response {
    respond(state.service.delegate_media_grant(&scope, request).await)
}

async fn prepare(
    State(state): State<Arc<CollaborationApiState>>,
    MediaHost(scope): MediaHost,
    MediaBody(request): MediaBody<PrepareMediaEffectRequest>,
) -> Response {
    respond(state.service.prepare_media_effect(&scope, request).await)
}

async fn dispatch(
    State(state): State<Arc<CollaborationApiState>>,
    MediaHost(scope): MediaHost,
    Path(id): Path<String>,
    MediaBody(request): MediaBody<DispatchMediaEffectRequest>,
) -> Response {
    respond(state.service.claim_media_effect(&scope, &id, request).await)
}

async fn cancel(
    State(state): State<Arc<CollaborationApiState>>,
    MediaHost(scope): MediaHost,
    Path(id): Path<String>,
    MediaBody(request): MediaBody<MediaCommandRequest>,
) -> Response {
    respond(state.service.cancel_media_effect(&scope, &id, request).await)
}

async fn outcome(
    State(state): State<Arc<CollaborationApiState>>,
    MediaHost(scope): MediaHost,
    Path(id): Path<String>,
    MediaBody(request): MediaBody<MediaOutcomeRequest>,
) -> Response {
    respond(state.service.record_media_outcome(&scope, &id, request).await)
}

async fn reconcile(
    State(state): State<Arc<CollaborationApiState>>,
    MediaHost(scope): MediaHost,
    Path(id): Path<String>,
    MediaBody(request): MediaBody<MediaOutcomeRequest>,
) -> Response {
    respond(state.service.reconcile_media_effect(&scope, &id, request).await)
}
