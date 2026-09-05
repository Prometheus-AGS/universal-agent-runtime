//! LLM graph node — calls the configured LLM driver and stores the response.

use async_trait::async_trait;
use tracing::{debug, warn};

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
        let Some(host) = &ctx.tool_host else {
            return NodeResult::Error(state, "Graph model host is unavailable".into());
        };

        if state.messages.is_empty() && self.system_prompt.is_none() {
            return NodeResult::Error(state, "LlmNode: no messages to send".to_string());
        }

        let turn = match host
            .model_turn(
                &ctx.run_id,
                state.messages.clone(),
                self.system_prompt.clone(),
            )
            .await
        {
            Ok(turn) => turn,
            Err(error) => return NodeResult::Error(state, format!("LlmNode host error: {error}")),
        };
        state.messages = turn
            .messages
            .into_iter()
            .map(|message| serde_json::json!(message))
            .collect();
        if let Some(error) = turn.error {
            return NodeResult::Error(state, error);
        }
        if turn.text.is_empty() {
            warn!(node_id = %self.id, run_id = %ctx.run_id, "LlmNode received empty response");
        }
        state.set(&self.output_key, &turn.text);

        NodeResult::Continue(state)
    }
}
