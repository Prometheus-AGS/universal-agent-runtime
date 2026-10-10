//! Trusted root approval channels. A child can request a decision through a
//! captured channel, but cannot choose its root, resolve it, or approve itself.

#[path = "approvals/records.rs"]
mod records;
use records::ApprovalLedger;
use crate::uar::persistence::approval_decisions::{ApprovalRecord, ApprovalState};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, Weak};
use std::time::Duration;

use tokio::sync::{Mutex as AsyncMutex, oneshot};
use tokio_util::sync::CancellationToken;
use serde::Serialize;

use crate::uar::domain::events::{NormalizedEvent, RuntimeEventSink};

/// Resolution is separate from a queued request's lifetime.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ApprovalOutcome {
    Approved,
    Rejected,
    Cancelled,
    TimedOut,
    ChannelClosed,
}

struct PendingApproval {
    snapshot: PendingApprovalSnapshot,
    record: ApprovalRecord,
    legacy_root_request: bool,
    reply: oneshot::Sender<bool>,
}

/// Safe, owner-scoped projection of the one live approval waiter.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PendingApprovalSnapshot {
    pub version: u32,
    pub root_run_id: String,
    pub approval_id: String,
    pub issuer_id: String,
    pub challenge_id: String,
    pub admission_id: Option<String>,
    pub admission_owner: crate::uar::persistence::tool_admission::AdmissionOwner,
    /// Decision routing is distinct from the tool's effect admission ownership.
    pub decision_owner: crate::uar::persistence::tool_admission::AdmissionOwner,
    pub call_index: usize,
    pub tool_call_id: String,
    pub name: String,
    pub arguments_json: String,
    pub risk_reason: String,
}

struct RootLane {
    run_id: String,
    owner_id: String,
    events: Arc<dyn RuntimeEventSink>,
    cancellation: CancellationToken,
    serial: AsyncMutex<()>,
    resolution: AsyncMutex<()>,
    owner_key: String,
    workspace_id: Option<String>,
    ledger: Arc<ApprovalLedger>,
    pending: Mutex<Option<PendingApproval>>,
}

/// Host-only resolver index. Weak entries do not retain completed run emitters.
#[derive(Clone)]
pub(crate) struct ApprovalBroker {
    roots: Arc<Mutex<HashMap<String, Weak<RootLane>>>>,
    ledger: Arc<ApprovalLedger>,
}

impl ApprovalBroker {
    /// Register once for a new host-allocated root run. Descendants inherit the
    /// returned channel rather than registering another root under that ID.
    #[cfg(test)]
    pub(crate) fn register(
        &self,
        run_id: String,
        owner_id: String,
        events: Arc<dyn RuntimeEventSink>,
        cancellation: CancellationToken,
    ) -> anyhow::Result<RootApprovalChannel> {
        let owner_key = format!("v1:s:{}:{}", owner_id.len(), owner_id);
        self.register_scoped(run_id, owner_id, owner_key, None, events, cancellation)
    }

    pub(crate) fn register_scoped(
        &self, run_id: String, owner_id: String, owner_key: String, workspace_id: Option<String>,
        events: Arc<dyn RuntimeEventSink>, cancellation: CancellationToken,
    ) -> anyhow::Result<RootApprovalChannel> {
        let mut roots = self
            .roots
            .lock()
            .map_err(|_| anyhow::anyhow!("Approval index unavailable"))?;
        roots.retain(|_, root| root.strong_count() > 0);
        if roots.get(&run_id).and_then(Weak::upgrade).is_some() {
            anyhow::bail!("Root approval channel already registered");
        }
        let lane = Arc::new(RootLane {
            run_id: run_id.clone(),
            owner_id,
            events,
            cancellation,
            serial: AsyncMutex::new(()),
            resolution: AsyncMutex::new(()),
            owner_key, workspace_id, ledger: Arc::clone(&self.ledger),
            pending: Mutex::new(None),
        });
        roots.insert(run_id, Arc::downgrade(&lane));
        Ok(RootApprovalChannel {
            lane,
            legacy_root_request: true,
        })
    }

    pub(crate) fn pending(
        &self,
        owner_id: &str,
        run_id: &str,
    ) -> Option<PendingApprovalSnapshot> {
        let lane = self
            .roots
            .lock()
            .ok()
            .and_then(|roots| roots.get(run_id).and_then(Weak::upgrade))?;
        if lane.owner_id != owner_id || lane.cancellation.is_cancelled() {
            return None;
        }
        lane.pending
            .lock()
            .ok()
            .and_then(|pending| pending.as_ref().map(|pending| pending.snapshot.clone()))
    }
}

/// A request-only capability bound to one root emitter and cancellation token.
/// It has no deserializer or reference to the host's resolution API.
#[derive(Clone)]
pub(crate) struct RootApprovalChannel {
    lane: Arc<RootLane>,
    legacy_root_request: bool,
}

impl RootApprovalChannel {
    /// Identity only; this does not expose the human-resolution capability.
    pub(crate) fn root_run_id(&self) -> &str {
        &self.lane.run_id
    }

    /// A descendant keeps the same root queue but cannot accept a run-only
    /// decision that might have been intended for another child.
    pub(crate) fn for_child(&self) -> Self {
        Self {
            lane: Arc::clone(&self.lane),
            legacy_root_request: false,
        }
    }

    /// The deadline bounds queueing, publication, and the human decision; an
    /// in-flight persistence write is allowed to settle before cancellation. A
    /// dropped gate clears its own slot synchronously before another caller
    /// acquires the queue, so cancellation cannot leave an approvable orphan.
    pub(crate) async fn request(
        &self,
        admission_id: Option<String>,
        call_index: usize,
        tool_call_id: String,
        name: String,
        arguments_json: String,
        risk_reason: String,
        caller_cancel: &CancellationToken,
    ) -> ApprovalOutcome {
        self.request_with_admission_owner(
            admission_id,
            crate::uar::persistence::tool_admission::AdmissionOwner::PairedHost,
            crate::uar::persistence::tool_admission::AdmissionOwner::PairedHost,
            call_index,
            tool_call_id,
            name,
            arguments_json,
            risk_reason,
            caller_cancel,
        )
        .await
    }

    pub(crate) async fn request_with_admission_owner(
        &self,
        admission_id: Option<String>,
        admission_owner: crate::uar::persistence::tool_admission::AdmissionOwner,
        decision_owner: crate::uar::persistence::tool_admission::AdmissionOwner,
        call_index: usize,
        tool_call_id: String,
        name: String,
        arguments_json: String,
        risk_reason: String,
        caller_cancel: &CancellationToken,
    ) -> ApprovalOutcome {
        let deadline = tokio::time::Instant::now() + Duration::from_secs(300);
        let expires_at = Utc::now() + chrono::Duration::seconds(300);
        let _serial = tokio::select! {
            biased;
            _ = self.lane.cancellation.cancelled() => return ApprovalOutcome::Cancelled,
            _ = caller_cancel.cancelled() => return ApprovalOutcome::Cancelled,
            result = tokio::time::timeout_at(deadline, self.lane.serial.lock()) => {
                match result {
                    Ok(serial) => serial,
                    Err(_) => return ApprovalOutcome::TimedOut,
                }
            }
        };
        let id = uuid::Uuid::new_v4().to_string();
        let created_at = Utc::now();
        let record = ApprovalRecord {
            version: 1, issuer_id: self.lane.ledger.issuer_id.clone(), challenge_id: id.clone(),
            owner_key: self.lane.owner_key.clone(), workspace_id: self.lane.workspace_id.clone(),
            root_run_id: self.lane.run_id.clone(), admission_id: admission_id.clone(), admission_owner,
            tool_call_id: tool_call_id.clone(), tool_name: name.clone(), created_at,
            expires_at, state: ApprovalState::Pending,
            decision: None, updated_at: created_at,
        };
        // Finish the storage write before observing cancellation: dropping a
        // database request can leave an inserted challenge with unknown status.
        if self.lane.ledger.create(&record).await.is_err() {
            tracing::error!("Approval challenge could not be persisted");
            self.lane.finish_record(&record, ApprovalState::Interrupted).await;
            return ApprovalOutcome::ChannelClosed;
        }
        let operation = async {
            let (reply, receiver) = oneshot::channel();
            let snapshot = PendingApprovalSnapshot {
                version: 1,
                root_run_id: self.lane.run_id.clone(),
                approval_id: id.clone(),
                issuer_id: record.issuer_id.clone(),
                challenge_id: record.challenge_id.clone(),
                admission_id: admission_id.clone(),
                admission_owner,
                decision_owner,
                call_index,
                tool_call_id: tool_call_id.clone(),
                name: name.clone(),
                arguments_json: arguments_json.clone(),
                risk_reason: risk_reason.clone(),
            };
            {
                let _publication = self.lane.resolution.lock().await;
                let Ok(mut pending) = self.lane.pending.lock() else {
                    return ApprovalOutcome::ChannelClosed;
                };
                if pending.is_some() {
                    return ApprovalOutcome::ChannelClosed;
                }
                *pending = Some(PendingApproval {
                    snapshot,
                    record: record.clone(),
                    legacy_root_request: self.legacy_root_request,
                    reply,
                });
            }
            let _pending = PendingGuard {
                lane: Arc::clone(&self.lane),
                id: id.clone(),
            };
            self.lane.publish_record(&record).await;
            // Register before publication: an immediate human response must
            // not race a yet-to-be-inserted sender.
            self.lane
                .events
                .emit(NormalizedEvent::ToolCallApprovalRequired {
                    run_id: self.lane.run_id.clone(),
                    call_index,
                    tool_call_id,
                    name,
                    arguments_json,
                    risk_reason,
                    approval_id: Some(id.clone()),
                    admission_id,
                })
                .await;
            match receiver.await {
                Ok(true) => ApprovalOutcome::Approved,
                Ok(false) => ApprovalOutcome::Rejected,
                Err(_) => ApprovalOutcome::ChannelClosed,
            }
        };
        let outcome = tokio::select! {
            biased;
            _ = self.lane.cancellation.cancelled() => ApprovalOutcome::Cancelled,
            _ = caller_cancel.cancelled() => ApprovalOutcome::Cancelled,
            result = tokio::time::timeout_at(deadline, operation) => {
                result.unwrap_or(ApprovalOutcome::TimedOut)
            }
        };
        let terminal = match outcome {
            ApprovalOutcome::Cancelled => Some(ApprovalState::Cancelled),
            ApprovalOutcome::TimedOut => Some(ApprovalState::Expired),
            ApprovalOutcome::ChannelClosed => Some(ApprovalState::Interrupted),
            _ => None,
        };
        if let Some(state) = terminal { self.lane.finish_record(&record, state).await; }
        outcome
    }
}

struct PendingGuard {
    lane: Arc<RootLane>,
    id: String,
}

impl Drop for PendingGuard {
    fn drop(&mut self) {
        if let Ok(mut pending) = self.lane.pending.lock()
            && pending
                .as_ref()
                .is_some_and(|request| request.snapshot.approval_id == self.id)
        {
            pending.take();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct RecordingSink {
        events: AsyncMutex<Vec<NormalizedEvent>>,
    }

    #[async_trait::async_trait]
    impl RuntimeEventSink for RecordingSink {
        async fn emit(&self, event: NormalizedEvent) {
            self.events.lock().await.push(event);
        }
    }

    #[tokio::test]
    async fn child_request_uses_root_lane_and_requires_its_exact_approval_id() {
        let broker = ApprovalBroker::default();
        let sink = Arc::new(RecordingSink::default());
        let root = broker
            .register(
                "root-run".to_owned(),
                sink.clone(),
                CancellationToken::new(),
            )
            .expect("root approval lane must register");
        let child = root.for_child();
        let caller_cancel = CancellationToken::new();
        let request = tokio::spawn(async move {
            child
                .request(
                    2,
                    "child-call".to_owned(),
                    "write".to_owned(),
                    r#"{"path":"scoped"}"#.to_owned(),
                    "write effect".to_owned(),
                    &caller_cancel,
                )
                .await
        });

        let approval_id = tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                let events = sink.events.lock().await;
                if let Some(approval_id) = events.iter().find_map(|event| match event {
                    NormalizedEvent::ToolCallApprovalRequired {
                        run_id,
                        tool_call_id,
                        approval_id,
                        ..
                    } if run_id == "root-run" && tool_call_id == "child-call" => {
                        approval_id.clone()
                    }
                    _ => None,
                }) {
                    return approval_id;
                }
                drop(events);
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("child approval request must be published on the root lane");

        assert!(!broker.resolve("root-run", None, true));
        assert!(!request.is_finished());
        assert!(broker.resolve("root-run", Some(&approval_id), true));
        assert_eq!(
            request.await.expect("child approval task must join"),
            ApprovalOutcome::Approved
        );
    }
}
