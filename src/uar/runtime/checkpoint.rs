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

    /// Restore a [`GraphState`] from this checkpoint, reporting a corrupt
    /// state bag rather than silently substituting an empty one.
    ///
    /// # Errors
    ///
    /// Returns an error when `state` cannot be deserialized into the graph's
    /// data bag. Resuming with an empty state would look like a successful
    /// resume while discarding everything the checkpoint recorded.
    pub fn try_restore_state(&self) -> anyhow::Result<crate::uar::runtime::graph::GraphState> {
        let data = serde_json::from_value(self.state.clone()).map_err(|e| {
            anyhow::anyhow!(
                "checkpoint {} has an unreadable state bag: {e}",
                self.id
            )
        })?;
        Ok(crate::uar::runtime::graph::GraphState {
            data,
            messages: self.messages.clone(),
            iteration: self.iteration,
        })
    }

    /// Restore a [`GraphState`], falling back to an empty data bag.
    ///
    /// Prefer [`Self::try_restore_state`]; this exists for callers that
    /// genuinely tolerate a partial restore.
    #[must_use]
    pub fn restore_state(&self) -> crate::uar::runtime::graph::GraphState {
        let data = serde_json::from_value(self.state.clone()).unwrap_or_default();
        crate::uar::runtime::graph::GraphState {
            data,
            messages: self.messages.clone(),
            iteration: self.iteration,
        }
    }
}

/// Convert a checkpoint's recorded messages into typed conversation history.
///
/// This is what seeds a resumed run, so a malformed entry is an error: a run
/// that silently starts with no history is indistinguishable from a fresh run.
///
/// # Errors
///
/// Returns an error when any recorded message cannot be deserialized.
pub fn history_from_checkpoint(checkpoint: &Checkpoint) -> anyhow::Result<Vec<crate::llm::Message>> {
    checkpoint
        .messages
        .iter()
        .enumerate()
        .map(|(i, value)| {
            serde_json::from_value(value.clone()).map_err(|e| {
                anyhow::anyhow!(
                    "checkpoint {} message {i} is not a valid conversation message: {e}",
                    checkpoint.id
                )
            })
        })
        .collect()
}
