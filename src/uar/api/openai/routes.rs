use super::types::{
    ChatCompletionChunk, ChatCompletionChunkChoice, ChatCompletionChunkDelta,
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
    Json(req): Json<ChatCompletionRequest>,
) -> impl IntoResponse {
    let run_manager = &state.run_manager;
    let conversation_id = Uuid::new_v4().to_string();
    let created = unix_now_secs();

    // Extract last user message
    let last_message = req
        .messages
        .last()
        .map(|m| m.content.clone())
        .unwrap_or_default();

    let agent_id = match req.model.as_str() {
        "default" => "default-agent",
        "orchestrator" => "orchestrator-agent",
        id => id,
    };
    let agent = match run_manager.resolve_registered_agent(agent_id).await {
        Ok(agent) => agent,
        Err(error) => return agent_resolution_response(error),
    };

    // Start Run
    // Note: We are creating a NEW session for each request for simplicity in this adapter
    // because OpenAI API is stateless (except for message history passed in request).
    // Ideally we would map thread_id if UAR supported it in context, but UAR sessions are ID-based.
    // We'll create an ephemeral session ID here.
    let mut run_request =
        match crate::uar::runtime::turn::RunExecutionRequest::new(agent, last_message.clone())
            .with_user_context(&user_context)
        {
            Ok(request) => request,
            Err(_) => {
                return (StatusCode::UNAUTHORIZED, "Invalid run principal").into_response();
            }
        };
    run_request.session_id = Some(conversation_id.clone());
    run_request.presentation_negotiation = req.presentation_negotiation.clone();
    let run_id = run_manager.execute_request(run_request).await;

    // Subscribe to events
    let Some(mut rx) = run_manager.subscribe(&run_id).await else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to subscribe to run",
        )
            .into_response();
    };

    // Convert to SSE stream
    let stream = async_stream::stream! {
        let id = Uuid::new_v4().to_string();

        // Yield initial role chunk
         let initial_chunk = ChatCompletionChunk {
            id: id.clone(),
            object: "chat.completion.chunk".to_string(),
            created,
            model: req.model.clone(),
            choices: vec![ChatCompletionChunkChoice {
                index: 0,
                delta: ChatCompletionChunkDelta {
                    role: Some("assistant".to_string()),
                    content: Some(String::new()),
                },
                finish_reason: None,
            }],
        };
        yield Ok::<_, std::convert::Infallible>(json_event(initial_chunk));

        while let Ok(event) = rx.recv().await {
            match event.event {
                NormalizedEvent::ChatDelta { text_delta, .. } => {
                    let chunk = ChatCompletionChunk {
                        id: id.clone(),
                        object: "chat.completion.chunk".to_string(),
                        created,
                        model: req.model.clone(),
                        choices: vec![ChatCompletionChunkChoice {
                            index: 0,
                            delta: ChatCompletionChunkDelta {
                                role: None,
                                content: Some(text_delta),
                            },
                            finish_reason: None,
                        }],
                    };
                    yield Ok(json_event(chunk));
                }
                NormalizedEvent::RunDone { .. } | NormalizedEvent::RunDoneWithUsage { .. } => {
                     let chunk = ChatCompletionChunk {
                        id: id.clone(),
                        object: "chat.completion.chunk".to_string(),
                        created,
                        model: req.model.clone(),
                        choices: vec![ChatCompletionChunkChoice {
                            index: 0,
                            delta: ChatCompletionChunkDelta::default(),
                            finish_reason: Some("stop".to_string()),
                        }],
                    };
                    yield Ok(json_event(chunk));
                    yield Ok(Event::default().data("[DONE]"));
                    break;
                }
                NormalizedEvent::Error { message, .. } => {
                    tracing::error!("Error in chat completion: {}", message);
                    break;
                }
                _ => {}
            }
        }
    };

    Sse::new(stream)
        .keep_alive(axum::response::sse::KeepAlive::default())
        .into_response()
}
