//! Actor management API routes.
//!
//! RESTful endpoints for spawning, messaging, and managing agent actors.

use axum::{
    Json, Router,
    extract::{Extension, Path, Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::Response,
    routing::{delete, get, post},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::uar::runtime::actor::{
    messages::{ActorInfo, ActorOwner, AgentReply},
    system::ActorCollaboration,
};

/// Build the actors API router.
///
/// Mounted at `/api/uar/actors`.
pub fn build_router() -> Router<Arc<ActorCollaboration>> {
    Router::new()
        .route("/", get(list_actors).post(spawn_actor))
        .route("/{id}", delete(stop_actor))
        .route("/{id}/message", post(send_message))
        .route("/{id}/collaborate", post(collaborate))
        .route_layer(middleware::from_fn(require_actor_owner))
}

/// Require the authenticated host principal even when general local endpoints
/// permit anonymous access. Missing extensions return 401 before body parsing.
async fn require_actor_owner(mut request: Request, next: Next) -> Result<Response, StatusCode> {
    let user = request
        .extensions()
        .get::<crate::uar::security::claims::UserContext>()
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let owner = ActorOwner::from_verified_context(user).map_err(|_| StatusCode::UNAUTHORIZED)?;
    request.extensions_mut().insert(owner);
    Ok(next.run(request).await)
}

// =============================================================================
// HANDLERS
// =============================================================================

/// List all active actors.
async fn list_actors(
    State(system): State<Arc<ActorCollaboration>>,
    Extension(owner): Extension<ActorOwner>,
) -> Json<ActorsResponse> {
    let actors = system.list_actors(&owner).await;
    Json(ActorsResponse { actors })
}

/// Spawn a new agent actor.
async fn spawn_actor(
    State(system): State<Arc<ActorCollaboration>>,
    Extension(owner): Extension<ActorOwner>,
    Json(req): Json<SpawnActorRequest>,
) -> Result<(StatusCode, Json<SpawnActorResponse>), (StatusCode, Json<ErrorResponse>)> {
    let actor_name = req
        .name
        .unwrap_or_else(|| format!("actor-{}", uuid::Uuid::new_v4()));

    match system
        .spawn_agent(&owner, actor_name, req.agent_id.clone(), req.system_prompt)
        .await
    {
        Ok(id) => Ok((
            StatusCode::CREATED,
            Json(SpawnActorResponse {
                id,
                agent_id: req.agent_id,
            }),
        )),
        Err(e) => Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

/// Send a message to a named actor and wait for the reply.
async fn send_message(
    State(system): State<Arc<ActorCollaboration>>,
    Extension(owner): Extension<ActorOwner>,
    Path(id): Path<String>,
    Json(req): Json<MessageRequest>,
) -> Result<Json<AgentReply>, (StatusCode, Json<ErrorResponse>)> {
    system
        .send_prompt(&owner, &id, req.content)
        .await
        .map(Json)
        .map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
        })
}

/// Request collaboration between two actors.
async fn collaborate(
    State(system): State<Arc<ActorCollaboration>>,
    Extension(owner): Extension<ActorOwner>,
    Path(id): Path<String>,
    Json(req): Json<CollaborateRequest>,
) -> Result<Json<AgentReply>, (StatusCode, Json<ErrorResponse>)> {
    system
        .collaborate(&owner, &req.from_actor, &id, req.task)
        .await
        .map(Json)
        .map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
        })
}

/// Stop a named actor.
async fn stop_actor(
    State(system): State<Arc<ActorCollaboration>>,
    Extension(owner): Extension<ActorOwner>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    system.stop_actor(&owner, &id).await.map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;
    Ok(StatusCode::NO_CONTENT)
}

// =============================================================================
// REQUEST / RESPONSE TYPES
// =============================================================================

#[derive(Deserialize)]
struct SpawnActorRequest {
    /// Optional actor name (auto-generated if omitted).
    name: Option<String>,
    /// Agent artifact ID to run.
    agent_id: String,
    /// Optional system prompt.
    system_prompt: Option<String>,
}

#[derive(Serialize)]
struct SpawnActorResponse {
    id: String,
    agent_id: String,
}

#[derive(Deserialize)]
struct MessageRequest {
    content: String,
}

#[derive(Deserialize)]
struct CollaborateRequest {
    from_actor: String,
    task: String,
}

#[derive(Serialize)]
struct ActorsResponse {
    actors: Vec<ActorInfo>,
}

#[derive(Serialize, Deserialize)]
struct ErrorResponse {
    error: String,
}
