//! ACP JSON-RPC 2.0 method handlers.
//!
//! Each method corresponds to an ACP protocol operation. Sessions are stored
//! in an in-memory map guarded by a RwLock with TTL-based eviction.

use crate::uar::security::credential_capture::AuthenticatedCredentialCapture;
use super::types::*;
use crate::AppState;
use crate::uar::security::claims::UserContext;
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

    pub async fn create(&self, owner_id: &str, agent_id: impl Into<String>) -> AcpSession {
        self.create_with_tools(owner_id, agent_id, None).await
    }

    pub async fn create_with_tools(
        &self,
        owner_id: &str,
        agent_id: impl Into<String>,
        client_tools: Option<AcpClientTools>,
    ) -> AcpSession {
        let mut session = AcpSession::new(agent_id);
        session.client_tools = client_tools;
        let mut store = self.sessions.write().await;
        store.insert(
            crate::uar::persistence::tenant_storage_key(owner_id, &session.session_id),
            session.clone(),
        );
        session
    }

    pub async fn get(&self, owner_id: &str, session_id: &str) -> Option<AcpSession> {
        let mut store = self.sessions.write().await;
        let key = crate::uar::persistence::tenant_storage_key(owner_id, session_id);
        if let Some(s) = store.get_mut(&key) {
            s.last_active = Utc::now();
            return Some(s.clone());
        }
        None
    }

    pub async fn delete(&self, owner_id: &str, session_id: &str) -> bool {
        let mut store = self.sessions.write().await;
        let key = crate::uar::persistence::tenant_storage_key(owner_id, session_id);
        store.remove(&key).is_some()
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
    user: &UserContext,
) -> JsonRpcResponse {
    dispatch_with_capture(req, state, sessions, user, None).await
}

pub(crate) async fn dispatch_with_capture(
    req: JsonRpcRequest,
    state: Arc<AppState>,
    sessions: Arc<AcpSessionStore>,
    user: &UserContext,
    credential_capture: Option<AuthenticatedCredentialCapture>,
) -> JsonRpcResponse {
    let owner_key = match acp_owner_key(user) {
        Ok(owner) => owner,
        Err(_) => return JsonRpcResponse::err(req.id, RPC_INVALID_PARAMS, "Invalid session principal"),
    };
    let owner_id = owner_key.as_str();
    match req.method.as_str() {
        "agents/list" => handle_agents_list(req.id, &state).await,
        "agents/get" => handle_agents_get(req.id, req.params, &state).await,
        "sessions/create" => {
            handle_sessions_create(req.id, req.params, &state, &sessions, owner_id).await
        }
        "sessions/get" => handle_sessions_get(req.id, req.params, &sessions, owner_id).await,
        "sessions/delete" => handle_sessions_delete(req.id, req.params, &sessions, owner_id).await,
        "runs/submit_tool_results" => {
            let valid = req.params.as_ref()
                .and_then(|params| params.get("client_tools"))
                .and_then(|tools| tools.get("history"))
                .and_then(Value::as_array)
                .and_then(|history| history.last())
                .is_some_and(|message| message.get("role").and_then(Value::as_str) == Some("tool"));
            if !valid {
                return JsonRpcResponse::err(req.id, RPC_INVALID_PARAMS, "Complete client_tools.history with tool results is required");
            }
            handle_runs_create(req.id, req.params, &state, &sessions, user, credential_capture).await
        }
        "runs/create" => handle_runs_create(req.id, req.params, &state, &sessions, user, credential_capture).await,
        "runs/get" => handle_runs_get(req.id, req.params, &state, user).await,
        _ => JsonRpcResponse::err(
            req.id,
            RPC_METHOD_NOT_FOUND,
            format!("Method '{}' not found", req.method),
        ),
    }
}

fn acp_owner_key(user: &UserContext) -> anyhow::Result<String> {
    user.session_owner_key()
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

fn parse_client_tools(params: Option<&Value>) -> anyhow::Result<Option<AcpClientTools>> {
    let Some(value) = params.and_then(|params| params.get("client_tools")) else {
        return Ok(None);
    };
    let tools: AcpClientTools = serde_json::from_value(value.clone())?;
    tools.clone().into_input().validate()?;
    Ok(Some(tools))
}

async fn handle_sessions_create(
    id: Option<Value>,
    params: Option<Value>,
    _state: &AppState,
    sessions: &AcpSessionStore,
    owner_id: &str,
) -> JsonRpcResponse {
    let agent_id = params
        .as_ref()
        .and_then(|p| p.get("agent_id"))
        .and_then(Value::as_str)
        .unwrap_or("default")
        .to_string();
    let client_tools = match parse_client_tools(params.as_ref()) {
        Ok(tools) => tools,
        Err(error) => return JsonRpcResponse::err(id, RPC_INVALID_PARAMS, error.to_string()),
    };
    if client_tools.as_ref().is_some_and(|tools| !tools.history.is_empty()) {
        return JsonRpcResponse::err(id, RPC_INVALID_PARAMS, "Session declarations cannot contain continuation history");
    }
    let session = sessions.create_with_tools(owner_id, agent_id, client_tools).await;
    JsonRpcResponse::ok(id, serde_json::to_value(&session).unwrap_or(json!({})))
}

async fn handle_sessions_get(
    id: Option<Value>,
    params: Option<Value>,
    sessions: &AcpSessionStore,
    owner_id: &str,
) -> JsonRpcResponse {
    let session_id = match params
        .as_ref()
        .and_then(|p| p.get("session_id"))
        .and_then(Value::as_str)
    {
        Some(s) => s.to_string(),
        None => return JsonRpcResponse::err(id, RPC_INVALID_PARAMS, "Missing session_id"),
    };
    match sessions.get(owner_id, &session_id).await {
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
    owner_id: &str,
) -> JsonRpcResponse {
    let session_id = match params
        .as_ref()
        .and_then(|p| p.get("session_id"))
        .and_then(Value::as_str)
    {
        Some(s) => s.to_string(),
        None => return JsonRpcResponse::err(id, RPC_INVALID_PARAMS, "Missing session_id"),
    };
    let deleted = sessions.delete(owner_id, &session_id).await;
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
    user: &UserContext,
    credential_capture: Option<AuthenticatedCredentialCapture>,
) -> JsonRpcResponse {
    let owner_id = match acp_owner_key(user) {
        Ok(owner) => owner,
        Err(_) => return JsonRpcResponse::err(id, RPC_INVALID_PARAMS, "Invalid session principal"),
    };
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
        None if params.get("client_tools").and_then(|tools| tools.get("history")).and_then(Value::as_array).is_some_and(|history| !history.is_empty()) => String::new(),
        None => return JsonRpcResponse::err(id, RPC_INVALID_PARAMS, "Missing input"),
    };
    let presentation_negotiation = match serde_json::from_value::<
        crate::uar::a2ui::presentation_selection::PresentationNegotiation,
    >(params.clone())
    {
        Ok(negotiation) => negotiation,
        Err(_) => {
            return JsonRpcResponse::err(
                id,
                RPC_INVALID_PARAMS,
                "Invalid Presentation negotiation",
            );
        }
    };

    // Verify session exists
    let session = match sessions.get(&owner_id, &session_id).await {
        Some(session) => session,
        None => {
            // A durable pending batch can restore its exact owner-bound session
            // after restart; a caller-selected session ID alone grants nothing.
            let supplied_tools = match parse_client_tools(Some(&params)) {
                Ok(Some(tools)) => tools,
                _ => return JsonRpcResponse::err(id, ACP_SESSION_NOT_FOUND, format!("Session '{session_id}' not found")),
            };
            let call_id = supplied_tools.history.last()
                .filter(|message| message.get("role").and_then(Value::as_str) == Some("tool"))
                .and_then(|message| message.get("tool_call_id")).and_then(Value::as_str);
            let (Some(call_id), Some(store)) = (call_id, state.persistence.as_ref()) else {
                return JsonRpcResponse::err(id, ACP_SESSION_NOT_FOUND, format!("Session '{session_id}' not found"));
            };
            match crate::uar::api::client_tools::load_by_call_id(
                store.as_ref(), &owner_id, user.tenant_id.as_ref().map(|tenant| tenant.as_str()),
                &crate::uar::defaults::default_agent().id, call_id,
            ).await {
                Ok(Some(row)) if row.protocol == "acp" && row.session_id.as_deref() == Some(session_id.as_str()) => {
                    let mut restored = AcpSession::new("default");
                    restored.session_id = session_id.clone();
                    restored.client_tools = Some(supplied_tools);
                    sessions.sessions.write().await.insert(
                        crate::uar::persistence::tenant_storage_key(&owner_id, &session_id), restored.clone(),
                    );
                    restored
                }
                _ => return JsonRpcResponse::err(id, ACP_SESSION_NOT_FOUND, format!("Session '{session_id}' not found")),
            }
        }
    };
    let client_tools = match parse_client_tools(Some(&params)) {
        Ok(tools) => tools.or(session.client_tools),
        Err(error) => return JsonRpcResponse::err(id, RPC_INVALID_PARAMS, error.to_string()),
    };

    // Delegate to RunManager using the default agent
    let artifact = crate::uar::defaults::default_agent();
    let mut request = match crate::uar::runtime::turn::RunExecutionRequest::new(artifact, input)
        .with_user_context(user)
    {
        Ok(request) => request,
        Err(_) => return JsonRpcResponse::err(id, RPC_INVALID_PARAMS, "Invalid run principal"),
    };
    request = request.with_credential_capture(credential_capture);
    let mut continuation_claim = None;
    let has_client_tools = client_tools.is_some();
    if let Some(tools) = client_tools {
        let Some(store) = state.persistence.as_ref() else {
            return JsonRpcResponse::err(id, RPC_INVALID_PARAMS, "Durable caller-tool persistence is unavailable");
        };
        if !store.supports_durable_client_tools() {
            return JsonRpcResponse::err(id, RPC_INVALID_PARAMS, "Durable caller-tool persistence is unavailable");
        }
        let input = tools.into_input();
        if let Some(call_id) = input.history.last()
            .filter(|message| message.get("role").and_then(Value::as_str) == Some("tool"))
            .and_then(|message| message.get("tool_call_id")).and_then(Value::as_str)
        {
            match crate::uar::api::client_tools::load_by_call_id(
                store.as_ref(), &owner_id, user.tenant_id.as_ref().map(|tenant| tenant.as_str()),
                &request.artifact.id, call_id,
            ).await {
                Ok(Some(row)) if row.session_id.as_deref() == Some(session_id.as_str()) => {}
                Ok(_) => return JsonRpcResponse::err(id, RPC_INVALID_PARAMS, "Caller-tool continuation does not belong to this session"),
                Err(error) => return JsonRpcResponse::err(id, RPC_INVALID_PARAMS, error.to_string()),
            }
        }
        match crate::uar::api::client_tools::prepare(
            state.run_manager.as_ref(), store.as_ref(), &owner_id, user.tenant_id.as_ref().map(|tenant| tenant.as_str()),
            &request.artifact.id, &input,
        ).await {
            Ok(crate::uar::api::client_tools::ClientToolPreparation::Fresh(input)) => request.client_tools = Some(input),
            Ok(crate::uar::api::client_tools::ClientToolPreparation::Continue { input, claim }) => {
                request.client_tools = Some(input);
                continuation_claim = Some(claim);
            }
            Ok(crate::uar::api::client_tools::ClientToolPreparation::Replay { response, .. }) => return JsonRpcResponse::ok(id, response),
            Err(error) => return JsonRpcResponse::err(id, RPC_INVALID_PARAMS, error.to_string()),
        }
    }
    request.session_id = continuation_claim.as_ref().map_or(Some(session_id), |claim| claim.session_id.clone());
    request.presentation_negotiation = presentation_negotiation;
    let agent_id = request.artifact.id.clone();
    let run_id = state.run_manager.execute_request(request).await;
    if has_client_tools {
        let events = match collect_client_tool_run(state, &run_id).await {
            Ok(events) => events,
            Err(error) => return JsonRpcResponse::err(id, RPC_INTERNAL_ERROR, error.to_string()),
        };
        match state.run_manager.await_settled_run_status(&run_id).await {
            Ok(crate::uar::domain::runs::RunStatus::Done) => {}
            Ok(status) => return JsonRpcResponse::err(id, RPC_INTERNAL_ERROR, format!("Caller-tool run did not settle successfully: {status:?}")),
            Err(error) => return JsonRpcResponse::err(id, RPC_INTERNAL_ERROR, error.to_string()),
        }
        let pending_batch = events.iter().find_map(|event| {
            (event.get("type").and_then(Value::as_str) == Some("ClientToolsRequested"))
                .then(|| event.pointer("/data/batch_id").and_then(Value::as_str)).flatten()
                .map(str::to_owned)
        });
        if pending_batch.is_some() {
            let Some(store) = state.persistence.as_ref() else {
                return JsonRpcResponse::err(id, RPC_INTERNAL_ERROR, "Caller-tool persistence disappeared");
            };
            if let Err(error) = crate::uar::api::client_tools::confirm_handoff_for_run(
                state.run_manager.as_ref(), store.as_ref(), &owner_id,
                user.tenant_id.as_ref().map(|tenant| tenant.as_str()), &agent_id, &run_id,
            ).await {
                return JsonRpcResponse::err(id, RPC_INTERNAL_ERROR, error.to_string());
            }
        }
        let output = events.iter().filter(|event| event.get("type").and_then(Value::as_str) == Some("ChatDelta"))
            .filter_map(|event| event.pointer("/data/text_delta").and_then(Value::as_str)).collect::<String>();
        let status = if events.iter().any(|event| matches!(event.get("type").and_then(Value::as_str), Some("Error" | "Cancelled"))) { "error" } else { "done" };
        let mut result = json!({"run_id":run_id,"status":status,"output":output,"events":events});
        if let Some(batch_id) = pending_batch {
            let Some(store) = state.persistence.as_ref() else {
                return JsonRpcResponse::err(id, RPC_INTERNAL_ERROR, "Caller-tool persistence disappeared");
            };
            match store.load_client_tool_continuation(&owner_id, user.tenant_id.as_ref().map(|tenant| tenant.as_str()), &agent_id, &batch_id).await {
                Ok(Some(row)) => {
                    result["status"] = json!("client_tools_required");
                    let mut tools = row.client_schemas;
                    tools["history"] = row.visible_transcript;
                    result["client_tools"] = tools;
                    result["batch_id"] = json!(batch_id);
                }
                Ok(None) => return JsonRpcResponse::err(id, RPC_INTERNAL_ERROR, "Persisted caller-tool batch is unavailable"),
                Err(error) => return JsonRpcResponse::err(id, RPC_INTERNAL_ERROR, error.to_string()),
            }
        }
        if let Some(claim) = continuation_claim {
            let Some(store) = state.persistence.as_ref() else {
                return JsonRpcResponse::err(id, RPC_INTERNAL_ERROR, "Caller-tool persistence disappeared");
            };
            let persisted_events = result["events"].as_array().cloned().unwrap_or_default();
            if let Err(error) = crate::uar::api::client_tools::complete(store.as_ref(), &claim, result.clone(), persisted_events).await {
                return JsonRpcResponse::err(id, RPC_INTERNAL_ERROR, error.to_string());
            }
        }
        return JsonRpcResponse::ok(id, result);
    }

    JsonRpcResponse::ok(
        id,
        json!({
            "run_id": run_id,
            "status": "running"
        }),
    )
}

/// Subscribe before taking history so fast completions and live events share one cursor.
async fn collect_client_tool_run(state: &AppState, run_id: &str) -> anyhow::Result<Vec<Value>> {
    use crate::uar::domain::events::NormalizedEvent;
    let mut receiver = state.run_manager.subscribe(run_id).await
        .ok_or_else(|| anyhow::anyhow!("Run event stream is unavailable"))?;
    let mut history = state.run_manager.history_since(run_id, None).await
        .ok_or_else(|| anyhow::anyhow!("Run event history is unavailable"))?;
    let mut cursor = history.last().map(|event| event.id);
    loop {
        if history.last().is_some_and(|event| matches!(&event.event,
            NormalizedEvent::RunDone { .. } | NormalizedEvent::RunDoneWithUsage { .. }
            | NormalizedEvent::Error { .. } | NormalizedEvent::Cancelled { .. })) {
            return history.into_iter().map(|event| serde_json::to_value(event.event).map_err(Into::into)).collect();
        }
        match receiver.recv().await {
            Ok(event) if cursor.is_none_or(|cursor| event.id > cursor) => {
                cursor = Some(event.id);
                history.push(event);
            }
            Ok(_) => {}
            Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                let recovered = state.run_manager.history_since(run_id, cursor).await
                    .ok_or_else(|| anyhow::anyhow!("Caller-tool event history was lost"))?;
                anyhow::ensure!(!recovered.is_empty(), "Caller-tool event history was lost");
                cursor = recovered.last().map(|event| event.id);
                history.extend(recovered);
            }
            Err(error) => return Err(error.into()),
        }
    }
}

async fn handle_runs_get(
    id: Option<Value>,
    params: Option<Value>,
    state: &AppState,
    user: &UserContext,
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
    let run = state.run_manager.get_run_for_context(user, &run_id).await;
    match run {
        Some(r) => {
            let events = state.run_manager.history_since(&run_id, None).await.unwrap_or_default();
            let pending = events.iter().rev().find_map(|event| match &event.event {
                crate::uar::domain::events::NormalizedEvent::ClientToolsRequested { batch_id, calls, .. } => Some(json!({"batch_id":batch_id,"calls":calls})),
                _ => None,
            });
            let mut durable_client_tools = Value::Null;
            if let Some(store) = state.persistence.as_ref().filter(|store| store.supports_durable_client_tools()) {
                let owner = match acp_owner_key(user) {
                    Ok(owner) => owner,
                    Err(error) => return JsonRpcResponse::err(id, RPC_INVALID_PARAMS, error.to_string()),
                };
                match store.load_client_tool_continuation_for_run(
                    &owner, user.tenant_id.as_ref().map(|tenant| tenant.as_str()), &r.agent_id, &run_id,
                ).await {
                    Ok(Some(row)) if row.protocol == "acp" => {
                        if let Err(error) = crate::uar::api::client_tools::confirm_handoff_for_run(
                            state.run_manager.as_ref(), store.as_ref(), &owner,
                            user.tenant_id.as_ref().map(|tenant| tenant.as_str()), &r.agent_id, &run_id,
                        ).await {
                            return JsonRpcResponse::err(id, RPC_INTERNAL_ERROR, error.to_string());
                        }
                        let mut tools = row.client_schemas;
                        tools["history"] = row.visible_transcript;
                        durable_client_tools = json!({"batch_id":row.batch_id,"state":row.state,"client_tools":tools});
                    }
                    Ok(_) => {}
                    Err(error) => return JsonRpcResponse::err(id, RPC_INTERNAL_ERROR, error.to_string()),
                }
            }
            JsonRpcResponse::ok(
            id,
            json!({
                "run_id": run_id,
                "status": format!("{:?}", r.status),
                "agent_id": r.agent_id,
                "conversation_id": r.conversation_id,
                "pending_client_tools": pending,
                "client_tool_continuation": durable_client_tools,
            }),
        )
        },
        None => JsonRpcResponse::err(id, ACP_RUN_NOT_FOUND, format!("Run '{run_id}' not found")),
    }
}

#[cfg(test)]
mod tests {
    use super::AcpSessionStore;

    #[tokio::test]
    async fn session_store_denies_cross_owner_get_and_delete() {
        let store = AcpSessionStore::new();
        let session = store.create("alice", "default").await;

        assert!(store.get("alice", &session.session_id).await.is_some());
        assert!(store.get("bob", &session.session_id).await.is_none());
        assert!(!store.delete("bob", &session.session_id).await);
        assert!(store.get("alice", &session.session_id).await.is_some());
        assert!(store.delete("alice", &session.session_id).await);
    }
}
