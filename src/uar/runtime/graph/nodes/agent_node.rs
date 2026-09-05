//! AgentNode — delegates execution to a local or remote agent.
//!
//! # Routing
//!
//! | `agent_id_or_url` | Routing |
//! |-------------------|---------|
//! | Starts with `http://` or `https://` | Remote — calls the URL via A2A JSON-RPC 2.0 |
//! | Any other string | Local — spawns a persisted child through the host's thread service |
//!
//! # State keys read
//! - `_agent_input` — the message text to send (falls back to the last user message in
//!   `state.messages`, or an empty string).
//!
//! # State keys written
//! - `_agent_result_{node_id}` — typed persisted child outcome.
//! - `_agent_thread_id_{node_id}` — persisted child identity.
//! - `_agent_output_{node_id}` — text returned by either a local or remote agent.

use async_trait::async_trait;
use tracing::{debug, warn};

use crate::uar::runtime::graph::types::{GraphContext, GraphNode, GraphState, NodeResult};
use crate::uar::runtime::thread::{
    AgentThreadResult,
    spawn::{AgentSpawnRequest, HistoryForkMode},
};

/// A graph node that delegates to another agent through the run's thread service
/// (local IDs) or A2A (remote URLs).
pub struct AgentNode {
    id: String,
    /// Agent ID (local) or full A2A endpoint URL (remote).
    agent_id_or_url: String,
    history_fork: HistoryForkMode,
}

impl AgentNode {
    /// Create a new `AgentNode`.
    ///
    /// Pass a URL (`https://...`) for a remote A2A agent or an agent ID string
    /// for a registered local artifact executed through the shared turn kernel.
    #[must_use]
    pub fn new(id: impl Into<String>, agent_id_or_url: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            agent_id_or_url: agent_id_or_url.into(),
            history_fork: HistoryForkMode::None,
        }
    }

    /// Select how much parent dialogue the local child inherits. System
    /// instructions and tool traffic are never copied by the thread service.
    #[must_use]
    pub fn with_history_fork(mut self, mode: HistoryForkMode) -> Self {
        self.history_fork = mode;
        self
    }

    fn is_remote(&self) -> bool {
        self.agent_id_or_url.starts_with("http://") || self.agent_id_or_url.starts_with("https://")
    }

    fn output_key(&self) -> String {
        format!("_agent_output_{}", self.id)
    }

    async fn execute_local(
        &self,
        state: GraphState,
        ctx: &GraphContext,
        input_text: &str,
    ) -> NodeResult {
        let Some(delegate) = &ctx.thread_delegate else {
            return NodeResult::Error(
                state,
                "Graph child execution requires a host thread service".into(),
            );
        };
        let outcome = match delegate
            .execute(
                &ctx.run_id,
                state.iteration,
                AgentSpawnRequest {
                    artifact_id: self.agent_id_or_url.clone(),
                    delegated_prompt: input_text.to_owned(),
                    task_name: None,
                    history_fork: self.history_fork,
                },
            )
            .await
        {
            Ok(outcome) => outcome,
            Err(error) => {
                return NodeResult::Error(
                    state,
                    format!("AgentNode '{}' failed: {error}", self.id),
                );
            }
        };
        self.apply_outcome(state, outcome)
    }

    fn apply_outcome(
        &self,
        mut state: GraphState,
        outcome: crate::uar::runtime::thread::control::AgentTurnOutcome,
    ) -> NodeResult {
        state.set(
            &format!("_agent_thread_id_{}", self.id),
            &outcome.agent.thread_id,
        );
        state.set(&format!("_agent_result_{}", self.id), &outcome);
        match outcome.result {
            Some(AgentThreadResult::Completed { output }) if !output.trim().is_empty() => {
                state.set(&self.output_key(), output);
                NodeResult::Continue(state)
            }
            Some(AgentThreadResult::Completed { .. }) => NodeResult::Error(
                state,
                format!("AgentNode '{}' returned empty output", self.id),
            ),
            Some(AgentThreadResult::Failed { code, message }) => NodeResult::Error(
                state,
                format!("AgentNode '{}' child failed ({code}): {message}", self.id),
            ),
            Some(AgentThreadResult::Cancelled) => NodeResult::Error(
                state,
                format!("AgentNode '{}' child was cancelled", self.id),
            ),
            None => NodeResult::Error(state, "Terminal child has no result".into()),
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

    async fn execute(&self, state: GraphState, ctx: &GraphContext) -> NodeResult {
        // Determine the input message for the delegated agent.
        let input_text: String = state
            .get::<String>("_agent_input")
            .or_else(|| {
                state
                    .messages
                    .iter()
                    .rev()
                    .find(|message| {
                        message.get("role").and_then(|role| role.as_str()) == Some("user")
                    })
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

        if let Some(host) = &ctx.tool_host
            && let Err(error) = host.check_remote_compatibility(&ctx.run_id)
        {
            return NodeResult::Error(state, error.to_string());
        }
        if !matches!(
            self.history_fork,
            HistoryForkMode::None | HistoryForkMode::LastTurns(0)
        ) {
            return NodeResult::Error(
                state,
                "Remote A2A delegation does not support parent history forking".into(),
            );
        }
        let url = self.agent_id_or_url.clone();

        debug!(
            run_id = %ctx.run_id,
            node_id = %self.id,
            endpoint = %url,
            is_remote = self.is_remote(),
            "AgentNode delegating"
        );

        let Some(delegate) = &ctx.thread_delegate else {
            return NodeResult::Error(
                state,
                "Remote graph child execution requires a host thread service".into(),
            );
        };
        match delegate
            .execute_remote(&ctx.run_id, state.iteration, url.clone(), input_text)
            .await
        {
            Ok(outcome) => self.apply_outcome(state, outcome),
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
