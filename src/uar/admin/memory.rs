//! Transport-free memory administration over `PersistenceLayer`.
//!
//! Deliberately read-oriented. Memory WRITES go through the memory service,
//! which owns scoping, embedding and auto-capture; an admin surface that wrote
//! rows directly would bypass those invariants.

use std::sync::Arc;

use crate::uar::domain::memory::MemoryMatch;
use crate::uar::persistence::PersistenceLayer;

pub async fn search(
    store: &Arc<dyn PersistenceLayer>,
    agent_id: Option<&str>,
    query_vec: &[f32],
    limit: usize,
    min_score: f32,
) -> anyhow::Result<Vec<MemoryMatch>> {
    store.search_memory(agent_id, query_vec, limit, min_score).await
}
