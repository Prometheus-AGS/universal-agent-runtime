//! Shared validation for the three host-owned agent-thread stores.
//! Storage revisions cover every status write; history revisions do not.

use serde::{Deserialize, Serialize};

use crate::uar::runtime::thread::{AgentEdge, AgentThread, AgentThreadStatus, ThreadRecordError};

/// A thread plus its optimistic-concurrency token. Revisions start at zero.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PersistedAgentThread {
    pub thread: AgentThread,
    pub revision: u64,
}

/// Portable store failures. Database transport/transaction errors remain errors
/// too, and never imply that an interrupted write did not commit.
#[derive(Debug, thiserror::Error)]
pub enum AgentThreadStoreError {
    #[error(transparent)]
    InvalidRecord(#[from] ThreadRecordError),
    #[error("agent thread is outside the verified owner or requested tree")]
    ScopeMismatch,
    #[error("agent thread or required parent/root does not exist")]
    NotFound,
    #[error("agent thread ID or canonical path already exists")]
    AlreadyExists,
    #[error("agent thread changed since the supplied storage revision")]
    Conflict,
    #[error("agent thread lineage, artifact, or creation time cannot be changed")]
    ImmutableField,
    #[error("invalid persisted agent thread transition")]
    InvalidTransition,
    #[error("agent thread storage revision is exhausted")]
    RevisionExhausted,
}

impl PersistedAgentThread {
    /// Validate a record returned by storage, including its exact requested owner.
    ///
    /// # Errors
    /// Malformed state, scope mismatches, and unsupported revisions fail closed.
    pub fn validate(&self, owner_id: &str) -> Result<(), AgentThreadStoreError> {
        self.thread.validate()?;
        if owner_id != self.thread.owner_id {
            return Err(AgentThreadStoreError::ScopeMismatch);
        }
        // Both durable stores have signed 64-bit integers. Keep the same bound
        // in memory, rather than allowing a value another provider cannot store.
        if self.revision > i64::MAX as u64 {
            return Err(AgentThreadStoreError::RevisionExhausted);
        }
        if self.thread.status == AgentThreadStatus::Pending && self.thread.run_id.is_some() {
            return Err(AgentThreadStoreError::InvalidTransition);
        }
        Ok(())
    }
}

pub(crate) fn new_root(
    owner_id: &str,
    thread: &AgentThread,
) -> Result<PersistedAgentThread, AgentThreadStoreError> {
    let record = PersistedAgentThread {
        thread: thread.clone(),
        revision: 0,
    };
    record.validate(owner_id)?;
    if thread.parent_thread_id.is_some()
        || thread.run_id.as_ref() != Some(&thread.root_run_id)
        || thread.history_revision != 0
        || thread.status != AgentThreadStatus::Running
    {
        return Err(AgentThreadStoreError::InvalidTransition);
    }
    Ok(record)
}

pub(crate) fn new_child(
    owner_id: &str,
    thread: &AgentThread,
    edge: &AgentEdge,
    parent: &PersistedAgentThread,
    root: &PersistedAgentThread,
) -> Result<PersistedAgentThread, AgentThreadStoreError> {
    let record = PersistedAgentThread {
        thread: thread.clone(),
        revision: 0,
    };
    record.validate(owner_id)?;
    parent.validate(owner_id)?;
    root.validate(owner_id)?;
    if root.thread.parent_thread_id.is_some()
        || root.thread.thread_id != thread.root_thread_id
        || root.thread.root_run_id != thread.root_run_id
    {
        return Err(AgentThreadStoreError::ScopeMismatch);
    }
    if thread.status != AgentThreadStatus::Pending
        || thread.history_revision != 0
        || !matches!(
            parent.thread.status,
            AgentThreadStatus::Running | AgentThreadStatus::Waiting
        )
        || !matches!(
            root.thread.status,
            AgentThreadStatus::Running | AgentThreadStatus::Waiting
        )
    {
        return Err(AgentThreadStoreError::InvalidTransition);
    }
    if &AgentEdge::between(&parent.thread, thread)? != edge {
        return Err(AgentThreadStoreError::ScopeMismatch);
    }
    Ok(record)
}

pub(crate) fn next_record(
    owner_id: &str,
    current: &PersistedAgentThread,
    expected_revision: u64,
    next: &AgentThread,
) -> Result<PersistedAgentThread, AgentThreadStoreError> {
    current.validate(owner_id)?;
    next.validate()?;
    if next.owner_id != owner_id {
        return Err(AgentThreadStoreError::ScopeMismatch);
    }
    if current.revision != expected_revision {
        return Err(AgentThreadStoreError::Conflict);
    }
    let before = &current.thread;
    if before.thread_id != next.thread_id
        || before.root_thread_id != next.root_thread_id
        || before.root_run_id != next.root_run_id
        || before.parent_thread_id != next.parent_thread_id
        || before.canonical_path != next.canonical_path
        || before.artifact_id != next.artifact_id
        || before.created_at != next.created_at
    {
        return Err(AgentThreadStoreError::ImmutableField);
    }
    let remote_transition = before.remote == next.remote
        || (before.remote.is_none()
            && next.remote.is_some()
            && before.parent_thread_id.is_some()
            && matches!(
                before.status,
                AgentThreadStatus::Running | AgentThreadStatus::Waiting
            )
            && next.status == before.status
            && next.run_id == before.run_id
            && next.history_revision == before.history_revision);
    if !remote_transition {
        return Err(AgentThreadStoreError::ImmutableField);
    }
    let valid_transition = if before == next {
        true
    } else {
        match before.status {
            AgentThreadStatus::Pending => match next.status {
                AgentThreadStatus::Pending => next.run_id.is_none(),
                AgentThreadStatus::Running => {
                    next.run_id.is_some() && next.history_revision > before.history_revision
                }
                AgentThreadStatus::Failed | AgentThreadStatus::Cancelled => next.run_id.is_none(),
                _ => false,
            },
            AgentThreadStatus::Running | AgentThreadStatus::Waiting => {
                next.status != AgentThreadStatus::Pending && next.run_id == before.run_id
            }
            AgentThreadStatus::Completed
            | AgentThreadStatus::Failed
            | AgentThreadStatus::Cancelled => {
                next.status == AgentThreadStatus::Running
                    && next.run_id.is_some()
                    && next.run_id != before.run_id
                    && next.history_revision > before.history_revision
            }
        }
    };
    if !valid_transition || next.history_revision < before.history_revision {
        return Err(AgentThreadStoreError::InvalidTransition);
    }
    let revision = current
        .revision
        .checked_add(1)
        .filter(|value| *value <= i64::MAX as u64)
        .ok_or(AgentThreadStoreError::RevisionExhausted)?;
    let record = PersistedAgentThread {
        thread: next.clone(),
        revision,
    };
    record.validate(owner_id)?;
    Ok(record)
}

pub(crate) fn validate_lookup(
    record: &PersistedAgentThread,
    owner_id: &str,
    thread_id: &str,
) -> Result<(), AgentThreadStoreError> {
    record.validate(owner_id)?;
    if record.thread.thread_id != thread_id {
        return Err(AgentThreadStoreError::ScopeMismatch);
    }
    Ok(())
}

pub(crate) fn ordered_threads(
    mut records: Vec<PersistedAgentThread>,
    owner_id: &str,
    root_run_id: &str,
) -> Result<Vec<PersistedAgentThread>, AgentThreadStoreError> {
    for record in &records {
        record.validate(owner_id)?;
        if record.thread.root_run_id != root_run_id {
            return Err(AgentThreadStoreError::ScopeMismatch);
        }
    }
    records.sort_by(|left, right| left.thread.order_key().cmp(&right.thread.order_key()));
    Ok(records)
}

pub(crate) fn ordered_edges(
    mut edges: Vec<AgentEdge>,
    threads: &[PersistedAgentThread],
    owner_id: &str,
    root_run_id: &str,
) -> Result<Vec<AgentEdge>, AgentThreadStoreError> {
    for edge in &edges {
        if edge.owner_id != owner_id || edge.root_run_id != root_run_id {
            return Err(AgentThreadStoreError::ScopeMismatch);
        }
        let parent = threads
            .iter()
            .find(|record| record.thread.thread_id == edge.parent_thread_id)
            .ok_or(AgentThreadStoreError::NotFound)?;
        let child = threads
            .iter()
            .find(|record| record.thread.thread_id == edge.child_thread_id)
            .ok_or(AgentThreadStoreError::NotFound)?;
        if &AgentEdge::between(&parent.thread, &child.thread)? != edge {
            return Err(AgentThreadStoreError::ScopeMismatch);
        }
    }
    edges.sort_by(|left, right| left.order_key().cmp(&right.order_key()));
    Ok(edges)
}
