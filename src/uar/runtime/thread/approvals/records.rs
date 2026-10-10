use super::{ApprovalBroker, RootApprovalChannel, RootLane};
use crate::uar::persistence::{PersistenceLayer, approval_decisions::{ApprovalRecord, ApprovalRecordView, ApprovalState, ApprovalDecision}};
use crate::uar::domain::events::{NormalizedEvent, StatePatchOp};
use std::{collections::HashMap, sync::{Arc, Mutex}};
use chrono::Utc;

pub(super) struct ApprovalLedger {
    store: Option<Arc<dyn PersistenceLayer>>,
    ephemeral: Mutex<HashMap<String, ApprovalRecord>>,
    pub(super) issuer_id: String,
}

impl Default for ApprovalBroker {
    fn default() -> Self { Self::new(None, uuid::Uuid::new_v4().to_string()) }
}

impl ApprovalBroker {
    pub(crate) fn new(store: Option<Arc<dyn PersistenceLayer>>, issuer_id: String) -> Self {
        Self {
            roots: Arc::new(Mutex::new(HashMap::new())),
            ledger: Arc::new(ApprovalLedger {
                store,
                ephemeral: Mutex::new(HashMap::new()),
                issuer_id,
            }),
        }
    }

    pub(crate) async fn resolve_record(&self, run_id: &str, approval_id: Option<&str>, approved: bool) -> anyhow::Result<Option<(ApprovalRecord, bool)>> {
        // Legacy root provenance cannot authorize a run-only decision.
        if approval_id.is_none_or(|id| id.trim().is_empty()) { return Ok(None); }
        let lane = self.roots.lock().map_err(|_| anyhow::anyhow!("Approval index unavailable"))?
            .get(run_id).and_then(std::sync::Weak::upgrade);
        let Some(lane) = lane else { return Ok(None); };
        let _resolution = lane.resolution.lock().await;
        let record = {
            let pending = lane.pending.lock().map_err(|_| anyhow::anyhow!("Approval request unavailable"))?;
            let Some(request) = pending.as_ref() else { return Ok(None); };
            if lane.cancellation.is_cancelled() || request.caller_cancellation.is_cancelled() || request.reply.is_closed() || request.record.expires_at <= Utc::now() ||
                !approval_id.map_or(request.legacy_root_request, |id| id == request.snapshot.approval_id) {
                return Ok(None);
            }
            request.record.clone()
        };
        let next = record.finish(if approved { ApprovalState::Approved } else { ApprovalState::Denied }, Some(ApprovalDecision {
            decision_id: uuid::Uuid::new_v4().to_string(), actor: lane.owner_key.clone(), approved, decided_at: Utc::now(),
        }));
        if !self.ledger.transition(&record, &next).await? { return Ok(None); }
        let delivered = {
            let mut pending = lane.pending.lock().map_err(|_| anyhow::anyhow!("Approval request unavailable"))?;
            if pending.as_ref().is_some_and(|request| request.snapshot.approval_id == record.challenge_id) {
                pending.take().is_some_and(|request| !lane.cancellation.is_cancelled() && !request.caller_cancellation.is_cancelled() && request.record.expires_at > Utc::now() && request.reply.send(approved).is_ok())
            } else { false }
        };
        lane.publish_record(&next).await;
        Ok(Some((next, delivered)))
    }

    pub(crate) async fn records(
        &self, owner: &str, run: &str, workspace: Option<&str>,
    ) -> anyhow::Result<(bool, Vec<ApprovalRecordView>)> {
        let mut records = self.ledger.list(owner, run).await?;
        records.retain(|record| record.workspace_id.as_deref().is_none_or(|id| Some(id) == workspace));
        records.sort_by(|left, right| left.created_at.cmp(&right.created_at)
            .then_with(|| left.challenge_id.cmp(&right.challenge_id)));
        let lane = self.roots.lock().map_err(|_| anyhow::anyhow!("Approval index unavailable"))?
            .get(run).and_then(std::sync::Weak::upgrade);
        // A different issuer may still be active against shared storage. Reads
        // cannot infer its retirement or manufacture interruption evidence.
        let records = records.into_iter().map(|record| {
            let resolvable = record.state == ApprovalState::Pending
                && record.issuer_id == self.ledger.issuer_id
                && record.expires_at > Utc::now()
                && lane.as_ref().is_some_and(|lane| {
                    lane.owner_key == owner && !lane.cancellation.is_cancelled()
                        && lane.pending.lock().ok().is_some_and(|pending| {
                            pending.as_ref().is_some_and(|request| {
                                request.record == record && !request.caller_cancellation.is_cancelled() && !request.reply.is_closed()
                            })
                        })
                });
            ApprovalRecordView { record, resolvable }
        }).collect();
        Ok((self.ledger.store.as_ref().is_some_and(|store| store.supports_durable_approvals()), records))
    }
}

impl RootApprovalChannel {
    pub(crate) async fn finish_cancelled_root(&self, run_id: &str) -> anyhow::Result<()> {
        if self.lane.run_id != run_id || !self.lane.cancellation.is_cancelled() {
            return Ok(());
        }
        let _resolution = self.lane.resolution.lock().await;
        // The run owner may drop the approval future before its async cleanup.
        // Recover its persisted record after all root-owned execution has drained.
        let records = self.lane.ledger.list(&self.lane.owner_key, run_id).await?;
        for record in records {
            if record.issuer_id != self.lane.ledger.issuer_id || record.state != ApprovalState::Pending {
                continue;
            }
            let next = record.finish(ApprovalState::Cancelled, None);
            if self.lane.ledger.transition(&record, &next).await? {
                self.lane.publish_record(&next).await;
            }
        }
        Ok(())
    }
}

impl RootLane {
    pub(super) async fn publish_record(&self, record: &ApprovalRecord) {
        if let Ok(value) = serde_json::to_value(record) {
            self.events.emit(NormalizedEvent::StatePatch {
                run_id: self.run_id.clone(),
                patch: vec![StatePatchOp { op: "add".to_owned(), path: "/approval".to_owned(), value: Some(value) }],
            }).await;
        }
    }

    pub(super) async fn finish_record(&self, record: &ApprovalRecord, state: ApprovalState) {
        let _resolution = self.resolution.lock().await;
        let next = record.finish(state, None);
        match self.ledger.transition(record, &next).await {
            Ok(true) => self.publish_record(&next).await,
            Ok(false) => {},
            Err(_) => tracing::error!("Approval terminal state could not be persisted"),
        }
    }
}

impl ApprovalLedger {
    pub(super) async fn create(&self, record: &ApprovalRecord) -> anyhow::Result<()> {
        if let Some(store) = &self.store { return store.create_approval_record(record).await; }
        let mut records = self.ephemeral.lock().map_err(|_| anyhow::anyhow!("Approval history unavailable"))?;
        anyhow::ensure!(!records.contains_key(&record.storage_key()), "Approval already exists");
        records.insert(record.storage_key(), record.clone());
        Ok(())
    }

    async fn transition(&self, before: &ApprovalRecord, after: &ApprovalRecord) -> anyhow::Result<bool> {
        if let Some(store) = &self.store { return store.transition_approval_record(before, after).await; }
        before.validate_transition(after)?;
        let mut records = self.ephemeral.lock().map_err(|_| anyhow::anyhow!("Approval history unavailable"))?;
        if records.get(&before.storage_key()) != Some(before) { return Ok(false); }
        records.insert(before.storage_key(), after.clone());
        Ok(true)
    }

    async fn list(&self, owner: &str, run: &str) -> anyhow::Result<Vec<ApprovalRecord>> {
        if let Some(store) = &self.store { return store.list_approval_records(owner, run).await; }
        let records = self.ephemeral.lock().map_err(|_| anyhow::anyhow!("Approval history unavailable"))?;
        Ok(records.values().filter(|record| record.owner_key == owner && record.root_run_id == run).cloned().collect())
    }
}
