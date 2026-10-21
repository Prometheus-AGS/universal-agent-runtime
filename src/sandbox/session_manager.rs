use std::collections::{HashMap, HashSet};
use std::panic::AssertUnwindSafe;
use std::sync::{Arc, Mutex, MutexGuard};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use futures::{FutureExt, StreamExt, stream::FuturesUnordered};
use tokio::sync::{oneshot, watch};

use crate::uar::runtime::lifecycle::{AdmissionClosed, CloseReport, LifecycleState, ResourceScope, WeakResourceScope};

use super::runner::{RunnerCapabilities, SandboxRunner};
use super::types::*;

struct SessionEntry {
    handle: SandboxHandle,
    last_accessed: Instant,
    active_operations: usize,
}

#[derive(Default)]
struct SessionState {
    sessions: HashMap<String, SessionEntry>,
    creating: HashSet<String>,
    destroying: HashSet<String>,
    expiry_blocked: HashSet<String>,
    unknown_creations: HashSet<String>,
    unknown_destructions: HashSet<String>,
    unknown_operations: HashMap<String, usize>,
}

// Physical resources do not retain the facade or scope. The construction-time
// finalizer owns this state even when every service facade has been dropped.
struct SessionResources {
    state: Mutex<SessionState>,
    runner: Arc<dyn SandboxRunner>,
    ttl: Duration,
    max_concurrent: usize,
    work_generation: watch::Sender<u64>,
}

impl SessionResources {
    fn lock(&self) -> MutexGuard<'_, SessionState> {
        self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn notify_work(&self) {
        self.work_generation
            .send_modify(|generation| *generation = generation.saturating_add(1));
    }

    fn touch_locked(state: &mut SessionState, session_id: &str) -> Option<SandboxHandle> {
        if state.destroying.contains(session_id) || state.unknown_destructions.contains(session_id)
            || state.unknown_operations.contains_key(session_id) {
            return None;
        }
        let entry = state.sessions.get_mut(session_id)?;
        entry.last_accessed = Instant::now();
        let handle = entry.handle.clone();
        state.expiry_blocked.remove(session_id);
        Some(handle)
    }

    fn next_expiry(&self) -> Option<tokio::time::Instant> {
        let state = self.lock();
        state.sessions.iter()
            .filter(|(id, entry)| entry.active_operations == 0
                && !state.destroying.contains(*id)
                && !state.unknown_operations.contains_key(*id)
                && !state.expiry_blocked.contains(*id))
            .map(|(_, entry)| entry.last_accessed + self.ttl)
            .min().map(tokio::time::Instant::from_std)
    }

    async fn finalize(self: Arc<Self>, observer: WeakResourceScope) -> anyhow::Result<()> {
        // ResourceScope has already joined all creation, destruction, operation
        // and GC tasks. Keep handles in state until physical release succeeds.
        let handles = {
            let state = self.lock();
            state.sessions.iter()
                .filter(|(id, _)| !state.unknown_destructions.contains(*id)
                    && !state.unknown_operations.contains_key(*id))
                .map(|(id, entry)| (id.clone(), entry.handle.clone()))
                .collect::<Vec<_>>()
        };
        let mut pending = FuturesUnordered::new();
        for (id, handle) in handles {
            let resources = Arc::clone(&self);
            pending.push(async move {
                let resource_id = handle.id.clone();
                let outcome = AssertUnwindSafe(async { resources.runner.destroy(handle).await }).catch_unwind().await;
                match outcome {
                    Ok(Ok(())) => { resources.lock().sessions.remove(&id); None }
                    Ok(Err(SandboxError::OutcomeUnknown(_))) | Err(_) => {
                        resources.lock().unknown_destructions.insert(id);
                        Some(format!("sandbox resource {resource_id:?}: destruction outcome unknown"))
                    }
                    Ok(Err(_)) => Some(format!("sandbox resource {resource_id:?}: destruction failed")),
                }
            });
        }
        let mut failures = Vec::new();
        while let Some(failure) = pending.next().await {
            if let Some(failure) = failure { failures.push(failure); }
        }
        let state = self.lock();
        if let Some(scope) = observer.upgrade() {
            for id in state.creating.iter().chain(&state.unknown_creations) {
                scope.record_unresolved_effect(format!("sandbox session {id:?}: creation outcome unknown; no bound handle"));
            }
            for id in &state.unknown_destructions {
                if let Some(entry) = state.sessions.get(id) {
                    scope.record_unresolved_effect(format!("sandbox resource {:?}: destruction outcome unknown; handle retained", entry.handle.id));
                }
            }
            for (id, count) in &state.unknown_operations {
                if let Some(entry) = state.sessions.get(id) {
                    scope.record_unresolved_effect(format!("sandbox resource {:?}: {count} operation effect outcome(s) unknown; handle retained", entry.handle.id));
                }
            }
        }
        if !state.creating.is_empty() || !state.unknown_creations.is_empty() {
            failures.push("sandbox creation ended without a known resource outcome".to_owned());
        }
        if !state.unknown_operations.is_empty() {
            failures.push("sandbox operation ended without a known effect outcome".to_owned());
        }
        if failures.is_empty() { Ok(()) }
        else { Err(anyhow::anyhow!(failures.join("; "))) }
    }
}

/// The runner access capability shares the manager's ownership boundary. It
/// never exposes the raw backend or accepts a handle outside this manager.
struct ManagedRunner {
    resources: Arc<SessionResources>,
    scope: ResourceScope,
}

type Completion<T> = oneshot::Receiver<Result<T, SandboxError>>;

impl ManagedRunner {
    async fn get_or_create(&self, session_id: &str, config: SandboxConfig) -> Result<SandboxHandle, SandboxError> {
        let cancellation = self.scope.cancellation_token();
        loop {
            // Register the observer before inspecting reservations. A creator's
            // completion must not fall between the inspection and this wait.
            let mut work = self.resources.work_generation.subscribe();
            let completion = {
                let _admission = self.scope.admit_named("sandbox-session-lookup").map_err(admission_error)?;
                let mut state = self.resources.lock();
                if state.unknown_creations.contains(session_id) || state.unknown_destructions.contains(session_id)
                    || state.unknown_operations.contains_key(session_id) {
                    return Err(SandboxError::OutcomeUnknown("prior session resource outcome is unknown".into()));
                }
                if let Some(handle) = SessionResources::touch_locked(&mut state, session_id) {
                    self.resources.notify_work();
                    return Ok(handle);
                }
                if state.creating.contains(session_id) || state.destroying.contains(session_id) {
                    None
                } else {
                    if state.sessions.len().saturating_add(state.creating.len()).saturating_add(state.unknown_creations.len()) >= self.resources.max_concurrent {
                        return Err(SandboxError::CapacityExceeded(self.resources.max_concurrent));
                    }
                    state.creating.insert(session_id.to_owned());
                    let resources = Arc::clone(&self.resources);
                    let id = session_id.to_owned();
                    let config = config.clone();
                    let (sender, receiver) = oneshot::channel();
                    // Hold the reservation lock until task registration either
                    // succeeds or rolls back. No backend effect precedes it.
                    let admitted = self.scope.spawn("sandbox-session-create", async move {
                        let outcome = AssertUnwindSafe(async { resources.runner.create(config).await }).catch_unwind().await;
                        let result = {
                            let mut state = resources.lock();
                            state.creating.remove(&id);
                            match outcome {
                                Ok(Ok(handle)) => {
                                    state.sessions.insert(id, SessionEntry {
                                        handle: handle.clone(), last_accessed: Instant::now(), active_operations: 0,
                                    });
                                    Ok(handle)
                                }
                                Ok(Err(error)) => {
                                    if matches!(&error, SandboxError::OutcomeUnknown(_)) {
                                        state.unknown_creations.insert(id);
                                    }
                                    Err(error)
                                }
                                Err(_) => {
                                    state.unknown_creations.insert(id);
                                    Err(SandboxError::OutcomeUnknown("creation panicked; resource outcome unknown".into()))
                                }
                            }
                        };
                        resources.notify_work();
                        let _ = sender.send(result);
                        Ok(())
                    });
                    if let Err(error) = admitted {
                        state.creating.remove(session_id);
                        return Err(admission_error(error));
                    }
                    Some(receiver)
                }
            };
            if let Some(completion) = completion { return receive(completion).await; }
            tokio::select! {
                _ = cancellation.cancelled() => return Err(shutting_down()),
                changed = work.changed() => if changed.is_err() { return Err(shutting_down()); },
            }
        }
    }

    fn begin_destroy(&self, session_id: &str, only_expired: bool, expected: Option<&SandboxHandle>) -> Result<Completion<()>, SandboxError> {
        let mut state = self.resources.lock();
        if let Some(expected) = expected {
            if !state.sessions.get(session_id).is_some_and(|entry| {
                entry.handle.id == expected.id && entry.handle.session_id == expected.session_id
            }) {
                return Err(SandboxError::NotFound("sandbox handle binding changed".into()));
            }
        }
        if state.unknown_creations.contains(session_id) || state.unknown_destructions.contains(session_id)
            || state.unknown_operations.contains_key(session_id) {
            return Err(SandboxError::OutcomeUnknown("prior session resource outcome is unknown".into()));
        }
        // Busy/renewed entries are rechecked under the same lock as the
        // destruction reservation, not merely in the earlier GC snapshot.
        if only_expired && (state.expiry_blocked.contains(session_id)
            || state.sessions.get(session_id).is_none_or(|entry| {
                entry.active_operations != 0 || entry.last_accessed.elapsed() < self.resources.ttl
            })) {
            let (sender, receiver) = oneshot::channel();
            let _ = sender.send(Ok(()));
            return Ok(receiver);
        }
        if state.destroying.contains(session_id) {
            return Err(SandboxError::ExecutionFailed("sandbox destruction is already in progress".into()));
        }
        state.destroying.insert(session_id.to_owned());
        let resources = Arc::clone(&self.resources);
        let id = session_id.to_owned();
        let (sender, receiver) = oneshot::channel();
        let admitted = self.scope.spawn("sandbox-session-destroy", async move {
            let handle = loop {
                let mut work = resources.work_generation.subscribe();
                let available = {
                    let state = resources.lock();
                    if state.unknown_creations.contains(&id) || state.unknown_operations.contains_key(&id) {
                        Some(Err(SandboxError::OutcomeUnknown("session resource or effect outcome remains unknown".into())))
                    } else if state.creating.contains(&id) { None }
                    else if let Some(entry) = state.sessions.get(&id) {
                        if entry.active_operations != 0 { None } else { Some(Ok(Some(entry.handle.clone()))) }
                    } else { Some(Ok(None)) }
                };
                if let Some(handle) = available { break handle; }
                // Accepted destruction drains admitted use even after scope
                // cancellation. Cancellation is not a release acknowledgment.
                if work.changed().await.is_err() {
                    return Err(anyhow::anyhow!("sandbox work source ended while destruction was pending"));
                }
            };
            let attempted = matches!(&handle, Ok(Some(_)));
            let result = match handle {
                Ok(Some(handle)) => match AssertUnwindSafe(async { resources.runner.destroy(handle).await }).catch_unwind().await {
                    Ok(result) => result,
                    Err(_) => Err(SandboxError::OutcomeUnknown("destruction panicked; release outcome unknown".into())),
                },
                Ok(None) => Ok(()),
                Err(error) => Err(error),
            };
            {
                let mut state = resources.lock();
                state.destroying.remove(&id);
                if result.is_ok() {
                    state.sessions.remove(&id);
                    state.expiry_blocked.remove(&id);
                } else {
                    if attempted && matches!(&result, Err(SandboxError::OutcomeUnknown(_))) {
                        state.unknown_destructions.insert(id.clone());
                    }
                    state.expiry_blocked.insert(id);
                }
            }
            resources.notify_work();
            let _ = sender.send(result);
            Ok(())
        });
        if let Err(error) = admitted {
            state.destroying.remove(session_id);
            return Err(admission_error(error));
        }
        Ok(receiver)
    }

    fn begin_operation<T, F, Fut>(&self, handle: &SandboxHandle, name: &'static str, operation: F) -> Result<Completion<T>, SandboxError>
    where
        T: Send + 'static,
        F: FnOnce(Arc<dyn SandboxRunner>, SandboxHandle) -> Fut + Send + 'static,
        Fut: Future<Output = Result<T, SandboxError>> + Send + 'static,
    {
        let mut state = self.resources.lock();
        let id = state.sessions.iter().find(|(id, entry)| {
            entry.handle.id == handle.id && entry.handle.session_id == handle.session_id
                && !state.destroying.contains(*id) && !state.unknown_destructions.contains(*id)
                && !state.unknown_operations.contains_key(*id)
        }).map(|(id, _)| id.clone()).ok_or_else(|| SandboxError::NotFound("handle is not an available owned session".into()))?;
        let entry = state.sessions.get_mut(&id).ok_or_else(|| SandboxError::NotFound(id.clone()))?;
        entry.active_operations += 1;
        entry.last_accessed = Instant::now();
        state.expiry_blocked.remove(&id);
        let resources = Arc::clone(&self.resources);
        let backend = Arc::clone(&resources.runner);
        let handle = handle.clone();
        let task_id = id.clone();
        let (sender, receiver) = oneshot::channel();
        let admitted = self.scope.spawn(name, async move {
            let use_guard = SessionUse { resources: Arc::clone(&resources), id: task_id.clone() };
            let result = match AssertUnwindSafe(async move { operation(backend, handle).await }).catch_unwind().await {
                Ok(result) => result,
                Err(_) => {
                    Err(SandboxError::OutcomeUnknown("operation panicked; effect outcome unknown".into()))
                }
            };
            if name != "sandbox-file-read" && matches!(&result, Err(SandboxError::OutcomeUnknown(_))) {
                *resources.lock().unknown_operations.entry(task_id).or_default() += 1;
            }
            drop(use_guard);
            let _ = sender.send(result);
            Ok(())
        });
        if let Err(error) = admitted {
            if let Some(entry) = state.sessions.get_mut(&id) { entry.active_operations -= 1; }
            return Err(admission_error(error));
        }
        drop(state);
        self.resources.notify_work();
        Ok(receiver)
    }
}

struct SessionUse {
    resources: Arc<SessionResources>,
    id: String,
}

impl Drop for SessionUse {
    fn drop(&mut self) {
        {
            let mut state = self.resources.lock();
            if let Some(entry) = state.sessions.get_mut(&self.id) {
                entry.active_operations -= 1;
                entry.last_accessed = Instant::now();
            }
        }
        self.resources.notify_work();
    }
}

#[async_trait]
impl SandboxRunner for ManagedRunner {
    fn enforces_isolation(&self) -> bool { self.resources.runner.enforces_isolation() }

    async fn create(&self, config: SandboxConfig) -> Result<SandboxHandle, SandboxError> {
        self.get_or_create(&uuid::Uuid::new_v4().to_string(), config).await
    }

    async fn execute(&self, handle: &SandboxHandle, request: ExecutionRequest) -> Result<ExecutionResult, SandboxError> {
        receive(self.begin_operation(handle, "sandbox-execute", move |runner, handle| async move {
            runner.execute(&handle, request).await
        })?).await
    }

    async fn write_file(&self, handle: &SandboxHandle, path: &str, content: &[u8]) -> Result<(), SandboxError> {
        let path = path.to_owned();
        let content = content.to_vec();
        receive(self.begin_operation(handle, "sandbox-file-write", move |runner, handle| async move {
            runner.write_file(&handle, &path, &content).await
        })?).await
    }

    async fn read_file(&self, handle: &SandboxHandle, path: &str) -> Result<Vec<u8>, SandboxError> {
        let path = path.to_owned();
        receive(self.begin_operation(handle, "sandbox-file-read", move |runner, handle| async move {
            runner.read_file(&handle, &path).await
        })?).await
    }

    async fn destroy(&self, handle: SandboxHandle) -> Result<(), SandboxError> {
        let id = self.resources.lock().sessions.iter().find(|(_, entry)| {
            entry.handle.id == handle.id && entry.handle.session_id == handle.session_id
        }).map(|(id, _)| id.clone()).ok_or_else(|| SandboxError::NotFound("handle is not an owned session".into()))?;
        receive(self.begin_destroy(&id, false, Some(&handle))?).await
    }

    fn capabilities(&self) -> RunnerCapabilities { self.resources.runner.capabilities() }
}

pub struct SessionManager {
    runner: ManagedRunner,
    gc_started: AtomicBool,
}

impl std::fmt::Debug for SessionManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SessionManager")
            .field("ttl", &self.runner.resources.ttl)
            .field("max_concurrent", &self.runner.resources.max_concurrent)
            .field("active_count", &self.active_count())
            .finish_non_exhaustive()
    }
}

impl SessionManager {
    #[must_use]
    pub fn new(runner: Arc<dyn SandboxRunner>, ttl_secs: u64, max_concurrent: usize) -> Self {
        Self::new_in_scope(runner, ttl_secs, max_concurrent,
            ResourceScope::standalone("sandbox-sessions", "sandbox-session-manager"))
    }

    #[must_use]
    pub fn new_in_scope(runner: Arc<dyn SandboxRunner>, ttl_secs: u64,
        max_concurrent: usize, lifecycle_scope: ResourceScope) -> Self {
        let (work_generation, _) = watch::channel(0_u64);
        let resources = Arc::new(SessionResources {
            state: Mutex::new(SessionState::default()), runner,
            ttl: Duration::from_secs(ttl_secs), max_concurrent, work_generation,
        });
        let owned = Arc::clone(&resources);
        let observer = lifecycle_scope.downgrade();
        // Installation precedes every possible acquisition. Preserve the
        // infallible constructor API, but fail closed if a caller supplies an
        // already closed scope or a scope owned by another finalizer.
        let registered = lifecycle_scope.retain_resource_state(Arc::clone(&resources))
            .and_then(|()| lifecycle_scope.set_finalizer(async move { owned.finalize(observer).await }));
        if let Err(error) = registered {
            lifecycle_scope.record_failure(format!("sandbox finalizer registration failed: {error}"));
            lifecycle_scope.close_admission();
        }
        Self { runner: ManagedRunner { resources, scope: lifecycle_scope }, gc_started: AtomicBool::new(false) }
    }

    pub fn touch_session(&self, session_id: &str) {
        if SessionResources::touch_locked(&mut self.runner.resources.lock(), session_id).is_some() {
            self.runner.resources.notify_work();
        }
    }

    pub async fn get_or_create(&self, session_id: &str, config: SandboxConfig) -> Result<SandboxHandle, SandboxError> {
        self.runner.get_or_create(session_id, config).await
    }

    pub async fn destroy_session(&self, session_id: &str) -> Result<(), SandboxError> {
        receive(self.runner.begin_destroy(session_id, false, None)?).await
    }

    fn begin_cleanup_expired(&self) -> Vec<Completion<()>> {
        let expired = {
            let state = self.runner.resources.lock();
            state.sessions.iter().filter(|(id, entry)| {
                entry.active_operations == 0 && !state.destroying.contains(*id)
                    && !state.unknown_operations.contains_key(*id)
                    && !state.expiry_blocked.contains(*id)
                    && entry.last_accessed.elapsed() >= self.runner.resources.ttl
            }).map(|(id, _)| id.clone()).collect::<Vec<_>>()
        };
        expired.into_iter().filter_map(|id| self.runner.begin_destroy(&id, true, None).ok()).collect()
    }

    pub async fn cleanup_expired(&self) {
        for completion in self.begin_cleanup_expired() { let _ = receive(completion).await; }
    }

    /// The retained scope owns this one-shot expiry task. Busy sessions have
    /// no expiry deadline; failed destruction waits for real session activity.
    pub fn start_gc_task(self: &Arc<Self>) -> Result<(), AdmissionClosed> {
        if self.gc_started.swap(true, Ordering::AcqRel) { return Ok(()); }
        let manager = Arc::downgrade(self);
        let cancellation = self.runner.scope.cancellation_token();
        let mut lifecycle = self.runner.scope.subscribe();
        let mut work = self.runner.resources.work_generation.subscribe();
        let result = self.runner.scope.spawn("sandbox-session-expiry", async move {
            loop {
                if cancellation.is_cancelled() { return Ok(()); }
                if matches!(lifecycle.borrow().state, LifecycleState::AdmissionClosed
                    | LifecycleState::Draining | LifecycleState::Finalizing | LifecycleState::Released) {
                    return Ok(());
                }
                let Some(manager) = manager.upgrade() else { return Ok(()); };
                // Registration is synchronous; no strong controller is held
                // while waiting for physical cleanup or the next work event.
                let _ = manager.begin_cleanup_expired();
                let due = manager.runner.resources.next_expiry();
                drop(manager);
                match due {
                    Some(due) => tokio::select! {
                        _ = cancellation.cancelled() => return Ok(()),
                        changed = lifecycle.changed() => if changed.is_err() { return Ok(()); },
                        changed = work.changed() => if changed.is_err() { return Err(anyhow::anyhow!("sandbox session work source closed")); },
                        _ = tokio::time::sleep_until(due) => {},
                    },
                    None => tokio::select! {
                        _ = cancellation.cancelled() => return Ok(()),
                        changed = lifecycle.changed() => if changed.is_err() { return Ok(()); },
                        changed = work.changed() => if changed.is_err() { return Err(anyhow::anyhow!("sandbox session work source closed")); },
                    },
                }
            }
        });
        if result.is_err() { self.gc_started.store(false, Ordering::Release); }
        result
    }

    /// Await the scope's single retained result after all admitted work and
    /// physical sandbox destruction. A dropped waiter cannot abandon cleanup.
    pub async fn shutdown(&self) -> Result<CloseReport, SandboxError> {
        Ok(self.runner.scope.close().await)
    }

    /// Return a scoped facade. Backend operations retain a session-use lease
    /// through completion even when their result waiter is canceled.
    #[must_use]
    pub fn runner(&self) -> &dyn SandboxRunner { &self.runner }

    #[must_use]
    pub fn active_count(&self) -> usize { self.runner.resources.lock().sessions.len() }
}

async fn receive<T>(completion: Completion<T>) -> Result<T, SandboxError> {
    completion.await.map_err(|_| SandboxError::ExecutionFailed("owned sandbox task ended without a result".into()))?
}

fn admission_error(error: AdmissionClosed) -> SandboxError {
    SandboxError::CreationFailed(format!("sandbox session lifecycle admission closed: {error}"))
}
fn shutting_down() -> SandboxError { SandboxError::CreationFailed("sandbox session lifecycle is shutting down".into()) }
