//! A2UI REST routes.
//!
//! Mounts at `/api/uar/a2ui` in the main router.
//!
//! | Method | Path | Description |
//! |--------|------|-------------|
//! | GET | `/api/uar/a2ui/schemas` | List all registered artifact schemas |
//! | GET | `/api/uar/a2ui/schemas/{schema_id}` | Get a single schema by ID |
//! | POST | `/api/uar/runs/{run_id}/artifact-response` | Submit user response to an input artifact |

use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};

use super::registry::A2uiRegistry;
use crate::uar::{domain::events::NormalizedEvent, runtime::manager::RunManager};

// ── State ─────────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct A2uiApiState {
    pub registry: Arc<A2uiRegistry>,
    pub run_manager: Arc<RunManager>,
}

// ── Request / response types ──────────────────────────────────────────────────

/// Response submitted by the user after completing an A2UI artifact form.
#[derive(Debug, Deserialize)]
pub struct ArtifactResponsePayload {
    /// The artifact ID that generated this input request (from the SSE event).
    pub artifact_id: String,
    /// The user's response data. Shape depends on `artifact_type`:
    /// - `form`: `{ "field_name": "value", ... }`
    /// - `confirm`: `{ "accepted": true | false }`
    /// - `select`: `{ "value": "selected_option_value" }`
    /// - `text_input`: `{ "text": "user input" }`
    pub response: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct ArtifactResponseAck {
    run_id: String,
    artifact_id: String,
    status: &'static str,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// `GET /api/uar/a2ui/schemas`
///
/// Returns all schemas registered in the A2UI registry (built-ins + user-defined).
async fn list_schemas(State(state): State<A2uiApiState>) -> impl IntoResponse {
    let schemas = state.registry.list().await;
    Json(schemas)
}

/// `GET /api/uar/a2ui/schemas/{schema_id}`
///
/// Returns a single schema by its `schema_id` (e.g. `a2ui/form`).
/// URL-encodes slashes: `a2ui%2Fform`.
async fn get_schema(
    State(state): State<A2uiApiState>,
    Path(schema_id): Path<String>,
) -> impl IntoResponse {
    match state.registry.get(&schema_id).await {
        Some(schema) => (StatusCode::OK, Json(schema)).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": format!("schema '{}' not found", schema_id) })),
        )
            .into_response(),
    }
}

/// `POST /api/uar/runs/{run_id}/artifact-response`
///
/// Receives the user's response to an `agui.artifact_input_request` event and
/// injects it back into the paused agent run as a tool result, allowing the
/// agent to continue execution.
async fn submit_artifact_response(
    State(state): State<A2uiApiState>,
    Path(run_id): Path<String>,
    Json(payload): Json<ArtifactResponsePayload>,
) -> impl IntoResponse {
    // Verify the run exists and is active.
    let run = match state.run_manager.get_run(&run_id).await {
        Some(r) => r,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": format!("run '{}' not found", run_id) })),
            )
                .into_response();
        }
    };

    // Emit the artifact response as a tool result event into the run's event stream.
    // The RunManager will route this to the agent's ongoing tool-call loop.
    let tool_result_event = NormalizedEvent::ToolEnd {
        run_id: run.run_id.clone(),
        call_index: 0,
        tool_call_id: payload.artifact_id.clone(),
        tool: "a2ui.collect_input".to_string(),
        output: payload.response,
        ok: true,
    };

    state.run_manager.emit_to_run(&run.run_id, tool_result_event).await;

    (
        StatusCode::OK,
        Json(ArtifactResponseAck {
            run_id: run.run_id,
            artifact_id: payload.artifact_id,
            status: "accepted",
        }),
    )
        .into_response()
}

// ── Router builders ───────────────────────────────────────────────────────────

/// Build the A2UI schema listing router (mounted at `/api/uar/a2ui`).
pub fn build_schema_router() -> Router<A2uiApiState> {
    Router::new()
        .route("/schemas", get(list_schemas))
        .route("/schemas/{schema_id}", get(get_schema))
}

/// Build the artifact-response router (mounted at `/api/uar/runs`).
///
/// This is separate from the schema router because it shares the path prefix
/// with the main runs API.
pub fn build_response_router() -> Router<A2uiApiState> {
    Router::new()
        .route("/{run_id}/artifact-response", post(submit_artifact_response))
}
