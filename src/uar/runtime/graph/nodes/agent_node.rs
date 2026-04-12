//! AgentNode — delegates execution to a local or remote agent.
//!
//! # Routing
//!
//! | `agent_id_or_url` | Routing |
//! |-------------------|---------|
//! | Starts with `http://` or `https://` | Remote — calls the URL via A2A JSON-RPC 2.0 |
//! | Any other string | Local — calls `{UAR_LOCAL_A2A_URL}/{id}` or falls back to A2A at `/a2a/compiler` |
//!
//! # State keys read
//! - `_agent_input` — the message text to send (falls back to the last user message in
//!   `state.messages`, or an empty string).
//!
//! # State keys written
//! - `_agent_result_{node_id}` — JSON-serialised `Task` returned by the remote agent.
//! - `_agent_task_id_{node_id}` — task ID for follow-up polling.

use async_trait::async_trait;
use tracing::{debug, warn};

use crate::uar::{
    api::a2a::{A2AClient, types::Message},
    runtime::graph::types::{GraphContext, GraphNode, GraphState, NodeResult},
};

/// A graph node that delegates to another agent (local or remote) via A2A.
pub struct AgentNode {
    id: String,
    /// Agent ID (local) or full A2A endpoint URL (remote).
    agent_id_or_url: String,
}

impl AgentNode {
    /// Create a new `AgentNode`.
    ///
    /// Pass a URL (`https://...`) for a remote agent or an agent ID string for
    /// a local agent reachable at the UAR local A2A endpoint.
    #[must_use]
    pub fn new(id: impl Into<String>, agent_id_or_url: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            agent_id_or_url: agent_id_or_url.into(),
        }
    }

    fn is_remote(&self) -> bool {
        self.agent_id_or_url.starts_with("http://")
            || self.agent_id_or_url.starts_with("https://")
    }

    /// Resolve the A2A endpoint URL.
    fn endpoint_url(&self) -> String {
        if self.is_remote() {
            self.agent_id_or_url.clone()
        } else {
            // Local agent — resolve via UAR_LOCAL_A2A_URL env var or default.
            let base = std::env::var("UAR_LOCAL_A2A_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:3000".to_string());
            format!("{}/a2a/{}", base.trim_end_matches('/'), self.agent_id_or_url)
        }
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
        let url = self.endpoint_url();

        // Determine the input message for the delegated agent.
        let input_text: String = state
            .get::<String>("_agent_input")
            .or_else(|| {
                state.messages.last().and_then(|m| {
                    m.get("content").and_then(|c| c.as_str()).map(String::from)
                })
            })
            .unwrap_or_default();

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
                state.set(
                    &format!("_agent_result_{}", self.id),
                    task_json,
                );
                state.set(
                    &format!("_agent_task_id_{}", self.id),
                    task_id,
                );
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
