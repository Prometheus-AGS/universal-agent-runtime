//! Authenticated CRUD for reusable Presentation templates.
//!
//! Mount under `/api/uar/presentations` behind the host authentication layer.
//! Anonymous access is rejected even when local chat permits anonymous clients.

use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Extension, Path, Query, Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::Response,
    routing::get,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::uar::{
    a2ui::presentations::{Presentation, PresentationDraft},
    persistence::{PersistenceLayer, presentations::PresentationStoreError},
    security::claims::UserContext,
};

type ApiError = (StatusCode, Json<Value>);

#[cfg(test)]
#[path = "presentations_tests.rs"]
mod tests;

#[derive(Debug, Clone)]
struct CatalogOwner(String);

/// Build the catalog router with a required verified principal on every route.
pub fn build_router() -> Router<Arc<dyn PersistenceLayer>> {
    Router::new()
        .route("/", get(list).post(create))
        .route("/{id}", get(read).put(update).delete(delete))
        .route_layer(middleware::from_fn(require_owner))
}

async fn require_owner(mut request: Request, next: Next) -> Result<Response, StatusCode> {
    let user = request
        .extensions()
        .get::<UserContext>()
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let owner = crate::uar::runtime::actor::messages::ActorOwner::from_verified_context(user)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let key = owner.presentation_owner_key();
    request.extensions_mut().insert(CatalogOwner(key));
    Ok(next.run(request).await)
}

fn catalog_error(error: anyhow::Error) -> ApiError {
    let (status, message) = match error.downcast_ref::<PresentationStoreError>() {
        Some(PresentationStoreError::NotFound) => {
            (StatusCode::NOT_FOUND, "Presentation not found".to_string())
        }
        Some(PresentationStoreError::Conflict) => (
            StatusCode::CONFLICT,
            "This Presentation changed. Reload it before saving; your draft has not been applied."
                .to_string(),
        ),
        Some(PresentationStoreError::Invalid(message)) => {
            (StatusCode::UNPROCESSABLE_ENTITY, message.clone())
        }
        None => {
            tracing::error!(error = %error, "Presentation persistence failed");
            (StatusCode::INTERNAL_SERVER_ERROR, "Presentation storage is unavailable. Reload before retrying to check whether your change was saved.".to_string())
        }
    };
    (status, Json(json!({"error": message})))
}

#[derive(Debug, Serialize)]
struct CatalogResponse {
    owner_id: String,
    presentations: Vec<Presentation>,
}

async fn list(
    State(store): State<Arc<dyn PersistenceLayer>>,
    Extension(owner): Extension<CatalogOwner>,
) -> Result<Json<CatalogResponse>, ApiError> {
    let presentations = store
        .list_presentations(&owner.0)
        .await
        .map_err(catalog_error)?;
    Ok(Json(CatalogResponse {
        owner_id: owner.0,
        presentations,
    }))
}

async fn read(
    State(store): State<Arc<dyn PersistenceLayer>>,
    Extension(owner): Extension<CatalogOwner>,
    Path(id): Path<String>,
) -> Result<Json<Presentation>, ApiError> {
    store
        .get_presentation(&owner.0, &id)
        .await
        .map_err(catalog_error)?
        .map(Json)
        .ok_or_else(|| catalog_error(PresentationStoreError::NotFound.into()))
}

async fn create(
    State(store): State<Arc<dyn PersistenceLayer>>,
    Extension(owner): Extension<CatalogOwner>,
    Json(draft): Json<PresentationDraft>,
) -> Result<(StatusCode, Json<Presentation>), ApiError> {
    let record = store
        .create_presentation(&owner.0, &draft)
        .await
        .map_err(catalog_error)?;
    Ok((StatusCode::CREATED, Json(record)))
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct UpdateRequest {
    expected_revision: u64,
    content: PresentationDraft,
}

async fn update(
    State(store): State<Arc<dyn PersistenceLayer>>,
    Extension(owner): Extension<CatalogOwner>,
    Path(id): Path<String>,
    Json(request): Json<UpdateRequest>,
) -> Result<Json<Presentation>, ApiError> {
    store
        .update_presentation(&owner.0, &id, request.expected_revision, &request.content)
        .await
        .map(Json)
        .map_err(catalog_error)
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DeleteRequest {
    expected_revision: u64,
}

async fn delete(
    State(store): State<Arc<dyn PersistenceLayer>>,
    Extension(owner): Extension<CatalogOwner>,
    Path(id): Path<String>,
    Query(request): Query<DeleteRequest>,
) -> Result<StatusCode, ApiError> {
    store
        .delete_presentation(&owner.0, &id, request.expected_revision)
        .await
        .map_err(catalog_error)?;
    Ok(StatusCode::NO_CONTENT)
}
