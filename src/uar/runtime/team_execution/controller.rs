use std::{
    collections::BTreeMap,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use tokio::sync::{Mutex, Semaphore};
use tokio_util::sync::CancellationToken;

use crate::uar::{
    compiler::collaboration::{CollaborationCatalogService, CollaborationError},
    domain::team_execution::{TeamControlRequest, TeamExecutionAttempt, TeamReservation},
    persistence::PersistenceLayer,
    runtime::{actor::messages::ActorOwner, manager::RunManager},
};

struct Job {
    attempt: TeamExecutionAttempt,
    cancellation: CancellationToken,
    finished: AtomicBool,
    handle: Mutex<Option<tokio::task::JoinHandle<()>>>,
}

impl Job {
    async fn locally_live(&self) -> bool {
        if self.finished.load(Ordering::Acquire) {
            return false;
        }
        self.handle
            .lock()
            .await
            .as_ref()
            .is_some_and(|handle| !handle.is_finished())
    }
}

/// Process-local workers consume catalog-owned durable attempts. No model loop lives here.
pub struct TeamExecutionRuntime {
    pub(super) catalog: Arc<CollaborationCatalogService>,
    pub(super) manager: Arc<RunManager>,
    pub(super) persistence: Arc<dyn PersistenceLayer>,
    available: bool,
    lifecycle_scope: crate::uar::runtime::lifecycle::ResourceScope,
    cancellation: CancellationToken,
    jobs: Mutex<BTreeMap<String, Arc<Job>>>,
    members: Mutex<BTreeMap<String, Arc<Mutex<()>>>>,
    active: Arc<Semaphore>,
    owners: Mutex<BTreeMap<String, ActorOwner>>,
    last_team: Mutex<Option<String>>,
    pub(super) host_contexts: super::host::TeamHostContexts,
}

impl std::fmt::Debug for TeamExecutionRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TeamExecutionRuntime")
            .field("available", &self.available)
            .finish_non_exhaustive()
    }
}

impl TeamExecutionRuntime {
    pub fn new(
        catalog: Arc<CollaborationCatalogService>,
        manager: Arc<RunManager>,
        persistence: Arc<dyn PersistenceLayer>,
        available: bool,
    ) -> Arc<Self> {
        // The server finalizer owns this child scope. It must not borrow the
        // run-manager scope: team workers need to stop and join before the
        // manager begins releasing its own persistence dependencies.
        let lifecycle_scope = crate::uar::runtime::lifecycle::ResourceScope::standalone(
            "team-execution-workers",
            "team-execution-runtime",
        );
        let runtime = Arc::new(Self {
            catalog,
            cancellation: manager.root_cancellation_token().child_token(),
            manager,
            persistence,
            available,
            lifecycle_scope,
            jobs: Mutex::new(BTreeMap::new()),
            members: Mutex::new(BTreeMap::new()),
            active: Arc::new(Semaphore::new(
                std::env::var("UAR_TEAM_EXECUTION_MAX_ACTIVE")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .filter(|v| *v > 0)
                    .unwrap_or(4),
            )),
            owners: Mutex::new(BTreeMap::new()),
            last_team: Mutex::new(None),
            host_contexts: Mutex::new(BTreeMap::new()),
        });
        let weak = Arc::downgrade(&runtime);
        let notify = Arc::clone(&runtime.catalog.team_execution_notify);
        let cancel = runtime.cancellation.clone();
        let notifier_scope = runtime.lifecycle_scope.clone();
        let scope_cancellation = notifier_scope.cancellation_token();
        if let Err(error) = notifier_scope.spawn("team-execution-notifier", async move {
            loop {
                tokio::select! {
                    _ = cancel.cancelled() => break,
                    _ = scope_cancellation.cancelled() => break,
                    _ = notify.notified() => {
                        if let Some(runtime) = weak.upgrade() {
                            runtime.drain().await;
                        } else {
                            break;
                        }
                    }
                }
            }
            Ok(())
        }) {
            tracing::warn!(%error, "Team execution notifier was rejected during lifecycle shutdown");
        }
        runtime
    }

    pub async fn quiesce(
        &self,
    ) -> anyhow::Result<crate::uar::domain::team_execution::TeamExecutionFencingEvidence> {
        self.shutdown().await?;
        Ok(self.catalog.record_joined_execution_owner().await?)
    }

    pub fn available(&self) -> bool {
        self.available
    }

    /// Wait for this owner's exact durable attempt to acquire a usable run.
    /// Subscribe before inspection so registration or settlement cannot be lost.
    /// A queued attempt may remain pending; elapsed time never certifies dispatch.
    pub async fn await_run_registration(
        &self,
        owner: &ActorOwner,
        workspace: &str,
        team: &str,
        attempt_id: &str,
    ) -> Result<TeamExecutionAttempt, CollaborationError> {
        let mut registrations = self.manager.subscribe_run_registrations();
        let mut catalog = self.catalog.subscribe_catalog_generation();
        let mut committed_work = self.manager.subscribe_committed_work();
        loop {
            if self.cancellation.is_cancelled() {
                return Err(CollaborationError::Conflict("TEAM_EXECUTION_UNAVAILABLE".into()));
            }
            let attempt = self.catalog.get_team_attempt(
                &owner.presentation_owner_key(), workspace, team, attempt_id,
            ).await?;
            if self.manager.get_run_for_owner(owner, &attempt.run_id).await.is_some() {
                return Ok(attempt);
            }
            if !matches!(attempt.status.as_str(), "queued" | "running" | "cancellation_requested") {
                return Err(CollaborationError::Conflict("TEAM_RUN_NOT_REGISTERED".into()));
            }
            tokio::select! {
                () = self.cancellation.cancelled() => {
                    return Err(CollaborationError::Conflict("TEAM_EXECUTION_UNAVAILABLE".into()));
                },
                changed = registrations.changed() => {
                    if changed.is_err() {
                        return Err(CollaborationError::Conflict("TEAM_EXECUTION_UNAVAILABLE".into()));
                    }
                },
                changed = catalog.changed() => {
                    if changed.is_err() {
                        return Err(CollaborationError::Conflict("TEAM_EXECUTION_UNAVAILABLE".into()));
                    }
                },
                changed = committed_work.changed() => {
                    if changed.is_err() {
                        return Err(CollaborationError::Conflict("TEAM_EXECUTION_UNAVAILABLE".into()));
                    }
                },
            }
        }
    }

    pub async fn dispatch(
        self: &Arc<Self>,
        owner: ActorOwner,
        attempt: TeamExecutionAttempt,
    ) -> Result<TeamExecutionAttempt, CollaborationError> {
        if !self.available || self.cancellation.is_cancelled() {
            return Err(CollaborationError::Conflict(
                "Durable team execution is unavailable for this runtime storage".into(),
            ));
        }
        if owner.presentation_owner_key() != attempt.owner_id {
            return Err(CollaborationError::Conflict(
                "Team attempt belongs to another authenticated owner".into(),
            ));
        }
        let task_lease = self.lifecycle_scope.admit().map_err(|_| {
            CollaborationError::Conflict("Team execution is shutting down".into())
        })?;
        // A duplicate admission receipt never launches a second model turn.
        if attempt.status != "queued" {
            return Ok(attempt);
        }
        self.owners
            .lock()
            .await
            .insert(attempt.owner_id.clone(), owner.clone());
        let member_key = format!(
            "{}\u{1f}{}\u{1f}{}\u{1f}{}",
            attempt.owner_id, attempt.workspace_id, attempt.team_id, attempt.member_id
        );
        let lane = self
            .members
            .lock()
            .await
            .entry(member_key)
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone();
        let serial = match lane.clone().try_lock_owned() {
            Ok(guard) => guard,
            Err(_) => return Ok(attempt),
        };
        let permit = match self.active.clone().try_acquire_owned() {
            Ok(permit) => permit,
            Err(_) => return Ok(attempt),
        };
        // Resolve before claiming dispatch; unavailable resources cannot start a turn.
        let prepared = async {
            let bound = self.catalog.resolve_team_member_run(&attempt).await?;
            self.require_host_context(&attempt, &bound.effective_binding_receipt.effective)
                .await?;
            self.catalog
                .selected_team_context(&attempt, &bound.effective_binding_receipt.resolved_skills)
                .await?;
            Ok::<_, CollaborationError>(())
        }
        .await;
        if let Err(error) = prepared {
            if matches!(&error, CollaborationError::Conflict(code) if code == "TEAM_HOST_CONTEXT_REQUIRED")
            {
                return Err(error);
            }
            self.catalog
                .settle_team_attempt(
                    &attempt,
                    "failed",
                    Some(TeamReservation::default()),
                    None,
                    Some("member_binding_or_selected_context_denied_before_dispatch".into()),
                )
                .await?;
            return Err(error);
        }
        // Shutdown closes the scope before it snapshots direct join handles.
        // Do not publish a prepared task after that point; the admitted lease
        // then drops and the retained close observes the real outcome.
        if self.cancellation.is_cancelled() {
            return Err(CollaborationError::Conflict(
                "Team execution is shutting down".into(),
            ));
        }
        let mut jobs = self.jobs.lock().await;
        if self.cancellation.is_cancelled() {
            return Err(CollaborationError::Conflict(
                "Team execution is shutting down".into(),
            ));
        }
        if jobs.contains_key(&attempt.id) {
            drop(jobs);
            return self
                .catalog
                .get_team_attempt(
                    &attempt.owner_id,
                    &attempt.workspace_id,
                    &attempt.team_id,
                    &attempt.id,
                )
                .await;
        }
        let attempt = match self.catalog.claim_team_dispatch(&attempt).await {
            Ok(attempt) => attempt,
            Err(CollaborationError::Conflict(code)) if code == "TEAM_PENDING_LIMIT" => {
                return Ok(attempt);
            }
            Err(error) => return Err(error),
        };
        if self.cancellation.is_cancelled() {
            // Claiming the attempt is durable, but no model or tool operation
            // has started before this point. Settle that known zero-effect
            // outcome instead of leaving a local-shutdown race as `running`.
            drop(jobs);
            return self
                .catalog
                .settle_team_attempt(
                    &attempt,
                    "cancelled",
                    Some(TeamReservation::default()),
                    None,
                    Some("runtime_shutdown_before_team_worker_start".into()),
                )
                .await;
        }
        let job = Arc::new(Job {
            attempt: attempt.clone(),
            cancellation: self.cancellation.child_token(),
            finished: AtomicBool::new(false),
            handle: Mutex::new(None),
        });
        let runtime = Arc::clone(self);
        let worker = Arc::clone(&job);
        let worker_scope = self.lifecycle_scope.clone();
        let handle = tokio::spawn(async move {
            worker_scope.run_in_scope(async move {
                let _serial = serial;
                let _active = permit;
                runtime
                    .execute(owner, worker.attempt.clone(), worker.cancellation.clone())
                    .await;
                worker.finished.store(true, Ordering::Release);
                drop(_serial);
                drop(_active);
                runtime.catalog.team_execution_notify.notify_one();
                drop(task_lease);
            }).await
        });
        *job.handle.lock().await = Some(handle);
        jobs.insert(attempt.id.clone(), job);
        Ok(attempt)
    }

    pub async fn activate_workflows(self: &Arc<Self>, owner: ActorOwner) {
        self.owners
            .lock()
            .await
            .insert(owner.presentation_owner_key(), owner);
        self.drain().await;
    }

    async fn drain(self: &Arc<Self>) {
        if !self.available || self.cancellation.is_cancelled() {
            return;
        }
        let owners = self
            .owners
            .lock()
            .await
            .values()
            .cloned()
            .collect::<Vec<_>>();
        for owner in owners {
            if let Err(error) = self.drive_workflows(&owner).await {
                tracing::warn!(%error, "Workflow progression awaits reconciliation");
            }
        }
        let queued = match self.catalog.queued_team_attempts().await {
            Ok(v) => v,
            Err(_) => return,
        };
        let mut teams = std::collections::BTreeMap::<String, Vec<TeamExecutionAttempt>>::new();
        for attempt in queued {
            teams
                .entry(format!(
                    "{}:{}:{}",
                    attempt.owner_id, attempt.workspace_id, attempt.team_id
                ))
                .or_default()
                .push(attempt);
        }
        for attempts in teams.values_mut() {
            attempts.sort_by(|a, b| {
                a.queue_sequence
                    .cmp(&b.queue_sequence)
                    .then_with(|| a.created_at.cmp(&b.created_at))
                    .then_with(|| a.id.cmp(&b.id))
            });
        }
        let mut names: Vec<_> = teams.keys().cloned().collect();
        if let Some(last) = self.last_team.lock().await.clone() {
            if let Some(index) = names.iter().position(|name| name == &last) {
                let len = names.len();
                names.rotate_left((index + 1) % len);
            }
        }
        let mut progress = true;
        while progress && self.active.available_permits() > 0 {
            progress = false;
            for name in &names {
                if self.active.available_permits() == 0 {
                    break;
                }
                let Some(attempt) = teams.get_mut(name).and_then(|v| {
                    if v.is_empty() {
                        None
                    } else {
                        Some(v.remove(0))
                    }
                }) else {
                    continue;
                };
                progress = true;
                let owner = self.owners.lock().await.get(&attempt.owner_id).cloned();
                let Some(owner) = owner else {
                    continue;
                };
                match self.dispatch(owner, attempt).await {
                    Ok(a) if a.status == "running" => {
                        *self.last_team.lock().await = Some(name.clone());
                        progress = true;
                    }
                    _ => {}
                }
            }
        }
    }

    pub async fn cancel(
        &self,
        owner: &ActorOwner,
        workspace: &str,
        team: &str,
        attempt: &str,
        request: TeamControlRequest,
    ) -> Result<TeamExecutionAttempt, CollaborationError> {
        let receipt = self
            .catalog
            .cancel_team_attempt(
                &owner.presentation_owner_key(),
                workspace,
                team,
                attempt,
                request,
            )
            .await?;
        if let Some(job) = self.jobs.lock().await.get(attempt) {
            job.cancellation.cancel();
        }
        self.manager
            .cancel_run_for_owner(owner, &receipt.run_id)
            .await;
        Ok(receipt)
    }

    pub async fn revoke_member(&self, owner: &str, workspace: &str, team: &str, member: &str) {
        for job in self.jobs.lock().await.values().filter(|job| {
            job.attempt.owner_id == owner
                && job.attempt.workspace_id == workspace
                && job.attempt.team_id == team
                && job.attempt.member_id == member
        }) {
            job.cancellation.cancel();
        }
    }

    pub async fn recover(
        self: &Arc<Self>,
        owner: ActorOwner,
        workspace: &str,
        team: &str,
        request: TeamControlRequest,
    ) -> Result<Vec<TeamExecutionAttempt>, CollaborationError> {
        if let Some(receipt) = self
            .catalog
            .team_recovery_receipt(&owner.presentation_owner_key(), workspace, team, &request)
            .await?
        {
            return Ok(receipt);
        }
        let jobs = self.jobs.lock().await;
        for job in jobs.values().filter(|job| {
            job.attempt.owner_id == owner.presentation_owner_key()
                && job.attempt.workspace_id == workspace
                && job.attempt.team_id == team
        }) {
            if job.locally_live().await {
                return Err(CollaborationError::Conflict(
                    "Live team attempts must finish cancellation before recovery".into(),
                ));
            }
        }
        let joined_yields = self
            .confirmed_joined_yields(&owner, workspace, team)
            .await?;
        let queued = self
            .catalog
            .recover_team_attempts_control(
                &owner.presentation_owner_key(),
                workspace,
                team,
                request,
                joined_yields,
            )
            .await?;
        drop(jobs);
        let mut dispatched = Vec::new();
        for attempt in queued {
            dispatched.push(self.dispatch(owner.clone(), attempt).await?);
        }
        Ok(dispatched)
    }

    pub async fn shutdown(&self) -> anyhow::Result<()> {
        // Stop local admission before taking the job snapshot. A caller that
        // was already preparing remains an explicit scope obligation; it
        // cannot publish another direct worker after observing cancellation.
        self.lifecycle_scope.close_admission();
        self.cancellation.cancel();
        let mut failures = Vec::new();
        match self.catalog.execution_ownership_view().await {
            Ok(view) if view.owns_execution => {
                if let Err(error) = self.catalog.drain_execution_owner().await {
                    failures.push(format!("team execution owner drain failed: {error}"));
                }
            }
            Ok(_) => {}
            Err(error) => failures.push(format!("team execution owner inspection failed: {error}")),
        }
        let jobs = self.jobs.lock().await.values().cloned().collect::<Vec<_>>();
        for job in &jobs {
            job.cancellation.cancel();
        }
        for job in jobs {
            let mut handle = job.handle.lock().await;
            if let Some(worker) = handle.as_mut() {
                if let Err(error) = worker.await {
                    failures.push(format!("team execution worker join failed: {error}"));
                }
            }
            handle.take();
        }
        let report = self.lifecycle_scope.close().await;
        failures.extend(report.failures);
        failures.extend(
            report
                .unresolved_effects
                .into_iter()
                .map(|effect| format!("team execution unresolved effect: {effect}")),
        );
        if failures.is_empty() {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "team execution shutdown remains unconfirmed: {}",
                failures.join("; ")
            ))
        }
    }
}
