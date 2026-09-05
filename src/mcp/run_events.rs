//! Run-scoped lifecycle delivery while the trusted host owns MCP operations.
//! No observer task is detached, and observation grants no transport authority.

use std::collections::BTreeMap;
use std::sync::Arc;

use tokio::sync::{Mutex, broadcast};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::uar::domain::events::{NormalizedEvent, RuntimeEventSink};
use crate::uar::runtime::actor::messages::ActorOwner;

use super::binding_cache::{McpBindingError, McpBindingRequest};
use super::lifecycle::McpLifecycleSubscription;
use super::runtime::{McpRuntimeError, McpRuntimeManager};

pub(crate) struct McpRunEvents {
    run_id: String,
    owner: ActorOwner,
    sink: Arc<dyn RuntimeEventSink>,
    delivered: Mutex<BTreeMap<Uuid, u64>>,
}

impl std::fmt::Debug for McpRunEvents {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("McpRunEvents")
            .field("run_id", &self.run_id)
            .finish_non_exhaustive()
    }
}

impl McpRunEvents {
    pub(crate) fn new(run_id: String, owner: ActorOwner, sink: Arc<dyn RuntimeEventSink>) -> Self {
        Self {
            run_id,
            owner,
            sink,
            delivered: Mutex::new(BTreeMap::new()),
        }
    }

    async fn emit(&self, event: NormalizedEvent) {
        if let NormalizedEvent::McpServerStateChanged { lifecycle, .. } = event {
            // Repeated preparation and concurrent tool waits may observe the
            // same binding. Serialize publication and suppress stale snapshots.
            let mut delivered = self.delivered.lock().await;
            if let Some(previous) = delivered.get(&lifecycle.binding_id) {
                if lifecycle.sequence <= *previous {
                    return;
                }
                if lifecycle.sequence > previous.saturating_add(1) {
                    tracing::warn!(run_id = %self.run_id, server = %lifecycle.server,
                        "MCP lifecycle delivery resumes from a newer snapshot; intermediate transitions were not replayed");
                }
            }
            let binding_id = lifecycle.binding_id;
            let sequence = lifecycle.sequence;
            self.sink
                .emit(NormalizedEvent::McpServerStateChanged {
                    run_id: Some(self.run_id.clone()),
                    lifecycle,
                })
                .await;
            delivered.insert(binding_id, sequence);
        }
    }

    async fn resynchronize(&self, subscription: &mut McpLifecycleSubscription, skipped: u64) {
        tracing::warn!(run_id = %self.run_id, skipped,
            "MCP lifecycle receiver lagged; publishing current state, not replaying missed events");
        if let Some(event) = subscription.resynchronize() {
            self.emit(event).await;
        }
    }

    pub(crate) async fn forward<T>(
        &self,
        runtime: &McpRuntimeManager,
        request: &McpBindingRequest,
        operation: impl Future<Output = Result<T, McpRuntimeError>>,
        startup_cancellation: Option<&CancellationToken>,
    ) -> Result<T, McpRuntimeError> {
        if request.owner() != &self.owner {
            return Err(McpBindingError::InvalidBinding {
                server: request.definition().name().to_owned(),
            }
            .into());
        }
        let mut subscription = runtime.observe(request)?;
        let mut operation = Box::pin(operation);
        let mut observing = true;
        let result = loop {
            tokio::select! {
                biased;
                () = async {
                    match startup_cancellation {
                        Some(token) => token.cancelled().await,
                        None => std::future::pending::<()>().await,
                    }
                } => break Err(McpBindingError::Cancelled {
                    server: request.definition().name().to_owned(),
                }.into()),
                result = operation.as_mut() => break result,
                event = subscription.recv(), if observing => match event {
                    Ok(event) => self.emit(event).await,
                    Err(broadcast::error::RecvError::Lagged(skipped)) => {
                        self.resynchronize(&mut subscription, skipped).await;
                    }
                    Err(broadcast::error::RecvError::Closed) => observing = false,
                },
            }
        };
        // Startup cancellation drops its cache refresh guard before the final
        // drain, so the resulting Failed/Cancelled transition can be delivered.
        // Mutating calls never receive startup_cancellation and are not replayed.
        drop(operation);
        for _ in 0..subscription.pending_count() {
            match subscription.try_recv() {
                Ok(event) => self.emit(event).await,
                Err(broadcast::error::TryRecvError::Lagged(skipped)) => {
                    self.resynchronize(&mut subscription, skipped).await;
                    break;
                }
                Err(
                    broadcast::error::TryRecvError::Empty | broadcast::error::TryRecvError::Closed,
                ) => break,
            }
        }
        result
    }
}
