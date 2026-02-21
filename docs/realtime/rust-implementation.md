# UAR Realtime — Rust Implementation Guide

_Last updated: 2026-02-21_

This document covers everything a Rust developer needs to:
1. Understand how the `RealtimeBroker` fits into the existing `AppState`
2. Register the WebSocket endpoint in `server.rs`
3. Emit events from anywhere in the codebase with a single ergonomic call

---

## 1. Module Location

```
src/
  uar/
    realtime/
      mod.rs          ← re-exports; public API surface
      broker.rs       ← RealtimeBroker (topic registry + tokio broadcast)
      connection.rs   ← per-WebSocket connection task (read/write loops)
      filter.rs       ← subscription filter DSL evaluator
      envelope.rs     ← Envelope struct + serde
      emitter.rs      ← EventEmitter — ergonomic emit API for all subsystems
      topic.rs        ← topic pattern matching and access-control resolver
      presence.rs     ← presence state (CRDT-lite, in-memory)
      error.rs        ← RealtimeError enum
```

---

## 2. Core Types

### 2.1 `Envelope` — the wire message

```rust
// src/uar/realtime/envelope.rs

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The single message type shared by every direction of the protocol.
/// Corresponds to the JSON envelope defined in protocol.md.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope {
    /// Protocol version — always 1 for v1.
    pub v: u8,
    /// Client-assigned join reference (echoed by server). None for server pushes.
    pub join_ref: Option<String>,
    /// Client-assigned message reference for request/reply correlation.
    pub ref_: Option<String>,
    /// Channel topic identifier (e.g. "agent:run:abc123").
    pub topic: String,
    /// Event name (e.g. "agent:token:delta").
    pub event: String,
    /// Event payload — arbitrary JSON object.
    pub payload: Value,
}

impl Envelope {
    /// Construct a server-originated push (no join_ref or ref).
    pub fn push(topic: impl Into<String>, event: impl Into<String>, payload: Value) -> Self {
        Self {
            v: 1,
            join_ref: None,
            ref_: None,
            topic: topic.into(),
            event: event.into(),
            payload,
        }
    }

    /// Construct a reply to a specific client message (echoes join_ref and ref).
    pub fn reply(
        join_ref: Option<String>,
        ref_: Option<String>,
        topic: impl Into<String>,
        status: &str,
        response: Value,
    ) -> Self {
        Self {
            v: 1,
            join_ref,
            ref_,
            topic: topic.into(),
            event: "uar_reply".to_string(),
            payload: serde_json::json!({ "status": status, "response": response }),
        }
    }
}
```

---

### 2.2 `RealtimeBroker` — the in-process pub/sub bus

```rust
// src/uar/realtime/broker.rs

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use serde_json::Value;
use crate::uar::realtime::envelope::Envelope;

/// Capacity of each topic's broadcast channel.
/// Subscribers that fall behind this far will see messages dropped.
const CHANNEL_CAPACITY: usize = 1_024;

/// A handle to a single topic's broadcast sender.
#[derive(Clone)]
struct TopicChannel {
    sender: broadcast::Sender<Envelope>,
}

/// Central pub/sub broker.
///
/// Held as `Arc<RealtimeBroker>` inside `AppState`. All subsystems call
/// `broker.publish(...)` to emit events; WebSocket connection tasks call
/// `broker.subscribe(topic)` to receive them.
#[derive(Clone, Default)]
pub struct RealtimeBroker {
    channels: Arc<RwLock<HashMap<String, TopicChannel>>>,
}

impl RealtimeBroker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a receiver for the given topic, lazily creating the channel.
    pub async fn subscribe(&self, topic: &str) -> broadcast::Receiver<Envelope> {
        let read = self.channels.read().await;
        if let Some(ch) = read.get(topic) {
            return ch.sender.subscribe();
        }
        drop(read);

        let mut write = self.channels.write().await;
        // Re-check after acquiring write lock (double-checked locking).
        let sender = write
            .entry(topic.to_string())
            .or_insert_with(|| {
                let (tx, _) = broadcast::channel(CHANNEL_CAPACITY);
                TopicChannel { sender: tx }
            })
            .sender
            .clone();
        sender.subscribe()
    }

    /// Publish an event to a topic. Returns the number of active receivers.
    /// If no subscribers exist for this topic, the call is a no-op.
    pub async fn publish(&self, topic: &str, event: &str, payload: Value) -> usize {
        let envelope = Envelope::push(topic, event, payload);
        let read = self.channels.read().await;
        if let Some(ch) = read.get(topic) {
            ch.sender.send(envelope).unwrap_or(0)
        } else {
            0
        }
    }

    /// Publish a pre-built envelope (useful for forwarding).
    pub async fn publish_envelope(&self, envelope: Envelope) -> usize {
        let topic = envelope.topic.clone();
        let read = self.channels.read().await;
        if let Some(ch) = read.get(&topic) {
            ch.sender.send(envelope).unwrap_or(0)
        } else {
            0
        }
    }

    /// Remove a topic channel when its last subscriber leaves.
    pub async fn cleanup(&self, topic: &str) {
        let mut write = self.channels.write().await;
        if let Some(ch) = write.get(topic) {
            if ch.sender.receiver_count() == 0 {
                write.remove(topic);
            }
        }
    }
}
```

---

### 2.3 `EventEmitter` — ergonomic emit API

This is the primary interface that all Rust developers use to emit events. It wraps `RealtimeBroker` and provides typed, named constructors for every standard event.

```rust
// src/uar/realtime/emitter.rs

use std::sync::Arc;
use serde_json::{Value, json};
use crate::uar::realtime::broker::RealtimeBroker;

/// Ergonomic event emission API.
///
/// Obtain via `AppState::realtime.emitter()` or by cloning from anywhere
/// that already holds an `Arc<RealtimeBroker>`.
///
/// # Example
///
/// ```rust
/// state.realtime.emit()
///     .agent_token_delta("run_abc123", "Hello, world!", 0)
///     .await;
/// ```
#[derive(Clone)]
pub struct EventEmitter {
    broker: Arc<RealtimeBroker>,
}

impl EventEmitter {
    pub fn new(broker: Arc<RealtimeBroker>) -> Self {
        Self { broker }
    }

    // ── Generic escape hatch ────────────────────────────────────────────────

    /// Emit any arbitrary event to any topic. Use typed helpers below when possible.
    pub async fn emit(&self, topic: &str, event: &str, payload: Value) -> usize {
        self.broker.publish(topic, event, payload).await
    }

    // ── Agent Run Events ────────────────────────────────────────────────────

    pub async fn agent_run_started(&self, run_id: &str, agent_id: &str, session_id: &str, model: &str) -> usize {
        self.emit(
            &format!("agent:run:{run_id}"),
            "agent:run:started",
            json!({ "run_id": run_id, "agent_id": agent_id, "session_id": session_id, "model": model }),
        ).await
    }

    pub async fn agent_run_completed(&self, run_id: &str, finish_reason: &str, total_tokens: u32, duration_ms: u64) -> usize {
        self.emit(
            &format!("agent:run:{run_id}"),
            "agent:run:completed",
            json!({ "run_id": run_id, "finish_reason": finish_reason, "total_tokens": total_tokens, "duration_ms": duration_ms }),
        ).await
    }

    pub async fn agent_run_failed(&self, run_id: &str, error: &str, code: &str) -> usize {
        self.emit(
            &format!("agent:run:{run_id}"),
            "agent:run:failed",
            json!({ "run_id": run_id, "error": error, "code": code }),
        ).await
    }

    pub async fn agent_token_delta(&self, run_id: &str, delta: &str, token_index: u64) -> usize {
        self.emit(
            &format!("agent:run:{run_id}"),
            "agent:token:delta",
            json!({ "delta": delta, "token_index": token_index }),
        ).await
    }

    pub async fn agent_tool_called(&self, run_id: &str, tool_name: &str, tool_call_id: &str, arguments: Value) -> usize {
        self.emit(
            &format!("agent:run:{run_id}"),
            "agent:tool:called",
            json!({ "tool_name": tool_name, "tool_call_id": tool_call_id, "arguments": arguments }),
        ).await
    }

    pub async fn agent_tool_result(&self, run_id: &str, tool_call_id: &str, result: Value, duration_ms: u64) -> usize {
        self.emit(
            &format!("agent:run:{run_id}"),
            "agent:tool:result",
            json!({ "tool_call_id": tool_call_id, "result": result, "duration_ms": duration_ms }),
        ).await
    }

    // ── Session Events ──────────────────────────────────────────────────────

    pub async fn session_created(&self, session_id: &str, title: &str) -> usize {
        self.emit(
            &format!("session:{session_id}"),
            "session:created",
            json!({ "session_id": session_id, "title": title }),
        ).await
    }

    pub async fn session_message_created(&self, session_id: &str, message_id: &str, role: &str) -> usize {
        self.emit(
            &format!("session:{session_id}"),
            "session:message:created",
            json!({ "message_id": message_id, "role": role }),
        ).await
    }

    // ── System Events ───────────────────────────────────────────────────────

    pub async fn system_notification(&self, level: &str, title: &str, body: &str) -> usize {
        self.emit(
            "system:notifications",
            "system:notification",
            json!({ "level": level, "title": title, "body": body }),
        ).await
    }

    // ── Plugin Events ───────────────────────────────────────────────────────

    /// Emit a plugin-namespaced event. Plugin name is enforced as prefix.
    pub async fn plugin_event(&self, plugin_name: &str, scope: &str, event_suffix: &str, payload: Value) -> usize {
        self.emit(
            &format!("plugin:{plugin_name}:{scope}"),
            &format!("plugin:{plugin_name}:{event_suffix}"),
            payload,
        ).await
    }
}
```

---

## 3. Adding `RealtimeBroker` to `AppState`

Edit **`src/lib.rs`** — add one field to the existing `AppState` struct:

```rust
// src/lib.rs  (add to AppState)
use uar::realtime::broker::RealtimeBroker;
use uar::realtime::emitter::EventEmitter;

#[derive(Clone, Debug)]
pub struct AppState {
    // ... existing fields ...

    /// Central pub/sub broker for the UAR Realtime system.
    /// All subsystems emit events through `AppState::emit()`.
    pub realtime: Arc<RealtimeBroker>,
}

impl AppState {
    /// Convenience accessor — returns an `EventEmitter` bound to the
    /// realtime broker. Cheap to call; `EventEmitter` is a thin wrapper.
    pub fn emit(&self) -> EventEmitter {
        EventEmitter::new(Arc::clone(&self.realtime))
    }
}
```

Edit **`src/server.rs`** — construct and add the broker in `start_server`:

```rust
// src/server.rs — inside start_server(), before the `let state = AppState { ... }` block

let realtime = Arc::new(RealtimeBroker::new());

let state = AppState {
    // ... existing fields ...
    realtime: Arc::clone(&realtime),
};
```

---

## 4. Registering the WebSocket Endpoint

Add the route in **`src/server.rs`** inside the `Router::new()` chain:

```rust
// src/server.rs — in the `let app = Router::new()` chain

.route(
    "/api/realtime",
    get(uar::realtime::handler::ws_handler),
)
.route(
    "/api/realtime/spec",
    get(uar::realtime::handler::spec_handler),
)
```

---

## 5. WebSocket Handler

```rust
// src/uar/realtime/handler.rs

use axum::{
    extract::{State, WebSocketUpgrade, ws::WebSocket},
    response::{IntoResponse, Response},
};
use std::sync::Arc;
use crate::AppState;
use super::{connection::RealtimeConnection, auth};

/// HTTP → WebSocket upgrade handler for /api/realtime.
///
/// Performs JWT authentication on the handshake, then hands off to
/// `RealtimeConnection::run()` for the full session lifecycle.
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Response {
    // Extract JWT from Authorization header or ?token= query param.
    let token = auth::extract_token(&headers, &params);

    let user_ctx = match auth::verify_token(token.as_deref(), &state.config.security.jwt_secret) {
        Ok(ctx) => ctx,
        Err(e) => {
            return (axum::http::StatusCode::UNAUTHORIZED, e.to_string()).into_response();
        }
    };

    ws.protocols(["uar.realtime.v1"])
        .on_upgrade(move |socket| async move {
            let conn = RealtimeConnection::new(socket, state, user_ctx);
            conn.run().await;
        })
}

/// Serves the AsyncAPI 3.0 spec YAML.
pub async fn spec_handler() -> impl IntoResponse {
    static SPEC: &str = include_str!("../../../../docs/realtime/asyncapi.yaml");
    (
        [(axum::http::header::CONTENT_TYPE, "application/yaml")],
        SPEC,
    )
}
```

---

## 6. Connection Task

Each WebSocket connection runs two concurrent tasks: a **reader** that processes client messages (joins, leaves, heartbeats, filter updates) and a **writer** that drains per-subscription receivers.

```rust
// src/uar/realtime/connection.rs  (abridged — shows the essential structure)

use axum::extract::ws::{Message, WebSocket};
use futures::{SinkExt, StreamExt};
use std::collections::HashMap;
use tokio::sync::broadcast;
use crate::{AppState, uar::security::claims::UserContext};
use super::{
    broker::RealtimeBroker,
    envelope::Envelope,
    filter::{SubscriptionFilter, evaluate_filter},
};

pub struct RealtimeConnection {
    socket: WebSocket,
    state: AppState,
    user: UserContext,
}

impl RealtimeConnection {
    pub fn new(socket: WebSocket, state: AppState, user: UserContext) -> Self {
        Self { socket, state, user }
    }

    pub async fn run(self) {
        let (mut sink, mut stream) = self.socket.split();
        let broker = Arc::clone(&self.state.realtime);

        // Active subscriptions: join_ref -> (topic, filter, rx)
        let mut subs: HashMap<String, (String, SubscriptionFilter, broadcast::Receiver<Envelope>)> =
            HashMap::new();

        loop {
            tokio::select! {
                // ── Inbound: messages from the client ────────────────────
                msg = stream.next() => {
                    match msg {
                        Some(Ok(Message::Text(text))) => {
                            if let Ok(env) = serde_json::from_str::<Envelope>(&text) {
                                handle_client_message(
                                    &env, &mut subs, &broker, &self.user, &mut sink
                                ).await;
                            }
                        }
                        None | Some(Err(_)) => break, // connection closed
                        _ => {}
                    }
                }

                // ── Outbound: fan-out from subscribed topics ──────────────
                // Poll all active subscription receivers
                _ = drain_subscriptions(&mut subs, &mut sink) => {}
            }
        }

        // Cleanup: remove empty channels
        for (_, (topic, _, _)) in &subs {
            broker.cleanup(topic).await;
        }
    }
}

/// Process a single inbound client envelope.
async fn handle_client_message(
    env: &Envelope,
    subs: &mut HashMap<String, (String, SubscriptionFilter, broadcast::Receiver<Envelope>)>,
    broker: &Arc<RealtimeBroker>,
    user: &UserContext,
    sink: &mut impl SinkExt<Message, Error = axum::Error> + Unpin,
) {
    match env.event.as_str() {
        "uar_join" => {
            // Parse filter from payload, authorize topic, subscribe
            let filter = SubscriptionFilter::from_payload(&env.payload);
            // TODO: call topic access control resolver here
            let rx = broker.subscribe(&env.topic).await;
            let join_ref = env.join_ref.clone().unwrap_or_default();
            subs.insert(join_ref.clone(), (env.topic.clone(), filter, rx));

            let reply = Envelope::reply(
                env.join_ref.clone(),
                env.ref_.clone(),
                &env.topic,
                "ok",
                serde_json::json!({ "channel_id": uuid::Uuid::new_v4().to_string() }),
            );
            let _ = sink.send(Message::Text(serde_json::to_string(&reply).unwrap())).await;
        }

        "uar_subscribe" => {
            // Batch join: payload.subscriptions is an array
            if let Some(subscriptions) = env.payload.get("subscriptions").and_then(|s| s.as_array()) {
                let mut results = vec![];
                for sub in subscriptions {
                    let topic = sub.get("topic").and_then(|t| t.as_str()).unwrap_or("");
                    let join_ref = sub.get("join_ref").and_then(|r| r.as_str()).unwrap_or("");
                    let filter = SubscriptionFilter::from_payload(sub);
                    // TODO: access control per topic
                    let rx = broker.subscribe(topic).await;
                    subs.insert(join_ref.to_string(), (topic.to_string(), filter, rx));
                    results.push(serde_json::json!({ "topic": topic, "join_ref": join_ref, "status": "ok" }));
                }
                let reply = Envelope::reply(
                    env.join_ref.clone(),
                    env.ref_.clone(),
                    "uar",
                    "ok",
                    serde_json::json!({ "results": results }),
                );
                let _ = sink.send(Message::Text(serde_json::to_string(&reply).unwrap())).await;
            }
        }

        "uar_leave" => {
            if let Some(join_ref) = &env.join_ref {
                subs.remove(join_ref);
            }
            let reply = Envelope::reply(env.join_ref.clone(), env.ref_.clone(), &env.topic, "ok", serde_json::json!({}));
            let _ = sink.send(Message::Text(serde_json::to_string(&reply).unwrap())).await;
        }

        "uar_filter_update" => {
            if let Some(join_ref) = &env.join_ref {
                if let Some((_, filter, _)) = subs.get_mut(join_ref) {
                    *filter = SubscriptionFilter::from_payload(&env.payload.get("filter").cloned().unwrap_or_default());
                }
            }
            let reply = Envelope::reply(env.join_ref.clone(), env.ref_.clone(), &env.topic, "ok", serde_json::json!({}));
            let _ = sink.send(Message::Text(serde_json::to_string(&reply).unwrap())).await;
        }

        "uar_heartbeat" => {
            let reply = Envelope::reply(env.join_ref.clone(), env.ref_.clone(), "phoenix", "ok", serde_json::json!({}));
            let _ = sink.send(Message::Text(serde_json::to_string(&reply).unwrap())).await;
        }

        _ => {} // Unknown client events are silently ignored
    }
}

/// Non-blocking drain: forward any pending broker messages to the WebSocket
/// that pass subscription filters.
async fn drain_subscriptions(
    subs: &mut HashMap<String, (String, SubscriptionFilter, broadcast::Receiver<Envelope>)>,
    sink: &mut impl SinkExt<Message, Error = axum::Error> + Unpin,
) {
    for (_join_ref, (_, filter, rx)) in subs.iter_mut() {
        // Drain all pending without blocking
        loop {
            match rx.try_recv() {
                Ok(envelope) => {
                    if filter.matches(&envelope) {
                        let _ = sink.send(Message::Text(serde_json::to_string(&envelope).unwrap())).await;
                    }
                }
                Err(broadcast::error::TryRecvError::Empty) => break,
                Err(broadcast::error::TryRecvError::Lagged(n)) => {
                    tracing::warn!("Realtime subscriber lagged by {n} messages");
                    break;
                }
                Err(broadcast::error::TryRecvError::Closed) => break,
            }
        }
    }
}
```

---

## 7. Filter Engine

```rust
// src/uar/realtime/filter.rs

use serde_json::Value;
use crate::uar::realtime::envelope::Envelope;

/// Parsed subscription filter — stored per active subscription.
/// Evaluated for every incoming broadcast before forwarding.
#[derive(Clone, Default)]
pub struct SubscriptionFilter {
    /// Event name glob patterns. Empty = accept all.
    pub events: Vec<String>,
    /// JSON predicate tree. None = accept all payloads.
    pub where_clause: Option<Value>,
}

impl SubscriptionFilter {
    /// Parse a filter from the `filter` field of a join payload.
    pub fn from_payload(v: &Value) -> Self {
        let events = v.get("events")
            .and_then(|e| e.as_array())
            .map(|arr| arr.iter().filter_map(|s| s.as_str().map(String::from)).collect())
            .unwrap_or_default();

        let where_clause = v.get("where").cloned();

        Self { events, where_clause }
    }

    /// Returns true if the envelope should be delivered to this subscriber.
    pub fn matches(&self, envelope: &Envelope) -> bool {
        self.event_matches(&envelope.event) && self.predicate_matches(&envelope.payload)
    }

    fn event_matches(&self, event: &str) -> bool {
        if self.events.is_empty() {
            return true;
        }
        self.events.iter().any(|pattern| glob_match(pattern, event))
    }

    fn predicate_matches(&self, payload: &Value) -> bool {
        match &self.where_clause {
            None => true,
            Some(predicate) => evaluate_predicate(predicate, payload),
        }
    }
}

/// Simple glob matching: `*` matches one colon-delimited segment, `**` matches any.
fn glob_match(pattern: &str, value: &str) -> bool {
    if pattern == "**" { return true; }
    if !pattern.contains('*') { return pattern == value; }

    let p_parts: Vec<&str> = pattern.split(':').collect();
    let v_parts: Vec<&str> = value.split(':').collect();

    if p_parts.len() != v_parts.len() && !p_parts.contains(&"**") {
        return false;
    }

    p_parts.iter().zip(v_parts.iter()).all(|(p, v)| *p == "*" || *p == "**" || p == v)
}

/// Recursively evaluate a predicate tree against a JSON payload.
/// Supports $eq, $neq, $gt, $gte, $lt, $lte, $in, $nin, $exists,
/// $contains, $starts_with, $ends_with, $between, $and, $or, $not.
pub fn evaluate_predicate(predicate: &Value, payload: &Value) -> bool {
    let Some(obj) = predicate.as_object() else { return true };

    for (key, operand) in obj {
        match key.as_str() {
            "$and" => {
                if let Some(arr) = operand.as_array() {
                    if !arr.iter().all(|p| evaluate_predicate(p, payload)) {
                        return false;
                    }
                }
            }
            "$or" => {
                if let Some(arr) = operand.as_array() {
                    if !arr.iter().any(|p| evaluate_predicate(p, payload)) {
                        return false;
                    }
                }
            }
            "$not" => {
                if evaluate_predicate(operand, payload) {
                    return false;
                }
            }
            field_path => {
                // field_path is like "payload.level" or "event"
                let field_value = resolve_path(field_path, payload);
                if !evaluate_field_predicate(operand, &field_value) {
                    return false;
                }
            }
        }
    }
    true
}

/// Resolve a dot-separated path into the payload JSON.
fn resolve_path<'a>(path: &str, payload: &'a Value) -> Option<&'a Value> {
    // Strip leading "payload." prefix — we receive the payload directly.
    let stripped = path.strip_prefix("payload.").unwrap_or(path);
    stripped.split('.').try_fold(payload, |val, key| val.get(key))
}

/// Evaluate a single field's operator map against its actual value.
fn evaluate_field_predicate(ops: &Value, actual: &Option<&Value>) -> bool {
    let Some(op_obj) = ops.as_object() else { return true };

    for (op, operand) in op_obj {
        let matched = match op.as_str() {
            "$exists" => {
                let want = operand.as_bool().unwrap_or(true);
                actual.is_some() == want
            }
            _ => {
                let Some(val) = actual else { return false };
                match op.as_str() {
                    "$eq"          => val == operand,
                    "$neq"         => val != operand,
                    "$gt"          => num_cmp(val, operand, |a, b| a > b),
                    "$gte"         => num_cmp(val, operand, |a, b| a >= b),
                    "$lt"          => num_cmp(val, operand, |a, b| a < b),
                    "$lte"         => num_cmp(val, operand, |a, b| a <= b),
                    "$in"          => operand.as_array().map_or(false, |arr| arr.contains(val)),
                    "$nin"         => operand.as_array().map_or(true, |arr| !arr.contains(val)),
                    "$contains"    => str_op(val, operand, |v, o| v.contains(o)),
                    "$starts_with" => str_op(val, operand, |v, o| v.starts_with(o)),
                    "$ends_with"   => str_op(val, operand, |v, o| v.ends_with(o)),
                    "$between"     => {
                        if let (Some(v), Some(arr)) = (val.as_f64(), operand.as_array()) {
                            arr.len() == 2
                                && arr[0].as_f64().map_or(false, |lo| v >= lo)
                                && arr[1].as_f64().map_or(false, |hi| v <= hi)
                        } else { false }
                    }
                    _ => true, // unknown operator — pass-through (4009 returned at join time)
                }
            }
        };
        if !matched { return false; }
    }
    true
}

fn num_cmp(val: &Value, operand: &Value, f: impl Fn(f64, f64) -> bool) -> bool {
    match (val.as_f64(), operand.as_f64()) {
        (Some(a), Some(b)) => f(a, b),
        _ => false,
    }
}

fn str_op(val: &Value, operand: &Value, f: impl Fn(&str, &str) -> bool) -> bool {
    match (val.as_str(), operand.as_str()) {
        (Some(a), Some(b)) => f(a, b),
        _ => false,
    }
}
```

---

## 8. Emitting from Existing Subsystems

With the above in place, emitting realtime events from any handler or service is a one-liner:

### From an Axum handler (has `State<AppState>`)

```rust
async fn my_handler(State(state): State<AppState>) -> impl IntoResponse {
    // ... do work ...

    state.emit().agent_run_started("run_abc", "agent_1", "sess_xyz", "gpt-4o").await;

    StatusCode::OK
}
```

### From `RunManager` (after adding broker dependency)

```rust
// Inside RunManager — after streaming a token:
self.emitter.agent_token_delta(&run_id, &token_text, token_index).await;

// After run completes:
self.emitter.agent_run_completed(&run_id, "stop", total_tokens, duration_ms).await;
```

### From any async task with access to `Arc<RealtimeBroker>`

```rust
let emitter = EventEmitter::new(Arc::clone(&broker));

// Fire-and-forget (ignore subscriber count)
emitter.system_notification("warning", "Maintenance", "Restarting in 5 min").await;
```

### Completely custom event (escape hatch)

```rust
state.emit().emit(
    "plugin:canvas:board:brd_42",
    "plugin:canvas:cursor:moved",
    json!({ "user_id": "usr_abc", "x": 100, "y": 200 }),
).await;
```

---

## 9. `Cargo.toml` Dependencies

Add these to the existing `[dependencies]` block:

```toml
# WebSocket support for Axum (enable the ws feature)
axum = { version = "0.7", features = ["ws", "macros"] }

# Already present — tokio broadcast is part of tokio::sync
tokio = { version = "1", features = ["full"] }
```

No additional broker library (Redis, NATS, etc.) is required for the in-process single-node case. When horizontal scaling is needed, swap `broadcast::channel` for a Redis pub/sub adapter behind the `RealtimeBroker` interface.

---

## 10. Horizontal Scaling Path

For multi-node deployments, `RealtimeBroker::publish` is the only method that needs to change — replace the in-process `broadcast::Sender` with a Redis PUBLISH command. All consumer code (`connection.rs`, `EventEmitter`) is unchanged.

```rust
// Future: RealtimeBroker backed by Redis
impl RealtimeBroker {
    pub async fn publish(&self, topic: &str, event: &str, payload: Value) -> usize {
        let envelope = Envelope::push(topic, event, payload);
        let serialized = serde_json::to_string(&envelope).unwrap();
        self.redis.publish(topic, serialized).await.unwrap_or(0)
    }
}
```
