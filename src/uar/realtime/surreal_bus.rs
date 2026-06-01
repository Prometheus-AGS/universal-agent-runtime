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
use tokio::sync::broadcast;
use tracing::{debug, info, warn};

use super::{EntityTopic, LiveAction, LiveEvent, RealtimeBus};

/// Broadcast channel capacity per topic. Slow consumers get a `Lagged` error
/// but the publisher never blocks.
const CHANNEL_CAPACITY: usize = 256;

/// Supervised reconnect bounds for a dropped live stream.
const RECONNECT_INITIAL: Duration = Duration::from_millis(250);
const RECONNECT_MAX: Duration = Duration::from_secs(30);

#[derive(Clone)]
pub struct LiveQueryBus {
    senders: Arc<HashMap<EntityTopic, broadcast::Sender<LiveEvent>>>,
}

impl std::fmt::Debug for LiveQueryBus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LiveQueryBus")
            .field("topics", &self.senders.keys().collect::<Vec<_>>())
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
        for topic in EntityTopic::ALL {
            let (tx, _) = broadcast::channel(CHANNEL_CAPACITY);
            senders.insert(*topic, tx.clone());
            let db = db.clone();
            let topic = *topic;
            tokio::spawn(async move {
                supervise_topic(db, topic, tx).await;
            });
        }
        Self {
            senders: Arc::new(senders),
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

        // `send` only errors when there are no subscribers — that's fine, just
        // means nobody is listening yet.
        let _ = tx.send(event);
    }
    Ok(())
}
