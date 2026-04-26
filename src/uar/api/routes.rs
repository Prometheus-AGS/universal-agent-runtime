use crate::uar::{
    api::sse::build_sse_response,
    domain::artifact::AgentArtifact,
    runtime::{checkpoint::Checkpoint, manager::RunManager},
};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::BroadcastStream;

pub fn build_router() -> Router<Arc<RunManager>> {
    Router::new()
        .route("/runs", post(create_run))
        .route("/runs/{id}/stream", get(stream_run))
        .route("/runs/{run_id}/tool-approval", post(api_tool_approval))
        .route("/runs/{run_id}/checkpoints", get(list_checkpoints))
        .route("/runs/{run_id}/resume", post(resume_run))
        .route(
            "/runs/{run_id}/resume/{checkpoint_id}",
            post(resume_run_from_checkpoint),
        )
        .route("/resolve-model", get(resolve_model))
}

#[derive(Deserialize)]
struct CreateRunRequest {
    artifact: AgentArtifact,
    input: String,
    session_id: Option<String>,
}

#[derive(serde::Serialize)]
struct CreateRunResponse {
    run_id: String,
    stream_url: String,
}

#[derive(Deserialize)]
struct StreamParams {
    last_event_id: Option<u64>,
}

async fn create_run(
    State(manager): State<Arc<RunManager>>,
    Json(req): Json<CreateRunRequest>,
) -> Json<CreateRunResponse> {
    let run_id = manager
        .start_run(req.artifact, req.input, req.session_id, None, vec![])
        .await;
    Json(CreateRunResponse {
        run_id: run_id.clone(),
        stream_url: format!("/api/uar/runs/{run_id}/stream"),
    })
}

async fn stream_run(
    State(manager): State<Arc<RunManager>>,
    Path(run_id): Path<String>,
    Query(params): Query<StreamParams>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let Some(rx) = manager.subscribe(&run_id).await else {
        return axum::http::StatusCode::NOT_FOUND.into_response();
    };

    let last_event_id = params.last_event_id.or_else(|| {
        headers
            .get("last-event-id")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse::<u64>().ok())
    });

    let replay = manager
        .history_since(&run_id, last_event_id)
        .await
        .unwrap_or_default();
    let replay_max_id = replay.last().map_or(0, |event| event.id);

    // Convert Broadcast Receiver to Stream
    let live_stream = BroadcastStream::new(rx)
        .filter_map(Result::ok)
        .filter(move |event| event.id > replay_max_id);

    let stream = tokio_stream::iter(replay).chain(live_stream);

    build_sse_response(stream).into_response()
}

#[derive(Deserialize)]
struct ToolApprovalRequest {
    approved: bool,
}

/// POST /api/uar/runs/{run_id}/tool-approval
///
/// Submit an approval or rejection decision for a pending tool call.
/// Returns 200 OK if the decision was delivered, 404 if no pending approval exists.
async fn api_tool_approval(
    State(manager): State<Arc<RunManager>>,
    Path(run_id): Path<String>,
    Json(body): Json<ToolApprovalRequest>,
) -> impl IntoResponse {
    if manager.resolve_approval(&run_id, body.approved).await {
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}

// ── Checkpoint endpoints ──────────────────────────────────────────────────────

#[derive(Serialize)]
struct CheckpointListResponse {
    run_id: String,
    checkpoints: Vec<Checkpoint>,
}

/// GET /api/uar/runs/{run_id}/checkpoints
///
/// List all persisted checkpoints for a run, ordered by creation time.
/// Returns 503 if no persistence layer is configured.
async fn list_checkpoints(
    State(manager): State<Arc<RunManager>>,
    Path(run_id): Path<String>,
) -> impl IntoResponse {
    let Some(db) = &manager.persistence else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "persistence not configured"})),
        )
            .into_response();
    };

    match db.list_checkpoints(&run_id).await {
        Ok(checkpoints) => Json(CheckpointListResponse {
            run_id,
            checkpoints,
        })
        .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
struct ResumeRequest {
    artifact: AgentArtifact,
    /// Optional new input message; defaults to restoring the last checkpoint state.
    input: Option<String>,
    session_id: Option<String>,
}

/// POST /api/uar/runs/{run_id}/resume
///
/// Resume a run from its latest checkpoint (if any), or start fresh.
async fn resume_run(
    State(manager): State<Arc<RunManager>>,
    Path(run_id): Path<String>,
    Json(req): Json<ResumeRequest>,
) -> impl IntoResponse {
    let input = req.input.unwrap_or_else(|| {
        // No explicit input — use a standard resume message.
        format!("Resuming run {run_id}")
    });

    let new_run_id = manager
        .start_run(req.artifact, input, req.session_id, None, vec![])
        .await;

    Json(serde_json::json!({
        "resumed_from_run_id": run_id,
        "run_id": new_run_id,
        "stream_url": format!("/api/uar/runs/{new_run_id}/stream"),
    }))
    .into_response()
}

/// POST /api/uar/runs/{run_id}/resume/{checkpoint_id}
///
/// Resume a run from a specific named checkpoint.
/// The checkpoint's saved state is injected as context into the new run.
async fn resume_run_from_checkpoint(
    State(manager): State<Arc<RunManager>>,
    Path((run_id, checkpoint_id)): Path<(String, String)>,
    Json(req): Json<ResumeRequest>,
) -> impl IntoResponse {
    let Some(db) = &manager.persistence else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "persistence not configured"})),
        )
            .into_response();
    };

    let checkpoint = match db.load_checkpoint(&checkpoint_id).await {
        Ok(Some(cp)) => cp,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "checkpoint not found"})),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response();
        }
    };

    let input = req.input.unwrap_or_else(|| {
        format!(
            "Resuming run {run_id} from checkpoint {} (node: {})",
            checkpoint.id, checkpoint.node_id
        )
    });

    let new_run_id = manager
        .start_run(req.artifact, input, req.session_id, None, vec![])
        .await;

    Json(serde_json::json!({
        "resumed_from_run_id": run_id,
        "checkpoint_id": checkpoint_id,
        "checkpoint_node_id": checkpoint.node_id,
        "checkpoint_iteration": checkpoint.iteration,
        "run_id": new_run_id,
        "stream_url": format!("/api/uar/runs/{new_run_id}/stream"),
    }))
    .into_response()
}

/// GET /api/uar/resolve-model
///
/// Returns the resolved default model configuration (provider + model) or an error
/// if no model is available. Used by the frontend to guard chat before starting a run.
async fn resolve_model(State(manager): State<Arc<RunManager>>) -> impl IntoResponse {
    let model = manager.resolve_default_model().await;
    match model {
        Some((provider_id, model_id)) => Json(serde_json::json!({
            "ok": true,
            "provider_id": provider_id,
            "model_id": model_id,
        }))
        .into_response(),
        None => (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({
                "ok": false,
                "error": "No model configured. Add a provider and set a default model in Settings → Providers.",
            })),
        )
            .into_response(),
    }
}
