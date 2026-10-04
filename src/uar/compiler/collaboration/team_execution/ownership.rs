//! One nonexpiring catalog execution authority. Local permits never grant authority.
use super::{CollaborationCatalogService, CollaborationError, MAX_CAS_ATTEMPTS};
use crate::uar::domain::{collaboration::CollaborationCatalogState, team_execution::*};
use chrono::Utc;
use uuid::Uuid;

fn conflict(code: &str) -> CollaborationError { CollaborationError::Conflict(code.into()) }

pub(super) fn require_attempt_fence(
    state: &CollaborationCatalogState, attempt: &TeamExecutionAttempt, cleanup: bool,
) -> Result<(), CollaborationError> {
    let admitted = attempt.execution_fence.as_ref().ok_or_else(|| conflict("TEAM_EXECUTION_EPOCH_STALE"))?;
    let effective = state.execution_transfers.get(&attempt.id).unwrap_or(admitted);
    let claim = state.execution_claim.as_ref().ok_or_else(|| conflict("TEAM_EXECUTION_OWNER_CONFLICT"))?;
    if &claim.fence != effective || (claim.state != "held" && !(cleanup && claim.state == "draining")) {
        return Err(conflict("TEAM_EXECUTION_EPOCH_STALE"));
    }
    Ok(())
}

impl CollaborationCatalogService {
    pub fn execution_identity(&self) -> Result<TeamExecutionFence, CollaborationError> {
        self.execution_identity.read().map(|f| f.clone()).map_err(|_| CollaborationError::Storage("execution identity lock poisoned".into()))
    }

    pub(in crate::uar::compiler::collaboration) fn require_execution_owner(&self, state: &CollaborationCatalogState, cleanup: bool) -> Result<TeamExecutionFence, CollaborationError> {
        let identity = self.execution_identity()?;
        let claim = state.execution_claim.as_ref().ok_or_else(|| conflict("TEAM_EXECUTION_OWNER_CONFLICT"))?;
        if identity != claim.fence || (claim.state != "held" && !(cleanup && claim.state == "draining")) {
            return Err(conflict("TEAM_EXECUTION_OWNER_CONFLICT"));
        }
        Ok(identity)
    }

    pub async fn execution_ownership_view(&self) -> Result<TeamExecutionOwnershipView, CollaborationError> {
        let state = self.load_state().await?;
        Ok(TeamExecutionOwnershipView { claim: state.execution_claim.clone(), current_fence: self.execution_identity()?, owns_execution: self.require_execution_owner(&state, false).is_ok() })
    }

    pub async fn acquire_execution_owner(&self) -> Result<TeamExecutionClaim, CollaborationError> {
        for _ in 0..MAX_CAS_ATTEMPTS {
            let current = self.load_state().await?;
            let mut identity = self.execution_identity()?;
            if let Some(prior) = &current.execution_claim {
                if prior.fence == identity && prior.state == "held" { return Ok(prior.clone()); }
                if prior.state != "released" { return Err(conflict("TEAM_EXECUTION_OWNER_CONFLICT")); }
                identity.epoch = prior.fence.epoch.checked_add(1).ok_or_else(|| conflict("TEAM_EXECUTION_EPOCH_STALE"))?;
            } else { identity.epoch = 1; }
            let claim = TeamExecutionClaim { fence: identity.clone(), state: "held".into(), acquired_at: Utc::now(), audit_receipt_id: Uuid::new_v4().to_string(), released_at: None };
            let mut next = current.clone(); next.generation += 1; next.execution_claim = Some(claim.clone());
            next.execution_claim_history.insert(claim.audit_receipt_id.clone(), claim.clone());
            if let Some(prior) = &current.execution_claim {
                for item in next.team_execution_attempts.values().filter(|a| a.status == "queued" && current.execution_transfers.get(&a.id).or(a.execution_fence.as_ref()) == Some(&prior.fence)) { next.execution_transfers.insert(item.id.clone(), identity.clone()); }
            }
            if self.cas(current.generation, &next).await? {
                *self.execution_identity.write().map_err(|_| CollaborationError::Storage("execution identity lock poisoned".into()))? = identity;
                return Ok(claim);
            }
        }
        Err(conflict("TEAM_EXECUTION_OWNER_CONFLICT"))
    }

    pub async fn drain_execution_owner(&self) -> Result<(), CollaborationError> {
        for _ in 0..MAX_CAS_ATTEMPTS {
            let current = self.load_state().await?;
            self.require_execution_owner(&current, true)?;
            let mut next = current.clone(); next.generation += 1;
            if let Some(claim) = next.execution_claim.as_mut() { claim.state = "draining".into(); claim.audit_receipt_id = Uuid::new_v4().to_string(); next.execution_claim_history.insert(claim.audit_receipt_id.clone(), claim.clone()); }
            if self.cas(current.generation, &next).await? { return Ok(()); }
        }
        Err(conflict("TEAM_EXECUTION_OWNER_CONFLICT"))
    }

    /// Called only after the trusted runtime joined its exact owned roots/children.
    pub(crate) async fn record_joined_execution_owner(&self) -> Result<TeamExecutionFencingEvidence, CollaborationError> {
        for _ in 0..MAX_CAS_ATTEMPTS {
            let current = self.load_state().await?;
            let fence = self.require_execution_owner(&current, true)?;
            if current.execution_claim.as_ref().is_none_or(|c| c.state != "draining") { return Err(conflict("TEAM_EFFECTS_UNCERTAIN")); }
            let evidence = TeamExecutionFencingEvidence { id: Uuid::new_v4().to_string(), fence, root_and_children_stopped: true, source: "trusted-runtime-joined".into(), recorded_at: Utc::now() };
            let mut next = current.clone(); next.generation += 1;
            // Joining execution excludes future effects, not already-issued effects.
            // Preserve the latter as uncertain without preventing a safe owner stop.
            for attempt in next.team_execution_attempts.values_mut() {
                if current.execution_transfers.get(&attempt.id).or(attempt.execution_fence.as_ref()) == Some(&evidence.fence)
                    && matches!(attempt.status.as_str(), "running" | "cancellation_requested") {
                    attempt.status = "uncertain".into();
                    attempt.effect_disposition = "uncertain".into();
                    attempt.accounting_state = "reserved-unknown".into();
                    attempt.state_reason = Some("TEAM_EFFECTS_UNCERTAIN".into());
                    attempt.updated_at = evidence.recorded_at;
                    if let Some(team) = next.team_instances.get_mut(&super::team_key(&attempt.owner_id, &attempt.workspace_id, &attempt.team_id)) {
                        if let Some(task) = team.tasks.iter_mut().find(|task| task.id == attempt.task_id && task.ownership_epoch == attempt.ownership_epoch) {
                            task.status = "blocked".into(); task.state_reason = Some("TEAM_EFFECTS_UNCERTAIN".into()); task.revision += 1; task.updated_at = evidence.recorded_at;
                        }
                        team.revision += 1; team.updated_at = evidence.recorded_at;
                    }
                }
            }
            next.execution_fencing_evidence.insert(evidence.id.clone(), evidence.clone());
            if self.cas(current.generation, &next).await? { return Ok(evidence); }
        }
        Err(conflict("TEAM_EXECUTION_OWNER_CONFLICT"))
    }

    pub async fn release_execution_owner(&self) -> Result<(), CollaborationError> {
        let evidence = self.record_joined_execution_owner().await?;
        for _ in 0..MAX_CAS_ATTEMPTS {
            let current = self.load_state().await?;
            self.require_execution_owner(&current, true)?;
            if current.execution_claim.as_ref().is_none_or(|c| c.fence != evidence.fence || c.state != "draining") { return Err(conflict("TEAM_EXECUTION_EPOCH_STALE")); }
            let mut next = current.clone(); next.generation += 1;
            if let Some(claim) = next.execution_claim.as_mut() { claim.state = "released".into(); claim.released_at = Some(Utc::now()); claim.audit_receipt_id = Uuid::new_v4().to_string(); next.execution_claim_history.insert(claim.audit_receipt_id.clone(), claim.clone()); }
            if self.cas(current.generation, &next).await? { return Ok(()); }
        }
        Err(conflict("TEAM_EXECUTION_OWNER_CONFLICT"))
    }

    /// API caller must establish current privileged host administration first.
    pub(crate) async fn reclaim_execution_owner(&self, actor: &str, decision: &str, request: TeamExecutionReclaimRequest) -> Result<TeamExecutionReclaimReceipt, CollaborationError> {
        for id in [&request.command_id, &request.catalog_id, &request.replacement_service_instance_id, &request.fencing_evidence_ref] { super::super::validation::validate_id(id)?; }
        if request.reason.trim().is_empty() || request.reason.len() > 2048 { return Err(CollaborationError::Invalid("TEAM_RECLAIM_EVIDENCE_REQUIRED".into())); }
        let digest = super::super::validation::canonical_digest(&serde_json::to_value(&request)?)?;
        let receipt_key = format!("{actor}\u{1f}{}", request.command_id);
        for _ in 0..MAX_CAS_ATTEMPTS {
            let current = self.load_state().await?;
            if let Some(prior) = current.execution_reclaim_receipts.get(&receipt_key) {
                if prior.request_digest != digest { return Err(conflict("TEAM_COMMAND_CONFLICT")); }
                return Ok(prior.clone());
            }
            let claim = current.execution_claim.as_ref().ok_or_else(|| conflict("TEAM_EXECUTION_OWNER_CONFLICT"))?;
            if request.catalog_id != claim.fence.catalog_id || request.expected_epoch != claim.fence.epoch { return Err(conflict("TEAM_EXECUTION_EPOCH_STALE")); }
            let evidence = current.execution_fencing_evidence.get(&request.fencing_evidence_ref).filter(|e| e.fence == claim.fence && e.root_and_children_stopped && e.source == "trusted-runtime-joined").ok_or_else(|| conflict("TEAM_RECLAIM_EVIDENCE_REQUIRED"))?;
            let mut replacement = self.execution_identity()?;
            if replacement.service_instance_id != request.replacement_service_instance_id || replacement.catalog_id != request.catalog_id { return Err(conflict("TEAM_RECLAIM_UNAUTHORIZED")); }
            if replacement.incarnation_id == claim.fence.incarnation_id { return Err(conflict("TEAM_EXECUTION_OWNER_CONFLICT")); }
            replacement.epoch = claim.fence.epoch.checked_add(1).ok_or_else(|| conflict("TEAM_EXECUTION_EPOCH_STALE"))?;
            let mut next = current.clone(); let mut transferred = Vec::new(); let mut uncertain = Vec::new();
            for item in next.team_execution_attempts.values_mut() {
                if current.execution_transfers.get(&item.id).or(item.execution_fence.as_ref()) != Some(&claim.fence) { continue; }
                if item.status == "queued" {
                    next.execution_transfers.insert(item.id.clone(), replacement.clone()); transferred.push(item.id.clone());
                } else if matches!(item.status.as_str(), "running" | "cancellation_requested") {
                    item.status = "uncertain".into(); item.effect_disposition = "uncertain".into(); item.accounting_state = "reserved-unknown".into(); item.state_reason = Some("TEAM_EFFECTS_UNCERTAIN".into()); item.updated_at = Utc::now(); uncertain.push(item.id.clone());
                } else if item.status == "uncertain" { uncertain.push(item.id.clone());
                }
            }
            let now = Utc::now();
            let receipt = TeamExecutionReclaimReceipt { command_id: request.command_id.clone(), request_digest: digest.clone(), authenticated_actor_id: actor.into(), reason: request.reason.trim().into(), previous_fence: claim.fence.clone(), replacement_fence: replacement.clone(), authorization_decision_ref: decision.into(), fencing_evidence_ref: evidence.id.clone(), transferred_queued_attempt_ids: transferred, uncertain_attempt_ids: uncertain, committed_at: now };
            next.execution_claim = Some(TeamExecutionClaim { fence: replacement.clone(), state: "held".into(), acquired_at: now, audit_receipt_id: Uuid::new_v4().to_string(), released_at: None });
            if let Some(claim) = &next.execution_claim { next.execution_claim_history.insert(claim.audit_receipt_id.clone(), claim.clone()); }
            next.execution_reclaim_receipts.insert(receipt_key.clone(), receipt.clone()); next.generation += 1;
            if self.cas(current.generation, &next).await? {
                *self.execution_identity.write().map_err(|_| CollaborationError::Storage("execution identity lock poisoned".into()))? = replacement;
                return Ok(receipt);
            }
        }
        Err(conflict("TEAM_EXECUTION_OWNER_CONFLICT"))
    }
}
