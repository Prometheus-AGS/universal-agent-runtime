//! ACP HTTP route definitions.
//!
//! Mounts the ACP JSON-RPC endpoint at the path configured via `acp.path`
//! (default: `/acp`). All requests are POST to the root path; SSE streaming
//! is available at `<path>/stream` for clients that support it.

use super::handler::{AcpSessionStore, dispatch};
use super::types::{JsonRpcRequest, JsonRpcResponse, RPC_PARSE_ERROR};
use crate::AppState;
use axum::{
    Json, Router,
    http::{HeaderMap, StatusCode},
    response::{
        IntoResponse, Response,
        sse::{Event, Sse},
    },
    routing::post,
};
use futures::stream;
use serde_json::{Value, json};
use std::convert::Infallible;
use std::sync::Arc;

// =============================================================================
// Router builder
// =============================================================================

#[derive(Debug)]
pub struct AcpRouter {
    pub sessions: Arc<AcpSessionStore>,
}

impl AcpRouter {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(AcpSessionStore::new()),
        }
    }

    /// Build Axum router. Call `.nest()` on the parent router with the configured path.
    pub fn into_router(self, app_state: Arc<AppState>) -> Router {
        let sessions = self.sessions;
        Router::new()
            .route(
                "/",
                post({
                    let sess = Arc::clone(&sessions);
                    let state = Arc::clone(&app_state);
                    move |headers: HeaderMap, Json(req): Json<serde_json::Value>| {
                        let sess = Arc::clone(&sess);
                        let state = Arc::clone(&state);
                        async move { handle_rpc(req, state, sess, headers).await }
                    }
                }),
            )
            .route(
                "/stream",
                post({
                    let sess = Arc::clone(&sessions);
                    let state = Arc::clone(&app_state);
                    move |headers: HeaderMap, Json(req): Json<serde_json::Value>| {
                        let sess = Arc::clone(&sess);
                        let state = Arc::clone(&state);
                        async move { handle_rpc_stream(req, state, sess, headers).await }
                    }
                }),
            )
    }
}

// =============================================================================
// Handlers
// =============================================================================

async fn handle_rpc(
    req_val: Value,
    state: Arc<AppState>,
    sessions: Arc<AcpSessionStore>,
    _headers: HeaderMap,
) -> Response {
    let req: JsonRpcRequest = match serde_json::from_value(req_val) {
        Ok(r) => r,
        Err(e) => {
            let resp = JsonRpcResponse::err(None, RPC_PARSE_ERROR, format!("Parse error: {}", e));
            return (StatusCode::OK, Json(resp)).into_response();
        }
    };
    let resp = dispatch(req, state, sessions).await;
    (StatusCode::OK, Json(resp)).into_response()
}

/// SSE streaming endpoint — sends ACP run events as Server-Sent Events.
async fn handle_rpc_stream(
    req_val: Value,
    state: Arc<AppState>,
    sessions: Arc<AcpSessionStore>,
    headers: HeaderMap,
) -> Response {
    // For non-runs/create methods, fall back to regular JSON response.
    let method = req_val.get("method").and_then(Value::as_str).unwrap_or("");
    if method != "runs/create" {
        return handle_rpc(req_val, state, sessions, headers).await;
    }

    let req: JsonRpcRequest = match serde_json::from_value(req_val) {
        Ok(r) => r,
        Err(e) => {
            let resp = JsonRpcResponse::err(None, RPC_PARSE_ERROR, format!("Parse error: {}", e));
            return (StatusCode::OK, Json(resp)).into_response();
        }
    };

    // Start the run and subscribe to its event stream.
    let resp = dispatch(req, Arc::clone(&state), Arc::clone(&sessions)).await;
    if resp.error.is_some() {
        return (StatusCode::OK, Json(resp)).into_response();
    }

    let run_id = resp
        .result
        .as_ref()
        .and_then(|r| r.get("run_id"))
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap_or_default();

    let receiver = state.run_manager.subscribe(&run_id).await;
    let sse_stream = match receiver {
        Some(rx) => {
            let stream = stream::unfold(rx, |mut rx| async move {
                match rx.recv().await {
                    Ok(evt) => {
                        let data = serde_json::to_string(&evt.event).unwrap_or_default();
                        let sse = Event::default().data(data);
                        Some((Ok::<Event, Infallible>(sse), rx))
                    }
                    Err(_) => None, // Channel closed — run is done
                }
            });
            Sse::new(stream).into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Run not found"})),
        )
            .into_response(),
    };
    sse_stream
}
