//! LLM graph node — calls the configured LLM driver and stores the response.

use async_trait::async_trait;
use futures::StreamExt;
use tracing::{debug, warn};

use crate::llm::{LlmRequest, MessageRole};
use crate::normalized::NormalizedEvent;
use crate::uar::runtime::graph::types::{GraphContext, GraphNode, GraphState, NodeResult};

/// A graph node that sends the current conversation to the LLM driver and
/// appends the assistant reply to `state.messages`.
///
/// # State keys written
/// - `state.messages` — assistant reply appended
/// - `_llm_response` (configurable via [`LlmNode::with_output_key`]) — raw text
pub struct LlmNode {
    id: String,
    /// Optional system prompt injected as the first message.
    system_prompt: Option<String>,
    /// State key to write the raw text response into.
    output_key: String,
}

impl LlmNode {
    /// Create an LLM node with the given unique ID.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            system_prompt: None,
            output_key: "_llm_response".to_string(),
        }
    }

    /// Set a system prompt that is prepended to every request.
    #[must_use]
    pub fn with_system(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }

    /// Override the state key used to store the text response.
    #[must_use]
    pub fn with_output_key(mut self, key: impl Into<String>) -> Self {
        self.output_key = key.into();
        self
    }
}

impl std::fmt::Debug for LlmNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LlmNode").field("id", &self.id).finish()
    }
}

#[async_trait]
impl GraphNode for LlmNode {
    fn id(&self) -> &str {
        &self.id
    }

    async fn execute(&self, mut state: GraphState, ctx: &GraphContext) -> NodeResult {
        debug!(node_id = %self.id, run_id = %ctx.run_id, "LlmNode executing");

        // Build the message list: optional system prompt + conversation history
        let mut messages: Vec<serde_json::Value> = Vec::new();

        if let Some(ref sys) = self.system_prompt {
            messages.push(serde_json::json!({
                "role": MessageRole::System,
                "content": sys
            }));
        }

        messages.extend(state.messages.clone());

        if messages.is_empty() {
            return NodeResult::Error(state, "LlmNode: no messages to send".to_string());
        }

        let req = LlmRequest {
            messages,
            // The MCP registry's tools, in OpenAI shape.
            //
            // This was `Vec::new()` with the comment "Graph nodes don't
            // handle tool calls directly". The effect was that NO tools
            // reached any provider: the model saw tool names only if they
            // appeared in prompt text, invented a call from that, and the
            // provider had no schema to validate it against — so the call
            // came back as prose and nothing executed. Observed 2026-09-01
            // against a local ferrox sidecar, where the model correctly
            // named `file_read` and `time__current_time` and neither ran.
            //
            // `openai_tools_json` already existed on the registry, and the
            // registry is already on `GraphContext`; only the wiring was
            // missing.
            tools: ctx.mcp.openai_tools_json(),
            cache_strategy: ctx.cache_strategy.clone(),
            thinking_config: None,
            anthropic_system: None,
            extra_params: None,
        };

        let stream = match ctx.driver.stream(req).await {
            Ok(s) => s,
            Err(e) => {
                return NodeResult::Error(state, format!("LlmNode driver error: {e}"));
            }
        };

        futures::pin_mut!(stream);

        let mut text = String::new();

        while let Some(event_result) = stream.next().await {
            match event_result {
                Ok(NormalizedEvent::MessageDelta { text: delta }) => {
                    text.push_str(&delta);
                }
                Ok(NormalizedEvent::Done) => break,
                Ok(NormalizedEvent::Error { message, .. }) => {
                    return NodeResult::Error(state, format!("LlmNode stream error: {message}"));
                }
                Ok(_) => {} // ignore other events (stream start, tool calls, etc.)
                Err(e) => {
                    return NodeResult::Error(state, format!("LlmNode stream error: {e}"));
                }
            }
        }

        if text.is_empty() {
            warn!(node_id = %self.id, run_id = %ctx.run_id, "LlmNode received empty response");
        }

        // Store raw text in data bag
        state.set(&self.output_key, &text);

        // Append as assistant message in history
        state.messages.push(serde_json::json!({
            "role": "assistant",
            "content": text
        }));

        NodeResult::Continue(state)
    }
}
