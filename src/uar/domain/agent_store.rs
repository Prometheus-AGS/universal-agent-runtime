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

/// Validate the catalog invariants shared by every agent write path.
///
/// # Errors
///
/// Returns [`AgentStoreError::Invalid`] when the artifact cannot be admitted to
/// the runtime catalog.
pub fn validate_agent(agent: &AgentArtifact) -> Result<(), AgentStoreError> {
    if agent.id.trim().is_empty() {
        return Err(AgentStoreError::Invalid(
            "agent id must not be empty".to_string(),
        ));
    }
    if agent.kind != "agent" {
        return Err(AgentStoreError::Invalid(
            "agent kind must be 'agent'".to_string(),
        ));
    }
    if agent.version.trim().is_empty() {
        return Err(AgentStoreError::Invalid(
            "agent version must not be empty".to_string(),
        ));
    }
    if agent.metadata.title.trim().is_empty() {
        return Err(AgentStoreError::Invalid(
            "agent title must not be empty".to_string(),
        ));
    }
    if agent.runtime.entry.trim().is_empty() {
        return Err(AgentStoreError::Invalid(
            "agent runtime entry must not be empty".to_string(),
        ));
    }
    Ok(())
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
) -> Result<AgentArtifact, AgentStoreError> {
    if agent.id.is_empty() {
        agent.id = Uuid::new_v4().to_string();
    }
    agent.kind = "agent".to_string();
    if persistence
        .load_agent(&agent.id)
        .await
        .map_err(AgentStoreError::Backend)?
        .is_some()
    {
        return Err(AgentStoreError::Conflict);
    }
    agent = agent.with_catalog_metadata("uar_api");
    validate_agent(&agent)?;
    persistence
        .save_agent(&agent)
        .await
        .context("saving new agent")
        .map_err(AgentStoreError::Backend)?;
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
    agent: AgentArtifact,
) -> Result<AgentArtifact, AgentStoreError> {
    replace_agent_if_revision(persistence, id, agent, None).await
}

/// Replace an agent only when the caller's observed catalog revision remains
/// current. The persistence CAS closes the race between the read and write.
pub async fn replace_agent_if_revision(
    persistence: &dyn PersistenceLayer,
    id: impl Into<String>,
    mut agent: AgentArtifact,
    expected_revision: Option<&str>,
) -> Result<AgentArtifact, AgentStoreError> {
    agent.id = id.into();
    let existing = persistence
        .load_agent(&agent.id)
        .await
        .map_err(AgentStoreError::Backend)?;
    if let (Some(expected), Some(current)) = (expected_revision, existing.as_ref())
        && current
            .clone()
            .with_catalog_metadata("uar_api")
            .content_revision()
            != expected
    {
        return Err(AgentStoreError::Conflict);
    }
    agent = agent.with_catalog_metadata("uar_api");
    validate_agent(&agent)?;
    if let Some(existing) = existing {
        if !persistence
            .save_agent_if_unchanged(&existing, &agent)
            .await
            .map_err(AgentStoreError::Backend)?
        {
            return Err(AgentStoreError::Conflict);
        }
    } else {
        persistence
            .save_agent(&agent)
            .await
            .context("saving agent")
            .map_err(AgentStoreError::Backend)?;
    }
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
) -> Result<(), AgentStoreError> {
    let agent = agent.clone().with_catalog_metadata("embedded");
    validate_agent(&agent)?;
    persistence
        .save_agent(&agent)
        .await
        .context("upserting agent")
        .map_err(AgentStoreError::Backend)
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
    patch_agent_if_revision(persistence, id, patch, None).await
}

/// Merge-patch an agent only when its observed catalog revision is current.
pub async fn patch_agent_if_revision(
    persistence: &dyn PersistenceLayer,
    id: &str,
    patch: &serde_json::Value,
    expected_revision: Option<&str>,
) -> Result<AgentArtifact, AgentStoreError> {
    let existing = persistence
        .load_agent(id)
        .await
        .map_err(AgentStoreError::Backend)?
        .ok_or_else(|| AgentStoreError::NotFound(id.to_string()))?;
    if expected_revision.is_some_and(|expected| {
        existing
            .clone()
            .with_catalog_metadata("uar_api")
            .content_revision()
            != expected
    }) {
        return Err(AgentStoreError::Conflict);
    }

    let mut base = serde_json::to_value(&existing)
        .map_err(|e| AgentStoreError::Backend(anyhow::anyhow!(e)))?;
    json_merge(&mut base, patch);

    let mut agent: AgentArtifact = serde_json::from_value(base)
        .map_err(|e| AgentStoreError::Invalid(format!("invalid agent after merge: {e}")))?;
    agent.id = id.to_string();
    agent = agent.with_catalog_metadata("uar_api");
    validate_agent(&agent)?;

    if !persistence
        .save_agent_if_unchanged(&existing, &agent)
        .await
        .map_err(AgentStoreError::Backend)?
    {
        return Err(AgentStoreError::Conflict);
    }
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
    persistence
        .list_agents()
        .await
        .context("listing agents")
        .map(|agents| {
            agents
                .into_iter()
                .map(|agent| agent.with_catalog_metadata("uar_api"))
                .collect()
        })
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
    persistence
        .load_agent(id)
        .await
        .context("loading agent")
        .map(|agent| agent.map(|agent| agent.with_catalog_metadata("uar_api")))
}

/// Resolve an explicitly selected runtime agent without fallback substitution.
///
/// Persisted definitions take precedence over built-ins so the specialist IDs
/// retain their documented operator-overridable behavior.
///
/// # Errors
///
/// Returns [`AgentStoreError::NotFound`] for an unknown explicit id and
/// [`AgentStoreError::Backend`] when catalog storage cannot be read.
pub async fn resolve_registered_agent(
    persistence: Option<&dyn PersistenceLayer>,
    agent_id: &str,
) -> Result<AgentArtifact, AgentStoreError> {
    if agent_id.trim().is_empty() {
        return Err(AgentStoreError::Invalid(
            "agent id must not be empty".to_string(),
        ));
    }
    if let Some(persistence) = persistence
        && let Some(agent) = persistence
            .load_agent(agent_id)
            .await
            .map_err(AgentStoreError::Backend)?
    {
        let agent = agent.with_catalog_metadata("uar_api");
        validate_agent(&agent)?;
        return Ok(agent);
    }
    let agent = match agent_id {
        "default-agent" => crate::uar::defaults::default_agent(),
        "orchestrator-agent" => crate::uar::defaults::orchestrator_agent(),
        "general-purpose" => crate::uar::defaults::general_purpose_agent(),
        "rust-reviewer" => crate::uar::defaults::rust_reviewer_agent(),
        "compiler-agent" => crate::uar::defaults::compiler_agent(),
        _ => return Err(AgentStoreError::NotFound(agent_id.to_string())),
    };
    let agent = agent.with_catalog_metadata("builtin");
    validate_agent(&agent)?;
    Ok(agent)
}

/// List the complete local runtime catalog, surfacing storage failures and
/// reconciling built-ins that have not yet been seeded.
///
/// # Errors
///
/// Returns a typed backend or validation error instead of an empty catalog.
pub async fn list_registered_agents(
    persistence: Option<&dyn PersistenceLayer>,
) -> Result<Vec<AgentArtifact>, AgentStoreError> {
    let mut agents = match persistence {
        Some(persistence) => persistence
            .list_agents()
            .await
            .map_err(AgentStoreError::Backend)?,
        None => Vec::new(),
    };
    agents = agents
        .into_iter()
        .map(|agent| agent.with_catalog_metadata("uar_api"))
        .collect();
    for agent in &agents {
        validate_agent(agent)?;
    }
    for builtin in [
        crate::uar::defaults::default_agent(),
        crate::uar::defaults::orchestrator_agent(),
        crate::uar::defaults::general_purpose_agent(),
        crate::uar::defaults::rust_reviewer_agent(),
        crate::uar::defaults::compiler_agent(),
    ] {
        if !agents.iter().any(|agent| agent.id == builtin.id) {
            agents.push(builtin.with_catalog_metadata("builtin"));
        }
    }
    Ok(agents)
}

/// Typed failures from the mutating agent-store operations so callers can map
/// them to the right transport response (e.g. 404 vs 403 vs 500).
#[derive(Debug, thiserror::Error)]
pub enum AgentStoreError {
    /// Another edit committed after the patch read its baseline.
    #[error("Agent changed; reload before saving")]
    Conflict,
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
