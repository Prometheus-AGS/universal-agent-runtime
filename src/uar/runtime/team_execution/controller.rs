use std::{
    collections::BTreeMap,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use tokio::sync::Mutex;
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
    cancellation: CancellationToken,
    jobs: Mutex<BTreeMap<String, Arc<Job>>>,
    members: Mutex<BTreeMap<String, Arc<Mutex<()>>>>,
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
        Arc::new(Self {
            catalog,
            cancellation: manager.root_cancellation_token().child_token(),
            manager,
            persistence,
            available,
            jobs: Mutex::new(BTreeMap::new()),
            members: Mutex::new(BTreeMap::new()),
        })
    }

    pub fn available(&self) -> bool {
        self.available
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
        // A duplicate admission receipt never launches a second model turn.
        if attempt.status != "queued" {
            return Ok(attempt);
        }
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
        let mut jobs = self.jobs.lock().await;
        if jobs.contains_key(&attempt.id) {
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
        // Resolve before claiming dispatch; unavailable resources cannot start a turn.
        let prepared = async {
            self.catalog.resolve_team_member_run(&attempt).await?;
            self.catalog.selected_team_context(&attempt).await?;
            Ok::<_, CollaborationError>(())
        }
        .await;
        if let Err(error) = prepared {
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
        let attempt = self.catalog.claim_team_dispatch(&attempt).await?;
        let job = Arc::new(Job {
            attempt: attempt.clone(),
            cancellation: self.cancellation.child_token(),
            finished: AtomicBool::new(false),
            handle: Mutex::new(None),
        });
        let runtime = Arc::clone(self);
        let worker = Arc::clone(&job);
        let handle = tokio::spawn(async move {
            let _serial = lane.lock().await;
            runtime
                .execute(owner, worker.attempt.clone(), worker.cancellation.clone())
                .await;
            worker.finished.store(true, Ordering::Release);
        });
        *job.handle.lock().await = Some(handle);
        jobs.insert(attempt.id.clone(), job);
        Ok(attempt)
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
        let queued = self
            .catalog
            .recover_team_attempts_control(
                &owner.presentation_owner_key(),
                workspace,
                team,
                request,
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
        self.cancellation.cancel();
        let jobs = self.jobs.lock().await.values().cloned().collect::<Vec<_>>();
        for job in &jobs {
            job.cancellation.cancel();
        }
        let mut failure = None;
        for job in jobs {
            let mut handle = job.handle.lock().await;
            if let Some(worker) = handle.as_mut() {
                if let Err(error) = worker.await {
                    failure.get_or_insert(error);
                }
            }
            handle.take();
        }
        failure.map_or(Ok(()), |error| Err(error.into()))
    }
}
