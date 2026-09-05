//! Tree-wide host admission. One lock checks and reserves every limit together;
//! a reservation that never reaches persistence is rolled back on drop. Once
//! persistence starts, an unknown write outcome retains lifetime capacity.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

use super::{AgentEdge, AgentThread, AgentThreadStatus, ThreadRecordError};

/// Root-run ceilings. Child requests may only reduce them; zero disables admission.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AgentTreeLimits {
    pub max_concurrent_children: usize,
    pub max_depth: usize,
    pub max_total_children: usize,
}

impl Default for AgentTreeLimits {
    fn default() -> Self {
        Self {
            max_concurrent_children: 4,
            max_depth: 3,
            max_total_children: 16,
        }
    }
}

impl AgentTreeLimits {
    /// Intersect limits without letting a child increase a root ceiling.
    pub fn narrowed_by(self, requested: Self) -> Self {
        Self {
            max_concurrent_children: self
                .max_concurrent_children
                .min(requested.max_concurrent_children),
            max_depth: self.max_depth.min(requested.max_depth),
            max_total_children: self.max_total_children.min(requested.max_total_children),
        }
    }
}

/// Typed admission refusals; none silently queue or widen a limit.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AgentLimitError {
    #[error(transparent)]
    InvalidThread(#[from] ThreadRecordError),
    #[error("maximum concurrent children reached ({limit})")]
    ConcurrentChildren { limit: usize },
    #[error("maximum child depth exceeded ({limit})")]
    Depth { limit: usize },
    #[error("maximum total children reached ({limit})")]
    TotalChildren { limit: usize },
    #[error("thread '{thread_id}' is not a committed member of this root tree")]
    UnknownThread { thread_id: String },
    #[error("child identity or canonical path already exists in this root tree")]
    DuplicateChild,
    #[error("thread '{thread_id}' already has a concurrent turn")]
    AlreadyRunning { thread_id: String },
    #[error("tree admission state is unavailable")]
    Unavailable,
    #[error("child reservation has not entered the persistence boundary")]
    PersistenceNotStarted,
}

#[derive(Debug)]
struct ChildEntry {
    thread: AgentThread,
    committed: bool,
}

#[derive(Debug)]
struct AdmissionState {
    root: AgentThread,
    children: BTreeMap<String, ChildEntry>,
    active: BTreeSet<String>,
}

/// Inspectable counters include in-flight reservations to prevent oversubscription.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentTreeCounts {
    pub total_children: usize,
    pub concurrent_children: usize,
}

/// Cloneable host-side admission gate shared by all adapters in a root run.
#[derive(Debug, Clone)]
pub struct AgentTreeAdmission {
    limits: AgentTreeLimits,
    state: Arc<Mutex<AdmissionState>>,
}

impl AgentTreeAdmission {
    /// Create a gate for a root record, not for an arbitrary descendant.
    ///
    /// # Errors
    /// Rejects malformed records or a child supplied as the root.
    pub fn new(root: AgentThread, limits: AgentTreeLimits) -> Result<Self, AgentLimitError> {
        root.validate()?;
        if root.parent_thread_id.is_some() {
            return Err(ThreadRecordError::LineageMismatch.into());
        }
        Ok(Self {
            // Four concurrent, depth three, and sixteen total are hard root
            // ceilings. Caller configuration can reduce, never raise them.
            limits: AgentTreeLimits::default().narrowed_by(limits),
            state: Arc::new(Mutex::new(AdmissionState {
                root,
                children: BTreeMap::new(),
                active: BTreeSet::new(),
            })),
        })
    }

    /// Reserve a new child after policy/budget authorization, before persistence.
    /// The pending reservation counts against all siblings immediately.
    ///
    /// # Errors
    /// Rejects foreign lineage, uncommitted parents, duplicate paths, and any
    /// exceeded depth/concurrency/lifetime limit. Poisoned state fails closed.
    pub fn reserve_child(&self, child: &AgentThread) -> Result<ChildReservation, AgentLimitError> {
        child.validate()?;
        if child.status != AgentThreadStatus::Pending
            || child.run_id.is_some()
            || child.history_revision != 0
        {
            return Err(ThreadRecordError::InvalidStatus.into());
        }
        let mut state = self
            .state
            .lock()
            .map_err(|_| AgentLimitError::Unavailable)?;
        let parent_id = child
            .parent_thread_id
            .as_deref()
            .ok_or(ThreadRecordError::LineageMismatch)?;
        let parent = if parent_id == state.root.thread_id {
            &state.root
        } else {
            &state
                .children
                .get(parent_id)
                .filter(|entry| entry.committed)
                .ok_or_else(|| AgentLimitError::UnknownThread {
                    thread_id: parent_id.into(),
                })?
                .thread
        };
        AgentEdge::between(parent, child)?;
        if child.depth() > self.limits.max_depth {
            return Err(AgentLimitError::Depth {
                limit: self.limits.max_depth,
            });
        }
        if state.children.contains_key(&child.thread_id)
            || state
                .children
                .values()
                .any(|entry| entry.thread.canonical_path == child.canonical_path)
        {
            return Err(AgentLimitError::DuplicateChild);
        }
        if state.active.len() >= self.limits.max_concurrent_children {
            return Err(AgentLimitError::ConcurrentChildren {
                limit: self.limits.max_concurrent_children,
            });
        }
        if state.children.len() >= self.limits.max_total_children {
            return Err(AgentLimitError::TotalChildren {
                limit: self.limits.max_total_children,
            });
        }
        state.children.insert(
            child.thread_id.clone(),
            ChildEntry {
                thread: child.clone(),
                committed: false,
            },
        );
        state.active.insert(child.thread_id.clone());
        Ok(ChildReservation {
            state: Arc::clone(&self.state),
            thread_id: child.thread_id.clone(),
            persistence_started: false,
            resolved: false,
        })
    }

    /// Reserve another turn on a persisted child without counting a new child.
    ///
    /// # Errors
    /// Rejects unknown/uncommitted children, concurrent reuse of a child, and a
    /// full concurrent-child quota. It does not perform policy or budget approval.
    pub fn reserve_turn(&self, thread_id: &str) -> Result<ActiveChildPermit, AgentLimitError> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| AgentLimitError::Unavailable)?;
        if !state
            .children
            .get(thread_id)
            .is_some_and(|entry| entry.committed)
        {
            return Err(AgentLimitError::UnknownThread {
                thread_id: thread_id.into(),
            });
        }
        if state.active.contains(thread_id) {
            return Err(AgentLimitError::AlreadyRunning {
                thread_id: thread_id.into(),
            });
        }
        if state.active.len() >= self.limits.max_concurrent_children {
            return Err(AgentLimitError::ConcurrentChildren {
                limit: self.limits.max_concurrent_children,
            });
        }
        state.active.insert(thread_id.into());
        Ok(ActiveChildPermit {
            state: Arc::clone(&self.state),
            thread_id: thread_id.into(),
        })
    }

    /// Snapshot counters, including work admitted but not yet persisted.
    ///
    /// # Errors
    /// Fails closed if admission state was poisoned.
    pub fn counts(&self) -> Result<AgentTreeCounts, AgentLimitError> {
        let state = self
            .state
            .lock()
            .map_err(|_| AgentLimitError::Unavailable)?;
        Ok(AgentTreeCounts {
            total_children: state.children.len(),
            concurrent_children: state.active.len(),
        })
    }
}

/// Admission guard. Drop before persistence rolls back both counters. Drop
/// during persistence retains the unknown child's lifetime count but releases
/// concurrency; confirmed commit transfers concurrency to a live-turn permit.
#[derive(Debug)]
#[must_use = "retain the reservation until persistence commits or aborts"]
pub struct ChildReservation {
    state: Arc<Mutex<AdmissionState>>,
    thread_id: String,
    persistence_started: bool,
    resolved: bool,
}

impl ChildReservation {
    /// Mark the boundary immediately before awaiting an atomic thread+edge write.
    /// Cancellation after this point cannot prove that the database did not commit.
    pub fn begin_persistence(&mut self) {
        self.persistence_started = true;
    }

    /// Release a reservation only after the host has confirmed that persistence
    /// did not write the child. An ambiguous timeout is not such confirmation.
    ///
    /// # Errors
    /// Fails closed if admission state is unavailable.
    pub fn abort_unpersisted(mut self) -> Result<(), AgentLimitError> {
        {
            let mut state = self
                .state
                .lock()
                .map_err(|_| AgentLimitError::Unavailable)?;
            state.children.remove(&self.thread_id);
            state.active.remove(&self.thread_id);
            self.resolved = true;
        }
        Ok(())
    }

    /// Commit only after persistence confirms the thread and edge were written.
    ///
    /// # Errors
    /// Returns unavailable state or a missing reservation; it never recreates one.
    pub fn commit(mut self) -> Result<ActiveChildPermit, AgentLimitError> {
        if !self.persistence_started {
            return Err(AgentLimitError::PersistenceNotStarted);
        }
        {
            let mut state = self
                .state
                .lock()
                .map_err(|_| AgentLimitError::Unavailable)?;
            let entry = state.children.get_mut(&self.thread_id).ok_or_else(|| {
                AgentLimitError::UnknownThread {
                    thread_id: self.thread_id.clone(),
                }
            })?;
            entry.committed = true;
            self.resolved = true;
        }
        Ok(ActiveChildPermit {
            state: Arc::clone(&self.state),
            thread_id: self.thread_id.clone(),
        })
    }
}

impl Drop for ChildReservation {
    fn drop(&mut self) {
        if !self.resolved
            && let Ok(mut state) = self.state.lock()
        {
            if !self.persistence_started {
                state.children.remove(&self.thread_id);
            }
            state.active.remove(&self.thread_id);
        }
    }
}

/// Retained by the host for the entire child turn, including waits on descendants.
/// Releasing it frees concurrency but cannot erase a committed child from the tree.
#[derive(Debug)]
#[must_use = "retain the permit for the entire child turn"]
pub struct ActiveChildPermit {
    state: Arc<Mutex<AdmissionState>>,
    thread_id: String,
}

impl Drop for ActiveChildPermit {
    fn drop(&mut self) {
        if let Ok(mut state) = self.state.lock() {
            state.active.remove(&self.thread_id);
        }
    }
}
