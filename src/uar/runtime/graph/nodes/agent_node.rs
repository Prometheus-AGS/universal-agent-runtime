//! AgentNode — delegates execution to a local or remote agent.
//!
//! # Routing
//!
//! | `agent_id_or_url` | Routing |
//! |-------------------|---------|
//! | Starts with `http://` or `https://` | Remote — calls the URL via A2A JSON-RPC 2.0 |
//! | Any other string | Local — calls the run's configured LLM driver with the sub-agent identity |
//!
//! # State keys read
//! - `_agent_input` — the message text to send (falls back to the last user message in
//!   `state.messages`, or an empty string).
//!
//! # State keys written
//! - `_agent_result_{node_id}` — JSON-serialised `Task` returned by the remote agent.
//! - `_agent_task_id_{node_id}` — task ID for follow-up polling.
//! - `_agent_output_{node_id}` — text returned by either a local or remote agent.

use async_trait::async_trait;
use futures::StreamExt;
use tracing::{debug, warn};

use crate::llm::{LlmRequest, MessageRole};
use crate::normalized::NormalizedEvent;
use crate::uar::{
    api::a2a::{
        A2AClient,
        types::{Message, Part, Role, Task},
    },
    runtime::graph::types::{GraphContext, GraphNode, GraphState, NodeResult},
};

/// A graph node that delegates to another agent through the run's LLM driver
/// (local IDs) or A2A (remote URLs).
pub struct AgentNode {
    id: String,
    /// Agent ID (local) or full A2A endpoint URL (remote).
    agent_id_or_url: String,
}

impl AgentNode {
    /// Create a new `AgentNode`.
    ///
    /// Pass a URL (`https://...`) for a remote A2A agent or an agent ID string
    /// for a local sub-agent executed through the run's configured LLM driver.
    #[must_use]
    pub fn new(id: impl Into<String>, agent_id_or_url: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            agent_id_or_url: agent_id_or_url.into(),
        }
    }

    fn is_remote(&self) -> bool {
        self.agent_id_or_url.starts_with("http://") || self.agent_id_or_url.starts_with("https://")
    }

    fn output_key(&self) -> String {
        format!("_agent_output_{}", self.id)
    }

    fn remote_task_text(task: &Task) -> Option<String> {
        fn text_from_parts<'a>(parts: impl Iterator<Item = &'a Part>) -> Option<String> {
            let text = parts
                .filter_map(|part| match part {
                    Part::Text { text } => Some(text.as_str()),
                    Part::File { .. } | Part::Data { .. } => None,
                })
                .collect::<Vec<_>>()
                .join("\n");
            (!text.trim().is_empty()).then_some(text)
        }

        task.status
            .message
            .as_ref()
            .and_then(|message| text_from_parts(message.parts.iter()))
            .or_else(|| {
                task.history
                    .iter()
                    .rev()
                    .find(|message| message.role == Role::Agent)
                    .and_then(|message| text_from_parts(message.parts.iter()))
            })
            .or_else(|| {
                task.artifacts
                    .iter()
                    .rev()
                    .find_map(|artifact| text_from_parts(artifact.parts.iter()))
            })
    }

    async fn execute_local(
        &self,
        mut state: GraphState,
        ctx: &GraphContext,
        input_text: &str,
    ) -> NodeResult {
        let request = LlmRequest {
            messages: vec![
                serde_json::json!({
                    "role": MessageRole::System,
                    "content": format!(
                        "You are the '{}' sub-agent. Answer only the delegated task and identify concrete evidence for your conclusion.",
                        self.agent_id_or_url
                    )
                }),
                serde_json::json!({
                    "role": MessageRole::User,
                    "content": input_text
                }),
            ],
            tools: Vec::new(),
            cache_strategy: None,
            thinking_config: None,
            anthropic_system: None,
            extra_params: None,
        };
        let stream = match ctx.driver.stream(request).await {
            Ok(stream) => stream,
            Err(error) => {
                return NodeResult::Error(
                    state,
                    format!("AgentNode '{}' driver error: {error}", self.id),
                );
            }
        };
        futures::pin_mut!(stream);

        let mut output = String::new();
        while let Some(event) = stream.next().await {
            match event {
                Ok(NormalizedEvent::MessageDelta { text }) => output.push_str(&text),
                Ok(NormalizedEvent::Done) => break,
                Ok(NormalizedEvent::Error { message, .. }) => {
                    return NodeResult::Error(
                        state,
                        format!("AgentNode '{}' stream error: {message}", self.id),
                    );
                }
                Ok(_) => {}
                Err(error) => {
                    return NodeResult::Error(
                        state,
                        format!("AgentNode '{}' stream error: {error}", self.id),
                    );
                }
            }
        }

        if output.trim().is_empty() {
            return NodeResult::Error(
                state,
                format!("AgentNode '{}' returned empty output", self.id),
            );
        }

        state.set(&self.output_key(), output);
        NodeResult::Continue(state)
    }
}

impl std::fmt::Debug for AgentNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AgentNode")
            .field("id", &self.id)
            .field("agent_id_or_url", &self.agent_id_or_url)
            .finish()
    }
}

#[async_trait]
impl GraphNode for AgentNode {
    fn id(&self) -> &str {
        &self.id
    }

    async fn execute(&self, mut state: GraphState, ctx: &GraphContext) -> NodeResult {
        // Determine the input message for the delegated agent.
        let input_text: String = state
            .get::<String>("_agent_input")
            .or_else(|| {
                state
                    .messages
                    .last()
                    .and_then(|m| m.get("content").and_then(|c| c.as_str()).map(String::from))
            })
            .unwrap_or_default();

        if !self.is_remote() {
            debug!(
                run_id = %ctx.run_id,
                node_id = %self.id,
                agent_id = %self.agent_id_or_url,
                "AgentNode delegating locally"
            );
            return self.execute_local(state, ctx, &input_text).await;
        }

        let url = self.agent_id_or_url.clone();

        debug!(
            run_id = %ctx.run_id,
            node_id = %self.id,
            endpoint = %url,
            is_remote = self.is_remote(),
            "AgentNode delegating"
        );

        let msg = Message::user_text(&input_text);
        let client = A2AClient::new();

        match client.send_message(&url, &msg).await {
            Ok(task) => {
                // Store the full task JSON and convenience task_id in state.
                let task_id = task.id.clone();
                let task_json = serde_json::to_value(&task).unwrap_or_default();
                state.set(&format!("_agent_result_{}", self.id), task_json);
                state.set(&format!("_agent_task_id_{}", self.id), task_id);
                if let Some(output) = Self::remote_task_text(&task) {
                    state.set(&self.output_key(), output);
                }
                NodeResult::Continue(state)
            }
            Err(e) => {
                warn!(
                    run_id = %ctx.run_id,
                    node_id = %self.id,
                    endpoint = %url,
                    error = %e,
                    "AgentNode delegation failed"
                );
                NodeResult::Error(state, format!("AgentNode '{}' failed: {e}", self.id))
            }
        }
    }
}
