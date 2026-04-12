//! State checkpoint types for graph-based run persistence.
//!
//! A [`Checkpoint`] is a snapshot of [`GraphState`] at a named node boundary.
//! Checkpoints allow runs to be paused and resumed from any saved point.

use serde::{Deserialize, Serialize};

/// A persisted snapshot of graph execution state at a named boundary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    /// Unique checkpoint ID (UUID v4).
    pub id: String,
    /// The run that produced this checkpoint.
    pub run_id: String,
    /// The conversation/session thread ID.
    pub thread_id: String,
    /// The graph node that recorded this checkpoint.
    pub node_id: String,
    /// Graph iteration count at the time of the checkpoint.
    pub iteration: u32,
    /// Serialised `GraphState.data` bag.
    pub state: serde_json::Value,
    /// Serialised `GraphState.messages` at this point in execution.
    pub messages: Vec<serde_json::Value>,
    /// ISO 8601 UTC timestamp.
    pub created_at: String,
}

impl Checkpoint {
    /// Create a new checkpoint from the current graph state.
    pub fn new(
        run_id: impl Into<String>,
        thread_id: impl Into<String>,
        node_id: impl Into<String>,
        state: &crate::uar::runtime::graph::GraphState,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            run_id: run_id.into(),
            thread_id: thread_id.into(),
            node_id: node_id.into(),
            iteration: state.iteration,
            state: serde_json::to_value(&state.data).unwrap_or_default(),
            messages: state.messages.clone(),
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Restore a [`GraphState`] from this checkpoint.
    pub fn restore_state(&self) -> crate::uar::runtime::graph::GraphState {
        let data = serde_json::from_value(self.state.clone()).unwrap_or_default();
        crate::uar::runtime::graph::GraphState {
            data,
            messages: self.messages.clone(),
            iteration: self.iteration,
        }
    }
}
