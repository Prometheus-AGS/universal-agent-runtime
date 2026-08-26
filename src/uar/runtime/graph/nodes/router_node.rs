//! Router graph node — uses the LLM to choose between multiple routing options.

use async_trait::async_trait;
use futures::StreamExt;
use tracing::{debug, warn};

use crate::llm::{LlmRequest, MessageRole};
use crate::normalized::NormalizedEvent;
use crate::uar::runtime::graph::types::{GraphContext, GraphNode, GraphState, NodeResult};

/// A graph node that asks the LLM to select one of a fixed set of options,
/// then stores the chosen option in state under `_route`.
///
/// Pair this with a `Conditional` edge that reads `state.get::<String>("_route")`.
///
/// # State keys written
/// - `_route` (configurable via [`RouterNode::with_route_key`]) — chosen option name.
pub struct RouterNode {
    id: String,
    /// The question / framing to present to the LLM.
    prompt: String,
    /// Valid routing options (the LLM must return exactly one of these).
    options: Vec<String>,
    /// State key where the routing decision is stored.
    route_key: String,
}

impl RouterNode {
    /// Create a router node with the given ID, question prompt, and options.
    #[must_use]
    pub fn new(id: impl Into<String>, prompt: impl Into<String>, options: Vec<String>) -> Self {
        Self {
            id: id.into(),
            prompt: prompt.into(),
            options,
            route_key: "_route".to_string(),
        }
    }

    /// Override the state key used to store the routing decision.
    #[must_use]
    pub fn with_route_key(mut self, key: impl Into<String>) -> Self {
        self.route_key = key.into();
        self
    }
}

impl std::fmt::Debug for RouterNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RouterNode")
            .field("id", &self.id)
            .field("options", &self.options)
            .finish()
    }
}

#[async_trait]
impl GraphNode for RouterNode {
    fn id(&self) -> &str {
        &self.id
    }

    async fn execute(&self, mut state: GraphState, ctx: &GraphContext) -> NodeResult {
        debug!(
            node_id = %self.id,
            run_id = %ctx.run_id,
            options = ?self.options,
            "RouterNode executing"
        );

        // Build a routing prompt that asks the LLM to pick exactly one option
        let options_list = self.options.join(", ");
        let user_input = state
            .get::<String>("_agent_input")
            .or_else(|| {
                state.messages.iter().rev().find_map(|message| {
                    (message.get("role").and_then(|role| role.as_str()) == Some("user"))
                        .then(|| message.get("content").and_then(|content| content.as_str()))
                        .flatten()
                        .map(str::to_string)
                })
            })
            .unwrap_or_default();
        let router_prompt = format!(
            "{}\n\nUser request:\n{}\n\nYou MUST respond with exactly one of the following options (no extra text): {options_list}",
            self.prompt, user_input
        );

        let messages: Vec<serde_json::Value> = vec![
            serde_json::json!({
                "role": MessageRole::System,
                "content": "You are a routing assistant. Respond with exactly one option from the provided list."
            }),
            serde_json::json!({
                "role": MessageRole::User,
                "content": router_prompt
            }),
        ];

        let req = LlmRequest {
            messages,
            tools: Vec::new(),
            cache_strategy: ctx.cache_strategy.clone(),
            thinking_config: None,
            anthropic_system: None,
            extra_params: None,
        };

        let stream = match ctx.driver.stream(req).await {
            Ok(s) => s,
            Err(e) => {
                return NodeResult::Error(state, format!("RouterNode driver error: {e}"));
            }
        };

        futures::pin_mut!(stream);

        let mut response_text = String::new();

        while let Some(event_result) = stream.next().await {
            match event_result {
                Ok(NormalizedEvent::MessageDelta { text }) => {
                    response_text.push_str(&text);
                }
                Ok(NormalizedEvent::Done) => break,
                Ok(NormalizedEvent::Error { message, .. }) => {
                    return NodeResult::Error(state, format!("RouterNode stream error: {message}"));
                }
                Ok(_) => {}
                Err(e) => {
                    return NodeResult::Error(state, format!("RouterNode stream error: {e}"));
                }
            }
        }

        // Match response against known options (case-insensitive, trimmed)
        let trimmed = response_text.trim().to_lowercase();
        let chosen = self
            .options
            .iter()
            .find(|opt| trimmed == opt.to_lowercase())
            .cloned();

        let route = if let Some(opt) = chosen {
            opt
        } else {
            warn!(
                node_id = %self.id,
                run_id = %ctx.run_id,
                response = %response_text,
                options = ?self.options,
                "RouterNode: LLM response did not match any option — using first option as fallback"
            );
            self.options.first().cloned().unwrap_or_default()
        };

        state.set(&self.route_key, &route);
        NodeResult::Continue(state)
    }
}
