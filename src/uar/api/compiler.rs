//! Compiler REST API — spec management, compilation, reports, and sessions.
//!
//! Mounted at `/api/uar/compiler`.
//!
//! | Endpoint                         | Method | Description                              |
//! |----------------------------------|--------|------------------------------------------|
//! | `/specs`                         | POST   | Submit + store a UAR-AGENT-MD document   |
//! | `/specs`                         | GET    | List stored specifications               |
//! | `/specs/{id}`                    | GET    | Retrieve a stored spec                   |
//! | `/specs/{id}`                    | DELETE | Delete a stored spec                     |
//! | `/specs/{id}/compile`            | POST   | Compile a stored spec (single-shot)      |
//! | `/compile`                       | POST   | Compile inline (no storage)              |
//! | `/compile-and-register`          | POST   | Compile + register a runtime artifact    |
//! | `/reports/{id}`                  | GET    | Retrieve a compile report                |
//! | `/sessions`                      | POST   | Start a conversational session           |
//! | `/sessions`                      | GET    | List all sessions                        |
//! | `/sessions/{id}`                 | GET    | Get session state + completeness         |
//! | `/sessions/{id}/cancel`          | POST   | Cancel a session                         |
//! | `/sessions/{id}/compile`         | POST   | Force-compile session's partial IR       |

use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};

use crate::uar::compiler::completeness::CompletenessAnalyzer;
use crate::uar::compiler::service::CompilerService;
use crate::uar::compiler::session::CompilerSession;
use crate::uar::domain::artifact::AgentArtifact;
use crate::uar::persistence::PersistenceLayer;

// ─────────────────────────────────────────────────────────────────────────────
// State
// ─────────────────────────────────────────────────────────────────────────────

/// Shared state for the compiler API.
#[derive(Debug, Clone)]
pub struct CompilerApiState {
    pub compiler_service: Arc<CompilerService>,
    /// Persistence layer used by `compile-and-register` to store the resulting
    /// runtime artifact. `None` disables that endpoint (503).
    pub persistence: Option<Arc<dyn PersistenceLayer>>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Request / Response types
// ─────────────────────────────────────────────────────────────────────────────

/// Request body for `POST /specs`.
#[derive(Debug, Deserialize)]
pub struct SubmitSpecRequest {
    /// The raw UAR-AGENT-MD Markdown content.
    pub content: String,
}

/// Request body for `POST /compile` (inline, no storage).
#[derive(Debug, Deserialize)]
pub struct InlineCompileRequest {
    /// The raw UAR-AGENT-MD Markdown content.
    pub content: String,
}

/// Request body for `POST /compile-and-register`.
#[derive(Debug, Deserialize)]
pub struct CompileAndRegisterRequest {
    /// The raw UAR-AGENT-MD Markdown content.
    pub content: String,
}

/// Response body for `POST /compile-and-register`.
#[derive(Debug, Serialize)]
pub struct CompileAndRegisterResponse {
    /// The registered runtime artifact (also persisted via the persistence layer).
    pub artifact: AgentArtifact,
    /// The full compile report.
    pub report: crate::uar::compiler::report::CompileReport,
    /// Ed25519 signature of the compiled descriptor (hex-encoded).
    pub signature: String,
}

/// Session state response (includes completeness analysis).
#[derive(Debug, Serialize)]
pub struct SessionStateResponse {
    #[serde(flatten)]
    pub session: CompilerSession,
    /// Completeness analysis of the partial IR.
    pub completeness: CompletenessResponse,
}

/// Completeness summary for a session.
#[derive(Debug, Serialize)]
pub struct CompletenessResponse {
    pub filled_required: usize,
    pub total_required: usize,
    pub ready_to_compile: bool,
    pub missing_sections: Vec<String>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Router
// ─────────────────────────────────────────────────────────────────────────────

/// Build the compiler API router.
pub fn build_router() -> Router<Arc<CompilerApiState>> {
    Router::new()
        // Spec management
        .route("/specs", post(submit_spec).get(list_specs))
        .route("/specs/{id}", get(get_spec).delete(delete_spec))
        .route("/specs/{id}/compile", post(compile_spec))
        // Inline compilation (no storage)
        .route("/compile", post(compile_inline))
        // Compile + register the resulting runtime artifact in one call
        .route("/compile-and-register", post(compile_and_register))
        // Report retrieval
        .route("/reports/{id}", get(get_report))
        // Conversational sessions
        .route("/sessions", post(create_session).get(list_sessions))
        .route("/sessions/{id}", get(get_session))
        .route("/sessions/{id}/cancel", post(cancel_session))
        .route("/sessions/{id}/compile", post(compile_session))
}

// ─────────────────────────────────────────────────────────────────────────────
// Spec handlers
// ─────────────────────────────────────────────────────────────────────────────

/// `POST /specs` — store a new spec document.
async fn submit_spec(
    State(state): State<Arc<CompilerApiState>>,
    Json(req): Json<SubmitSpecRequest>,
) -> impl IntoResponse {
    match state.compiler_service.store_spec(req.content).await {
        Ok(spec) => (StatusCode::CREATED, Json(spec)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

/// `GET /specs` — list all stored specs.
async fn list_specs(State(state): State<Arc<CompilerApiState>>) -> impl IntoResponse {
    match state.compiler_service.list_specs().await {
        Ok(specs) => Json(specs).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

/// `GET /specs/{id}` — retrieve a stored spec.
async fn get_spec(
    State(state): State<Arc<CompilerApiState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.compiler_service.get_spec(&id).await {
        Ok(Some(spec)) => Json(spec).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "spec not found" })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

/// `DELETE /specs/{id}` — delete a stored spec.
async fn delete_spec(
    State(state): State<Arc<CompilerApiState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.compiler_service.delete_spec(&id).await {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "spec not found" })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

/// `POST /specs/{id}/compile` — compile a stored spec.
async fn compile_spec(
    State(state): State<Arc<CompilerApiState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.compiler_service.compile_spec(&id).await {
        Ok(output) => Json(output).into_response(),
        Err(e) => {
            let status = if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::UNPROCESSABLE_ENTITY
            };
            (status, Json(serde_json::json!({ "error": e.to_string() }))).into_response()
        }
    }
}

/// `POST /compile` — compile inline (no storage).
async fn compile_inline(
    State(state): State<Arc<CompilerApiState>>,
    Json(req): Json<InlineCompileRequest>,
) -> impl IntoResponse {
    match state.compiler_service.compile_content(&req.content).await {
        Ok(output) => Json(output).into_response(),
        Err(e) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

/// `POST /compile-and-register` — compile raw UAR-AGENT-MD, convert the compiled
/// descriptor's IR payload into a runtime [`AgentArtifact`], persist it via the
/// persistence layer (the same `save_agent` call `POST /api/agents` uses), and
/// return the registered artifact alongside the compile report + signature.
async fn compile_and_register(
    State(state): State<Arc<CompilerApiState>>,
    Json(req): Json<CompileAndRegisterRequest>,
) -> impl IntoResponse {
    let Some(persistence) = state.persistence.clone() else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "No persistence layer" })),
        )
            .into_response();
    };

    // Compile (parse → 8-stage pipeline). Reuses the same path as `/compile`.
    let output = match state.compiler_service.compile_content(&req.content).await {
        Ok(output) => output,
        Err(e) => {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response();
        }
    };

    // Convert the compiled descriptor's IR payload into a runtime artifact.
    let artifact = AgentArtifact::from(&output.descriptor.payload);

    // Persist the artifact (same call `create_agent` uses).
    if let Err(e) = persistence.save_agent(&artifact).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response();
    }

    (
        StatusCode::CREATED,
        Json(CompileAndRegisterResponse {
            artifact,
            report: output.report,
            signature: output.signature,
        }),
    )
        .into_response()
}

// ─────────────────────────────────────────────────────────────────────────────
// Report handlers
// ─────────────────────────────────────────────────────────────────────────────

/// `GET /reports/{id}` — retrieve a compile report.
async fn get_report(
    State(state): State<Arc<CompilerApiState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.compiler_service.get_report(&id).await {
        Ok(Some(report)) => Json(report).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "report not found" })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Session handlers
// ─────────────────────────────────────────────────────────────────────────────

/// `POST /sessions` — create a new conversational compiler session.
async fn create_session(State(state): State<Arc<CompilerApiState>>) -> impl IntoResponse {
    let session = state.compiler_service.create_session().await;
    (StatusCode::CREATED, Json(session)).into_response()
}

/// `GET /sessions` — list all sessions.
async fn list_sessions(State(state): State<Arc<CompilerApiState>>) -> impl IntoResponse {
    let sessions = state.compiler_service.list_sessions().await;
    Json(sessions).into_response()
}

/// `GET /sessions/{id}` — get session state with completeness analysis.
async fn get_session(
    State(state): State<Arc<CompilerApiState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.compiler_service.get_session(&id).await {
        Some(session) => {
            let completeness_state = CompletenessAnalyzer::analyze(&session.partial_ir);
            let response = SessionStateResponse {
                session,
                completeness: CompletenessResponse {
                    filled_required: completeness_state.present.len(),
                    total_required: completeness_state.present.len()
                        + completeness_state.missing.len(),
                    ready_to_compile: completeness_state.is_ready,
                    missing_sections: completeness_state.missing,
                },
            };
            Json(response).into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "session not found" })),
        )
            .into_response(),
    }
}

/// `POST /sessions/{id}/cancel` — cancel a session.
async fn cancel_session(
    State(state): State<Arc<CompilerApiState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    if state.compiler_service.cancel_session(&id).await {
        StatusCode::NO_CONTENT.into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "session not found" })),
        )
            .into_response()
    }
}

/// `POST /sessions/{id}/compile` — force-compile a session's partial IR.
async fn compile_session(
    State(state): State<Arc<CompilerApiState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.compiler_service.compile_session(&id).await {
        Ok(output) => Json(output).into_response(),
        Err(e) => {
            let status = if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else if e.to_string().contains("not ready") {
                StatusCode::CONFLICT
            } else {
                StatusCode::UNPROCESSABLE_ENTITY
            };
            (status, Json(serde_json::json!({ "error": e.to_string() }))).into_response()
        }
    }
}
