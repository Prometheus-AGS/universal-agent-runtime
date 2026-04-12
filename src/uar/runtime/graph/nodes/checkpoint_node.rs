//! Checkpoint graph node — records execution progress for resumability.
//!
//! In this V1 implementation the node logs the checkpoint and stores it in
//! state.  Full persistence via SurrealDB is wired in Phase 4 (Task 4.x).

use async_trait::async_trait;
use tracing::info;

use crate::uar::runtime::graph::types::{GraphContext, GraphNode, GraphState, NodeResult};

/// A graph node that marks a checkpoint in the execution flow.
///
/// # State keys written
/// - `_last_checkpoint` — the label of the most recently hit checkpoint.
/// - `_checkpoint_iteration_{label}` — the iteration count when this checkpoint was hit.
pub struct CheckpointNode {
    id: String,
    /// Human-readable label for this checkpoint (used in logs and state keys).
    label: String,
}

impl CheckpointNode {
    /// Create a checkpoint node with the given unique ID and label.
    #[must_use]
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
        }
    }
}

impl std::fmt::Debug for CheckpointNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CheckpointNode")
            .field("id", &self.id)
            .field("label", &self.label)
            .finish()
    }
}

#[async_trait]
impl GraphNode for CheckpointNode {
    fn id(&self) -> &str {
        &self.id
    }

    async fn execute(&self, mut state: GraphState, ctx: &GraphContext) -> NodeResult {
        info!(
            node_id = %self.id,
            label = %self.label,
            run_id = %ctx.run_id,
            iteration = state.iteration,
            "Graph checkpoint reached"
        );

        // Record checkpoint metadata in state
        state.set("_last_checkpoint", &self.label);
        state.set(
            &format!("_checkpoint_iteration_{}", self.label),
            state.iteration,
        );

        // Phase 4 will add: persist checkpoint to SurrealDB here.

        NodeResult::Continue(state)
    }
}
