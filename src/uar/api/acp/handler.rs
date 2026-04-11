//! ACP JSON-RPC 2.0 method handlers.
//!
//! Each method corresponds to an ACP protocol operation. Sessions are stored
//! in an in-memory map guarded by a RwLock with TTL-based eviction.

use super::types::*;
use crate::AppState;
use chrono::Utc;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

// =============================================================================
// ACP Session Store
// =============================================================================

#[derive(Default, Debug)]
pub struct AcpSessionStore {
    sessions: RwLock<HashMap<String, AcpSession>>,
}

impl AcpSessionStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn create(&self, agent_id: impl Into<String>) -> AcpSession {
        let session = AcpSession::new(agent_id);
        let mut store = self.sessions.write().await;
        store.insert(session.session_id.clone(), session.clone());
        session
    }

    pub async fn get(&self, session_id: &str) -> Option<AcpSession> {
        let mut store = self.sessions.write().await;
        if let Some(s) = store.get_mut(session_id) {
            s.last_active = Utc::now();
            return Some(s.clone());
        }
        None
    }

    pub async fn delete(&self, session_id: &str) -> bool {
        let mut store = self.sessions.write().await;
        store.remove(session_id).is_some()
    }

    pub async fn evict_expired(&self, ttl_secs: u64) {
        let cutoff = Utc::now() - chrono::Duration::seconds(ttl_secs as i64);
        let mut store = self.sessions.write().await;
        store.retain(|_, s| s.last_active > cutoff);
    }
}

// =============================================================================
// Dispatch
// =============================================================================

/// Dispatch an ACP JSON-RPC request to the appropriate handler.
pub async fn dispatch(
    req: JsonRpcRequest,
    state: Arc<AppState>,
    sessions: Arc<AcpSessionStore>,
) -> JsonRpcResponse {
    match req.method.as_str() {
        "agents/list" => handle_agents_list(req.id, &state).await,
        "agents/get" => handle_agents_get(req.id, req.params, &state).await,
        "sessions/create" => handle_sessions_create(req.id, req.params, &state, &sessions).await,
        "sessions/get" => handle_sessions_get(req.id, req.params, &sessions).await,
        "sessions/delete" => handle_sessions_delete(req.id, req.params, &sessions).await,
        "runs/create" => handle_runs_create(req.id, req.params, &state, &sessions).await,
        "runs/get" => handle_runs_get(req.id, req.params, &state).await,
        _ => JsonRpcResponse::err(
            req.id,
            RPC_METHOD_NOT_FOUND,
            format!("Method '{}' not found", req.method),
        ),
    }
}

// =============================================================================
// agents/*
// =============================================================================

async fn handle_agents_list(id: Option<Value>, state: &AppState) -> JsonRpcResponse {
    // Return all registered agents from A2A registry if available, else default agent.
    let agents = vec![json!({
        "id": "default",
        "name": "Default Assistant",
        "description": "A helpful general-purpose AI assistant.",
        "version": "1.0.0",
        "capabilities": {
            "streaming": true,
            "tool_use": true,
            "memory": state.memory_service.is_some(),
            "multi_turn": true
        }
    })];
    JsonRpcResponse::ok(id, json!({ "agents": agents }))
}

async fn handle_agents_get(
    id: Option<Value>,
    params: Option<Value>,
    state: &AppState,
) -> JsonRpcResponse {
    let agent_id = params
        .as_ref()
        .and_then(|p| p.get("agent_id"))
        .and_then(Value::as_str)
        .unwrap_or("default")
        .to_string();

    if agent_id != "default" {
        // Look up in federated agent registry
        match state.federated_agent_registry.get_agent(&agent_id).await {
            Ok(Some(agent)) => {
                return JsonRpcResponse::ok(
                    id,
                    json!({
                        "id": agent.id,
                        "name": agent.name,
                        "description": agent.description,
                        "version": "1.0.0",
                        "capabilities": {
                            "streaming": true,
                            "tool_use": true,
                            "memory": false,
                            "multi_turn": true
                        }
                    }),
                );
            }
            Ok(None) => {
                return JsonRpcResponse::err(
                    id,
                    ACP_AGENT_NOT_FOUND,
                    format!("Agent '{agent_id}' not found"),
                );
            }
            Err(e) => {
                return JsonRpcResponse::err(id, RPC_INTERNAL_ERROR, e.to_string());
            }
        }
    }

    JsonRpcResponse::ok(
        id,
        json!({
            "id": "default",
            "name": "Default Assistant",
            "description": "A helpful general-purpose AI assistant.",
            "version": "1.0.0",
            "capabilities": { "streaming": true, "tool_use": true, "memory": state.memory_service.is_some(), "multi_turn": true }
        }),
    )
}

// =============================================================================
// sessions/*
// =============================================================================

async fn handle_sessions_create(
    id: Option<Value>,
    params: Option<Value>,
    _state: &AppState,
    sessions: &AcpSessionStore,
) -> JsonRpcResponse {
    let agent_id = params
        .as_ref()
        .and_then(|p| p.get("agent_id"))
        .and_then(Value::as_str)
        .unwrap_or("default")
        .to_string();
    let session = sessions.create(agent_id).await;
    JsonRpcResponse::ok(id, serde_json::to_value(&session).unwrap_or(json!({})))
}

async fn handle_sessions_get(
    id: Option<Value>,
    params: Option<Value>,
    sessions: &AcpSessionStore,
) -> JsonRpcResponse {
    let session_id = match params
        .as_ref()
        .and_then(|p| p.get("session_id"))
        .and_then(Value::as_str)
    {
        Some(s) => s.to_string(),
        None => return JsonRpcResponse::err(id, RPC_INVALID_PARAMS, "Missing session_id"),
    };
    match sessions.get(&session_id).await {
        Some(s) => JsonRpcResponse::ok(id, serde_json::to_value(&s).unwrap_or(json!({}))),
        None => JsonRpcResponse::err(
            id,
            ACP_SESSION_NOT_FOUND,
            format!("Session '{session_id}' not found"),
        ),
    }
}

async fn handle_sessions_delete(
    id: Option<Value>,
    params: Option<Value>,
    sessions: &AcpSessionStore,
) -> JsonRpcResponse {
    let session_id = match params
        .as_ref()
        .and_then(|p| p.get("session_id"))
        .and_then(Value::as_str)
    {
        Some(s) => s.to_string(),
        None => return JsonRpcResponse::err(id, RPC_INVALID_PARAMS, "Missing session_id"),
    };
    let deleted = sessions.delete(&session_id).await;
    JsonRpcResponse::ok(id, json!({ "deleted": deleted, "session_id": session_id }))
}

// =============================================================================
// runs/*
// =============================================================================

async fn handle_runs_create(
    id: Option<Value>,
    params: Option<Value>,
    state: &AppState,
    sessions: &AcpSessionStore,
) -> JsonRpcResponse {
    let params = match params {
        Some(p) => p,
        None => return JsonRpcResponse::err(id, RPC_INVALID_PARAMS, "Missing params"),
    };
    let session_id = match params.get("session_id").and_then(Value::as_str) {
        Some(s) => s.to_string(),
        None => return JsonRpcResponse::err(id, RPC_INVALID_PARAMS, "Missing session_id"),
    };
    let input = match params.get("input").and_then(Value::as_str) {
        Some(s) => s.to_string(),
        None => return JsonRpcResponse::err(id, RPC_INVALID_PARAMS, "Missing input"),
    };

    // Verify session exists
    if sessions.get(&session_id).await.is_none() {
        return JsonRpcResponse::err(
            id,
            ACP_SESSION_NOT_FOUND,
            format!("Session '{session_id}' not found"),
        );
    }

    // Delegate to RunManager using the default agent
    let artifact = crate::uar::defaults::default_agent();
    let run_id = state
        .run_manager
        .start_run(artifact, input, Some(session_id), None, vec![])
        .await;

    JsonRpcResponse::ok(
        id,
        json!({
            "run_id": run_id,
            "status": "running"
        }),
    )
}

async fn handle_runs_get(
    id: Option<Value>,
    params: Option<Value>,
    state: &AppState,
) -> JsonRpcResponse {
    let run_id = match params
        .as_ref()
        .and_then(|p| p.get("run_id"))
        .and_then(Value::as_str)
    {
        Some(r) => r.to_string(),
        None => return JsonRpcResponse::err(id, RPC_INVALID_PARAMS, "Missing run_id"),
    };
    // Check if run exists in run manager
    let run = state.run_manager.get_run(&run_id).await;
    match run {
        Some(r) => JsonRpcResponse::ok(
            id,
            json!({
                "run_id": run_id,
                "status": format!("{:?}", r.status),
                "agent_id": r.agent_id,
                "conversation_id": r.conversation_id,
            }),
        ),
        None => JsonRpcResponse::err(id, ACP_RUN_NOT_FOUND, format!("Run '{run_id}' not found")),
    }
}
