//! SurrealDB-backed live-query bus.
//!
//! Opens one `.select(table).live()` stream per [`EntityTopic`] at startup and
//! forwards `Notification<Value>` events into a per-topic broadcast channel.
//! SSE handlers subscribe to the receiver side. Streams are wrapped in a
//! supervised reconnect loop with exponential backoff to survive transient
//! SurrealDB disconnects.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use futures::StreamExt;
use surrealdb::types::Action;
use surrealdb::{Notification, Surreal, engine::any::Any};
use tokio::sync::{Mutex, broadcast};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, warn};

use super::{
    EntityTopic, LiveAction, LiveEvent, RealtimeBus, RealtimePublishError,
    is_governance_enabled_event,
};

/// Broadcast channel capacity per topic. Slow consumers get a `Lagged` error
/// but the publisher never blocks.
const CHANNEL_CAPACITY: usize = 256;

/// Supervised reconnect bounds for a dropped live stream.
const RECONNECT_INITIAL: Duration = Duration::from_millis(250);
const RECONNECT_MAX: Duration = Duration::from_secs(30);

#[derive(Clone)]
pub struct LiveQueryBus {
    senders: Arc<HashMap<EntityTopic, broadcast::Sender<LiveEvent>>>,
    lifecycle: Arc<LiveQueryLifecycle>,
}

struct LiveQueryLifecycle {
    shutdown: CancellationToken,
    tasks: Mutex<Option<Vec<JoinHandle<()>>>>,
}

impl std::fmt::Debug for LiveQueryBus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LiveQueryBus")
            .field("topics", &self.senders.keys().collect::<Vec<_>>())
            .field("shutting_down", &self.lifecycle.shutdown.is_cancelled())
            .finish()
    }
}

impl LiveQueryBus {
    /// Construct the bus and spawn a supervised task per topic.
    ///
    /// The `db` handle must already have a namespace + database selected. The
    /// constructor returns immediately; live streams start opening in the
    /// background. If a topic's stream cannot be opened, that topic's tasks
    /// retry with exponential backoff — the bus itself stays usable for any
    /// topics whose streams are healthy.
    pub fn start(db: Surreal<Any>) -> Self {
        let mut senders = HashMap::with_capacity(EntityTopic::ALL.len());
        let shutdown = CancellationToken::new();
        let mut tasks = Vec::with_capacity(EntityTopic::ALL.len());
        for topic in EntityTopic::ALL {
            let (tx, _) = broadcast::channel(CHANNEL_CAPACITY);
            senders.insert(*topic, tx.clone());
            let db = db.clone();
            let topic = *topic;
            let topic_shutdown = shutdown.clone();
            tasks.push(tokio::spawn(async move {
                tokio::select! {
                    () = topic_shutdown.cancelled() => {
                        info!(topic = %topic, "live stream supervisor shutting down");
                    }
                    () = supervise_topic(db, topic, tx) => {}
                }
            }));
        }
        Self {
            senders: Arc::new(senders),
            lifecycle: Arc::new(LiveQueryLifecycle {
                shutdown,
                tasks: Mutex::new(Some(tasks)),
            }),
        }
    }

    /// Cancel and join every UAR-owned live-query supervisor.
    pub(crate) async fn shutdown(&self) {
        self.lifecycle.shutdown.cancel();
        let tasks = self.lifecycle.tasks.lock().await.take().unwrap_or_default();
        for task in tasks {
            if let Err(error) = task.await {
                warn!(%error, "live stream supervisor task failed during shutdown");
            }
        }
    }

    /// Subscribe to a topic's broadcast stream.
    pub fn subscribe(&self, topic: EntityTopic) -> Option<broadcast::Receiver<LiveEvent>> {
        self.senders.get(&topic).map(|s| s.subscribe())
    }

    /// Number of active subscribers on a topic (for observability).
    pub fn subscriber_count(&self, topic: EntityTopic) -> usize {
        self.senders
            .get(&topic)
            .map(|s| s.receiver_count())
            .unwrap_or(0)
    }
}

impl RealtimeBus for LiveQueryBus {
    fn subscribe(&self, topic: EntityTopic) -> Option<broadcast::Receiver<LiveEvent>> {
        LiveQueryBus::subscribe(self, topic)
    }

    fn subscriber_count(&self, topic: EntityTopic) -> usize {
        LiveQueryBus::subscriber_count(self, topic)
    }

    fn publish(
        &self,
        event: LiveEvent,
    ) -> std::result::Result<(), RealtimePublishError> {
        let sender = self
            .senders
            .get(&event.topic)
            .ok_or(RealtimePublishError::TopicUnavailable(event.topic))?;
        let _ = sender.send(event);
        Ok(())
    }
}

async fn supervise_topic(db: Surreal<Any>, topic: EntityTopic, tx: broadcast::Sender<LiveEvent>) {
    let mut backoff = RECONNECT_INITIAL;
    loop {
        match run_live_stream(&db, topic, &tx).await {
            Ok(()) => {
                // Stream ended cleanly (host shutdown, table dropped, etc.).
                info!(topic = %topic, "live stream ended cleanly; topic supervisor exiting");
                return;
            }
            Err(err) => {
                let err_str = err.to_string();
                // "table does not exist" is normal early-life for empty tables;
                // log at debug and use the maximum backoff so we don't spam.
                if err_str.contains("does not exist") {
                    debug!(
                        topic = %topic,
                        "table not yet created; waiting for first write"
                    );
                    tokio::time::sleep(RECONNECT_MAX).await;
                } else {
                    warn!(
                        topic = %topic,
                        error = %err,
                        backoff_ms = backoff.as_millis() as u64,
                        "live stream failed; reconnecting"
                    );
                    tokio::time::sleep(backoff).await;
                    backoff = (backoff * 2).min(RECONNECT_MAX);
                }
            }
        }
    }
}

async fn run_live_stream(
    db: &Surreal<Any>,
    topic: EntityTopic,
    tx: &broadcast::Sender<LiveEvent>,
) -> Result<()> {
    let mut stream = db
        .select::<Vec<surrealdb::types::Value>>(topic.table())
        .live()
        .await?;
    info!(topic = %topic, table = %topic.table(), "live stream opened");

    while let Some(notification) = stream.next().await {
        let notification: Notification<surrealdb::types::Value> = match notification {
            Ok(n) => n,
            Err(err) => {
                warn!(topic = %topic, error = %err, "notification error");
                return Err(anyhow::anyhow!(err));
            }
        };

        let action = match notification.action {
            Action::Create => LiveAction::Create,
            Action::Update => LiveAction::Update,
            Action::Delete => LiveAction::Delete,
            _ => continue,
        };

        let json = match crate::uar::persistence::providers::surreal::value_to_json(
            notification.data.clone(),
        ) {
            Ok(v) => v,
            Err(err) => {
                debug!(topic = %topic, error = %err, "could not serialize notification payload");
                continue;
            }
        };
        let id = json
            .get("id")
            .and_then(|v| v.as_str())
            .map(str::to_string)
            .unwrap_or_default();

        let event = LiveEvent {
            action,
            topic,
            id,
            data: json,
        };

        // Governance publishes this event explicitly only after the coherent
        // runtime snapshot has advanced. Forwarding the database notification
        // here could make clients refetch the old snapshot in the write→publish
        // interval.
        if is_governance_enabled_event(&event) {
            continue;
        }

        // `send` only errors when there are no subscribers — that's fine, just
        // means nobody is listening yet.
        let _ = tx.send(event);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn shutdown_is_idempotent_and_joins_topic_supervisors() {
        let scratch = tempfile::tempdir().expect("create SurrealKV scratch directory");
        let endpoint = format!("surrealkv://{}", scratch.path().join("db").display());
        let db = surrealdb::engine::any::connect(&endpoint)
            .await
            .expect("connect temporary SurrealKV database");
        db.use_ns("test")
            .use_db("test")
            .await
            .expect("select test namespace and database");
        let bus = LiveQueryBus::start(db);

        bus.shutdown().await;
        bus.shutdown().await;

        assert!(bus.lifecycle.shutdown.is_cancelled());
        assert!(bus.lifecycle.tasks.lock().await.is_none());
    }
}
