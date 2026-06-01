//! Postgres `LISTEN/NOTIFY`-backed realtime bus.
//!
//! Mirrors [`super::surreal_bus::LiveQueryBus`]: it owns one broadcast channel
//! per [`EntityTopic`] and fans `LiveEvent`s out to SSE subscribers. The change
//! SOURCE is a single Postgres `LISTEN uar_entity_change` connection (opened via
//! [`sqlx::postgres::PgListener`]) rather than per-table `SurrealDB` live queries.
//!
//! Database triggers (see the `*_notify_entity_change.sql` migration) emit thin
//! `pg_notify('uar_entity_change', json)` payloads of the shape
//! `{ "topic": "...", "id": "...", "action": "create|update|delete" }` on
//! INSERT/UPDATE/DELETE of the entity tables. Payloads are intentionally thin
//! (no row data) — the client refetches on signal, which also keeps us clear of
//! the 8000-byte `NOTIFY` limit.
//!
//! The listener runs in one supervised task with the same exponential-backoff
//! reconnect policy as the `SurrealDB` bus, so a dropped connection re-`LISTEN`s.

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use serde::Deserialize;
use sqlx::PgPool;
use sqlx::postgres::PgListener;
use tokio::sync::broadcast;
use tracing::{debug, info, warn};

use super::{EntityTopic, LiveAction, LiveEvent, RealtimeBus};

/// Broadcast channel capacity per topic. Matches the `SurrealDB` bus: slow
/// consumers get a `Lagged` error but the publisher never blocks.
const CHANNEL_CAPACITY: usize = 256;

/// Single Postgres `NOTIFY` channel all entity triggers emit on.
const NOTIFY_CHANNEL: &str = "uar_entity_change";

/// Supervised reconnect bounds for a dropped listener — identical to the
/// `SurrealDB` bus (250ms → 30s).
const RECONNECT_INITIAL: Duration = Duration::from_millis(250);
const RECONNECT_MAX: Duration = Duration::from_secs(30);

/// Wire shape of a `uar_entity_change` notification payload.
#[derive(Debug, Deserialize)]
struct NotifyPayload {
    topic: String,
    id: String,
    action: String,
    #[serde(default)]
    data: Option<serde_json::Value>,
}

#[derive(Clone)]
pub struct PostgresNotifyBus {
    senders: Arc<HashMap<EntityTopic, broadcast::Sender<LiveEvent>>>,
}

impl std::fmt::Debug for PostgresNotifyBus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PostgresNotifyBus")
            .field("topics", &self.senders.keys().collect::<Vec<_>>())
            .finish()
    }
}

impl PostgresNotifyBus {
    /// Build the per-topic channels and spawn one supervised `LISTEN` task.
    ///
    /// The constructor returns immediately; the listener starts connecting in
    /// the background. If the listener cannot connect (or drops), it retries
    /// with exponential backoff — the bus itself stays usable, it just won't
    /// deliver events until the connection is healthy.
    pub fn start(pool: PgPool) -> Self {
        let mut senders = HashMap::with_capacity(EntityTopic::ALL.len());
        for topic in EntityTopic::ALL {
            let (tx, _) = broadcast::channel(CHANNEL_CAPACITY);
            senders.insert(*topic, tx);
        }
        let senders = Arc::new(senders);

        let senders_for_task = Arc::clone(&senders);
        tokio::spawn(async move {
            supervise_listener(pool, senders_for_task).await;
        });

        Self { senders }
    }
}

impl RealtimeBus for PostgresNotifyBus {
    fn subscribe(&self, topic: EntityTopic) -> Option<broadcast::Receiver<LiveEvent>> {
        self.senders.get(&topic).map(broadcast::Sender::subscribe)
    }

    fn subscriber_count(&self, topic: EntityTopic) -> usize {
        self.senders
            .get(&topic)
            .map_or(0, broadcast::Sender::receiver_count)
    }
}

async fn supervise_listener(
    pool: PgPool,
    senders: Arc<HashMap<EntityTopic, broadcast::Sender<LiveEvent>>>,
) {
    let mut backoff = RECONNECT_INITIAL;
    loop {
        match run_listener(&pool, &senders).await {
            Ok(()) => {
                info!("postgres NOTIFY listener ended cleanly; supervisor exiting");
                return;
            }
            Err(err) => {
                warn!(
                    error = %err,
                    backoff_ms = backoff.as_millis() as u64,
                    "postgres NOTIFY listener failed; reconnecting"
                );
                tokio::time::sleep(backoff).await;
                backoff = (backoff * 2).min(RECONNECT_MAX);
            }
        }
    }
}

async fn run_listener(
    pool: &PgPool,
    senders: &HashMap<EntityTopic, broadcast::Sender<LiveEvent>>,
) -> Result<()> {
    let mut listener = PgListener::connect_with(pool).await?;
    listener.listen(NOTIFY_CHANNEL).await?;
    info!(channel = NOTIFY_CHANNEL, "postgres NOTIFY listener opened");

    loop {
        // `recv` resolves on the next notification or errors when the
        // underlying connection drops — the latter bubbles up to the
        // supervisor for backoff + reconnect.
        let notification = listener.recv().await?;
        let raw = notification.payload();

        let payload: NotifyPayload = match serde_json::from_str(raw) {
            Ok(p) => p,
            Err(err) => {
                debug!(error = %err, payload = %raw, "skipping malformed NOTIFY payload");
                continue;
            }
        };

        let Ok(topic) = EntityTopic::from_str(&payload.topic) else {
            debug!(topic = %payload.topic, "skipping NOTIFY for unknown topic");
            continue;
        };

        let action = match payload.action.as_str() {
            "create" | "insert" => LiveAction::Create,
            "update" => LiveAction::Update,
            "delete" => LiveAction::Delete,
            other => {
                debug!(action = %other, "skipping NOTIFY with unknown action");
                continue;
            }
        };

        let event = LiveEvent {
            action,
            topic,
            id: payload.id,
            data: payload.data.unwrap_or(serde_json::Value::Null),
        };

        if let Some(tx) = senders.get(&topic) {
            // `send` only errors when there are no subscribers — that's fine.
            let _ = tx.send(event);
        }
    }
}
