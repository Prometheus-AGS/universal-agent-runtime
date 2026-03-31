//! [`ractor::Actor`] implementation that wraps a UAR agent.
//!
//! Each `AgentActor` represents a single agent running as an independently
//! addressable actor. It can receive user prompts, tool results, and
//! collaboration requests from other agents.

use std::sync::Arc;

use ractor::{Actor, ActorProcessingErr, ActorRef};
use tracing::{error, info};

use crate::config::LlmConfig;
use crate::llm::{Message, MessageContent, MessageRole, Orchestrator};
use crate::mcp::registry::McpRegistry;
use crate::uar::runtime::native_skill::NativeSkillRegistry;

use super::messages::{ActorStatus, AgentMessage, AgentReply};

// ---------------------------------------------------------------------------
// Actor state
// ---------------------------------------------------------------------------

/// Internal mutable state held by a running agent actor.
#[derive(Debug)]
pub struct AgentActorState {
    /// Unique agent identifier (matches the agent artifact ID).
    pub agent_id: String,
    /// Current lifecycle status.
    pub status: ActorStatus,
    /// Conversation history for this actor's session.
    pub history: Vec<Message>,
    /// LLM orchestrator for processing prompts.
    pub orchestrator: Arc<Orchestrator>,
}

// ---------------------------------------------------------------------------
// Actor definition
// ---------------------------------------------------------------------------

/// An agent running as an actor.
///
/// Spawned by [`super::system::ActorCollaboration`], each `AgentActor` wraps
/// an orchestrator and holds its own conversation history so it can
/// independently process messages.
#[derive(Debug)]
pub struct AgentActor;

/// Arguments passed to [`AgentActor::pre_start`] when spawning.
#[derive(Debug)]
pub struct AgentActorArgs {
    /// Agent artifact ID.
    pub agent_id: String,
    /// LLM config for this agent's orchestrator.
    pub llm_config: LlmConfig,
    /// MCP registry shared across the runtime.
    pub mcp: Arc<McpRegistry>,
    /// Native skill registry shared across the runtime.
    pub native_skills: Arc<NativeSkillRegistry>,
    /// Optional system prompt to seed the conversation.
    pub system_prompt: Option<String>,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Collect streamed events into a single response string.
async fn collect_stream_response(
    stream: impl futures::Stream<Item = crate::normalized::NormalizedEvent> + Send,
    agent_id: &str,
) -> (String, bool) {
    use crate::normalized::NormalizedEvent;
    use futures::StreamExt;

    futures::pin_mut!(stream);

    let mut response_text = String::new();
    let mut success = true;

    while let Some(event) = stream.next().await {
        match event {
            NormalizedEvent::MessageDelta { text } => {
                response_text.push_str(&text);
            }
            NormalizedEvent::Error { message: err, .. } => {
                error!(agent_id = %agent_id, error = %err, "LLM error");
                response_text = err;
                success = false;
                break;
            }
            NormalizedEvent::Done => break,
            _ => {} // Ignore other events
        }
    }

    (response_text, success)
}

// ---------------------------------------------------------------------------
// Actor trait implementation
// ---------------------------------------------------------------------------

impl Actor for AgentActor {
    type Msg = AgentMessage;
    type State = AgentActorState;
    type Arguments = AgentActorArgs;

    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        info!(agent_id = %args.agent_id, "Agent actor starting");

        let orchestrator = Arc::new(Orchestrator::new(
            args.llm_config,
            args.mcp,
            args.native_skills,
        )?);

        let mut history = Vec::new();
        if let Some(system_prompt) = args.system_prompt {
            history.push(Message {
                role: MessageRole::System,
                content: MessageContent::text(system_prompt),
                tool_call_id: None,
                tool_calls: None,
            });
        }

        Ok(AgentActorState {
            agent_id: args.agent_id,
            status: ActorStatus::Running,
            history,
            orchestrator,
        })
    }

    async fn handle(
        &self,
        myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            AgentMessage::UserPrompt { content, reply } => {
                info!(agent_id = %state.agent_id, "Processing user prompt");

                // Add the user message to history
                state.history.push(Message {
                    role: MessageRole::User,
                    content: MessageContent::text(content),
                    tool_call_id: None,
                    tool_calls: None,
                });

                // Use the orchestrator to get a response
                let (response_text, success) = match state
                    .orchestrator
                    .chat_with_history(state.history.clone())
                    .await
                {
                    Ok(stream) => collect_stream_response(stream, &state.agent_id).await,
                    Err(e) => {
                        error!(agent_id = %state.agent_id, error = %e, "Orchestrator error");
                        (format!("Error: {e}"), false)
                    }
                };

                // Add assistant response to history
                if success {
                    state.history.push(Message {
                        role: MessageRole::Assistant,
                        content: MessageContent::text(&response_text),
                        tool_call_id: None,
                        tool_calls: None,
                    });
                }

                // Reply if a channel was provided
                if let Some(reply_tx) = reply {
                    let _ = reply_tx.send(AgentReply {
                        content: response_text,
                        success,
                        metadata: serde_json::json!({}),
                    });
                }
            }

            AgentMessage::Collaborate {
                from_agent_id,
                task,
                reply,
            } => {
                info!(
                    agent_id = %state.agent_id,
                    from = %from_agent_id,
                    "Received collaboration request"
                );

                let collab_prompt =
                    format!("[Collaboration request from agent {from_agent_id}]: {task}");

                state.history.push(Message {
                    role: MessageRole::User,
                    content: MessageContent::text(collab_prompt),
                    tool_call_id: None,
                    tool_calls: None,
                });

                let (response_text, success) = match state
                    .orchestrator
                    .chat_with_history(state.history.clone())
                    .await
                {
                    Ok(stream) => collect_stream_response(stream, &state.agent_id).await,
                    Err(e) => (format!("Error: {e}"), false),
                };

                if success {
                    state.history.push(Message {
                        role: MessageRole::Assistant,
                        content: MessageContent::text(&response_text),
                        tool_call_id: None,
                        tool_calls: None,
                    });
                }

                let _ = reply.send(AgentReply {
                    content: response_text,
                    success,
                    metadata: serde_json::json!({
                        "collaboration_from": from_agent_id,
                    }),
                });
            }

            AgentMessage::ToolResult {
                tool_call_id,
                content,
                success: _,
            } => {
                info!(
                    agent_id = %state.agent_id,
                    tool_call_id = %tool_call_id,
                    "Received tool result"
                );

                state.history.push(Message {
                    role: MessageRole::Tool,
                    content: MessageContent::text(
                        serde_json::to_string(&content).unwrap_or_default(),
                    ),
                    tool_call_id: Some(tool_call_id),
                    tool_calls: None,
                });
            }

            AgentMessage::Shutdown => {
                info!(agent_id = %state.agent_id, "Agent actor shutting down");
                state.status = ActorStatus::Stopping;
                myself.stop(None);
            }
        }

        Ok(())
    }

    async fn post_stop(
        &self,
        _myself: ActorRef<Self::Msg>,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        info!(agent_id = %state.agent_id, "Agent actor stopped");
        state.status = ActorStatus::Stopped;
        Ok(())
    }
}
