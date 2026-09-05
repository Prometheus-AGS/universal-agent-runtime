//! Shared agent-thread contracts. Persistence and execution belong to the
//! trusted host; these records and read handles do not grant tool authority.

pub(crate) mod actor_host;
pub(crate) mod approvals;
pub mod artifacts;
pub mod control;
pub mod events;
pub(crate) mod execution;
pub(crate) mod graph_host;
pub mod kernel;
pub mod limits;
pub mod messages;
pub mod policy_intersection;
pub mod service;
pub mod spawn;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::watch;
use uuid::Uuid;

/// State of the most recent turn on an agent thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentThreadStatus {
    Pending,
    Running,
    Waiting,
    Completed,
    Failed,
    Cancelled,
}

impl AgentThreadStatus {
    /// A terminal turn releases its concurrent-child slot, not its lifetime count.
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }
}

/// Durable turn result. Lifecycle events must project the variant only, never
/// serialize this content-bearing result or the entire thread record.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AgentThreadResult {
    Completed { output: String },
    Failed { code: String, message: String },
    Cancelled,
}

impl AgentThreadResult {
    pub const fn status(&self) -> AgentThreadStatus {
        match self {
            Self::Completed { .. } => AgentThreadStatus::Completed,
            Self::Failed { .. } => AgentThreadStatus::Failed,
            Self::Cancelled => AgentThreadStatus::Cancelled,
        }
    }
}

impl std::fmt::Debug for AgentThreadResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AgentThreadResult")
            .field("status", &self.status())
            .field("content", &"<redacted>")
            .finish()
    }
}

/// Persisted lineage and latest-turn state. Owner/root identity comes from the
/// root host context and is inherited unchanged by every child.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentThread {
    pub thread_id: String,
    pub owner_id: String,
    pub root_thread_id: String,
    pub root_run_id: String,
    pub parent_thread_id: Option<String>,
    pub canonical_path: String,
    pub artifact_id: String,
    /// Exact accepted peer identity for a governed remote child. Persisted as
    /// soon as the first send receipt arrives so later turns and recovery do
    /// not create or guess another remote task.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote: Option<RemoteThreadBinding>,
    pub run_id: Option<String>,
    pub status: AgentThreadStatus,
    pub history_revision: u64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub result: Option<AgentThreadResult>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteThreadBinding {
    pub target_instance_id: String,
    pub endpoint: String,
    pub task_id: String,
    pub context_id: String,
    pub contract_digest: String,
}

impl RemoteThreadBinding {
    pub fn validate(&self) -> Result<(), ThreadRecordError> {
        for (field, value) in [
            (
                "remote.target_instance_id",
                self.target_instance_id.as_str(),
            ),
            ("remote.endpoint", self.endpoint.as_str()),
            ("remote.task_id", self.task_id.as_str()),
            ("remote.context_id", self.context_id.as_str()),
            ("remote.contract_digest", self.contract_digest.as_str()),
        ] {
            if value.trim().is_empty()
                || value != value.trim()
                || value.chars().any(char::is_control)
            {
                return Err(ThreadRecordError::EmptyField { field });
            }
        }
        Ok(())
    }
}

/// Parent-to-child edge stored atomically with the new thread by the host.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentEdge {
    pub owner_id: String,
    pub root_thread_id: String,
    pub root_run_id: String,
    pub parent_thread_id: String,
    pub child_thread_id: String,
    pub canonical_path: String,
    pub artifact_id: String,
    pub created_at: DateTime<Utc>,
}

/// Invalid stored or proposed lineage is not repaired by guessing an owner.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ThreadRecordError {
    #[error("agent thread field '{field}' must not be empty")]
    EmptyField { field: &'static str },
    #[error("agent task name must be one nonempty path segment")]
    InvalidName,
    #[error("invalid agent canonical path")]
    InvalidPath,
    #[error("agent thread owner, root, parent, or canonical path does not match its lineage")]
    LineageMismatch,
    #[error("agent thread status and result disagree")]
    InvalidStatus,
    #[error("agent thread already has an active turn")]
    ActiveTurn,
    #[error("agent thread history revision is exhausted")]
    RevisionExhausted,
}

/// Validate a single canonical task-path segment without excluding Unicode names.
pub fn validate_task_name(name: &str) -> Result<(), ThreadRecordError> {
    if name.trim().is_empty()
        || matches!(name, "." | "..")
        || name
            .chars()
            .any(|ch| ch == '/' || ch == '\\' || ch.is_control())
    {
        return Err(ThreadRecordError::InvalidName);
    }
    Ok(())
}

impl AgentThread {
    /// Create the root record for an already authorized root run.
    ///
    /// # Errors
    /// Returns an error for missing owner, artifact, or root-run identity.
    pub fn root(
        owner_id: String,
        artifact_id: String,
        root_run_id: String,
    ) -> Result<Self, ThreadRecordError> {
        let thread_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let thread = Self {
            root_thread_id: thread_id.clone(),
            thread_id,
            owner_id,
            run_id: Some(root_run_id.clone()),
            root_run_id,
            parent_thread_id: None,
            canonical_path: "/root".into(),
            artifact_id,
            remote: None,
            status: AgentThreadStatus::Running,
            history_revision: 0,
            created_at: now,
            updated_at: now,
            result: None,
        };
        thread.validate()?;
        Ok(thread)
    }

    /// Allocate a pending child with inherited identity, without starting a run.
    ///
    /// # Errors
    /// Returns invalid-parent, artifact, or task-name errors. Admission limits
    /// and policy intersection remain separate mandatory host gates.
    pub fn child(
        parent: &Self,
        artifact_id: String,
        name: Option<&str>,
    ) -> Result<Self, ThreadRecordError> {
        parent.validate()?;
        let thread_id = Uuid::new_v4().to_string();
        let name = name.unwrap_or(&thread_id);
        validate_task_name(name)?;
        let canonical_path = format!("{}/{name}", parent.canonical_path);
        let now = Utc::now();
        let thread = Self {
            thread_id,
            owner_id: parent.owner_id.clone(),
            root_thread_id: parent.root_thread_id.clone(),
            root_run_id: parent.root_run_id.clone(),
            parent_thread_id: Some(parent.thread_id.clone()),
            canonical_path,
            artifact_id,
            remote: None,
            run_id: None,
            status: AgentThreadStatus::Pending,
            history_revision: 0,
            created_at: now,
            updated_at: now,
            result: None,
        };
        thread.validate()?;
        Ok(thread)
    }

    /// Validate the record independently of its persistence provider.
    ///
    /// # Errors
    /// Returns malformed identity/path/status errors; parent existence and exact
    /// lineage are additionally checked using [`AgentEdge::between`].
    pub fn validate(&self) -> Result<(), ThreadRecordError> {
        for (field, value) in [
            ("thread_id", &self.thread_id),
            ("owner_id", &self.owner_id),
            ("root_thread_id", &self.root_thread_id),
            ("root_run_id", &self.root_run_id),
            ("artifact_id", &self.artifact_id),
        ] {
            if value.trim().is_empty() {
                return Err(ThreadRecordError::EmptyField { field });
            }
        }
        if self.run_id.as_ref().is_some_and(|id| id.trim().is_empty()) {
            return Err(ThreadRecordError::EmptyField { field: "run_id" });
        }
        if let Some(remote) = &self.remote {
            remote.validate()?;
            if self.parent_thread_id.is_none() {
                return Err(ThreadRecordError::LineageMismatch);
            }
        }
        match &self.parent_thread_id {
            None if self.thread_id == self.root_thread_id && self.canonical_path == "/root" => {}
            Some(parent)
                if !parent.trim().is_empty()
                    && parent != &self.thread_id
                    && self.thread_id != self.root_thread_id =>
            {
                let Some(relative) = self.canonical_path.strip_prefix("/root/") else {
                    return Err(ThreadRecordError::InvalidPath);
                };
                for segment in relative.split('/') {
                    validate_task_name(segment)?;
                }
            }
            _ => return Err(ThreadRecordError::LineageMismatch),
        }
        if self.status.is_terminal() {
            if (self.status == AgentThreadStatus::Completed && self.run_id.is_none())
                || self.result.as_ref().map(AgentThreadResult::status) != Some(self.status)
            {
                return Err(ThreadRecordError::InvalidStatus);
            }
        } else if self.result.is_some()
            || (self.status != AgentThreadStatus::Pending && self.run_id.is_none())
        {
            return Err(ThreadRecordError::InvalidStatus);
        }
        Ok(())
    }

    /// Root depth is zero; each canonical path segment adds one level.
    pub fn depth(&self) -> usize {
        self.canonical_path.split('/').count().saturating_sub(2)
    }

    /// Stable provider-independent list ordering.
    pub fn order_key(&self) -> (&str, &str) {
        (&self.canonical_path, &self.thread_id)
    }

    /// Begin another authorized turn without changing root or owner identity.
    ///
    /// # Errors
    /// Returns an error for an already active turn, empty run id, or exhausted revision.
    pub fn begin_turn(&mut self, run_id: String) -> Result<(), ThreadRecordError> {
        self.validate()?;
        if matches!(
            self.status,
            AgentThreadStatus::Running | AgentThreadStatus::Waiting
        ) {
            return Err(ThreadRecordError::ActiveTurn);
        }
        if run_id.trim().is_empty() {
            return Err(ThreadRecordError::EmptyField { field: "run_id" });
        }
        let revision = self
            .history_revision
            .checked_add(1)
            .ok_or(ThreadRecordError::RevisionExhausted)?;
        self.run_id = Some(run_id);
        self.history_revision = revision;
        self.status = AgentThreadStatus::Running;
        self.result = None;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Bind the first authenticated peer receipt to this persisted child.
    /// Repeating the exact receipt is idempotent; switching peers/tasks fails.
    pub fn bind_remote(&mut self, binding: RemoteThreadBinding) -> Result<(), ThreadRecordError> {
        self.validate()?;
        binding.validate()?;
        if self.parent_thread_id.is_none() {
            return Err(ThreadRecordError::LineageMismatch);
        }
        match &self.remote {
            Some(existing) if existing == &binding => Ok(()),
            Some(_) => Err(ThreadRecordError::LineageMismatch),
            None => {
                self.remote = Some(binding);
                self.updated_at = Utc::now();
                Ok(())
            }
        }
    }

    /// Complete the current turn in a proposed record; the host persists it
    /// before publishing lifecycle notifications.
    ///
    /// # Errors
    /// Returns an error if no live turn can receive this outcome.
    pub fn finish_turn(&mut self, result: AgentThreadResult) -> Result<(), ThreadRecordError> {
        self.validate()?;
        // Preflight failure or cancellation can terminate a persisted pending
        // child before the host has started a model run. It cannot complete it.
        let pending_failure = self.status == AgentThreadStatus::Pending
            && result.status() != AgentThreadStatus::Completed;
        if !pending_failure
            && !matches!(
                self.status,
                AgentThreadStatus::Running | AgentThreadStatus::Waiting
            )
        {
            return Err(ThreadRecordError::InvalidStatus);
        }
        self.status = result.status();
        self.result = Some(result);
        self.updated_at = Utc::now();
        Ok(())
    }
}

impl AgentEdge {
    /// Derive one edge only from a parent and a matching child record.
    ///
    /// # Errors
    /// Rejects malformed records, cross-owner/root links, and false parent paths.
    pub fn between(parent: &AgentThread, child: &AgentThread) -> Result<Self, ThreadRecordError> {
        parent.validate()?;
        child.validate()?;
        if parent.owner_id != child.owner_id
            || parent.root_thread_id != child.root_thread_id
            || parent.root_run_id != child.root_run_id
            || child.parent_thread_id.as_deref() != Some(parent.thread_id.as_str())
            || child
                .canonical_path
                .rsplit_once('/')
                .map(|(prefix, _)| prefix)
                != Some(parent.canonical_path.as_str())
        {
            return Err(ThreadRecordError::LineageMismatch);
        }
        Ok(Self {
            owner_id: child.owner_id.clone(),
            root_thread_id: child.root_thread_id.clone(),
            root_run_id: child.root_run_id.clone(),
            parent_thread_id: parent.thread_id.clone(),
            child_thread_id: child.thread_id.clone(),
            canonical_path: child.canonical_path.clone(),
            artifact_id: child.artifact_id.clone(),
            created_at: child.created_at,
        })
    }

    pub fn order_key(&self) -> (&str, &str) {
        (&self.canonical_path, &self.child_thread_id)
    }
}

/// Read-only subscription to a host-published thread record. Dropping a waiter
/// never cancels execution; cancellation is a separate governed host operation.
#[derive(Clone)]
pub struct AgentHandle {
    identity: (String, String, String, String),
    state: watch::Receiver<AgentThread>,
}

impl std::fmt::Debug for AgentHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AgentHandle")
            .field("identity", &self.identity)
            .finish_non_exhaustive()
    }
}

/// A closed publisher is not evidence that a thread completed successfully.
#[derive(Debug, thiserror::Error)]
pub enum AgentWaitError {
    #[error(transparent)]
    InvalidRecord(#[from] ThreadRecordError),
    #[error("thread '{thread_id}' publisher closed before a terminal state")]
    PublisherClosed { thread_id: String },
}

impl AgentHandle {
    /// Create a read handle and a publisher retained exclusively by the host.
    ///
    /// # Errors
    /// Returns an invalid-record error before creating the subscription.
    pub fn channel(
        thread: AgentThread,
    ) -> Result<(Self, watch::Sender<AgentThread>), ThreadRecordError> {
        thread.validate()?;
        let identity = (
            thread.owner_id.clone(),
            thread.root_thread_id.clone(),
            thread.root_run_id.clone(),
            thread.thread_id.clone(),
        );
        let (sender, state) = watch::channel(thread);
        Ok((Self { identity, state }, sender))
    }

    /// Return the latest state after checking it still belongs to this handle.
    ///
    /// # Errors
    /// Rejects invalid records or an accidental cross-thread publication.
    pub fn snapshot(&self) -> Result<AgentThread, ThreadRecordError> {
        let snapshot = self.state.borrow().clone();
        self.validate_identity(&snapshot)?;
        Ok(snapshot)
    }

    /// Wait for the latest observed turn to be terminal; this is a state
    /// subscription, not a log of earlier turns that have already been resumed.
    ///
    /// # Errors
    /// Returns a malformed publication or premature channel-close error.
    pub async fn wait_until_terminal(&self) -> Result<AgentThread, AgentWaitError> {
        let mut receiver = self.state.clone();
        loop {
            let snapshot = receiver.borrow_and_update().clone();
            self.validate_identity(&snapshot)?;
            if snapshot.status.is_terminal() {
                return Ok(snapshot);
            }
            receiver
                .changed()
                .await
                .map_err(|_| AgentWaitError::PublisherClosed {
                    thread_id: self.identity.3.clone(),
                })?;
        }
    }

    fn validate_identity(&self, thread: &AgentThread) -> Result<(), ThreadRecordError> {
        thread.validate()?;
        if (
            &thread.owner_id,
            &thread.root_thread_id,
            &thread.root_run_id,
            &thread.thread_id,
        ) != (
            &self.identity.0,
            &self.identity.1,
            &self.identity.2,
            &self.identity.3,
        ) {
            return Err(ThreadRecordError::LineageMismatch);
        }
        Ok(())
    }
}
