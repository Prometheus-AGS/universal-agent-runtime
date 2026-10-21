//! Trusted-host lifecycle ownership for runtime resources.
//!
//! A [`ResourceScope`] closes admission and accounts for accepted work under
//! one synchronous gate. Completion is retained: dropping an HTTP request or
//! another waiter cannot cancel a scope finalizer once it has started.

use std::{
    collections::{BTreeMap, BTreeSet},
    pin::Pin,
    panic::AssertUnwindSafe,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};

use futures::{FutureExt, StreamExt, stream::FuturesUnordered};
use tokio::sync::{Mutex as AsyncMutex, watch};
use tokio_util::{
    sync::CancellationToken,
    task::TaskTracker,
};
use uuid::Uuid;

tokio::task_local! {
    static CURRENT_RESOURCE_SCOPE: ResourceScope;
}

/// The trusted host scope carried by the currently admitted task, if any.
#[must_use]
pub fn current_scope() -> Option<ResourceScope> {
    CURRENT_RESOURCE_SCOPE.try_with(Clone::clone).ok()
}

/// Start blocking work with its admission lease inside the actual worker.
/// Dropping or timing out its join waiter cannot release that lease. Work
/// started by an admitted host task inherits its lifecycle authority; direct
/// compatibility callers retain an independent operation scope instead.
pub fn spawn_blocking<F, T>(
    operation_name: impl Into<String>,
    work: F,
) -> Result<tokio::task::JoinHandle<T>, AdmissionClosed>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    let operation_name = operation_name.into();
    let scope = current_scope().unwrap_or_else(|| {
        ResourceScope::standalone("blocking-operation", "compatibility-caller")
    });
    scope.spawn_blocking(operation_name, work)
}

/// A resource's retained lifecycle state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LifecycleState {
    Registered,
    Ready,
    AdmissionClosed,
    Draining,
    Finalizing,
    Released,
    Failed,
}

/// Which layer owns the underlying resource rather than merely a scoped
/// capability to use it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResourceOwnership {
    Owned,
    Borrowed,
}

/// Inspectable state for one trusted-host resource.
#[derive(Clone, Debug)]
pub struct ResourceSnapshot {
    pub resource_id: String,
    pub owner: String,
    pub ownership: ResourceOwnership,
    pub dependencies: Vec<String>,
    pub generation: u64,
    pub state: LifecycleState,
    pub in_flight_operations: usize,
    pub outstanding_operation_ids: Vec<String>,
    pub tracked_tasks: usize,
    pub failure: Option<String>,
    /// Sanitized lifecycle failure categories retained for on-demand control
    /// inspection. Detailed failures remain in the terminal close report.
    pub failure_categories: Vec<String>,
    /// Durable effects whose outcome remains unresolved after local cleanup.
    pub unresolved_effects: Vec<String>,
    pub dependency_failures: Vec<LifecycleDependencyFailure>,
}

/// The terminal result retained by a resource scope.
#[derive(Clone, Debug)]
pub struct CloseReport {
    pub resource_id: String,
    pub generation: u64,
    pub released: bool,
    pub completed_operations: usize,
    pub completed_tasks: usize,
    pub failures: Vec<String>,
    pub unresolved_effects: Vec<String>,
    /// Reports from scopes finalized by this scope. These preserve the actual
    /// child failure and uncertainty rather than replacing them with a generic
    /// supervisor error.
    pub child_reports: Vec<CloseReport>,
    pub dependency_failures: Vec<LifecycleDependencyFailure>,
}

impl CloseReport {
    fn empty(resource_id: String, generation: u64) -> Self {
        Self {
            resource_id,
            generation,
            released: false,
            completed_operations: 0,
            completed_tasks: 0,
            failures: Vec::new(),
            unresolved_effects: Vec::new(),
            child_reports: Vec::new(),
            dependency_failures: Vec::new(),
        }
    }
}

/// The current availability of shutdown inventory for a budget-limited
/// observer. `SnapshotPending` never means that no resources remain.
#[derive(Clone, Debug)]
pub enum ShutdownOutstandingResources {
    SnapshotPending,
    Resources(Vec<ResourceSnapshot>),
}

/// A bounded observation of retained supervisor shutdown work.
#[derive(Clone, Debug)]
pub enum ShutdownObservation {
    Pending {
        shutdown_id: String,
        outstanding_resources: ShutdownOutstandingResources,
    },
    Completed {
        shutdown_id: String,
        report: CloseReport,
    },
}

/// Admission was closed before an operation could be registered.
#[derive(Clone, Debug, thiserror::Error)]
#[error("runtime resource {resource_id} is no longer admitting work")]
pub struct AdmissionClosed {
    resource_id: String,
}

/// A resource cannot be published as ready after admission closes or a tracked
/// task has already made its lifecycle unavailable.
#[derive(Clone, Debug, thiserror::Error)]
pub enum LifecycleReadinessError {
    #[error("runtime resource {resource_id} is no longer admitting work")]
    AdmissionClosed { resource_id: String },
    #[error("runtime resource {resource_id} cannot become ready after failure: {failure}")]
    Failed { resource_id: String, failure: String },
}

/// A lifecycle resource can no longer accept registration changes.
#[derive(Clone, Debug, thiserror::Error)]
pub enum LifecycleRegistrationError {
    #[error("runtime resource {resource_id} is no longer admitting work")]
    AdmissionClosed { resource_id: String },
    #[error("runtime resource {resource_id} already has a finalizer")]
    FinalizerAlreadyRegistered { resource_id: String },
    #[error("runtime supervisor is no longer registering resources")]
    SupervisorClosed,
    #[error("runtime resource {resource_id} is not registered")]
    UnknownResource { resource_id: String },
    #[error("runtime resource {resource_id} already has a reconnect handler")]
    ReconnectAlreadyRegistered { resource_id: String },
    #[error("runtime resource {resource_id} already retains resource state")]
    ResourceStateAlreadyRegistered { resource_id: String },
}

/// A dependency graph cannot establish an ordered finalization for a resource.
#[derive(Clone, Debug)]
pub enum LifecycleDependencyFailure {
    Missing {
        resource_id: String,
        dependency_id: String,
    },
    Cycle {
        resource_ids: Vec<String>,
    },
}

type ResourceFinalizer =
    Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'static>>;

#[derive(Debug)]
struct ScopeGate {
    admission_closed: bool,
    state: LifecycleState,
    in_flight_operations: usize,
    outstanding_operation_ids: BTreeSet<String>,
    next_operation_id: u64,
    completed_operations: usize,
    completed_tasks: usize,
    failures: Vec<String>,
    unresolved_effects: Vec<String>,
    child_reports: Vec<CloseReport>,
    dependency_failures: Vec<LifecycleDependencyFailure>,
    close_started: bool,
}

fn lifecycle_failure_categories(gate: &ScopeGate) -> Vec<String> {
    let mut categories = BTreeSet::new();
    for failure in &gate.failures {
        let category = if failure.starts_with("lifecycle dependency failure") {
            "dependency"
        } else if failure.contains("panicked") {
            "panic"
        } else if failure.starts_with("resource finalizer") {
            "finalizer"
        } else if failure.contains(": task ") {
            "task"
        } else {
            "cleanup"
        };
        categories.insert(category.to_owned());
    }
    categories.into_iter().collect()
}

fn lifecycle_error_category(gate: &ScopeGate) -> &'static str {
    if !gate.dependency_failures.is_empty() {
        "dependency"
    } else if gate.failures.iter().any(|failure| failure.contains("panicked")) {
        "panic"
    } else if !gate.failures.is_empty() {
        "cleanup"
    } else if !gate.unresolved_effects.is_empty() {
        "outcome_unknown"
    } else {
        "none"
    }
}

#[derive(Debug)]
struct ScopeMetadata {
    ownership: ResourceOwnership,
    dependencies: BTreeSet<String>,
    close_admission_with_host: bool,
}

struct ScopeInner {
    resource_id: String,
    owner: String,
    generation: u64,
    cancellation: CancellationToken,
    tasks: TaskTracker,
    gate: Mutex<ScopeGate>,
    metadata: Mutex<ScopeMetadata>,
    finalizer: Mutex<Option<ResourceFinalizer>>,
    retained_state: Mutex<Option<Arc<dyn std::any::Any + Send + Sync>>>,
    snapshot: watch::Sender<ResourceSnapshot>,
    close_result: watch::Sender<Option<CloseReport>>,
}

impl ScopeInner {
    fn publish_locked(&self, gate: &ScopeGate) {
        let metadata = self
            .metadata
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let previous = self.snapshot.borrow().clone();
        let next = ResourceSnapshot {
            resource_id: self.resource_id.clone(),
            owner: self.owner.clone(),
            ownership: metadata.ownership.clone(),
            dependencies: metadata.dependencies.iter().cloned().collect(),
            generation: self.generation,
            state: gate.state.clone(),
            in_flight_operations: gate.in_flight_operations,
            outstanding_operation_ids: gate.outstanding_operation_ids.iter().cloned().collect(),
            tracked_tasks: self.tasks.len(),
            failure: gate.failures.last().cloned(),
            failure_categories: lifecycle_failure_categories(gate),
            unresolved_effects: gate.unresolved_effects.clone(),
            dependency_failures: gate.dependency_failures.clone(),
        };
        if previous.state != next.state
            || previous.ownership != next.ownership
            || previous.dependencies != next.dependencies
        {
            tracing::info!(
                resource_id = %next.resource_id,
                generation = next.generation,
                owner = %next.owner,
                from = ?previous.state,
                to = ?next.state,
                ownership = ?next.ownership,
                dependencies = ?next.dependencies,
                outstanding_operation_ids = ?next.outstanding_operation_ids,
                error_category = lifecycle_error_category(gate),
                "runtime resource lifecycle transition"
            );
        }
        self.snapshot.send_replace(next);
    }

    fn close_admission(&self) {
        let mut gate = self
            .gate
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !gate.admission_closed {
            gate.admission_closed = true;
            gate.state = LifecycleState::AdmissionClosed;
            self.tasks.close();
            self.publish_locked(&gate);
        }
    }

    async fn finalize(self: Arc<Self>) {
        self.close_admission();
        self.cancellation.cancel();
        {
            let mut gate = self
                .gate
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            gate.state = LifecycleState::Draining;
            self.publish_locked(&gate);
        }

        let mut snapshots = self.snapshot.subscribe();
        loop {
            if snapshots.borrow().in_flight_operations == 0 {
                break;
            }
            // `watch` retains the current count, so a completion that happens
            // between inspection and waiting cannot be lost.
            if snapshots.changed().await.is_err() {
                break;
            }
        }

        {
            let mut gate = self
                .gate
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            gate.state = LifecycleState::Finalizing;
            self.publish_locked(&gate);
        }
        self.tasks.wait().await;
        let finalizer = self
            .finalizer
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
        if let Some(finalizer) = finalizer {
            let result = AssertUnwindSafe(finalizer).catch_unwind().await;
            let mut gate = self
                .gate
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            match result {
                Ok(Ok(())) => {}
                Ok(Err(error)) => gate.failures.push(format!("resource finalizer: {error:#}")),
                Err(_) => gate.failures.push("resource finalizer panicked".into()),
            }
            self.publish_locked(&gate);
        }

        let report = {
            let mut gate = self
                .gate
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let released = gate.failures.is_empty() && gate.unresolved_effects.is_empty();
            gate.state = if released {
                LifecycleState::Released
            } else {
                LifecycleState::Failed
            };
            self.publish_locked(&gate);
            CloseReport {
                resource_id: self.resource_id.clone(),
                generation: self.generation,
                released,
                completed_operations: gate.completed_operations,
                completed_tasks: gate.completed_tasks,
                // Retain terminal obligations in the scope snapshot as well
                // as the close report so on-demand inspection after shutdown
                // cannot erase the evidence it is meant to report.
                failures: gate.failures.clone(),
                unresolved_effects: gate.unresolved_effects.clone(),
                child_reports: gate.child_reports.clone(),
                dependency_failures: gate.dependency_failures.clone(),
            }
        };
        self.close_result.send_replace(Some(report));
    }
}

/// One trusted-host scope. Clones are access capabilities, never ownership
/// transfers; resource finalization remains retained by the shared scope.
#[derive(Clone)]
pub struct ResourceScope {
    inner: Arc<ScopeInner>,
}

/// A non-owning observation capability for finalizers and controller facades.
/// It prevents a resource's retained finalizer from strongly owning its scope.
#[derive(Clone)]
pub struct WeakResourceScope {
    inner: std::sync::Weak<ScopeInner>,
}

impl std::fmt::Debug for WeakResourceScope {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WeakResourceScope")
            .field("upgradable", &(self.inner.strong_count() > 0))
            .finish()
    }
}

impl WeakResourceScope {
    #[must_use]
    pub fn upgrade(&self) -> Option<ResourceScope> {
        self.inner.upgrade().map(|inner| ResourceScope { inner })
    }
}

impl std::fmt::Debug for ResourceScope {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let snapshot = self.snapshot();
        formatter
            .debug_struct("ResourceScope")
            .field("resource_id", &snapshot.resource_id)
            .field("owner", &snapshot.owner)
            .field("generation", &snapshot.generation)
            .field("state", &snapshot.state)
            .finish_non_exhaustive()
    }
}

impl ResourceScope {
    /// Carry host ownership through directly awaited async work. Tokio task
    /// locals do not propagate through a separately spawned task or a blocking
    /// runtime entry; those boundaries must install this scope explicitly.
    pub async fn run_in_scope<F>(&self, future: F) -> F::Output
    where
        F: Future,
    {
        CURRENT_RESOURCE_SCOPE.scope(self.clone(), future).await
    }

    /// Admit before dispatch and retain the lease in the blocking closure,
    /// including after a waiting caller disappears. Panic is an acknowledged
    /// failed worker outcome and remains in the owning resource report.
    pub fn spawn_blocking<F, T>(
        &self,
        operation_name: impl Into<String>,
        work: F,
    ) -> Result<tokio::task::JoinHandle<T>, AdmissionClosed>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        let operation_name = operation_name.into();
        let lease = self.admit_named(&operation_name)?;
        let scope = self.clone();
        Ok(tokio::task::spawn_blocking(move || {
            let result = std::panic::catch_unwind(AssertUnwindSafe(work));
            match result {
                Ok(output) => {
                    lease.complete();
                    output
                }
                Err(panic) => {
                    scope.record_failure(format!("{operation_name}: blocking worker panicked"));
                    lease.complete();
                    std::panic::resume_unwind(panic)
                }
            }
        }))
    }

    #[must_use]
    pub fn downgrade(&self) -> WeakResourceScope {
        WeakResourceScope { inner: Arc::downgrade(&self.inner) }
    }

    /// Retain the resource's actual state after a failed finalizer, so failure
    /// reporting cannot abandon ownership of handles needing explicit recovery.
    /// The state must use weak scope references to avoid an ownership cycle.
    pub fn retain_resource_state<T>(&self, state: Arc<T>) -> Result<(), LifecycleRegistrationError>
    where T: std::any::Any + Send + Sync {
        let gate = self.inner.gate.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if gate.admission_closed {
            return Err(LifecycleRegistrationError::AdmissionClosed { resource_id: self.inner.resource_id.clone() });
        }
        let mut retained = self.inner.retained_state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if retained.is_some() {
            return Err(LifecycleRegistrationError::ResourceStateAlreadyRegistered { resource_id: self.inner.resource_id.clone() });
        }
        *retained = Some(state);
        Ok(())
    }

    /// Observe retained host resource state without reopening admission.
    #[must_use]
    pub fn resource_state<T>(&self) -> Option<Arc<T>>
    where T: std::any::Any + Send + Sync {
        self.inner.retained_state.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()?.clone().downcast::<T>().ok()
    }

    /// Create an independently owned scope for an embedding that does not use
    /// a process-wide supervisor. The embedding must call [`Self::close`] as
    /// part of its own shutdown contract.
    #[must_use]
    pub fn standalone(resource_id: impl Into<String>, owner: impl Into<String>) -> Self {
        let scope = Self::new(resource_id.into(), owner.into(), 1);
        let _ = scope.mark_ready();
        scope
    }

    fn new(resource_id: String, owner: String, generation: u64) -> Self {
        let initial = ResourceSnapshot {
            resource_id: resource_id.clone(),
            owner: owner.clone(),
            ownership: ResourceOwnership::Owned,
            dependencies: Vec::new(),
            generation,
            state: LifecycleState::Registered,
            in_flight_operations: 0,
            outstanding_operation_ids: Vec::new(),
            tracked_tasks: 0,
            failure: None,
            failure_categories: Vec::new(),
            unresolved_effects: Vec::new(),
            dependency_failures: Vec::new(),
        };
        let (snapshot, _) = watch::channel(initial);
        let (close_result, _) = watch::channel(None);
        let scope = Self {
            inner: Arc::new(ScopeInner {
                resource_id,
                owner,
                generation,
                cancellation: CancellationToken::new(),
                tasks: TaskTracker::new(),
                gate: Mutex::new(ScopeGate {
                    admission_closed: false,
                    state: LifecycleState::Registered,
                    in_flight_operations: 0,
                    outstanding_operation_ids: BTreeSet::new(),
                    next_operation_id: 1,
                    completed_operations: 0,
                    completed_tasks: 0,
                    failures: Vec::new(),
                    unresolved_effects: Vec::new(),
                    child_reports: Vec::new(),
                    dependency_failures: Vec::new(),
                    close_started: false,
                }),
                metadata: Mutex::new(ScopeMetadata {
                    ownership: ResourceOwnership::Owned,
                    dependencies: BTreeSet::new(),
                    close_admission_with_host: false,
                }),
                finalizer: Mutex::new(None),
                retained_state: Mutex::new(None),
                snapshot,
                close_result,
            }),
        };
        tracing::info!(
            resource_id = %scope.inner.resource_id,
            generation = scope.inner.generation,
            owner = %scope.inner.owner,
            from = "unregistered",
            to = "registered",
            ownership = ?ResourceOwnership::Owned,
            dependencies = ?Vec::<String>::new(),
            outstanding_operation_ids = ?Vec::<String>::new(),
            error_category = "none",
            "runtime resource lifecycle transition"
        );
        scope
    }

    /// Publish that this scope can accept work after its owner has completed
    /// all startup dependencies.
    pub fn mark_ready(&self) -> Result<(), LifecycleReadinessError> {
        let mut gate = self
            .inner
            .gate
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if gate.admission_closed {
            return Err(LifecycleReadinessError::AdmissionClosed {
                resource_id: self.inner.resource_id.clone(),
            });
        }
        if let Some(failure) = gate.failures.last().cloned() {
            gate.state = LifecycleState::Failed;
            self.inner.publish_locked(&gate);
            return Err(LifecycleReadinessError::Failed {
                resource_id: self.inner.resource_id.clone(),
                failure,
            });
        }
        gate.state = LifecycleState::Ready;
        self.inner.publish_locked(&gate);
        Ok(())
    }

    /// Register the one resource-specific closure operation before the owner
    /// exposes the resource to other runtime work.
    pub fn set_finalizer<F>(&self, finalizer: F) -> Result<(), LifecycleRegistrationError>
    where
        F: Future<Output = anyhow::Result<()>> + Send + 'static,
    {
        let gate = self
            .inner
            .gate
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if gate.admission_closed {
            return Err(LifecycleRegistrationError::AdmissionClosed {
                resource_id: self.inner.resource_id.clone(),
            });
        }
        let mut slot = self
            .inner
            .finalizer
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if slot.is_some() {
            return Err(LifecycleRegistrationError::FinalizerAlreadyRegistered {
                resource_id: self.inner.resource_id.clone(),
            });
        }
        *slot = Some(Box::pin(finalizer));
        Ok(())
    }

    /// Declare whether this scope owns the underlying resource. Borrowed
    /// resources release only their UAR capability during finalization.
    pub fn set_ownership(
        &self,
        ownership: ResourceOwnership,
    ) -> Result<(), AdmissionClosed> {
        let gate = self
            .inner
            .gate
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if gate.admission_closed {
            return Err(AdmissionClosed {
                resource_id: self.inner.resource_id.clone(),
            });
        }
        let mut metadata = self
            .inner
            .metadata
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        metadata.ownership = ownership;
        drop(metadata);
        self.inner.publish_locked(&gate);
        Ok(())
    }

    /// Declare resources that must remain usable while this scope finalizes.
    /// The supervisor exposes these edges in every resource snapshot.
    pub fn declare_dependencies<I, S>(&self, dependencies: I) -> Result<(), AdmissionClosed>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.configure_dependencies(dependencies, false)
    }

    fn configure_dependencies<I, S>(
        &self,
        dependencies: I,
        allow_closed_admission: bool,
    ) -> Result<(), AdmissionClosed>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let gate = self
            .inner
            .gate
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if gate.close_started || (gate.admission_closed && !allow_closed_admission) {
            return Err(AdmissionClosed {
                resource_id: self.inner.resource_id.clone(),
            });
        }
        let mut metadata = self
            .inner
            .metadata
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        metadata.dependencies.extend(dependencies.into_iter().map(Into::into));
        drop(metadata);
        self.inner.publish_locked(&gate);
        Ok(())
    }

    /// Atomically verify admission and account for an operation.
    pub fn admit(&self) -> Result<OperationLease, AdmissionClosed> {
        self.admit_named("operation")
    }

    /// Atomically register a named operation. The stable generated ID remains
    /// visible in snapshots until acknowledged completion.
    pub fn admit_named(&self, name: impl AsRef<str>) -> Result<OperationLease, AdmissionClosed> {
        let mut gate = self
            .inner
            .gate
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if gate.admission_closed {
            return Err(AdmissionClosed {
                resource_id: self.inner.resource_id.clone(),
            });
        }
        gate.in_flight_operations = gate.in_flight_operations.saturating_add(1);
        let sequence = gate.next_operation_id;
        gate.next_operation_id = gate.next_operation_id.saturating_add(1);
        let id = format!(
            "{}:{}:{}:{sequence}",
            self.inner.resource_id,
            name.as_ref(),
            self.inner.generation
        );
        gate.outstanding_operation_ids.insert(id.clone());
        self.inner.publish_locked(&gate);
        tracing::debug!(resource = %self.inner.resource_id, operation_id = %id, "lifecycle operation registered");
        Ok(OperationLease {
            inner: Arc::clone(&self.inner),
            id,
            resolved: false,
        })
    }

    /// Register a task only after its operation lease has been admitted.
    ///
    /// Task errors and panics are retained in the close report. The task keeps
    /// no strong reference to a supervisor, only to this resource scope.
    pub fn spawn<F>(&self, task_name: impl Into<String>, future: F) -> Result<(), AdmissionClosed>
    where
        F: Future<Output = anyhow::Result<()>> + Send + 'static,
    {
        let task_name = task_name.into();
        // Keep the admission gate locked through `TaskTracker::spawn`. Closing
        // the tracker alone cannot reject a concurrent spawn.
        let mut gate = self
            .inner
            .gate
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if gate.admission_closed {
            return Err(AdmissionClosed {
                resource_id: self.inner.resource_id.clone(),
            });
        }
        gate.in_flight_operations = gate.in_flight_operations.saturating_add(1);
        let sequence = gate.next_operation_id;
        gate.next_operation_id = gate.next_operation_id.saturating_add(1);
        let id = format!(
            "{}:task:{task_name}:{}:{sequence}",
            self.inner.resource_id,
            self.inner.generation
        );
        gate.outstanding_operation_ids.insert(id.clone());
        let lease = OperationLease {
            inner: Arc::clone(&self.inner),
            id,
            resolved: false,
        };
        let inner = Arc::clone(&self.inner);
        let execution_scope = self.clone();
        self.inner.tasks.spawn(async move {
            let result = AssertUnwindSafe(execution_scope.run_in_scope(future))
                .catch_unwind().await;
            {
                let mut gate = inner
                    .gate
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                gate.completed_tasks = gate.completed_tasks.saturating_add(1);
                match result {
                    Ok(Ok(())) => {}
                    Ok(Err(error)) => gate
                        .failures
                        .push(format!("{task_name}: {error:#}")),
                    Err(_) => gate.failures.push(format!("{task_name}: task panicked")),
                }
                if !gate.failures.is_empty()
                    && matches!(gate.state, LifecycleState::Registered | LifecycleState::Ready)
                {
                    gate.state = LifecycleState::Failed;
                }
                inner.publish_locked(&gate);
            }
            drop(lease);
        });
        self.inner.publish_locked(&gate);
        Ok(())
    }

    /// Stop new work. This operation is idempotent and does not await cleanup.
    pub fn close_admission(&self) {
        self.inner.close_admission();
    }

    /// A cancellation signal for work already admitted to this scope.
    #[must_use]
    pub fn cancellation_token(&self) -> CancellationToken {
        self.inner.cancellation.child_token()
    }

    /// Retained lifecycle state for current and late subscribers.
    #[must_use]
    pub fn subscribe(&self) -> watch::Receiver<ResourceSnapshot> {
        self.inner.snapshot.subscribe()
    }

    #[must_use]
    pub fn snapshot(&self) -> ResourceSnapshot {
        self.inner.snapshot.borrow().clone()
    }

    /// Read the retained terminal report without starting or awaiting closure.
    /// Control-plane callers use this to inspect unresolved durable effects
    /// after an independently-owned finalizer has completed.
    #[must_use]
    pub fn close_report(&self) -> Option<CloseReport> {
        self.inner.close_result.borrow().clone()
    }

    /// Report a durable effect whose outcome cannot be established locally.
    pub fn record_unresolved_effect(&self, effect: impl Into<String>) {
        let mut gate = self
            .inner
            .gate
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        gate.unresolved_effects.push(effect.into());
        self.inner.publish_locked(&gate);
    }

    /// Retain a local cleanup failure without fabricating a released state.
    pub fn record_failure(&self, failure: impl Into<String>) {
        let mut gate = self
            .inner
            .gate
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        gate.failures.push(failure.into());
        if matches!(gate.state, LifecycleState::Registered | LifecycleState::Ready) {
            gate.state = LifecycleState::Failed;
        }
        self.inner.publish_locked(&gate);
    }

    /// Preserve a child scope's terminal report in this scope's final result.
    pub fn record_child_report(&self, report: CloseReport) {
        let mut gate = self
            .inner
            .gate
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        gate.child_reports.push(report);
        self.inner.publish_locked(&gate);
    }

    fn record_dependency_failure(&self, failure: LifecycleDependencyFailure) {
        let mut gate = self
            .inner
            .gate
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        gate.failures.push(format!("lifecycle dependency failure: {failure:?}"));
        gate.dependency_failures.push(failure);
        if matches!(gate.state, LifecycleState::Registered | LifecycleState::Ready) {
            gate.state = LifecycleState::Failed;
        }
        self.inner.publish_locked(&gate);
    }

    /// Begin one retained finalizer. It remains owned by the scope after the
    /// caller that requested closure is canceled.
    fn begin_close_retained(&self) {
        let start_finalizer = {
            let mut gate = self
                .inner
                .gate
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if gate.close_started {
                false
            } else {
                gate.close_started = true;
                true
            }
        };
        if start_finalizer {
            self.inner.close_admission();
            self.inner.cancellation.cancel();
            let inner = Arc::clone(&self.inner);
            tokio::spawn(async move {
                inner.finalize().await;
            });
        }
    }

    /// Begin one retained finalizer and await its retained result.
    ///
    /// A cancelled caller only drops its wait. The spawned finalizer remains
    /// owned by this scope and all later callers observe the same report.
    pub async fn close(&self) -> CloseReport {
        self.begin_close_retained();
        let mut result = self.inner.close_result.subscribe();
        loop {
            if let Some(report) = result.borrow().clone() {
                return report;
            }
            if result.changed().await.is_err() {
                let mut report =
                    CloseReport::empty(self.inner.resource_id.clone(), self.inner.generation);
                report
                    .failures
                    .push("lifecycle finalizer ended without a completion report".into());
                return report;
            }
        }
    }
}

/// An admitted operation. It is completed exactly once when released.
pub struct OperationLease {
    inner: Arc<ScopeInner>,
    id: String,
    resolved: bool,
}

impl std::fmt::Debug for OperationLease {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("OperationLease")
            .field("id", &"***redacted***")
            .field("resolved", &self.resolved)
            .finish_non_exhaustive()
    }
}

impl OperationLease {
    #[must_use]
    pub fn id(&self) -> &str { &self.id }
    /// Retain an unresolved durable outcome for the resource's final report.
    pub fn unresolved(self, effect: impl Into<String>) {
        {
            let mut gate = self
                .inner
                .gate
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            gate.unresolved_effects.push(effect.into());
            self.inner.publish_locked(&gate);
        }
        self.complete();
    }

    /// Complete this operation before dropping the lease.
    pub fn complete(mut self) {
        self.finish();
    }

    fn finish(&mut self) {
        if self.resolved {
            return;
        }
        let mut gate = self
            .inner
            .gate
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        gate.in_flight_operations = gate.in_flight_operations.saturating_sub(1);
        gate.outstanding_operation_ids.remove(&self.id);
        gate.completed_operations = gate.completed_operations.saturating_add(1);
        self.inner.publish_locked(&gate);
        tracing::debug!(resource = %self.inner.resource_id, operation_id = %self.id, "lifecycle operation completed");
        self.resolved = true;
    }
}

impl Drop for OperationLease {
    fn drop(&mut self) {
        self.finish();
    }
}

struct SupervisorRegistry {
    closing: bool,
    resources: BTreeMap<String, ResourceScope>,
    registration_order: Vec<String>,
    reconnect_handlers: BTreeMap<String, ReconnectHandler>,
}

#[derive(Default)]
struct SupervisorShutdownState {
    initiated: bool,
    outstanding_resources: Option<BTreeMap<String, ResourceScope>>,
}

type ReconnectHandler = Arc<
    dyn Fn(u64) -> Pin<Box<dyn Future<Output = anyhow::Result<u64>> + Send>> + Send + Sync,
>;

fn known_dependency_closure(
    roots: &BTreeSet<String>,
    scopes: &BTreeMap<String, ResourceScope>,
) -> BTreeSet<String> {
    let mut closure = roots.clone();
    let mut pending = roots.iter().cloned().collect::<Vec<_>>();
    while let Some(resource_id) = pending.pop() {
        let Some(resource) = scopes.get(&resource_id) else {
            continue;
        };
        for dependency_id in resource.snapshot().dependencies {
            if scopes.contains_key(&dependency_id) && closure.insert(dependency_id.clone()) {
                pending.push(dependency_id);
            }
        }
    }
    closure
}

fn reaches_resource(
    start: &str,
    target: &str,
    remaining: &BTreeSet<String>,
    scopes: &BTreeMap<String, ResourceScope>,
) -> bool {
    let mut visited = BTreeSet::new();
    let mut pending = vec![start.to_owned()];
    while let Some(resource_id) = pending.pop() {
        if !remaining.contains(&resource_id) || !visited.insert(resource_id.clone()) {
            continue;
        }
        if resource_id == target {
            return true;
        }
        if let Some(resource) = scopes.get(&resource_id) {
            pending.extend(
                resource
                    .snapshot()
                    .dependencies
                    .into_iter()
                    .filter(|dependency_id| remaining.contains(dependency_id)),
            );
        }
    }
    false
}

fn cycle_members(
    remaining: &BTreeSet<String>,
    scopes: &BTreeMap<String, ResourceScope>,
) -> BTreeSet<String> {
    remaining
        .iter()
        .filter(|resource_id| {
            scopes
                .get(resource_id.as_str())
                .is_some_and(|resource| {
                    resource
                        .snapshot()
                        .dependencies
                        .iter()
                        .filter(|dependency_id| remaining.contains(*dependency_id))
                        .any(|dependency_id| {
                            dependency_id == *resource_id
                                || reaches_resource(
                                    dependency_id,
                                    resource_id,
                                    remaining,
                                    scopes,
                                )
                        })
                })
        })
        .cloned()
        .collect()
}

struct SupervisorInner {
    registry: AsyncMutex<SupervisorRegistry>,
    generations: AtomicU64,
    shutdown: ResourceScope,
    shutdown_id: String,
    shutdown_state: Mutex<SupervisorShutdownState>,
}

/// Trusted host owner for all runtime resource scopes.
#[derive(Clone)]
pub struct RuntimeSupervisor {
    inner: Arc<SupervisorInner>,
}

impl std::fmt::Debug for RuntimeSupervisor {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("RuntimeSupervisor(..)")
    }
}

/// A cloneable command and observation capability for a supervisor.
#[derive(Clone)]
pub struct RuntimeHandle {
    supervisor: RuntimeSupervisor,
}

impl std::fmt::Debug for RuntimeHandle {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("RuntimeHandle(..)")
    }
}

impl RuntimeSupervisor {
    #[must_use]
    pub fn new(owner: impl Into<String>) -> Self {
        let generation = 1;
        let shutdown = ResourceScope::new(
            "runtime-supervisor".into(),
            owner.into(),
            generation,
        );
        let _ = shutdown.mark_ready();
        Self {
            inner: Arc::new(SupervisorInner {
                registry: AsyncMutex::new(SupervisorRegistry {
                    closing: false,
                    resources: BTreeMap::new(),
                    registration_order: Vec::new(),
                    reconnect_handlers: BTreeMap::new(),
                }),
                generations: AtomicU64::new(generation),
                shutdown,
                shutdown_id: format!("runtime-supervisor-shutdown-{}", Uuid::new_v4()),
                shutdown_state: Mutex::new(SupervisorShutdownState::default()),
            }),
        }
    }

    #[must_use]
    pub fn handle(&self) -> RuntimeHandle {
        RuntimeHandle {
            supervisor: self.clone(),
        }
    }

    /// Register a resource before it acquires external state or starts work.
    pub async fn register(
        &self,
        resource_id: impl Into<String>,
        owner: impl Into<String>,
    ) -> Result<ResourceScope, LifecycleRegistrationError> {
        self.register_scope(resource_id.into(), owner.into(), false).await
    }

    /// Register external intake whose admission closes immediately when host
    /// shutdown begins, even while watcher/producers are still being joined.
    /// Accepted work still drains in dependency order. Persistence admission is
    /// left available for the finalizers that legitimately require it.
    pub async fn register_external(
        &self,
        resource_id: impl Into<String>,
        owner: impl Into<String>,
    ) -> Result<ResourceScope, LifecycleRegistrationError> {
        self.register_scope(resource_id.into(), owner.into(), true).await
    }

    async fn register_scope(
        &self,
        resource_id: String,
        owner: String,
        close_admission_with_host: bool,
    ) -> Result<ResourceScope, LifecycleRegistrationError> {
        let mut registry = self.inner.registry.lock().await;
        if registry.closing {
            return Err(LifecycleRegistrationError::SupervisorClosed);
        }
        if let Some(scope) = registry.resources.get(&resource_id) {
            if close_admission_with_host {
                scope.inner.metadata.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
                    .close_admission_with_host = true;
            }
            return Ok(scope.clone());
        }
        let generation = self
            .inner
            .generations
            .fetch_add(1, Ordering::Relaxed)
            .saturating_add(1);
        let scope = ResourceScope::new(resource_id.clone(), owner, generation);
        scope.inner.metadata.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
            .close_admission_with_host = close_admission_with_host;
        registry.registration_order.push(resource_id.clone());
        registry.resources.insert(resource_id, scope.clone());
        Ok(scope)
    }

    /// Register a resource and its closure operation as one acquisition step.
    pub async fn register_with_finalizer<F>(
        &self,
        resource_id: impl Into<String>,
        owner: impl Into<String>,
        finalizer: F,
    ) -> Result<ResourceScope, LifecycleRegistrationError>
    where
        F: Future<Output = anyhow::Result<()>> + Send + 'static,
    {
        let resource_id = resource_id.into();
        let mut registry = self.inner.registry.lock().await;
        if registry.closing {
            return Err(LifecycleRegistrationError::SupervisorClosed);
        }
        if registry.resources.contains_key(&resource_id) {
            return Err(LifecycleRegistrationError::FinalizerAlreadyRegistered { resource_id });
        }
        let generation = self
            .inner
            .generations
            .fetch_add(1, Ordering::Relaxed)
            .saturating_add(1);
        let scope = ResourceScope::new(resource_id.clone(), owner.into(), generation);
        scope.set_finalizer(finalizer)?;
        registry.registration_order.push(resource_id.clone());
        registry.resources.insert(resource_id, scope.clone());
        Ok(scope)
    }

    /// Look up a registered scope so composition code can attach its
    /// dependency metadata after all startup resources have been acquired.
    pub async fn resource_scope(
        &self,
        resource_id: &str,
    ) -> Result<ResourceScope, LifecycleRegistrationError> {
        let registry = self.inner.registry.lock().await;
        registry
            .resources
            .get(resource_id)
            .cloned()
            .ok_or_else(|| LifecycleRegistrationError::UnknownResource {
                resource_id: resource_id.to_owned(),
            })
    }

    /// Declare finalization prerequisites without retaining a resource handle
    /// in server composition code.
    pub async fn declare_dependencies<I, S>(
        &self,
        resource_id: &str,
        dependencies: I,
    ) -> Result<(), LifecycleRegistrationError>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let registry = self.inner.registry.lock().await;
        if registry.closing {
            return Err(LifecycleRegistrationError::SupervisorClosed);
        }
        let scope = registry
            .resources
            .get(resource_id)
            .cloned()
            .ok_or_else(|| LifecycleRegistrationError::UnknownResource {
                resource_id: resource_id.to_owned(),
            })?;
        scope
            // Partial startup can close intake before the acquired prefix has
            // its final graph. Registry admission is still held above; attach
            // cleanup dependencies before any finalization is allowed to start.
            .configure_dependencies(dependencies, true)
            .map_err(|_| LifecycleRegistrationError::AdmissionClosed {
                resource_id: resource_id.to_owned(),
            })
    }

    /// Register the host-authorized reconnect implementation for one resource.
    /// Reconnection is scoped work, so a dropped control request cannot detach
    /// an already-admitted connection attempt from finalization.
    pub async fn register_reconnect<F, Fut>(
        &self,
        resource_id: &str,
        handler: F,
    ) -> Result<(), LifecycleRegistrationError>
    where
        F: Fn(u64) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = anyhow::Result<u64>> + Send + 'static,
    {
        let mut registry = self.inner.registry.lock().await;
        if registry.closing {
            return Err(LifecycleRegistrationError::SupervisorClosed);
        }
        if !registry.resources.contains_key(resource_id) {
            return Err(LifecycleRegistrationError::UnknownResource {
                resource_id: resource_id.to_owned(),
            });
        }
        if registry.reconnect_handlers.contains_key(resource_id) {
            return Err(LifecycleRegistrationError::ReconnectAlreadyRegistered {
                resource_id: resource_id.to_owned(),
            });
        }
        registry.reconnect_handlers.insert(
            resource_id.to_owned(),
            Arc::new(move |generation| Box::pin(handler(generation))),
        );
        Ok(())
    }

    #[must_use]
    pub async fn resources(&self) -> Vec<ResourceSnapshot> {
        let scopes = {
            let registry = self.inner.registry.lock().await;
            registry
                .registration_order
                .iter()
                .filter_map(|resource_id| registry.resources.get(resource_id).cloned())
                .collect::<Vec<_>>()
        };
        scopes.into_iter().map(|scope| scope.snapshot()).collect()
    }

    /// Read one resource's retained closure result without requesting shutdown.
    pub async fn resource_report(
        &self,
        resource_id: &str,
    ) -> Result<Option<CloseReport>, LifecycleRegistrationError> {
        Ok(self.resource_scope(resource_id).await?.close_report())
    }

    /// Read every retained terminal resource report without requesting any
    /// lifecycle transition. Resources still closing are omitted.
    #[must_use]
    pub async fn close_reports(&self) -> Vec<CloseReport> {
        let scopes = {
            let registry = self.inner.registry.lock().await;
            registry
                .registration_order
                .iter()
                .filter_map(|resource_id| registry.resources.get(resource_id).cloned())
                .collect::<Vec<_>>()
        };
        scopes
            .into_iter()
            .filter_map(|scope| scope.close_report())
            .collect()
    }

    fn initiate_shutdown(&self) {
        let inner = Arc::clone(&self.inner);
        let supervisor_scope = inner.shutdown.clone();
        let mut state = inner
            .shutdown_state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if state.initiated {
            return;
        }

        let worker_inner = Arc::clone(&inner);
        let worker_scope = supervisor_scope.clone();
        let spawned = supervisor_scope.spawn("runtime-supervisor-finalizer", async move {
            RuntimeSupervisor::shutdown_worker(worker_inner, worker_scope).await
        });
        state.initiated = true;
        match spawned {
            Ok(_) => supervisor_scope.begin_close_retained(),
            Err(error) => {
                supervisor_scope.record_failure(format!(
                    "runtime supervisor shutdown finalizer could not be retained: {error}"
                ));
                supervisor_scope.begin_close_retained();
            }
        }
    }

    fn outstanding_shutdown_resources(&self) -> ShutdownOutstandingResources {
        let scopes = {
            let state = self
                .inner
                .shutdown_state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            state
                .outstanding_resources
                .as_ref()
                .map(|resources| resources.values().cloned().collect::<Vec<_>>())
        };
        match scopes {
            Some(scopes) => ShutdownOutstandingResources::Resources(
                scopes.iter().map(ResourceScope::snapshot).collect(),
            ),
            None => ShutdownOutstandingResources::SnapshotPending,
        }
    }

    /// Observe retained shutdown work for no longer than `budget`.
    ///
    /// The budget limits only this observer. Accepted finalization remains
    /// owned by the supervisor until all actual joins complete.
    pub async fn shutdown_with_budget(&self, budget: Duration) -> ShutdownObservation {
        self.initiate_shutdown();
        let supervisor_scope = self.inner.shutdown.clone();
        let shutdown_id = self.inner.shutdown_id.clone();
        if let Some(report) = supervisor_scope.close_report() {
            return ShutdownObservation::Completed {
                shutdown_id,
                report,
            };
        }
        match tokio::time::timeout(budget, supervisor_scope.close()).await {
            Ok(report) => ShutdownObservation::Completed {
                shutdown_id,
                report,
            },
            Err(_) => {
                let outstanding_resources = self.outstanding_shutdown_resources();
                if let Some(report) = supervisor_scope.close_report() {
                    ShutdownObservation::Completed {
                        shutdown_id,
                        report,
                    }
                } else {
                    ShutdownObservation::Pending {
                        shutdown_id,
                        outstanding_resources,
                    }
                }
            }
        }
    }

    /// Retain shutdown work, then close external intake and await registered
    /// scopes in their declared dependency order. Reverse registration is only
    /// a tie-breaker.
    pub async fn shutdown(&self) -> CloseReport {
        self.initiate_shutdown();
        self.inner.shutdown.close().await
    }

    async fn shutdown_worker(
        inner: Arc<SupervisorInner>,
        scope: ResourceScope,
    ) -> anyhow::Result<()> {
        let (registration_order, scopes) = {
            let mut registry = inner.registry.lock().await;
            if registry.closing {
                scope.record_failure(
                    "runtime supervisor shutdown finalizer started after registry closure",
                );
                return Ok(());
            }
            registry.closing = true;
            for resource in registry.resources.values() {
                let closes_early = resource
                    .inner
                    .metadata
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .close_admission_with_host;
                if closes_early {
                    resource.close_admission();
                }
            }
            let registration_order = registry.registration_order.clone();
            let scopes = registry.resources.clone();
            {
                let mut state = inner
                    .shutdown_state
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                state.outstanding_resources = Some(scopes.clone());
            }
            (registration_order, scopes)
        };
        let registration_index = registration_order
            .iter()
            .enumerate()
            .map(|(index, resource_id)| (resource_id.clone(), index))
            .collect::<BTreeMap<_, _>>();
        let mut remaining = scopes.keys().cloned().collect::<BTreeSet<_>>();
        let mut missing_roots = BTreeSet::new();
        for (resource_id, resource) in &scopes {
            for dependency_id in resource.snapshot().dependencies {
                if !scopes.contains_key(&dependency_id) {
                    let failure = LifecycleDependencyFailure::Missing {
                        resource_id: resource_id.clone(),
                        dependency_id,
                    };
                    resource.record_dependency_failure(failure.clone());
                    scope.record_dependency_failure(failure);
                    missing_roots.insert(resource_id.clone());
                }
            }
        }
        // A scope that cannot establish every dependency must stay open,
        // together with all resources it still needs. Closing its known
        // prerequisites would fabricate an ordered shutdown.
        let missing_blocked = known_dependency_closure(&missing_roots, &scopes);
        remaining.retain(|resource_id| !missing_blocked.contains(resource_id));
        let mut finalizers: FuturesUnordered<
            Pin<Box<dyn Future<Output = (String, CloseReport)> + Send>>,
        > = FuturesUnordered::new();
        let mut finalizing = BTreeSet::new();

        while !remaining.is_empty() {
            let mut leaves = remaining
                .iter()
                .filter(|candidate| {
                    !finalizing.contains(*candidate) && !remaining.iter().any(|resource_id| {
                        resource_id.as_str() != candidate.as_str()
                            && scopes
                                .get(resource_id)
                                .is_some_and(|scope| {
                                    scope
                                        .snapshot()
                                        .dependencies
                                        .iter()
                                        .any(|dependency| dependency.as_str() == candidate.as_str())
                                })
                    })
                })
                .cloned()
                .collect::<Vec<_>>();
            if !leaves.is_empty() {
                leaves.sort_by_key(|resource_id| {
                    std::cmp::Reverse(*registration_index.get(resource_id).unwrap_or(&0))
                });
                for resource_id in leaves {
                    let resource = scopes
                        .get(&resource_id)
                        .expect("registered resource remains in supervisor map")
                        .clone();
                    finalizing.insert(resource_id.clone());
                    finalizers.push(Box::pin(async move {
                        let report = resource.close().await;
                        (resource_id, report)
                    }));
                }
                continue;
            }
            if let Some((resource_id, report)) = finalizers.next().await {
                // A completed close acknowledgement, including a failure
                // report, releases only this resource's ordering edge.
                // Other independent leaves can finalize concurrently, and
                // a dependent becomes eligible only after this receipt.
                remaining.remove(&resource_id);
                finalizing.remove(&resource_id);
                for failure in &report.failures {
                    scope.record_failure(format!(
                        "{} cleanup failed: {failure}",
                        report.resource_id
                    ));
                }
                for unresolved_effect in &report.unresolved_effects {
                    scope.record_unresolved_effect(format!(
                        "{}: {unresolved_effect}",
                        report.resource_id
                    ));
                }
                let released = report.released;
                scope.record_child_report(report);
                if released {
                    let mut state = inner
                        .shutdown_state
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    if let Some(resources) = &mut state.outstanding_resources {
                        resources.remove(&resource_id);
                    }
                }
                continue;
            }
            {
                let cycle = cycle_members(&remaining, &scopes);
                if cycle.is_empty() {
                    scope.record_failure(
                        "lifecycle dependency graph has no finalization leaf or cycle member",
                    );
                    for resource_id in remaining {
                        scope.record_unresolved_effect(format!(
                            "{resource_id}: finalization order remains unresolved"
                        ));
                    }
                    break;
                }
                let cycle_ids = cycle.iter().cloned().collect::<Vec<_>>();
                for resource_id in &cycle {
                    if let Some(resource) = scopes.get(resource_id) {
                        resource.record_dependency_failure(
                            LifecycleDependencyFailure::Cycle {
                                resource_ids: cycle_ids.clone(),
                            },
                        );
                    }
                }
                scope.record_dependency_failure(LifecycleDependencyFailure::Cycle {
                    resource_ids: cycle_ids.clone(),
                });
                // Resources outside the SCC are retained because a cycle
                // still needs them, not because they themselves form one.
                let cycle_blocked = known_dependency_closure(&cycle, &scopes);
                for resource_id in cycle_blocked.difference(&cycle) {
                    if let Some(resource) = scopes.get(resource_id) {
                        resource.record_unresolved_effect(format!(
                            "required by cyclic lifecycle resources: {}",
                            cycle_ids.join(", "),
                        ));
                    }
                }
                for resource_id in cycle_blocked {
                    scope.record_unresolved_effect(format!(
                        "{resource_id}: retained because finalization cycle requires it"
                    ));
                }
                break;
            }
        }
        for resource_id in missing_blocked {
            let reason = if missing_roots.contains(&resource_id) {
                "has a missing declared dependency"
            } else {
                "is required by a resource with a missing declared dependency"
            };
            if !missing_roots.contains(&resource_id) {
                if let Some(resource) = scopes.get(&resource_id) {
                    resource.record_unresolved_effect(reason);
                }
            }
            scope.record_unresolved_effect(format!(
                "{resource_id}: retained because it {reason}"
            ));
        }
        Ok(())
    }
}

impl RuntimeHandle {
    #[must_use]
    pub async fn resources(&self) -> Vec<ResourceSnapshot> {
        self.supervisor.resources().await
    }

    /// Observe a retained resource report without changing lifecycle state.
    pub async fn resource_report(
        &self,
        resource_id: &str,
    ) -> Result<Option<CloseReport>, LifecycleRegistrationError> {
        self.supervisor.resource_report(resource_id).await
    }

    /// Observe all completed resource reports without initiating shutdown.
    #[must_use]
    pub async fn close_reports(&self) -> Vec<CloseReport> {
        self.supervisor.close_reports().await
    }

    pub async fn shutdown(&self) -> CloseReport {
        self.supervisor.shutdown().await
    }

    /// Observe retained shutdown work for no longer than `budget`.
    pub async fn shutdown_with_budget(&self, budget: Duration) -> ShutdownObservation {
        self.supervisor.shutdown_with_budget(budget).await
    }

    /// Request an explicit host-authorized reconnect for the expected resource
    /// generation. The result comes from the retained operation rather than
    /// from the lifetime of this control waiter.
    pub async fn reconnect(
        &self,
        resource_id: &str,
        generation: u64,
    ) -> anyhow::Result<u64> {
        let (scope, handler) = {
            let registry = self.supervisor.inner.registry.lock().await;
            if registry.closing {
                anyhow::bail!("runtime supervisor is shutting down");
            }
            let scope = registry
                .resources
                .get(resource_id)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("runtime resource {resource_id} is not registered"))?;
            let handler = registry
                .reconnect_handlers
                .get(resource_id)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("runtime resource {resource_id} has no reconnect handler"))?;
            (scope, handler)
        };
        let (sender, receiver) = tokio::sync::oneshot::channel();
        scope
            .spawn("resource-reconnect", async move {
                let result = handler(generation).await.map_err(|error| error.to_string());
                let _ = sender.send(result);
                Ok(())
            })
            .map_err(|error| anyhow::anyhow!(error.to_string()))?;
        receiver
            .await
            .map_err(|_| anyhow::anyhow!("reconnect operation ended without a result"))?
            .map_err(anyhow::Error::msg)
    }
}
