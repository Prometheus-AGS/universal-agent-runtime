# UAR Realtime — Subscription DSL

_Last updated: 2026-02-21_

This document defines how clients (UI code, API callers, plugins) subscribe to **multiple channels and events simultaneously** with **server-side filter criteria** that limit which notifications are delivered.

---

## 1. Design Goals

| Goal | Description |
|---|---|
| **Multi-channel** | Subscribe to any number of channels in a single call |
| **Event selection** | Choose which event types to receive per channel |
| **Payload predicates** | Server evaluates conditions against event payload fields before delivering |
| **Extensible** | Predicate operators can be expanded without breaking existing clients |
| **Composable** | Conditions combine with AND/OR/NOT logic |
| **Efficient** | Filtering happens server-side — no wasted bandwidth on dropped messages |

---

## 2. Multi-Channel Subscription

### 2.1 Subscribe to Multiple Channels at Once

Rather than sending a separate `uar_join` for each channel, clients send a single `uar_subscribe` message on the reserved `uar` topic. This atomically subscribes to an array of channels:

```json
{
  "v": 1,
  "join_ref": null,
  "ref": "sub-1",
  "topic": "uar",
  "event": "uar_subscribe",
  "payload": {
    "subscriptions": [
      {
        "topic": "agent:run:abc123",
        "join_ref": "jr-1",
        "config": { "presence": false },
        "filter": { "events": ["agent:token:delta", "agent:run:*"] }
      },
      {
        "topic": "session:sess_xyz",
        "join_ref": "jr-2",
        "config": { "presence": false },
        "filter": {
          "events": ["session:message:created", "session:run:*"]
        }
      },
      {
        "topic": "system:notifications",
        "join_ref": "jr-3",
        "filter": {
          "events": ["system:notification"],
          "where": {
            "payload.level": { "$in": ["warning", "error", "critical"] }
          }
        }
      }
    ]
  }
}
```

### 2.2 Server Reply

The server replies once with a result for each requested channel:

```json
{
  "v": 1,
  "join_ref": null,
  "ref": "sub-1",
  "topic": "uar",
  "event": "uar_reply",
  "payload": {
    "status": "ok",
    "results": [
      { "topic": "agent:run:abc123",     "join_ref": "jr-1", "status": "ok" },
      { "topic": "session:sess_xyz",     "join_ref": "jr-2", "status": "ok" },
      { "topic": "system:notifications", "join_ref": "jr-3", "status": "ok" }
    ]
  }
}
```

Partial failures are reported per-entry:

```json
{
  "status": "partial",
  "results": [
    { "topic": "agent:run:abc123",     "join_ref": "jr-1", "status": "ok" },
    { "topic": "session:sess_xyz",     "join_ref": "jr-2", "status": "error", "error": { "reason": "unauthorized", "code": 4001 } },
    { "topic": "system:notifications", "join_ref": "jr-3", "status": "ok" }
  ]
}
```

> **Note:** Individual `uar_join` messages still work per-channel. `uar_subscribe` is the batched convenience form.

---

## 3. Filter Grammar

Each subscription entry accepts a `filter` object:

```
filter := {
  events?:  EventPattern[]     // which event names to receive
  where?:   Predicate          // payload conditions
  limit?:   LimitConfig        // delivery constraints (rate, first-N)
}
```

---

### 3.1 `events` — Event Name Matching

A list of event name patterns. An event is delivered if it matches **any** pattern.

| Pattern | Matches |
|---|---|
| `"agent:run:completed"` | Exact match only |
| `"agent:run:*"` | Any event starting with `agent:run:` (single segment wildcard) |
| `"agent:**"` | Any event starting with `agent:` (recursive wildcard) |
| `"**"` | All events on this channel |

```json
"filter": {
  "events": ["agent:run:*", "agent:tool:called"]
}
```

An absent or empty `events` array is equivalent to `["**"]` — all events are delivered.

---

### 3.2 `where` — Payload Predicates

The `where` clause is a **JSON predicate object** evaluated server-side against `event.payload`. The syntax is inspired by MongoDB query operators, which are widely understood and already used in the Supabase Realtime Postgres Changes filter.

#### Predicate Shape

```json
"where": {
  "<payload.path>": { "<$operator>": <value> },
  ...
}
```

Multiple top-level keys in `where` are implicitly AND-ed.

#### Supported Operators

| Operator | Type | Description | Example |
|---|---|---|---|
| `$eq` | any | Exact equality | `{ "$eq": "stopped" }` |
| `$neq` | any | Not equal | `{ "$neq": "running" }` |
| `$gt` | number | Greater than | `{ "$gt": 1000 }` |
| `$gte` | number | Greater than or equal | `{ "$gte": 500 }` |
| `$lt` | number | Less than | `{ "$lt": 100 }` |
| `$lte` | number | Less than or equal | `{ "$lte": 200 }` |
| `$in` | array | Value is in array | `{ "$in": ["error", "warning"] }` |
| `$nin` | array | Value is not in array | `{ "$nin": ["info"] }` |
| `$exists` | boolean | Field presence check | `{ "$exists": true }` |
| `$contains` | string | Substring match | `{ "$contains": "timeout" }` |
| `$starts_with` | string | Prefix match | `{ "$starts_with": "agent" }` |
| `$ends_with` | string | Suffix match | `{ "$ends_with": ".pdf" }` |
| `$regex` | string | Regular expression (RE2 syntax) | `{ "$regex": "^err_[0-9]+" }` |
| `$between` | [min, max] | Inclusive range | `{ "$between": [1, 100] }` |

#### Logical Combinators

| Operator | Description |
|---|---|
| `$and` | All conditions must be true |
| `$or` | At least one condition must be true |
| `$not` | Negates a predicate |

```json
"where": {
  "$or": [
    { "payload.level": { "$eq": "critical" } },
    { "$and": [
      { "payload.level": { "$eq": "warning" } },
      { "payload.code": { "$gte": 5000 } }
    ]}
  ]
}
```

#### Field Path Notation

Nested payload fields are accessed with dot notation:

```json
"payload.run.duration_ms": { "$gt": 5000 }
"payload.tool.name": { "$in": ["web_search", "code_exec"] }
"payload.user.role": { "$eq": "admin" }
```

The root `payload` prefix is required to distinguish payload fields from message envelope fields. Envelope fields can also be matched:

```json
"event": { "$starts_with": "agent:tool:" }
"topic": { "$contains": ":abc123" }
```

---

### 3.3 `limit` — Delivery Constraints (Optional)

```json
"limit": {
  "first": 50,          // stop delivering after 50 events (auto-unsubscribe)
  "rate_per_second": 10 // server throttles delivery to max 10/s (drops rest)
}
```

---

## 4. Full Examples

### 4.1 UI Code: Track a Run with Selective Events

Subscribe only to the token stream and final result for a run — skip internal tool call events:

```json
{
  "v": 1,
  "join_ref": null,
  "ref": "sub-run",
  "topic": "uar",
  "event": "uar_subscribe",
  "payload": {
    "subscriptions": [
      {
        "topic": "agent:run:abc123",
        "join_ref": "jr-run",
        "filter": {
          "events": ["agent:token:delta", "agent:token:complete", "agent:run:completed", "agent:run:failed"]
        }
      }
    ]
  }
}
```

---

### 4.2 UI Code: System Alerts — Only High Severity

```json
{
  "subscriptions": [
    {
      "topic": "system:notifications",
      "join_ref": "jr-sys",
      "filter": {
        "events": ["system:notification", "system:maintenance:*"],
        "where": {
          "payload.level": { "$in": ["warning", "error", "critical"] }
        }
      }
    }
  ]
}
```

---

### 4.3 API Caller: Watch Multiple Sessions

```json
{
  "subscriptions": [
    {
      "topic": "session:sess_aaa",
      "join_ref": "jr-s1",
      "filter": { "events": ["session:run:completed", "session:message:created"] }
    },
    {
      "topic": "session:sess_bbb",
      "join_ref": "jr-s2",
      "filter": { "events": ["session:run:completed", "session:message:created"] }
    }
  ]
}
```

---

### 4.4 API Caller: Only Long-Running Tool Calls

Receive tool call events only when the estimated token cost is above a threshold:

```json
{
  "subscriptions": [
    {
      "topic": "agent:run:abc123",
      "join_ref": "jr-tools",
      "filter": {
        "events": ["agent:tool:called"],
        "where": {
          "$or": [
            { "payload.tool_name": { "$in": ["code_exec", "web_browser"] } },
            { "payload.estimated_tokens": { "$gt": 2000 } }
          ]
        }
      }
    }
  ]
}
```

---

### 4.5 Plugin: Cross-Channel Aggregated Subscription

A plugin subscribes to its own channel plus agent lifecycle events, scoped to a specific agent:

```json
{
  "subscriptions": [
    {
      "topic": "plugin:canvas:board:brd_42",
      "join_ref": "jr-plugin",
      "config": { "presence": true },
      "filter": { "events": ["**"] }
    },
    {
      "topic": "agent:run:abc123",
      "join_ref": "jr-agent",
      "filter": {
        "events": ["agent:run:completed", "agent:run:failed"],
        "where": {
          "payload.agent_id": { "$eq": "agent_canvas_worker" }
        }
      }
    }
  ]
}
```

---

## 5. Runtime Update: Modifying a Subscription

After joining, a client can update the filter on a channel without leaving and rejoining using `uar_filter_update`:

```json
{
  "v": 1,
  "join_ref": "jr-run",
  "ref": "upd-1",
  "topic": "agent:run:abc123",
  "event": "uar_filter_update",
  "payload": {
    "filter": {
      "events": ["agent:token:delta", "agent:tool:*", "agent:run:completed"],
      "where": {
        "payload.token_index": { "$gte": 100 }
      }
    }
  }
}
```

The server applies the new filter immediately and responds with `uar_reply`:

```json
{
  "v": 1,
  "join_ref": "jr-run",
  "ref": "upd-1",
  "topic": "agent:run:abc123",
  "event": "uar_reply",
  "payload": { "status": "ok", "response": {} }
}
```

---

## 6. SDK Convenience API

### TypeScript

```typescript
// Single channel with predicate filter
const channel = rt.channel('system:notifications')
  .filter({
    events: ['system:notification'],
    where: {
      'payload.level': { $in: ['warning', 'error', 'critical'] }
    }
  });

channel.on('system:notification', (payload) => {
  showAlert(payload);
}).subscribe();

// Multi-channel subscription
const subs = await rt.subscribe([
  {
    topic: 'agent:run:abc123',
    filter: { events: ['agent:token:delta', 'agent:run:*'] }
  },
  {
    topic: 'session:sess_xyz',
    filter: {
      events: ['session:message:created'],
      where: { 'payload.role': { $eq: 'assistant' } }
    }
  },
  {
    topic: 'system:notifications',
    filter: {
      events: ['system:notification'],
      where: { 'payload.level': { $in: ['error', 'critical'] } }
    }
  }
]);

// Listen to all channels via the subscription handle
subs.on('agent:token:delta', (payload, meta) => {
  console.log(`Token from ${meta.topic}:`, payload.delta);
});

// Update filter on an active channel
await subs.updateFilter('agent:run:abc123', {
  events: ['agent:token:delta', 'agent:tool:*'],
  where: { 'payload.token_index': { $gte: 100 } }
});

// Unsubscribe all
await subs.unsubscribeAll();
```

### Rust

```rust
use uar_realtime::{Subscription, Filter, Predicate};

let subs = client.subscribe(vec![
    Subscription::new("agent:run:abc123")
        .events(["agent:token:delta", "agent:run:*"])
        .build(),
    Subscription::new("system:notifications")
        .events(["system:notification"])
        .where_(Predicate::field("payload.level").r#in(["warning", "error", "critical"]))
        .build(),
])
.await?;

while let Some(event) = subs.next().await {
    println!("[{}] {}: {:?}", event.topic, event.event, event.payload);
}
```

---

## 7. Extensibility

The predicate system is designed to be additive. New operators are registered server-side with:

```rust
// Server-side operator registration (Rust pseudocode)
broker.register_predicate_operator("$fuzzy", |field_value, operand| {
    // custom fuzzy string matching
    fuzzy_match(field_value.as_str()?, operand.as_str()?)
});
```

Plugin developers can register custom operators in their plugin manifest:

```yaml
# plugin.yaml
realtime:
  predicate_operators:
    - name: "$semantic_similar"
      description: "Semantic similarity threshold filter"
      input_type: number   # threshold 0.0–1.0
      server_handler: "handlers/semantic_filter.wasm"
```

Clients that use an unrecognized operator receive a `4009` (`unknown_operator`) error on join — older clients with no `$semantic_similar` filter simply don't include it and are unaffected.

---

## 8. Server-Side Evaluation Model

```
Incoming Event
     │
     ▼
Topic Router → finds all channel subscribers
     │
     ▼  (per subscriber)
Event Name Matcher  →  does event match subscriber's `events` patterns?
     │ yes
     ▼
Payload Predicate Evaluator  →  does payload satisfy `where`?
     │ yes
     ▼
Rate Limiter (`limit.rate_per_second`)
     │ pass
     ▼
Write to subscriber's send queue
```

All filtering is **synchronous and in-process** — no additional round trips. Predicate evaluation is bounded to O(depth of predicate tree) per event per subscriber.
