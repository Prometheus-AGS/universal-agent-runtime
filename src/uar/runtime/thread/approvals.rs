//! Trusted root approval channels. A child can request a decision through a
//! captured channel, but cannot choose its root, resolve it, or approve itself.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, Weak};
use std::time::Duration;

use tokio::sync::{Mutex as AsyncMutex, oneshot};
use tokio_util::sync::CancellationToken;

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
    id: String,
    legacy_root_request: bool,
    reply: oneshot::Sender<bool>,
}

struct RootLane {
    run_id: String,
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
            events,
            cancellation,
            serial: AsyncMutex::new(()),
            pending: Mutex::new(None),
        });
        roots.insert(run_id, Arc::downgrade(&lane));
        Ok(RootApprovalChannel {
            lane,
            legacy_root_request: true,
        })
    }

    /// Called only after the host has authorized the root run's human owner.
    /// Child requests require their exact opaque ID; legacy run-only decisions
    /// remain valid solely for ordinary root requests.
    pub(crate) fn resolve(&self, run_id: &str, approval_id: Option<&str>, approved: bool) -> bool {
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
        let matches = approval_id.map_or(request.legacy_root_request, |id| id == request.id);
        if !matches || request.reply.is_closed() {
            return false;
        }
        pending
            .take()
            .is_some_and(|request| request.reply.send(approved).is_ok())
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

    /// The timeout bounds queueing, publication, and the human decision. A
    /// dropped gate clears its own slot synchronously before another caller
    /// acquires the queue, so cancellation cannot leave an approvable orphan.
    pub(crate) async fn request(
        &self,
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
            {
                let Ok(mut pending) = self.lane.pending.lock() else {
                    return ApprovalOutcome::ChannelClosed;
                };
                if pending.is_some() {
                    return ApprovalOutcome::ChannelClosed;
                }
                *pending = Some(PendingApproval {
                    id: id.clone(),
                    legacy_root_request: self.legacy_root_request,
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
                .is_some_and(|request| request.id == self.id)
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
