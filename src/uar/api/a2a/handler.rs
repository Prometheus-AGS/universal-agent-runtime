//! A2A JSON-RPC dispatcher.
//!
//! Handles the following A2A RC v1.0 methods:
//! - `message/send` — send a message to the compiler agent (creates or continues a task)
//! - `tasks/get` — retrieve task state
//! - `tasks/cancel` — cancel a task
//!
//! The handler maps A2A tasks 1:1 to [`CompilerSession`]s:
//! - `contextId` → `session_id`
//! - `TaskState::Working` → `SessionStatus::Gathering`
//! - `TaskState::Completed` → `SessionStatus::Completed`
//! - `TaskState::Canceled` → `SessionStatus::Cancelled`

use std::sync::Arc;

use axum::{Extension, Json, extract::State, response::IntoResponse};
use tracing::{info, warn};

use super::{
    agent_card::build_agent_card,
    task_store::TaskStore,
    types::{
        JsonRpcRequest, JsonRpcResponse, Message, MessageSendParams, Part, Task, TaskCancelParams,
        TaskGetParams, TaskState, rpc_error,
    },
};
use crate::{
    config::SecurityConfig,
    uar::{
        compiler::service::CompilerService,
        security::claims::{TenantId, UserContext},
    },
};

// ─────────────────────────────────────────────────────────────────────────────
// State
// ─────────────────────────────────────────────────────────────────────────────

/// Shared state for the A2A endpoint.
#[derive(Debug, Clone)]
pub struct A2AState {
    pub compiler_service: Arc<CompilerService>,
    pub task_store: Arc<TaskStore>,
    pub security: SecurityConfig,
    /// Public base URL of this UAR instance (e.g. `http://localhost:3928`).
    pub base_url: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// Handlers
// ─────────────────────────────────────────────────────────────────────────────

/// `POST /a2a/compiler` — JSON-RPC 2.0 dispatcher.
pub async fn handle_rpc(
    State(state): State<Arc<A2AState>>,
    user_context: Option<Extension<UserContext>>,
    Json(req): Json<JsonRpcRequest>,
) -> impl IntoResponse {
    if req.jsonrpc != "2.0" {
        return Json(JsonRpcResponse::err(
            req.id,
            rpc_error::INVALID_REQUEST,
            "jsonrpc must be \"2.0\"",
        ));
    }

    let tenant_id = user_context.and_then(|Extension(context)| context.tenant_id);
    if state.security.jwt_required && tenant_id.is_none() {
        return Json(JsonRpcResponse::err(
            req.id,
            rpc_error::INVALID_REQUEST,
            "verified tenant claim required",
        ));
    }

    let response = match req.method.as_str() {
        "message/send" => {
            handle_message_send(&state, tenant_id.as_ref(), req.id.clone(), req.params).await
        }
        "tasks/get" => {
            handle_tasks_get(&state, tenant_id.as_ref(), req.id.clone(), req.params).await
        }
        "tasks/cancel" => {
            handle_tasks_cancel(&state, tenant_id.as_ref(), req.id.clone(), req.params).await
        }
        other => {
            warn!(method = other, "unknown A2A method");
            JsonRpcResponse::err(
                req.id,
                rpc_error::METHOD_NOT_FOUND,
                format!("method '{other}' not found"),
            )
        }
    };

    Json(response)
}

/// `GET /.well-known/agent.json` — serve the AgentCard.
pub async fn handle_agent_card(State(state): State<Arc<A2AState>>) -> impl IntoResponse {
    Json(build_agent_card(&state.base_url))
}

// ─────────────────────────────────────────────────────────────────────────────
// Method implementations
// ─────────────────────────────────────────────────────────────────────────────

/// Handle `message/send`.
///
/// If `context_id` is provided and maps to an existing task, the message is
/// appended to that task. Otherwise a new session + task is created.
async fn handle_message_send(
    state: &A2AState,
    tenant_id: Option<&TenantId>,
    id: Option<serde_json::Value>,
    params: Option<serde_json::Value>,
) -> JsonRpcResponse {
    let params: MessageSendParams = match parse_params(params) {
        Ok(p) => p,
        Err(e) => return JsonRpcResponse::err(id, rpc_error::INVALID_PARAMS, e),
    };

    // Extract text from the message
    let user_text = extract_text(&params.message);

    // Check if this is a continuation of an existing task
    if let Some(ref ctx_id) = params.context_id {
        if let Some(existing_task) = state.task_store.get_by_context(tenant_id, ctx_id).await {
            return continue_task(
                state,
                tenant_id,
                id,
                existing_task,
                params.message,
                user_text,
            )
            .await;
        }
    }

    // New task — create a session
    let session = state.compiler_service.create_session().await;
    let session_id = session.id.clone();

    info!(session_id = %session_id, "A2A: created new compiler session");

    // Create the A2A task
    let task = state
        .task_store
        .create(tenant_id, params.context_id, &session_id, params.message)
        .await;

    // Generate an initial response from the agent
    let agent_reply = Message::agent_text(
        "Welcome! I'm the UAR Compiler Agent. I can help you compile a UAR-AGENT-MD \
         specification into a signed agent descriptor.\n\n\
         You can:\n\
         • **Paste a complete spec** — I'll compile it immediately.\n\
         • **Describe what you want** — I'll guide you through building one step by step.\n\n\
         What would you like to do?",
    );

    state
        .task_store
        .append_message(tenant_id, &task.id, agent_reply.clone())
        .await;

    // Return the task with the agent's reply in status
    let mut result_task = state
        .task_store
        .get(tenant_id, &task.id)
        .await
        .unwrap_or(task);
    result_task.status.message = Some(agent_reply);

    JsonRpcResponse::ok(id, result_task)
}

/// Continue an existing task with a new message.
async fn continue_task(
    state: &A2AState,
    tenant_id: Option<&TenantId>,
    id: Option<serde_json::Value>,
    task: Task,
    message: Message,
    user_text: String,
) -> JsonRpcResponse {
    let task_id = task.id.clone();

    // Append user message
    state
        .task_store
        .append_message(tenant_id, &task_id, message)
        .await;

    // Try to detect if the user pasted a complete spec (heuristic: starts with "# Agent:")
    let trimmed = user_text.trim();
    if trimmed.starts_with("# Agent:") || trimmed.contains("## Metadata") {
        // Attempt single-shot compilation
        match state.compiler_service.compile_content(trimmed).await {
            Ok(output) => {
                let descriptor = serde_json::to_value(&output).unwrap_or(serde_json::Value::Null);
                state
                    .task_store
                    .complete_with_descriptor(tenant_id, &task_id, descriptor)
                    .await;

                let result_task = state
                    .task_store
                    .get(tenant_id, &task_id)
                    .await
                    .unwrap_or(task);
                return JsonRpcResponse::ok(id, result_task);
            }
            Err(e) => {
                warn!(error = %e, "A2A: inline compilation failed, continuing conversationally");
            }
        }
    }

    // Conversational response — acknowledge and ask for more
    let agent_reply = Message::agent_text(format!(
        "Got it. I've noted: \"{}\"\n\n\
         To compile your agent, I need a complete UAR-AGENT-MD specification covering all 15 sections. \
         You can paste the full spec at any time, or continue describing your agent and I'll help fill in the details.",
        user_text.chars().take(120).collect::<String>()
    ));

    state
        .task_store
        .append_message(tenant_id, &task_id, agent_reply.clone())
        .await;

    let mut result_task = state
        .task_store
        .get(tenant_id, &task_id)
        .await
        .unwrap_or(task);
    result_task.status.state = TaskState::InputRequired;
    result_task.status.message = Some(agent_reply);

    JsonRpcResponse::ok(id, result_task)
}

/// Handle `tasks/get`.
async fn handle_tasks_get(
    state: &A2AState,
    tenant_id: Option<&TenantId>,
    id: Option<serde_json::Value>,
    params: Option<serde_json::Value>,
) -> JsonRpcResponse {
    let params: TaskGetParams = match parse_params(params) {
        Ok(p) => p,
        Err(e) => return JsonRpcResponse::err(id, rpc_error::INVALID_PARAMS, e),
    };

    match state.task_store.get(tenant_id, &params.id).await {
        Some(task) => JsonRpcResponse::ok(id, task),
        None => JsonRpcResponse::err(id, rpc_error::TASK_NOT_FOUND, "task not found"),
    }
}

/// Handle `tasks/cancel`.
async fn handle_tasks_cancel(
    state: &A2AState,
    tenant_id: Option<&TenantId>,
    id: Option<serde_json::Value>,
    params: Option<serde_json::Value>,
) -> JsonRpcResponse {
    let params: TaskCancelParams = match parse_params(params) {
        Ok(p) => p,
        Err(e) => return JsonRpcResponse::err(id, rpc_error::INVALID_PARAMS, e),
    };

    // Cancel the A2A task
    let cancelled = state.task_store.cancel(tenant_id, &params.id).await;
    if !cancelled {
        return JsonRpcResponse::err(
            id,
            rpc_error::TASK_NOT_CANCELABLE,
            "task not found or already in a terminal state",
        );
    }

    // Also cancel the underlying compiler session (best-effort)
    if let Some(task) = state.task_store.get(tenant_id, &params.id).await {
        if let Some(ctx_id) = task.context_id {
            state.compiler_service.cancel_session(&ctx_id).await;
        }
    }

    match state.task_store.get(tenant_id, &params.id).await {
        Some(task) => JsonRpcResponse::ok(id, task),
        None => JsonRpcResponse::err(id, rpc_error::TASK_NOT_FOUND, "task not found"),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

fn parse_params<T: serde::de::DeserializeOwned>(
    params: Option<serde_json::Value>,
) -> Result<T, String> {
    let value = params.unwrap_or(serde_json::Value::Null);
    serde_json::from_value(value).map_err(|e| e.to_string())
}

fn extract_text(message: &Message) -> String {
    message
        .parts
        .iter()
        .filter_map(|p| match p {
            Part::Text { text } => Some(text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use axum::{
        Router,
        body::{Body, to_bytes},
        http::{Request, StatusCode},
        routing::post,
    };
    use tower::ServiceExt;

    use super::*;
    use crate::uar::security::claims::UserClaims;

    fn security(jwt_required: bool) -> SecurityConfig {
        SecurityConfig {
            jwt_required,
            jwt_secret: "tenant-test-secret".to_owned().into(),
            jwks_url: None,
            jwt_issuer: None,
            jwt_audience: None,
            jwt_validate_nbf: true,
            settings_mutation_auth_required: true,
        }
    }

    fn context(tenant: &str) -> UserContext {
        UserContext {
            user_id: "verified-user".to_owned(),
            tenant_id: Some(TenantId::for_test(tenant)),
            claims: UserClaims {
                sub: "verified-user".to_owned(),
                name: None,
                roles: None,
                tenant_id: Some(tenant.to_owned()),
                exp: usize::MAX,
            },
        }
    }

    fn state(task_store: Arc<TaskStore>, jwt_required: bool) -> Arc<A2AState> {
        Arc::new(A2AState {
            compiler_service: Arc::new(CompilerService::in_memory()),
            task_store,
            security: security(jwt_required),
            base_url: "http://127.0.0.1:3928".to_owned(),
        })
    }

    #[tokio::test]
    async fn body_query_and_header_tenant_values_cannot_override_verified_tenant() {
        let store = TaskStore::new();
        let tenant_a = TenantId::for_test("tenant-a");
        let tenant_b = TenantId::for_test("tenant-b");
        let task_a = store
            .create(
                Some(&tenant_a),
                Some("shared-context".to_owned()),
                "session-a",
                Message::user_text("tenant A task"),
            )
            .await;
        let task_b = store
            .create(
                Some(&tenant_b),
                Some("shared-context".to_owned()),
                "session-b",
                Message::user_text("tenant B task"),
            )
            .await;

        let app = Router::new()
            .route("/", post(handle_rpc))
            .with_state(state(Arc::clone(&store), true))
            .layer(Extension(context("tenant-a")));
        let request = Request::builder()
            .method("POST")
            .uri("/?tenant_id=tenant-b")
            .header("content-type", "application/json")
            .header("x-uar-tenant-id", "tenant-b")
            .body(Body::from(
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": "tenant-spoof",
                    "method": "message/send",
                    "tenant_id": "tenant-b",
                    "params": {
                        "message": {
                            "role": "user",
                            "parts": [{"type": "text", "text": "continue verified tenant"}],
                            "metadata": {"tenant_id": "tenant-b"}
                        },
                        "context_id": "shared-context",
                        "metadata": {"tenant_id": "tenant-b"},
                        "tenant_id": "tenant-b"
                    }
                })
                .to_string(),
            ))
            .expect("tenant spoof request must build");

        let response = app
            .oneshot(request)
            .await
            .expect("tenant spoof request must complete");
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("tenant spoof response body must read");
        let response: serde_json::Value =
            serde_json::from_slice(&body).expect("tenant spoof response must be JSON");

        assert_eq!(response["result"]["id"], task_a.id);
        assert_ne!(response["result"]["id"], task_b.id);
        assert_eq!(
            store
                .get(Some(&tenant_b), &task_b.id)
                .await
                .expect("tenant B task must remain present")
                .history
                .len(),
            1
        );
    }

    #[tokio::test]
    async fn required_jwt_without_verified_tenant_is_rejected() {
        let app = Router::new()
            .route("/", post(handle_rpc))
            .with_state(state(TaskStore::new(), true));
        let request = Request::builder()
            .method("POST")
            .uri("/")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": "missing-tenant",
                    "method": "tasks/get",
                    "params": {"id": "unknown"}
                })
                .to_string(),
            ))
            .expect("missing tenant request must build");

        let response = app
            .oneshot(request)
            .await
            .expect("missing tenant request must complete");
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("missing tenant response body must read");
        let response: serde_json::Value =
            serde_json::from_slice(&body).expect("missing tenant response must be JSON");

        assert_eq!(response["error"]["code"], rpc_error::INVALID_REQUEST);
        assert_eq!(
            response["error"]["message"],
            "verified tenant claim required"
        );
    }
}
