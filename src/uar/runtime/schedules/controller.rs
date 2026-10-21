use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
use chrono::Utc;
use crate::uar::{
    compiler::collaboration::CollaborationCatalogService,
    domain::schedules::{DurableSchedule, ScheduleGuard, ScheduleReceipt, ScheduleRequest, ScheduleView},
    persistence::accounting::AccountingState,
    runtime::{actor::messages::ActorOwner, instance::{AgentInstanceController, InstanceCommandStatus, InstanceRecovery}},
    security::claims::ScheduleOwnerGrant,
};
use super::{calendar, ScheduleError};

/// Trusted host control plane. The observer wakeup invokes `scan_once`; it
/// never executes a model or effect directly.
pub struct ScheduleController {
    catalog: Arc<CollaborationCatalogService>,
    instances: Arc<AgentInstanceController>,
    scan_offset: AtomicUsize,
    // Serializes receipt cleanup with replay in this owning process. Cross-host
    // execution authority is the durable supervisor fence, never this mutex.
    scan_lock: tokio::sync::Mutex<()>,
}

impl std::fmt::Debug for ScheduleController {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Avoid traversing the private catalog and runtime instance graph.
        f.debug_struct("ScheduleController").finish_non_exhaustive()
    }
}

impl ScheduleController {
    pub fn new(catalog: Arc<CollaborationCatalogService>, instances: Arc<AgentInstanceController>) -> Arc<Self> {
        Arc::new(Self { catalog, instances, scan_offset: AtomicUsize::new(0), scan_lock: tokio::sync::Mutex::new(()) })
    }

    fn require_store(&self) -> Result<(), ScheduleError> {
        self.catalog.require_schedule_store()?;
        self.instances.require_store().map_err(|_| ScheduleError::Unsupported)?;
        if !self.instances.store.supports_durable_accounting() || !self.instances.store.supports_durable_observers() { return Err(ScheduleError::Unsupported); }
        Ok(())
    }

    pub fn supported(&self) -> bool { self.require_store().is_ok() }

    /// The next actual business-time boundary. This does not poll or certify
    /// execution; the scan after the one-shot wakeup rechecks durable state.
    pub async fn next_due_at(&self) -> Result<Option<chrono::DateTime<Utc>>, ScheduleError> {
        self.require_store()?;
        let now = Utc::now();
        self.catalog
            .schedule_records()
            .await?
            .iter()
            .try_fold(None, |next, record| {
                let candidate = calendar::next_due_after(record, now)?;
                Ok(match (next, candidate) {
                    (Some(current), Some(candidate)) => Some(current.min(candidate)),
                    (current, None) => current,
                    (None, candidate) => candidate,
                })
            })
    }

    async fn destination(&self, owner: &ActorOwner, workspace: &str, instance: &str) -> Result<(), ScheduleError> {
        // Management can run on a non-owning runtime. Read without triggering
        // instance orphan recovery; only the fenced scanner may do that.
        let record = self.instances.store.load_agent_instance(&owner.presentation_owner_key(), workspace, instance)
            .await.map_err(|_| ScheduleError::Storage)?.ok_or(ScheduleError::Denied)?;
        record.validate(&owner.presentation_owner_key(), workspace).map_err(|_| ScheduleError::Denied)?;
        let bound = self.catalog.resolve_bound_agent_run(&record.owner_id, workspace, &record.effective_binding.id)
            .await.map_err(|error| match error {
                crate::uar::compiler::collaboration::CollaborationError::Storage(_) => ScheduleError::Storage,
                _ => ScheduleError::Denied,
            })?;
        if bound.binding.package != record.definition || bound.effective_binding_receipt.binding_ref != record.effective_binding {
            return Err(ScheduleError::Denied);
        }
        Ok(())
    }

    pub async fn create(&self, owner: &ActorOwner, workspace: &str, request: ScheduleRequest) -> Result<ScheduleView, ScheduleError> {
        self.require_store()?;
        let key = owner_key(owner)?;
        validate_request(&request, workspace)?;
        let now = Utc::now();
        calendar::validate(&request.recurrence, now)?;
        self.destination(owner, workspace, &request.instance_id).await?;
        let id = uuid::Uuid::new_v4().to_string();
        let grant = ScheduleOwnerGrant::capture(owner.credential_principal().ok_or(ScheduleError::Denied)?, &id, workspace, &request.instance_id);
        let record = DurableSchedule { id, owner_key: key, owner_grant: grant,
            workspace_id: workspace.into(), instance_id: request.instance_id,
            next_local_date: calendar::initial_date(&request.recurrence, now)?, recurrence: request.recurrence,
            prompt: request.prompt, revision: 1, paused: false, cancelled: false, effective_at: now,
            once_scanned: false, pending: None, guard: None, skipped: vec![], receipts: vec![], created_at: now, updated_at: now };
        self.catalog.create_schedule_record(record).await.map(|r| ScheduleView::from(&r))
    }

    pub async fn list(&self, owner: &ActorOwner, workspace: &str) -> Result<Vec<ScheduleView>, ScheduleError> {
        self.require_store()?;
        let key = owner_key(owner)?;
        Ok(self.catalog.schedule_records().await?.iter().filter(|r| r.owner_key == key && r.workspace_id == workspace)
            .map(ScheduleView::from).collect())
    }

    pub async fn get(&self, owner: &ActorOwner, workspace: &str, id: &str) -> Result<ScheduleView, ScheduleError> {
        self.require_store()?;
        self.catalog.schedule_record(&owner_key(owner)?, workspace, id).await.map(|r| ScheduleView::from(&r))
    }

    pub async fn revise(&self, owner: &ActorOwner, workspace: &str, id: &str, expected_revision: u64,
        request: ScheduleRequest,
    ) -> Result<ScheduleView, ScheduleError> {
        self.require_store()?;
        let key = owner_key(owner)?;
        validate_request(&request, workspace)?;
        calendar::validate(&request.recurrence, Utc::now())?;
        self.destination(owner, workspace, &request.instance_id).await?;
        let grant = ScheduleOwnerGrant::capture(owner.credential_principal().ok_or(ScheduleError::Denied)?, id, workspace, &request.instance_id);
        self.catalog.change_schedule(&key, workspace, id, false, |r, _| {
            expected(r, expected_revision)?;
            // Each successful CAS establishes the new effective instant; do not
            // backdate a revision to the time a conflicted request began.
            let now = Utc::now();
            calendar::validate(&request.recurrence, now)?;
            cancel_pending(r, "revision_changed");
            r.revision = r.revision.checked_add(1).ok_or(ScheduleError::Conflict("schedule revision exhausted"))?;
            r.owner_grant = grant.clone(); r.instance_id = request.instance_id.clone();
            r.prompt = request.prompt.clone(); r.recurrence = request.recurrence.clone();
            r.next_local_date = calendar::initial_date(&r.recurrence, now)?;
            r.effective_at = now; r.once_scanned = false;
            Ok(true)
        }).await.map(|r| ScheduleView::from(&r))
    }

    pub async fn control(&self, owner: &ActorOwner, workspace: &str, id: &str, expected_revision: u64,
        operation: &str,
    ) -> Result<ScheduleView, ScheduleError> {
        self.require_store()?;
        if !matches!(operation, "pause" | "resume" | "cancel") { return Err(ScheduleError::Invalid("unknown schedule operation")); }
        self.catalog.change_schedule(&owner_key(owner)?, workspace, id, false, |r, _| {
            expected(r, expected_revision)?;
            match operation {
                "pause" => r.paused = true,
                "resume" => r.paused = false,
                _ => { cancel_pending(r, "cancelled"); r.cancelled = true; },
            }
            // Revision changes fence concurrent management requests. Pending
            // intent remains valid across pause/resume and retains its instant.
            r.revision = r.revision.checked_add(1).ok_or(ScheduleError::Conflict("schedule revision exhausted"))?;
            if let Some(p) = &mut r.pending {
                p.revision = r.revision;
                p.id = format!("{}-{}-{}", r.id, r.revision, p.due_at.timestamp());
                p.command_id = format!("schedule-{}", p.id);
            }
            Ok(true)
        }).await.map(|r| ScheduleView::from(&r))
    }

    /// Competing runtimes use the same catalog CAS and supervisor claim.
    pub async fn scan_once(&self) -> Result<(), ScheduleError> {
        let _scan = self.scan_lock.lock().await;
        self.require_store()?;
        if !self.catalog.execution_ownership_view().await.map_err(|_| ScheduleError::Storage)?.owns_execution {
            return Ok(());
        }
        let records = self.catalog.schedule_records().await?;
        if records.is_empty() { return Ok(()); }
        let start = self.scan_offset.fetch_add(128, Ordering::Relaxed) % records.len();
        for offset in 0..records.len().min(128) {
            let record = &records[(start + offset) % records.len()];
            if let Err(error) = self.scan_schedule(record).await {
                tracing::warn!(schedule_id = %record.id, code = error.code(), "Schedule scan deferred");
            }
        }
        Ok(())
    }

    async fn scan_schedule(&self, listed: &DurableSchedule) -> Result<(), ScheduleError> {
        self.cleanup_receipts(listed).await?;
        self.resolve_guard(listed).await?;
        let owner = restored(listed, None)?;
        let authority = self.destination(&owner, &listed.workspace_id, &listed.instance_id).await;
        let record = self.catalog.change_schedule(&listed.owner_key, &listed.workspace_id, &listed.id, true, |r, fence| {
            if r.cancelled { return Ok(false); }
            let before = (r.next_local_date, r.once_scanned,
                r.pending.as_ref().map(|p| p.id.clone()), r.skipped.last().map(|s| (s.last.clone(), s.count)), r.guard.is_some());
            let complete = calendar::scan(r, Utc::now())?;
            if complete && !r.paused && r.guard.is_none() && authority.is_ok()
                && r.revision == listed.revision && r.instance_id == listed.instance_id {
                if let Some(pending) = r.pending.take() {
                    if pending.revision != r.revision { return Err(ScheduleError::Conflict("pending revision changed")); }
                    r.guard = Some(ScheduleGuard { occurrence: pending, instance_id: r.instance_id.clone(),
                        owner_grant: r.owner_grant.clone(), execution_fence: fence.cloned().ok_or(ScheduleError::Denied)?,
                        prompt: r.prompt.clone(), dispatch_started: false, command_observed: false,
                        run_id: None, claimed_at: Utc::now(), last_error: None });
                }
            }
            Ok(before != (r.next_local_date, r.once_scanned,
                r.pending.as_ref().map(|p| p.id.clone()), r.skipped.last().map(|s| (s.last.clone(), s.count)), r.guard.is_some()))
        }).await?;
        if record.guard.is_some() { self.dispatch(&record).await?; }
        Ok(())
    }

    async fn dispatch(&self, record: &DurableSchedule) -> Result<(), ScheduleError> {
        let Some(guard) = record.guard.as_ref() else { return Ok(()); };
        let owner = restored(record, Some(guard))?;
        self.destination(&owner, &record.workspace_id, &guard.instance_id).await?;
        let instance = self.instances.load(&owner, &record.workspace_id, &guard.instance_id).await.map_err(|_| ScheduleError::Storage)?;
        let command = instance.inbox.iter().find(|c| c.command_id == guard.occurrence.command_id);
        if command.is_some_and(|c| c.schedule_id.as_deref() != Some(record.id.as_str())) {
            return Err(ScheduleError::Conflict("schedule command provenance mismatch"));
        }
        if command.is_some_and(|c| c.status != InstanceCommandStatus::Accepted) { return Ok(()); }
        if command.is_none() && guard.command_observed { return Err(ScheduleError::Conflict("schedule command receipt is missing")); }
        let safely_queued = command.is_none_or(|c| c.attempt_id.is_none() && c.root_run_id.is_none())
            && instance.active_attempt.as_ref().is_none_or(|a| a.command_id != guard.occurrence.command_id);
        let claimed = self.catalog.change_schedule(&record.owner_key, &record.workspace_id, &record.id, true, |r, fence| {
            let current = matching_guard(r, &guard.occurrence.id)?;
            let fence = fence.ok_or(ScheduleError::Denied)?;
            // Supervisor reclaim excludes the predecessor, but only an intent
            // never submitted may acquire a different execution fence.
            if current.execution_fence != *fence {
                if current.dispatch_started && !safely_queued { return Err(ScheduleError::Conflict("prior scheduled execution requires reconciliation")); }
                current.execution_fence = fence.clone();
            }
            current.dispatch_started = true;
            Ok(true)
        }).await?;
        let guard = claimed.guard.as_ref().ok_or(ScheduleError::Conflict("schedule guard disappeared"))?;
        // Host-only inbox provenance pins this command's dedupe receipt. Replay
        // after a crash is the same command, never a new logical occurrence.
        let result = self.instances.submit_scheduled(&owner, &record.workspace_id, &record.id, guard).await;
        self.catalog.change_schedule(&record.owner_key, &record.workspace_id, &record.id, true, |r, _| {
            let current = matching_guard(r, &guard.occurrence.id)?;
            current.command_observed |= result.is_ok();
            current.last_error = result.as_ref().err().map(|_| "instance_admission_deferred".into());
            Ok(true)
        }).await?;
        result.map(|_| ()).map_err(|_| ScheduleError::Conflict("instance admission deferred"))
    }

    async fn resolve_guard(&self, record: &DurableSchedule) -> Result<(), ScheduleError> {
        let Some(guard) = record.guard.as_ref() else { return Ok(()); };
        let owner = restored(record, Some(guard))?;
        let instance = self.instances.load(&owner, &record.workspace_id, &guard.instance_id).await.map_err(|_| ScheduleError::Storage)?;
        let Some(command) = instance.inbox.iter().find(|c| c.command_id == guard.occurrence.command_id) else { return Ok(()); };
        if command.schedule_id.as_deref() != Some(record.id.as_str()) { return Err(ScheduleError::Conflict("schedule command provenance mismatch")); }
        self.catalog.change_schedule(&record.owner_key, &record.workspace_id, &record.id, true, |r, _| {
            let current = matching_guard(r, &guard.occurrence.id)?;
            let error = (command.status == InstanceCommandStatus::Uncertain).then(|| "execution_or_effect_uncertain".to_owned());
            if current.command_observed && current.run_id == command.root_run_id && current.last_error == error { return Ok(false); }
            current.command_observed = true; current.run_id = command.root_run_id.clone(); current.last_error = error;
            Ok(true)
        }).await?;
        let status = match command.status {
            InstanceCommandStatus::Completed => "completed",
            InstanceCommandStatus::Failed => "failed",
            InstanceCommandStatus::Cancelled => "cancelled",
            _ => return Ok(()),
        };
        if instance.recovery != InstanceRecovery::Ready || instance.active_attempt.as_ref().is_some_and(|a| a.command_id == command.command_id) { return Ok(()); }
        if let Some(run_id) = &command.root_run_id {
            if self.instances.has_unsettled_effect_claim(&owner, run_id).await.map_err(|_| ScheduleError::Storage)? { return Ok(()); }
            let calls = self.instances.store.list_accounting_calls(&record.owner_key, run_id).await.map_err(|_| ScheduleError::Storage)?;
            if calls.iter().any(|call| call.state != AccountingState::Settled) { return Ok(()); }
        }
        self.catalog.change_schedule(&record.owner_key, &record.workspace_id, &record.id, true, |r, _| {
            matching_guard(r, &guard.occurrence.id)?;
            r.receipts.push(ScheduleReceipt { occurrence: guard.occurrence.clone(), run_id: command.root_run_id.clone(),
                disposition: status.into(), resolved_at: Utc::now(), instance_id: guard.instance_id.clone(),
                owner_grant: guard.owner_grant.clone(), inbox_released: false });
            r.guard = None;
            Ok(true)
        }).await?;
        Ok(())
    }

    async fn cleanup_receipts(&self, record: &DurableSchedule) -> Result<(), ScheduleError> {
        for receipt in record.receipts.iter().filter(|r| !r.inbox_released) {
            let owner = ActorOwner::from_schedule_grant(&receipt.owner_grant, &record.owner_key, &record.id,
                &record.workspace_id, &receipt.instance_id).map_err(|_| ScheduleError::Denied)?;
            self.instances.release_schedule_command(&owner, &record.workspace_id, &receipt.instance_id,
                &record.id, &receipt.occurrence.command_id).await.map_err(|_| ScheduleError::Storage)?;
            self.catalog.change_schedule(&record.owner_key, &record.workspace_id, &record.id, true, |r, _| {
                let receipt = r.receipts.iter_mut().find(|r| r.occurrence.id == receipt.occurrence.id)
                    .ok_or(ScheduleError::Conflict("schedule receipt disappeared"))?;
                receipt.inbox_released = true;
                Ok(true)
            }).await?;
        }
        Ok(())
    }
}

fn owner_key(owner: &ActorOwner) -> Result<String, ScheduleError> {
    owner.credential_principal().map(|p| p.storage_key()).ok_or(ScheduleError::Denied)
}
fn restored(record: &DurableSchedule, guard: Option<&ScheduleGuard>) -> Result<ActorOwner, ScheduleError> {
    let (grant, instance) = guard.map(|g| (&g.owner_grant, &g.instance_id)).unwrap_or((&record.owner_grant, &record.instance_id));
    ActorOwner::from_schedule_grant(grant, &record.owner_key, &record.id, &record.workspace_id, instance).map_err(|_| ScheduleError::Denied)
}
fn validate_request(request: &ScheduleRequest, workspace: &str) -> Result<(), ScheduleError> {
    if workspace.trim().is_empty() || workspace.len() > 256 || request.instance_id.trim().is_empty()
        || request.instance_id.len() > 256 || request.prompt.trim().is_empty()
        || serde_json::to_vec(&serde_json::json!({"prompt": request.prompt})).map_err(|_| ScheduleError::Invalid("invalid schedule input"))?.len()
            > crate::uar::persistence::agent_instances::MAX_INSTANCE_COMMAND_BYTES {
        return Err(ScheduleError::Invalid("schedule workspace, destination or prompt is invalid"));
    }
    Ok(())
}
fn expected(record: &DurableSchedule, revision: u64) -> Result<(), ScheduleError> {
    if record.cancelled || record.revision != revision { return Err(ScheduleError::Conflict("schedule is cancelled or revision changed")); }
    Ok(())
}
fn cancel_pending(record: &mut DurableSchedule, reason: &str) {
    if let Some(pending) = record.pending.take() { calendar::skipped(record, reason, pending.due_at.to_rfc3339()); }
}
fn matching_guard<'a>(record: &'a mut DurableSchedule, id: &str) -> Result<&'a mut ScheduleGuard, ScheduleError> {
    record.guard.as_mut().filter(|g| g.occurrence.id == id).ok_or(ScheduleError::Conflict("schedule occurrence changed"))
}
