//! Trusted root approval channels. A child can request a decision through a
//! captured channel, but cannot choose its root, resolve it, or approve itself.

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
    caller_cancellation: CancellationToken,
    reply: oneshot::Sender<bool>,
}

/// Safe, owner-scoped projection of the one live approval waiter.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PendingApprovalSnapshot {
    pub version: u32,
    pub root_run_id: String,
    pub approval_id: String,
    pub admission_id: Option<String>,
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
    pending: Mutex<Option<PendingApproval>>,
}

/// Host-only resolver index. Weak entries do not retain completed run emitters.
#[derive(Clone, Default)]
pub(crate) struct ApprovalBroker {
    roots: Arc<Mutex<HashMap<String, Weak<RootLane>>>>,
}

impl ApprovalBroker {
    /// Register once for a new host-allocated root run. Descendants inherit the
    /// returned channel rather than registering another root under that ID.
    pub(crate) fn register(
        &self,
        run_id: String,
        owner_id: String,
        events: Arc<dyn RuntimeEventSink>,
        cancellation: CancellationToken,
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
            pending: Mutex::new(None),
        });
        roots.insert(run_id, Arc::downgrade(&lane));
        Ok(RootApprovalChannel { lane })
    }

    /// Called only after the host has authorized the root run's human owner.
    /// Every root and descendant decision requires its exact opaque ID.
    pub(crate) fn resolve(&self, run_id: &str, approval_id: Option<&str>, approved: bool) -> bool {
        let Some(approval_id) = approval_id.filter(|id| !id.trim().is_empty()) else { return false; };
        let lane = self
            .roots
            .lock()
            .ok()
            .and_then(|roots| roots.get(run_id).and_then(Weak::upgrade));
        let Some(lane) = lane else {
            return false;
        };
        if lane.cancellation.is_cancelled() {
            return false;
        }
        let Ok(mut pending) = lane.pending.lock() else {
            return false;
        };
        let Some(request) = pending.as_ref() else {
            return false;
        };
        if approval_id != request.snapshot.approval_id
            || lane.cancellation.is_cancelled()
            || request.caller_cancellation.is_cancelled()
            || request.reply.is_closed()
        {
            return false;
        }
        pending
            .take()
            .is_some_and(|request| request.reply.send(approved).is_ok())
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
            .and_then(|pending| pending.as_ref()
                .filter(|request| !request.caller_cancellation.is_cancelled() && !request.reply.is_closed())
                .map(|pending| pending.snapshot.clone()))
    }
}

/// A request-only capability bound to one root emitter and cancellation token.
/// It has no deserializer or reference to the host's resolution API.
#[derive(Clone)]
pub(crate) struct RootApprovalChannel {
    lane: Arc<RootLane>,
}

impl RootApprovalChannel {
    /// Identity only; this does not expose the human-resolution capability.
    pub(crate) fn root_run_id(&self) -> &str {
        &self.lane.run_id
    }

    /// A descendant keeps the same root queue and exact-identity contract.
    pub(crate) fn for_child(&self) -> Self {
        Self {
            lane: Arc::clone(&self.lane),
        }
    }

    /// The timeout bounds queueing, publication, and the human decision. A
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
        let operation = async {
            let _serial = self.lane.serial.lock().await;
            if self.lane.cancellation.is_cancelled() || caller_cancel.is_cancelled() {
                return ApprovalOutcome::Cancelled;
            }
            let id = uuid::Uuid::new_v4().to_string();
            let (reply, receiver) = oneshot::channel();
            let snapshot = PendingApprovalSnapshot {
                version: 1,
                root_run_id: self.lane.run_id.clone(),
                approval_id: id.clone(),
                admission_id: admission_id.clone(),
                call_index,
                tool_call_id: tool_call_id.clone(),
                name: name.clone(),
                arguments_json: arguments_json.clone(),
                risk_reason: risk_reason.clone(),
            };
            {
                let Ok(mut pending) = self.lane.pending.lock() else {
                    return ApprovalOutcome::ChannelClosed;
                };
                if pending.is_some() {
                    return ApprovalOutcome::ChannelClosed;
                }
                *pending = Some(PendingApproval {
                    snapshot,
                    caller_cancellation: caller_cancel.clone(),
                    reply,
                });
            }
            let _pending = PendingGuard {
                lane: Arc::clone(&self.lane),
                id: id.clone(),
            };
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
                    approval_id: Some(id),
                    admission_id,
                })
                .await;
            match receiver.await {
                Ok(true) => ApprovalOutcome::Approved,
                Ok(false) => ApprovalOutcome::Rejected,
                Err(_) => ApprovalOutcome::ChannelClosed,
            }
        };
        tokio::select! {
            biased;
            _ = self.lane.cancellation.cancelled() => ApprovalOutcome::Cancelled,
            _ = caller_cancel.cancelled() => ApprovalOutcome::Cancelled,
            result = tokio::time::timeout(Duration::from_secs(300), operation) => {
                result.unwrap_or(ApprovalOutcome::TimedOut)
            }
        }
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
#[path = "approval_tests.rs"]
mod tests;
