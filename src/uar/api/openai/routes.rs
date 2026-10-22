use crate::uar::security::credential_capture::AuthenticatedCredentialCapture;
use super::types::{
    ChatCompletionRequest, ModelCard, ModelList,
};
use crate::AppState;
use crate::uar::domain::events::NormalizedEvent;
use crate::uar::security::claims::UserContext;
use axum::{
    extract::{Extension, Json, State},
    http::StatusCode,
    response::{
        IntoResponse,
        sse::{Event, Sse},
    },
};
use serde::Serialize;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

fn unix_now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs()
}

pub async fn list_models(State(state): State<AppState>) -> impl IntoResponse {
    let now = unix_now_secs();
    let agents = match state.run_manager.list_registered_agents().await {
        Ok(agents) => agents,
        Err(error) => return agent_resolution_response(error),
    };
    let models = agents
        .into_iter()
        .map(|agent| ModelCard {
            id: agent.id,
            object: "model".to_string(),
            created: now,
            owned_by: "uar".to_string(),
        })
        .collect();

    Json(ModelList {
        object: "list".to_string(),
        data: models,
    })
    .into_response()
}

fn agent_resolution_response(
    error: crate::uar::domain::agent_store::AgentStoreError,
) -> axum::response::Response {
    match error {
        crate::uar::domain::agent_store::AgentStoreError::NotFound(id) => (
            StatusCode::NOT_FOUND,
            format!("Agent '{id}' is not registered"),
        )
            .into_response(),
        crate::uar::domain::agent_store::AgentStoreError::Invalid(message) => {
            (StatusCode::UNPROCESSABLE_ENTITY, message).into_response()
        }
        other => {
            tracing::error!(%other, "Agent catalog resolution failed");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                "Agent catalog is unavailable",
            )
                .into_response()
        }
    }
}

fn json_event<T: Serialize>(value: T) -> Event {
    match Event::default().json_data(value) {
        Ok(event) => event,
        Err(err) => {
            tracing::error!("failed to serialize SSE event: {}", err);
            Event::default().data(r#"{"error":"internal serialization error"}"#)
        }
    }
}

pub async fn chat_completions(
    State(state): State<AppState>,
    Extension(user_context): axum::Extension<UserContext>,
    credential_capture: Option<Extension<AuthenticatedCredentialCapture>>,
    Json(req): Json<ChatCompletionRequest>,
) -> impl IntoResponse {
    let run_manager = &state.run_manager;
    if req.messages.is_empty() {
        return openai_error(StatusCode::BAD_REQUEST, "invalid_request_error", "invalid_messages", "messages must not be empty");
    }

    let agent_id = match req.model.as_str() {
        "default" => "default-agent",
        "orchestrator" => "orchestrator-agent",
        id => id,
    };
    let agent = match run_manager.resolve_registered_agent(agent_id).await {
        Ok(agent) => agent,
        Err(error) => return agent_resolution_response(error),
    };

    let run_request = match crate::uar::runtime::turn::RunExecutionRequest::new(agent, String::new())
        .with_user_context(&user_context)
    {
        Ok(request) => request.with_credential_capture(credential_capture.map(|Extension(capture)| capture)),
        Err(_) => return openai_error(StatusCode::UNAUTHORIZED, "invalid_request_error", "invalid_principal", "Invalid run principal"),
    };
    execute_prepared_chat_completion(state, user_context, req, run_request, None, None).await
}

/// Project caller tools through the same host request already admitted by an ingress.
pub(crate) async fn execute_prepared_chat_completion(
    state: AppState,
    user_context: UserContext,
    req: ChatCompletionRequest,
    mut run_request: crate::uar::runtime::turn::RunExecutionRequest,
    input_finding: Option<crate::uar::guardrails::GuardrailFinding>,
    response_timeout_ms: Option<u64>,
) -> axum::response::Response {
    let run_manager = &state.run_manager;
    let conversation_id = run_request.session_id.clone().unwrap_or_else(|| Uuid::new_v4().to_string());
    let created = unix_now_secs();
    let agent_id = run_request.artifact.id.clone();
    let mut input = match req.client_tool_input() {
        Ok(input) => input,
        Err(error) => return openai_error(StatusCode::BAD_REQUEST, "invalid_request_error", "invalid_client_tools", error),
    };
    input.output_token_cap = run_request.output_token_cap;
    if let Some(binding) = &run_request.collaboration_binding {
        if let Err(error) = binding.revalidate(&run_request.artifact).await {
            return openai_error(StatusCode::CONFLICT, "invalid_request_error", "target_binding_changed", error.to_string());
        }
    }
    let owner_key = match user_context.session_owner_key() {
        Ok(owner) => owner,
        Err(error) => return openai_error(StatusCode::UNAUTHORIZED, "invalid_request_error", "invalid_principal", error.to_string()),
    };
    let needs_persistence = !input.tools.is_empty() || input.history.iter().any(|message| {
        message.get("tool_calls").is_some() || message.get("role").and_then(serde_json::Value::as_str) == Some("tool")
    });
    let store = state.persistence.clone();
    let (input, claim) = match store.as_deref() {
        Some(store) => {
            if needs_persistence && !store.supports_durable_client_tools() {
                return openai_error(StatusCode::SERVICE_UNAVAILABLE, "server_error", "durable_client_tools_unsupported", "Durable client tool continuations are unavailable");
            }
            match crate::uar::api::client_tools::prepare(run_manager.as_ref(), store, &owner_key, user_context.tenant_id.as_ref().map(|tenant| tenant.as_str()), &agent_id, &input).await {
                Ok(crate::uar::api::client_tools::ClientToolPreparation::Fresh(input)) => (input, None),
                Ok(crate::uar::api::client_tools::ClientToolPreparation::Continue { input, claim }) => (input, Some(claim)),
                Ok(crate::uar::api::client_tools::ClientToolPreparation::Replay { response, events }) => return completion_response(req.stream, response, events),
                Err(error) => return openai_error(StatusCode::CONFLICT, "invalid_request_error", "client_tool_continuation_rejected", error.to_string()),
            }
        }
        None if needs_persistence => return openai_error(StatusCode::SERVICE_UNAVAILABLE, "server_error", "durable_client_tools_unsupported", "Durable client tool continuations are unavailable"),
        None => (input, None),
    };
    run_request.input = None;
    run_request.client_tools = Some(input);
    run_request.session_id = match &claim { Some(claim) => claim.session_id.clone(), None => Some(conversation_id) };
    run_request.presentation_negotiation = req.presentation_negotiation.clone();
    let (run_id, observed, mut rx) = match run_manager.execute_request_observed(run_request).await {
        Ok(observed) => observed,
        Err(error) => return openai_error(StatusCode::BAD_GATEWAY, "server_error", "run_observation_lost", error.to_string()),
    };
    if let Some(finding) = input_finding {
        run_manager.emit_to_run(&run_id, NormalizedEvent::GuardrailFlagged {
            run_id: Some(run_id.clone()), category: finding.category.as_str().to_owned(), reason: finding.reason,
        }).await;
    }
    let runtime_agent_id = agent_id;
    let handoff_tenant = user_context.tenant_id.as_ref().map(|tenant| tenant.as_str().to_owned());
    let mut replay = observed.into_iter();
    let id = format!("chatcmpl-{run_id}");
    if req.stream {
        let settled_manager = std::sync::Arc::clone(run_manager);
        let stream = async_stream::stream! {
            let mut content = String::new();
            let mut pending_calls = Vec::new();
            let initial = chunk(&id, created, &req.model, serde_json::json!({"role":"assistant","content":""}), None);
            let mut events = vec![initial.clone()];
            yield Ok::<_, std::convert::Infallible>(json_event(initial));
            loop {
                let received = match replay.next() {
                    Some(event) => Ok(event),
                    None => rx.recv().await,
                };
                let event = match received {
                    Ok(event) => event.event,
                    Err(error) => {
                        yield Ok(json_event(error_value("server_error", "run_observation_lost", format!("Run observation lost: {error}"))));
                        yield Ok(Event::default().data("[DONE]"));
                        break;
                    }
                };
                let (calls, finish_reason, usage) = match event {
                    NormalizedEvent::ChatDelta { text_delta, .. } => {
                        content.push_str(&text_delta);
                        let delta = chunk(&id, created, &req.model, serde_json::json!({"content":text_delta}), None);
                        events.push(delta.clone());
                        yield Ok(json_event(delta));
                        continue;
                    }
                    NormalizedEvent::ClientToolsRequested { calls, .. } => {
                        pending_calls = calls;
                        continue;
                    }
                    terminal @ (NormalizedEvent::RunDone { .. } | NormalizedEvent::RunDoneWithUsage { .. }) => {
                        match settled_manager.await_settled_run_status(&run_id).await {
                            Ok(crate::uar::domain::runs::RunStatus::Done) => {}
                            Ok(status) => {
                                yield Ok(json_event(error_value("server_error", "run_settlement_failed", format!("Run settled without success: {status:?}"))));
                                yield Ok(Event::default().data("[DONE]"));
                                break;
                            }
                            Err(error) => {
                                yield Ok(json_event(error_value("server_error", "run_settlement_failed", error.to_string())));
                                yield Ok(Event::default().data("[DONE]"));
                                break;
                            }
                        }
                        let calls = std::mem::take(&mut pending_calls);
                        let finish_reason = if calls.is_empty() { "stop" } else { "tool_calls" };
                        (calls, finish_reason, super::usage::from_run_done(&terminal))
                    }
                    NormalizedEvent::Error { code, message, .. } => {
                        let kind = if code.starts_with("budget_") { "budget_error" } else { "server_error" };
                        yield Ok(json_event(error_value(kind, &code, message)));
                        yield Ok(Event::default().data("[DONE]"));
                        break;
                    }
                    NormalizedEvent::Cancelled { .. } => {
                        yield Ok(json_event(error_value("server_error", "run_cancelled", "Run cancelled".to_owned())));
                        yield Ok(Event::default().data("[DONE]"));
                        break;
                    }
                    _ => continue,
                };
                if !calls.is_empty() {
                    let Some(store) = store.as_deref() else {
                        yield Ok(json_event(error_value("server_error", "client_tool_handoff_unconfirmed", "Continuation storage unavailable".to_owned())));
                        yield Ok(Event::default().data("[DONE]"));
                        break;
                    };
                    if let Err(error) = crate::uar::api::client_tools::confirm_handoff_for_run(
                        settled_manager.as_ref(), store, &owner_key, handoff_tenant.as_deref(), &runtime_agent_id, &run_id,
                    ).await {
                        yield Ok(json_event(error_value("server_error", "client_tool_handoff_unconfirmed", error.to_string())));
                        yield Ok(Event::default().data("[DONE]"));
                        break;
                    }
                }
                let response = completion(&id, created, &req.model, &content, &calls, finish_reason, usage);
                let mut final_events = Vec::new();
                for (index, call) in calls.iter().enumerate() {
                    final_events.push(chunk(&id, created, &req.model, serde_json::json!({"tool_calls":[{
                        "index":index,"id":call.id,"type":"function",
                        "function":{"name":call.function.name,"arguments":call.function.arguments}
                    }]}), None));
                }
                final_events.push(chunk(&id, created, &req.model, serde_json::json!({}), Some(finish_reason)));
                events.extend(final_events.iter().cloned());
                if let Some(claim) = &claim {
                    let Some(store) = store.as_deref() else {
                        yield Ok(json_event(error_value("server_error", "client_tool_completion_failed", "Continuation storage unavailable".to_owned())));
                        yield Ok(Event::default().data("[DONE]"));
                        break;
                    };
                    if let Err(error) = crate::uar::api::client_tools::complete(store, claim, response, events).await {
                        yield Ok(json_event(error_value("server_error", "client_tool_completion_failed", error.to_string())));
                        yield Ok(Event::default().data("[DONE]"));
                        break;
                    }
                }
                for event in final_events { yield Ok(json_event(event)); }
                yield Ok(Event::default().data("[DONE]"));
                break;
            }
        };
        return Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::default()).into_response();
    }
    let response_deadline = response_timeout_ms.map(|millis| tokio::time::Instant::now() + Duration::from_millis(millis));
    // The deadline covers settlement and durable writes as well as observation.
    // Dropping this projection leaves the manager-owned producer handles tracked.
    let projection = async {
        let mut content = String::new();
        let mut pending_calls = Vec::new();
        let mut events = vec![chunk(&id, created, &req.model, serde_json::json!({"role":"assistant","content":""}), None)];
        let (calls, finish_reason, usage) = loop {
            let received = match replay.next() {
                Some(event) => Ok(event),
                None => rx.recv().await,
            };
            let event = match received {
                Ok(event) => event.event,
                Err(error) => return completion_failure(req.stream, events, "server_error", "run_observation_lost", format!("Run observation lost: {error}")),
            };
            match event {
                NormalizedEvent::ChatDelta { text_delta, .. } => {
                    content.push_str(&text_delta);
                    events.push(chunk(&id, created, &req.model, serde_json::json!({"content":text_delta}), None));
                }
                NormalizedEvent::ClientToolsRequested { calls, .. } => {
                    pending_calls = calls;
                }
                terminal @ (NormalizedEvent::RunDone { .. } | NormalizedEvent::RunDoneWithUsage { .. }) => {
                    match run_manager.await_settled_run_status(&run_id).await {
                        Ok(crate::uar::domain::runs::RunStatus::Done) => {}
                        Ok(status) => return openai_error(StatusCode::BAD_GATEWAY, "server_error", "run_settlement_failed", format!("Run settled without success: {status:?}")),
                        Err(error) => return openai_error(StatusCode::BAD_GATEWAY, "server_error", "run_settlement_failed", error.to_string()),
                    }
                    let calls = std::mem::take(&mut pending_calls);
                    let finish_reason = if calls.is_empty() { "stop" } else { "tool_calls" };
                    break (calls, finish_reason, super::usage::from_run_done(&terminal));
                }
                NormalizedEvent::Error { code, message, .. } => {
                    let kind = if code.starts_with("budget_") { "budget_error" } else { "server_error" };
                    return completion_failure(req.stream, events, kind, &code, message);
                }
                NormalizedEvent::Cancelled { .. } => return completion_failure(req.stream, events, "server_error", "run_cancelled", "Run cancelled".to_owned()),
                _ => {}
            }
        };
        if !calls.is_empty() {
            let Some(store) = store.as_deref() else {
                return openai_error(StatusCode::SERVICE_UNAVAILABLE, "server_error", "client_tool_handoff_unconfirmed", "Continuation storage unavailable");
            };
            if let Err(error) = crate::uar::api::client_tools::confirm_handoff_for_run(
                run_manager.as_ref(), store, &owner_key, handoff_tenant.as_deref(), &runtime_agent_id, &run_id,
            ).await {
                return openai_error(StatusCode::BAD_GATEWAY, "server_error", "client_tool_handoff_unconfirmed", error.to_string());
            }
        }
        let response = completion(&id, created, &req.model, &content, &calls, finish_reason, usage);
        for (index, call) in calls.iter().enumerate() {
            events.push(chunk(&id, created, &req.model, serde_json::json!({"tool_calls":[{
                "index":index,"id":call.id,"type":"function",
                "function":{"name":call.function.name,"arguments":call.function.arguments}
            }]}), None));
        }
        events.push(chunk(&id, created, &req.model, serde_json::json!({}), Some(finish_reason)));
        if let Some(claim) = &claim {
            let Some(store) = store.as_deref() else {
                return openai_error(StatusCode::SERVICE_UNAVAILABLE, "server_error", "client_tool_completion_failed", "Continuation storage unavailable");
            };
            if let Err(error) = crate::uar::api::client_tools::complete(store, claim, response.clone(), events.clone()).await {
                return openai_error(StatusCode::BAD_GATEWAY, "server_error", "client_tool_completion_failed", error.to_string());
            }
        }
        completion_response(req.stream, response, events)
    };
    match response_deadline {
        Some(deadline) => match tokio::time::timeout_at(deadline, projection).await {
            Ok(response) => response,
            Err(_) => openai_error(StatusCode::GATEWAY_TIMEOUT, "timeout_error", "model_response_timeout", "Timed out waiting for model response"),
        },
        None => projection.await,
    }
}

fn openai_error(status: StatusCode, kind: &str, code: &str, message: impl Into<String>) -> axum::response::Response {
    (status, Json(serde_json::json!({"error":{"message":message.into(),"type":kind,"param":null,"code":code}}))).into_response()
}

fn chunk(id: &str, created: u64, model: &str, delta: serde_json::Value, finish_reason: Option<&str>) -> serde_json::Value {
    serde_json::json!({
        "id":id,"object":"chat.completion.chunk","created":created,"model":model,
        "choices":[{"index":0,"delta":delta,"finish_reason":finish_reason}]
    })
}

fn completion_response(stream: bool, response: serde_json::Value, events: Vec<serde_json::Value>) -> axum::response::Response {
    if !stream { return Json(response).into_response(); }
    sse_response(events)
}

fn sse_response(events: Vec<serde_json::Value>) -> axum::response::Response {
    let stream = async_stream::stream! {
        for event in events {
            yield Ok::<_, std::convert::Infallible>(json_event(event));
        }
        yield Ok(Event::default().data("[DONE]"));
    };
    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::default()).into_response()
}

fn completion_failure(stream: bool, mut events: Vec<serde_json::Value>, kind: &str, code: &str, message: String) -> axum::response::Response {
    if !stream { return openai_error(StatusCode::BAD_GATEWAY, kind, code, message); }
    events.push(serde_json::json!({"error":{"message":message,"type":kind,"param":null,"code":code}}));
    sse_response(events)
}

fn error_value(kind: &str, code: &str, message: String) -> serde_json::Value {
    serde_json::json!({"error":{"message":message,"type":kind,"param":null,"code":code}})
}

fn completion(id: &str, created: u64, model: &str, content: &str, calls: &[crate::llm::ToolCall], finish_reason: &str, usage: Option<serde_json::Value>) -> serde_json::Value {
    let mut message = serde_json::json!({"role":"assistant","content":content});
    if !calls.is_empty() {
        if content.is_empty() { message["content"] = serde_json::Value::Null; }
        message["tool_calls"] = serde_json::json!(calls);
    }
    let mut response = serde_json::json!({
        "id":id,"object":"chat.completion","created":created,"model":model,
        "choices":[{"index":0,"message":message,"finish_reason":finish_reason}]
    });
    if let Some(usage) = usage { response["usage"] = usage; }
    response
}
