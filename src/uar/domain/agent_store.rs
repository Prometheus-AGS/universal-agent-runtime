//! Transport-free agent-definition CRUD over a persistence layer.
//!
//! These free functions hold the agent-store behavior shared by the HTTP service
//! handlers (`api/discovery.rs`) and the embedded SDK admin surface, so both edit
//! agent definitions through one code path. They own only the storage-shaping
//! rules (id assignment, the `agent` kind marker, RFC 7396 merge on patch, and the
//! built-in-agent delete guard); transport concerns (status codes, JSON framing)
//! stay in the callers.

use anyhow::{Context, Result};
use uuid::Uuid;

use crate::uar::domain::artifact::AgentArtifact;
use crate::uar::persistence::PersistenceLayer;

/// Built-in agents that must never be deleted through the store.
#[must_use]
pub fn is_protected_agent_id(id: &str) -> bool {
    matches!(id, "default-agent" | "orchestrator-agent")
}

/// Create a new agent, assigning an id when absent and marking its kind.
///
/// Returns the stored artifact (with any generated id) so callers can echo it
/// back.
///
/// # Errors
///
/// Returns an error if the persistence write fails.
pub async fn create_agent(
    persistence: &dyn PersistenceLayer,
    mut agent: AgentArtifact,
) -> Result<AgentArtifact> {
    if agent.id.is_empty() {
        agent.id = Uuid::new_v4().to_string();
    }
    agent.kind = "agent".to_string();
    persistence
        .save_agent(&agent)
        .await
        .context("saving new agent")?;
    Ok(agent)
}

/// Full-replacement upsert of an agent under a fixed id.
///
/// # Errors
///
/// Returns an error if the persistence write fails.
pub async fn replace_agent(
    persistence: &dyn PersistenceLayer,
    id: impl Into<String>,
    mut agent: AgentArtifact,
) -> Result<AgentArtifact> {
    agent.id = id.into();
    persistence
        .save_agent(&agent)
        .await
        .context("saving agent")?;
    Ok(agent)
}

/// Upsert an agent artifact as-is (used by the embedded admin surface).
///
/// Unlike [`create_agent`] this preserves the caller-provided id and kind, so a
/// host can round-trip an artifact it already owns without mutation.
///
/// # Errors
///
/// Returns an error if the persistence write fails.
pub async fn upsert_agent(
    persistence: &dyn PersistenceLayer,
    agent: &AgentArtifact,
) -> Result<()> {
    persistence
        .save_agent(agent)
        .await
        .context("upserting agent")
}

/// Apply an RFC 7396 JSON Merge Patch to an existing agent and persist it.
///
/// # Errors
///
/// Returns [`AgentStoreError::NotFound`] when the agent does not exist, or a
/// generic error if the merged value is not a valid agent or the write fails.
pub async fn patch_agent(
    persistence: &dyn PersistenceLayer,
    id: &str,
    patch: &serde_json::Value,
) -> Result<AgentArtifact, AgentStoreError> {
    let existing = persistence
        .load_agent(id)
        .await
        .map_err(AgentStoreError::Backend)?
        .ok_or_else(|| AgentStoreError::NotFound(id.to_string()))?;

    let mut base = serde_json::to_value(&existing)
        .map_err(|e| AgentStoreError::Backend(anyhow::anyhow!(e)))?;
    json_merge(&mut base, patch);

    let mut agent: AgentArtifact = serde_json::from_value(base)
        .map_err(|e| AgentStoreError::Invalid(format!("invalid agent after merge: {e}")))?;
    agent.id = id.to_string();

    persistence
        .save_agent(&agent)
        .await
        .map_err(AgentStoreError::Backend)?;
    Ok(agent)
}

/// Delete an agent by id, refusing to remove built-in agents.
///
/// # Errors
///
/// Returns [`AgentStoreError::Protected`] for built-in agents, or a backend error
/// if the delete fails.
pub async fn delete_agent(
    persistence: &dyn PersistenceLayer,
    id: &str,
) -> Result<(), AgentStoreError> {
    if is_protected_agent_id(id) {
        return Err(AgentStoreError::Protected(id.to_string()));
    }
    persistence
        .delete_agent(id)
        .await
        .map_err(AgentStoreError::Backend)
}

/// List persisted agent definitions.
///
/// # Errors
///
/// Returns an error if the persistence read fails.
pub async fn list_agents(persistence: &dyn PersistenceLayer) -> Result<Vec<AgentArtifact>> {
    persistence.list_agents().await.context("listing agents")
}

/// Load a single agent definition by id.
///
/// # Errors
///
/// Returns an error if the persistence read fails.
pub async fn get_agent(
    persistence: &dyn PersistenceLayer,
    id: &str,
) -> Result<Option<AgentArtifact>> {
    persistence.load_agent(id).await.context("loading agent")
}

/// Typed failures from the mutating agent-store operations so callers can map
/// them to the right transport response (e.g. 404 vs 403 vs 500).
#[derive(Debug, thiserror::Error)]
pub enum AgentStoreError {
    /// The requested agent id does not exist.
    #[error("agent '{0}' not found")]
    NotFound(String),
    /// The agent is built in and cannot be deleted.
    #[error("agent '{0}' is built in and cannot be deleted")]
    Protected(String),
    /// The merged agent value was not a valid artifact.
    #[error("{0}")]
    Invalid(String),
    /// An underlying persistence error.
    #[error(transparent)]
    Backend(anyhow::Error),
}

/// RFC 7396 JSON Merge Patch: recursively merge `patch` into `target`.
pub fn json_merge(target: &mut serde_json::Value, patch: &serde_json::Value) {
    if let serde_json::Value::Object(patch_map) = patch {
        if !target.is_object() {
            *target = serde_json::Value::Object(serde_json::Map::new());
        }
        if let Some(target_map) = target.as_object_mut() {
            for (key, value) in patch_map {
                if value.is_null() {
                    target_map.remove(key);
                } else if value.is_object() {
                    let entry = target_map
                        .entry(key.clone())
                        .or_insert(serde_json::Value::Object(serde_json::Map::new()));
                    json_merge(entry, value);
                } else {
                    target_map.insert(key.clone(), value.clone());
                }
            }
        }
    } else {
        *target = patch.clone();
    }
}
