//! PostgreSQL subscriptions with owned connection release and explicit recovery.
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::Result;
use serde::Deserialize;
use sqlx::{PgPool, postgres::{PgListener, PgPoolOptions}};
use tokio::sync::broadcast;
use tracing::debug;
use super::{EntityTopic, LiveAction, LiveEvent, RealtimeBus, RealtimePublishError,
    is_governance_enabled_event, lifecycle::BusLifecycle};

const CHANNEL_CAPACITY: usize = 256;
const NOTIFY_CHANNEL: &str = "uar_entity_change";

#[derive(Debug, Deserialize)]
struct NotifyPayload {
    topic: String,
    id: String,
    action: String,
    #[serde(default)]
    data: Option<serde_json::Value>,
}

/// One scope-owned LISTEN connection; it does not close the caller's pool.
#[derive(Clone, Debug)]
pub struct PostgresNotifyBus {
    senders: Arc<HashMap<EntityTopic, broadcast::Sender<LiveEvent>>>,
    lifecycle: Arc<BusLifecycle>,
}

impl PostgresNotifyBus {
    /// Start one attempt. Readiness must be awaited by the composing host.
    pub fn start(pool: PgPool) -> Self {
        let senders = Arc::new(EntityTopic::ALL.iter().map(|topic| {
            (*topic, broadcast::channel(CHANNEL_CAPACITY).0)
        }).collect::<HashMap<_, _>>());
        let lifecycle = BusLifecycle::new(Arc::clone(&senders));
        let owner = Arc::clone(&lifecycle);
        let channels = Arc::clone(&senders);
        lifecycle.spawn(async move { supervise_listener(pool, channels, owner).await });
        Self { senders, lifecycle }
    }
    /// Wait for LISTEN acknowledgement.
    pub async fn ready(&self) -> Result<()> { self.lifecycle.ready().await }
    /// Stop readers and await release of the dedicated listener pool.
    pub async fn shutdown(&self) -> Result<()> { self.lifecycle.shutdown().await }
}

impl RealtimeBus for PostgresNotifyBus {
    fn lifecycle(&self) -> Option<Arc<BusLifecycle>> { Some(Arc::clone(&self.lifecycle)) }
    fn subscribe(&self, topic: EntityTopic) -> Option<broadcast::Receiver<LiveEvent>> {
        self.senders.get(&topic).map(broadcast::Sender::subscribe)
    }
    fn subscriber_count(&self, topic: EntityTopic) -> usize {
        self.senders.get(&topic).map_or(0, broadcast::Sender::receiver_count)
    }
    fn publish(&self, event: LiveEvent) -> std::result::Result<(), RealtimePublishError> {
        let sender = self.senders.get(&event.topic)
            .ok_or(RealtimePublishError::TopicUnavailable(event.topic))?;
        let _ = sender.send(event);
        Ok(())
    }
}

async fn supervise_listener(pool: PgPool,
    senders: Arc<HashMap<EntityTopic, broadcast::Sender<LiveEvent>>>, owner: Arc<BusLifecycle>) -> Result<()>
{
    let stop = owner.stop_token();
    let mut reconnect = owner.reconnect_events();
    loop {
        if stop.is_cancelled() { return Ok(()); }
        let generation = *reconnect.borrow_and_update();
        // PgListener::Drop returns its connection asynchronously. A dedicated
        // owned pool provides the awaited release barrier without closing a
        // borrowed application pool.
        let listener_pool = PgPoolOptions::new().max_connections(1).min_connections(0)
            .max_lifetime(None).idle_timeout(None)
            .connect_with(pool.connect_options().as_ref().clone()).await;
        match listener_pool {
            Ok(listener_pool) => {
                let result = run_listener(&listener_pool, &senders, &owner, generation,
                    &stop, &mut reconnect).await;
                listener_pool.close().await;
                if stop.is_cancelled() {
                    return result.context("PostgreSQL listener cleanup failed during shutdown");
                }
                if result.is_ok() { continue; }
            }
            Err(_) => {}
        }
        for topic in EntityTopic::ALL.iter().copied() {
            owner.transition(topic, generation, false, Some("notification connection unavailable".to_owned()));
        }
        tokio::select! {
            () = stop.cancelled() => return Ok(()),
            changed = reconnect.changed() => {
                changed.map_err(|_| anyhow::anyhow!("notification control channel ended"))?;
            }
        }
    }
}

async fn run_listener(pool: &PgPool,
    senders: &HashMap<EntityTopic, broadcast::Sender<LiveEvent>>, owner: &BusLifecycle,
    generation: u64, stop: &tokio_util::sync::CancellationToken,
    reconnect: &mut tokio::sync::watch::Receiver<u64>) -> Result<()>
{
    let mut listener = PgListener::connect_with(pool).await?;
    listener.eager_reconnect(false);
    listener.listen(NOTIFY_CHANNEL).await?;
    for topic in EntityTopic::ALL.iter().copied() { owner.transition(topic, generation, true, None); }
    loop {
        let notification = tokio::select! {
            () = stop.cancelled() => break,
            changed = reconnect.changed() => { changed.map_err(|_| anyhow::anyhow!("control ended"))?; break; }
            next = listener.try_recv() => next?.ok_or_else(|| anyhow::anyhow!("notification connection lost"))?,
        };
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

        if is_governance_enabled_event(&event) {
            continue;
        }

        if let Some(tx) = senders.get(&topic) {
            // `send` only errors when there are no subscribers — that's fine.
            let _ = tx.send(event);
        }
    }
    listener.unlisten_all().await?;
    drop(listener);
    Ok(())
}
