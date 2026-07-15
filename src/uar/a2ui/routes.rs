//! A2UI REST routes.
//!
//! Mounts at `/api/uar/a2ui` in the main router.
//!
//! | Method | Path | Description |
//! |--------|------|-------------|
//! | GET | `/api/uar/a2ui/schemas` | List all registered artifact schemas |
//! | GET | `/api/uar/a2ui/schemas/{schema_id}` | Get a single schema by ID |
//! | POST | `/api/uar/runs/{run_id}/artifact-response` | Submit user response to an input artifact |
//! | POST | `/api/uar/runs/{run_id}/a2ui/test-trigger` | Trigger a real artifact input request for testing |
//! | POST | `/api/uar/runs/{run_id}/a2ui/surface-test-trigger` | Trigger a real A2UI surface state-patch event for testing (Change 20) |
//! | GET | `/api/uar/runs/{run_id}/a2ui/surface-replay` | Replay every surface state-patch published for this run so far (Change 20 late-join reattach) |

use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};

use super::realtime::{
    A2uiReplayBackbone, A2uiWireKind, InMemoryReplayBackbone, surface_message_to_state_patch,
};
use super::registry::A2uiRegistry;
use crate::uar::{
    domain::events::{ArtifactPayload, NormalizedEvent},
    runtime::manager::RunManager,
};

// ── State ─────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct A2uiApiState {
    pub registry: Arc<A2uiRegistry>,
    pub run_manager: Arc<RunManager>,
    /// Durable-replay backbone for A2UI surface state patches (Change 20,
    /// `a2ui-realtime-backbone-from-flint-realtime-fabric`). In-memory for
    /// now — see `realtime.rs` module docs for the deferred
    /// `flint-realtime-fabric`-backed implementation.
    pub realtime_backbone: Arc<InMemoryReplayBackbone>,
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

/// Request body for `POST /api/uar/runs/{run_id}/a2ui/test-trigger`.
///
/// Mirrors [`ArtifactPayload`] minus `artifact_id` (generated server-side)
/// and `language` (not needed for input-request artifact types).
#[derive(Debug, Deserialize)]
pub struct TestTriggerPayload {
    pub artifact_type: String,
    pub title: String,
    pub content: String,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

/// Request body for `POST /api/uar/runs/{run_id}/a2ui/surface-test-trigger`
/// (Change 20). `kind` is one of `"createSurface" | "updateComponents" |
/// "updateDataModel" | "deleteSurface"`.
#[derive(Debug, Deserialize)]
pub struct SurfaceTestTriggerPayload {
    pub surface_id: String,
    pub kind: String,
    #[serde(default)]
    pub payload: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct SurfaceTestTriggerAck {
    run_id: String,
    surface_id: String,
    status: &'static str,
}

#[derive(Debug, Serialize)]
struct TestTriggerAck {
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

    state
        .run_manager
        .emit_to_run(&run.run_id, tool_result_event)
        .await;

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

/// `POST /api/uar/runs/{run_id}/a2ui/test-trigger`
///
/// Emits a real `ArtifactInputRequest` event onto the given run's SSE stream,
/// using the exact same [`RunManager::emit_to_run`] path a live agent tool
/// call uses — for testing/validating the A2UI round-trip on demand rather
/// than waiting for an agent to naturally request input.
async fn test_trigger_artifact(
    State(state): State<A2uiApiState>,
    Path(run_id): Path<String>,
    Json(payload): Json<TestTriggerPayload>,
) -> impl IntoResponse {
    let run = match state.run_manager.get_run(&run_id).await {
        Some(r) => r,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": format!("run '{}' not found or not active", run_id) })),
            )
                .into_response();
        }
    };

    let artifact_id = uuid::Uuid::new_v4().to_string();
    let event = NormalizedEvent::ArtifactInputRequest {
        run_id: run.run_id.clone(),
        artifact: ArtifactPayload {
            artifact_id: artifact_id.clone(),
            artifact_type: payload.artifact_type,
            title: payload.title,
            content: payload.content,
            language: None,
            metadata: payload.metadata,
        },
    };

    state.run_manager.emit_to_run(&run.run_id, event).await;

    (
        StatusCode::OK,
        Json(TestTriggerAck {
            run_id: run.run_id,
            artifact_id,
            status: "triggered",
        }),
    )
        .into_response()
}

/// `POST /api/uar/runs/{run_id}/a2ui/surface-test-trigger` (Change 20).
///
/// Converts the given A2UI surface message into a
/// `NormalizedEvent::StatePatch` (via [`surface_message_to_state_patch`]),
/// publishes it to the durable replay backbone (so a subsequent
/// `GET .../surface-replay` call — or, once wired, a
/// `flint-realtime-fabric` late-joining subscriber — sees it), and emits it
/// onto the run's live SSE broadcast via the same [`RunManager::emit_to_run`]
/// path every other run event uses, so every currently-connected client
/// converges on it immediately.
async fn surface_test_trigger(
    State(state): State<A2uiApiState>,
    Path(run_id): Path<String>,
    Json(payload): Json<SurfaceTestTriggerPayload>,
) -> impl IntoResponse {
    let run = match state.run_manager.get_run(&run_id).await {
        Some(r) => r,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": format!("run '{}' not found or not active", run_id) })),
            )
                .into_response();
        }
    };

    let kind = match payload.kind.as_str() {
        "createSurface" => A2uiWireKind::CreateSurface,
        "updateComponents" => A2uiWireKind::UpdateComponents,
        "updateDataModel" => A2uiWireKind::UpdateDataModel,
        "deleteSurface" => A2uiWireKind::DeleteSurface,
        other => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": format!(
                        "unknown kind '{other}' -- expected one of createSurface, updateComponents, updateDataModel, deleteSurface"
                    )
                })),
            )
                .into_response();
        }
    };

    let op = surface_message_to_state_patch(&payload.surface_id, kind, payload.payload);

    state.realtime_backbone.publish(&run.run_id, op.clone());

    state
        .run_manager
        .emit_to_run(
            &run.run_id,
            NormalizedEvent::StatePatch {
                run_id: run.run_id.clone(),
                patch: vec![op],
            },
        )
        .await;

    (
        StatusCode::OK,
        Json(SurfaceTestTriggerAck {
            run_id: run.run_id,
            surface_id: payload.surface_id,
            status: "triggered",
        }),
    )
        .into_response()
}

/// `GET /api/uar/runs/{run_id}/a2ui/surface-replay` (Change 20).
///
/// Returns every A2UI surface state-patch op published for this run so far,
/// in publish order — the "late-join reattach" read path: a client that
/// connects to the run's SSE stream after some surface updates already
/// happened calls this once to catch up, then relies on the live SSE
/// broadcast for anything published from that point on.
async fn surface_replay(
    State(state): State<A2uiApiState>,
    Path(run_id): Path<String>,
) -> impl IntoResponse {
    let ops = state.realtime_backbone.replay(&run_id);
    (StatusCode::OK, Json(ops)).into_response()
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
        .route(
            "/{run_id}/artifact-response",
            post(submit_artifact_response),
        )
        .route("/{run_id}/a2ui/test-trigger", post(test_trigger_artifact))
        .route(
            "/{run_id}/a2ui/surface-test-trigger",
            post(surface_test_trigger),
        )
        .route("/{run_id}/a2ui/surface-replay", get(surface_replay))
}
