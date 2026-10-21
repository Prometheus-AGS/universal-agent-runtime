//! Scope-owned Surreal subscriptions with explicit reconnect generations.

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use futures::StreamExt;
use surrealdb::types::Action;
use surrealdb::{Surreal, engine::any::Any};
use tokio::sync::broadcast;

use super::{EntityTopic, LiveAction, LiveEvent, RealtimeBus, RealtimePublishError,
    is_governance_enabled_event, lifecycle::BusLifecycle};

const CHANNEL_CAPACITY: usize = 256;

/// Owns only subscriptions registered by this bus, including their cleanup.
#[derive(Clone, Debug)]
pub struct LiveQueryBus {
    senders: Arc<HashMap<EntityTopic, broadcast::Sender<LiveEvent>>>,
    lifecycle: Arc<BusLifecycle>,
}

impl LiveQueryBus {
    /// Start one initial subscription attempt per topic. Await `ready` before
    /// advertising readiness. The legacy sweep argument is intentionally ignored:
    /// table-wide query deletion cannot establish ownership on shared stores.
    pub fn start(db: Surreal<Any>, _legacy_sweep: bool) -> Self {
        Self::start_source(SubscriptionSource::Persistence(db),
            EntityTopic::ALL.iter().copied().filter(|topic| *topic != EntityTopic::Memory).collect())
    }

    /// Memory has a separate authenticated database and independent ownership.
    pub fn start_memory(client: crate::uar::memory::service::MemorySubscriptionClient) -> Self {
        Self::start_source(SubscriptionSource::Memory(client), vec![EntityTopic::Memory])
    }

    fn start_source(source: SubscriptionSource, topics: Vec<EntityTopic>) -> Self {
        let senders = Arc::new(topics.iter().map(|topic| {
            (*topic, broadcast::channel(CHANNEL_CAPACITY).0)
        }).collect::<HashMap<_, _>>());
        let lifecycle = BusLifecycle::new(Arc::clone(&senders));
        let transport_source = source.clone();
        let transport_owner = Arc::clone(&lifecycle);
        lifecycle.spawn(async move {
            let mut transport = transport_source.transport_events()?;
            let stop = transport_owner.stop_token();
            loop {
                tokio::select! {
                    () = stop.cancelled() => return Ok(()),
                    changed = transport.changed() => {
                        changed.map_err(|_| anyhow::anyhow!("transport state owner ended"))?;
                        let state = transport.borrow_and_update().clone();
                        match state {
                            // Availability is evidence for the trusted host,
                            // not authority to recreate subscriptions. Only a
                            // host-issued `request_reconnect` advances their
                            // generation and starts another registration.
                            surrealdb::TransportState::Available => {}
                            _ => {
                                let states = transport_owner.states().borrow().clone();
                                for (topic, state) in states {
                                    transport_owner.transition(topic, state.generation, false, Some("transport unavailable".to_owned()));
                                }
                            }
                        }
                    }
                }
            }
        });
        for topic in topics {
            let source = source.clone();
            let owner = Arc::clone(&lifecycle);
            let sender = senders[&topic].clone();
            lifecycle.spawn(async move { supervise_topic(source, topic, sender, owner).await });
        }
        Self { senders, lifecycle }
    }

    /// Observe database-acknowledged readiness for all enrolled topics.
    pub async fn ready(&self) -> Result<()> { self.lifecycle.ready().await }

    /// Stop, explicitly close each stream, and join every owned worker.
    pub async fn shutdown(&self) -> Result<()> { self.lifecycle.shutdown().await }
}

#[derive(Clone)]
enum SubscriptionSource {
    Persistence(Surreal<Any>),
    Memory(crate::uar::memory::service::MemorySubscriptionClient),
}

impl SubscriptionSource {
    fn transport_events(&self) -> Result<tokio::sync::watch::Receiver<surrealdb::TransportState>> {
        match self {
            Self::Persistence(db) => db.subscribe_transport_state().map_err(anyhow::Error::from),
            Self::Memory(client) => client.subscribe_transport_state(),
        }
    }
}

impl RealtimeBus for LiveQueryBus {
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

async fn supervise_topic(source: SubscriptionSource, topic: EntityTopic,
    sender: broadcast::Sender<LiveEvent>, owner: Arc<BusLifecycle>) -> Result<()>
{
    let stop = owner.stop_token();
    let mut reconnect = owner.reconnect_events();
    loop {
        if stop.is_cancelled() { return Ok(()); }
        let generation = *reconnect.borrow_and_update();
        let attempt = async {
            // Required subscriptions cannot depend on a later write creating a
            // table. This idempotent schema operation is part of initialization.
            match &source {
                SubscriptionSource::Persistence(db) => {
                    db.query(format!("DEFINE TABLE IF NOT EXISTS {} SCHEMALESS", topic.table()))
                        .await?.check()?;
                    db.select::<Vec<surrealdb::types::Value>>(topic.table()).live().await.map_err(anyhow::Error::from)
                }
                SubscriptionSource::Memory(client) => client.subscribe_memory().await,
            }
        }.await;
        match attempt {
            Ok(mut stream) => {
                owner.transition(topic, generation, true, None);
                let disconnected = loop {
                    tokio::select! {
                        () = stop.cancelled() => break false,
                        changed = reconnect.changed() => {
                            if changed.is_err() { break false; }
                            break false;
                        }
                        next = stream.next() => {
                            match next {
                                Some(Ok(notification)) => {
                                    let action = match notification.action {
                                        Action::Create => LiveAction::Create,
                                        Action::Update => LiveAction::Update,
                                        Action::Delete => LiveAction::Delete,
                                        _ => continue,
                                    };
                                    let json = crate::uar::persistence::providers::surreal::value_to_json(notification.data)?;
                                    let id = json.get("id").and_then(serde_json::Value::as_str)
                                        .unwrap_or_default().to_owned();
                                    let event = LiveEvent { action, topic, id, data: json };
                                    if !is_governance_enabled_event(&event) { let _ = sender.send(event); }
                                }
                                Some(Err(_)) | None => break true,
                            }
                        }
                    }
                };
                // Patched SDK closes in the original registration session and
                // acknowledges command/local registration cleanup. No Killed
                // notification is required or assumed to arrive.
                if disconnected {
                    owner.transition(topic, generation, false, Some("subscription ended".to_owned()));
                }
                if stream.close().await.is_err() {
                    owner.record_failure(format!("subscription cleanup outcome unknown: {topic}, generation {generation}"));
                }
                if stop.is_cancelled() { return Ok(()); }
                if !disconnected { continue; }
                owner.transition(topic, generation, false, Some("subscription ended".to_owned()));
            }
            Err(_) => owner.transition(topic, generation, false, Some("subscription registration failed".to_owned())),
        }
        tokio::select! {
            () = stop.cancelled() => return Ok(()),
            changed = reconnect.changed() => {
                changed.map_err(|_| anyhow::anyhow!("subscription control channel ended"))?;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn shutdown_is_idempotent_and_joins_topic_supervisors() {
        let scratch = tempfile::tempdir().expect("create SurrealKV scratch directory");
        let endpoint = format!("surrealkv://{}", scratch.path().join("db").display());
        let db = surrealdb::engine::any::connect(&endpoint).await.expect("connect temporary database");
        db.use_ns("test").use_db("test").await.expect("select namespace and database");
        let bus = LiveQueryBus::start(db.clone(), false);
        bus.ready().await.expect("subscriptions ready");
        bus.shutdown().await.expect("subscriptions closed");
        bus.shutdown().await.expect("same closure result");
        assert!(bus.lifecycle.states().borrow().values().all(|state| !state.ready));
        db.shutdown().await.expect("embedded engine closed");
        let reopened = surrealdb::engine::any::connect(&endpoint).await.expect("immediate reopen after close acknowledgement");
        reopened.shutdown().await.expect("close reopened engine");
    }
}
