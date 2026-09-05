//! Pure lifecycle projection of confirmed persistence results. The trusted host
//! calls this after committing a transition, then publishes through its run sink.

use crate::uar::domain::events::{
    AgentLifecycle, AgentLifecycleOutcome, AgentLifecycleStatus, NormalizedEvent,
};
use crate::uar::persistence::agent_threads::{
    AgentThreadStoreError, PersistedAgentThread, next_record,
};

use super::{AgentEdge, AgentThreadStatus};

impl PersistedAgentThread {
    /// Project a confirmed child insertion or update without exposing content.
    /// The parent is the host-captured parent turn associated with this child
    /// turn, not a later resumed parent run. Root records have no child event.
    ///
    /// Repeated state writes do not create another start/completion event. The
    /// source lifecycle ID and timestamp come from the stored revision, so a
    /// retry of this projection yields the same event rather than a new identity.
    ///
    /// # Errors
    /// Rejects foreign lineage, skipped/reordered revisions, or an invalid
    /// transition. The caller remains responsible for proof of committed writes.
    pub fn lifecycle_event(
        &self,
        owner_id: &str,
        parent: &PersistedAgentThread,
        previous: Option<&PersistedAgentThread>,
    ) -> Result<Option<NormalizedEvent>, AgentThreadStoreError> {
        self.validate(owner_id)?;
        parent.validate(owner_id)?;
        let thread = &self.thread;
        AgentEdge::between(&parent.thread, thread)?;
        if let Some(previous) = previous {
            let expected = next_record(owner_id, previous, previous.revision, thread)?;
            if &expected != self {
                return Err(AgentThreadStoreError::Conflict);
            }
            if previous.thread.status == thread.status && previous.thread.run_id == thread.run_id {
                return Ok(None);
            }
        } else if self.revision != 0 || thread.status != AgentThreadStatus::Pending {
            // A restored snapshot is not evidence of an unobserved transition.
            return Err(AgentThreadStoreError::InvalidTransition);
        }

        let (status, terminal_outcome) = match thread.status {
            AgentThreadStatus::Pending => (AgentLifecycleStatus::Pending, None),
            AgentThreadStatus::Running => (AgentLifecycleStatus::Running, None),
            AgentThreadStatus::Waiting => (AgentLifecycleStatus::Waiting, None),
            AgentThreadStatus::Completed => (
                AgentLifecycleStatus::Completed,
                Some(AgentLifecycleOutcome::Completed),
            ),
            AgentThreadStatus::Failed => (
                AgentLifecycleStatus::Failed,
                Some(AgentLifecycleOutcome::Failed),
            ),
            AgentThreadStatus::Cancelled => (
                AgentLifecycleStatus::Cancelled,
                Some(AgentLifecycleOutcome::Cancelled),
            ),
        };
        let lifecycle = AgentLifecycle {
            root_thread_id: thread.root_thread_id.clone(),
            parent_thread_id: parent.thread.thread_id.clone(),
            child_thread_id: thread.thread_id.clone(),
            parent_run_id: parent.thread.run_id.clone(),
            child_run_id: thread.run_id.clone(),
            canonical_path: thread.canonical_path.clone(),
            artifact_id: thread.artifact_id.clone(),
            status,
            terminal_outcome,
            revision: self.revision,
            lifecycle_id: format!("agent-thread:{}:{}", thread.thread_id, self.revision),
            timestamp: thread.updated_at,
        };
        let run_id = thread.root_run_id.clone();
        let event = match thread.status {
            AgentThreadStatus::Running
                if previous.is_some_and(|record| record.thread.run_id != thread.run_id) =>
            {
                NormalizedEvent::AgentThreadStarted { run_id, lifecycle }
            }
            AgentThreadStatus::Pending
            | AgentThreadStatus::Running
            | AgentThreadStatus::Waiting => {
                NormalizedEvent::AgentThreadUpdated { run_id, lifecycle }
            }
            AgentThreadStatus::Completed => {
                NormalizedEvent::AgentThreadFinished { run_id, lifecycle }
            }
            AgentThreadStatus::Failed | AgentThreadStatus::Cancelled => {
                NormalizedEvent::AgentThreadError { run_id, lifecycle }
            }
        };
        Ok(Some(event))
    }
}
