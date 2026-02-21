# UAR Realtime — Wire Protocol Specification

_Version: 1.0 (JSON) | Status: Proposed_  
_Last updated: 2026-02-21_

---

## 1. Transport

| Property | Value |
|---|---|
| Endpoint | `ws[s]://host/api/realtime` |
| Upgrade | Standard HTTP → WebSocket (RFC 6455) |
| Subprotocol | `uar.realtime.v1` (registered in handshake `Sec-WebSocket-Protocol`) |
| Frame type | Text (UTF-8 JSON), v2 will add binary (MessagePack) |
| TLS | Required in production (`wss://`) |
| Auth | JWT in `Authorization: Bearer <token>` header OR `?token=<token>` query param during handshake |

---

## 2. Message Envelope

Every message — client-to-server and server-to-client — shares the same envelope:

```json
{
  "v":       1,
  "join_ref": "<string | null>",
  "ref":      "<string | null>",
  "topic":    "<string>",
  "event":    "<string>",
  "payload":  { }
}
```

### Field Definitions

| Field | Type | Required | Description |
|---|---|---|---|
| `v` | integer | Yes | Protocol version. Currently always `1`. |
| `join_ref` | string \| null | Yes | Client-generated ID sent with `uar_join`; echoed by the server on all subsequent messages on that channel. `null` for server-originated pushes. |
| `ref` | string \| null | Yes | Client-generated message ID for request/reply correlation. Server echoes it in `uar_reply`. `null` for broadcasts from server. |
| `topic` | string | Yes | Channel topic identifier (see [channels.md](./channels.md)). |
| `event` | string | Yes | Event name (see §3 below). |
| `payload` | object | Yes | Event-specific data. May be `{}`. |

### Example

```json
{
  "v": 1,
  "join_ref": "1",
  "ref": "2",
  "topic": "agent:run:abc123",
  "event": "agent:token",
  "payload": {
    "delta": "Hello",
    "token_index": 42
  }
}
```

---

## 3. Event Taxonomy

Events are grouped into two namespaces: **lifecycle events** (prefixed `uar_`) are reserved for the protocol layer; **domain events** use `namespace:verb` naming.

### 3.1 Lifecycle Events (Reserved)

These events are handled by the UAR transport layer and are never forwarded to plugin handlers.

| Event | Direction | Description |
|---|---|---|
| `uar_join` | Client → Server | Subscribe to a single topic. |
| `uar_subscribe` | Client → Server | Subscribe to multiple topics in one call (see [subscriptions.md](./subscriptions.md)). |
| `uar_leave` | Client → Server | Unsubscribe from a topic. |
| `uar_filter_update` | Client → Server | Replace the active filter on a subscribed channel without rejoining. |
| `uar_reply` | Server → Client | Response to a client-sent event (echoes `ref`). |
| `uar_heartbeat` | Client → Server | Keep-alive ping (sent every 30s). |
| `uar_error` | Server → Client | Protocol-level error (auth failure, unknown topic, rate limit). |
| `uar_close` | Server → Client | Server is closing the channel (graceful shutdown or kick). |

### 3.2 Domain Event Format

Domain events follow `{namespace}:{verb}` or `{namespace}:{resource}:{verb}`:

```
agent:run:started
agent:run:completed
agent:run:failed
agent:token:delta
agent:tool:called
agent:tool:result
session:created
session:updated
session:deleted
user:presence:joined
user:presence:left
user:presence:sync
system:notification
system:maintenance:warning
plugin:{name}:{event}
```

---

## 4. Channel Lifecycle

### 4.1 Join a Channel

**Client sends:**
```json
{
  "v": 1,
  "join_ref": "1",
  "ref": "1",
  "topic": "agent:run:abc123",
  "event": "uar_join",
  "payload": {
    "config": {
      "presence": true,
      "filter": ["agent:token:delta", "agent:run:*"]
    }
  }
}
```

**Server replies (success):**
```json
{
  "v": 1,
  "join_ref": "1",
  "ref": "1",
  "topic": "agent:run:abc123",
  "event": "uar_reply",
  "payload": {
    "status": "ok",
    "response": {
      "channel_id": "ch_7f3a9b",
      "presence_state": {}
    }
  }
}
```

**Server replies (error):**
```json
{
  "v": 1,
  "join_ref": "1",
  "ref": "1",
  "topic": "agent:run:abc123",
  "event": "uar_reply",
  "payload": {
    "status": "error",
    "response": {
      "reason": "unauthorized",
      "code": 4001
    }
  }
}
```

### 4.2 Leave a Channel

**Client sends:**
```json
{
  "v": 1,
  "join_ref": "1",
  "ref": "3",
  "topic": "agent:run:abc123",
  "event": "uar_leave",
  "payload": {}
}
```

**Server confirms:**
```json
{
  "v": 1,
  "join_ref": "1",
  "ref": "3",
  "topic": "agent:run:abc123",
  "event": "uar_reply",
  "payload": { "status": "ok", "response": {} }
}
```

### 4.3 Heartbeat

Client sends every 30 seconds to prevent proxy/load-balancer timeouts:

```json
{
  "v": 1,
  "join_ref": null,
  "ref": "hb-1",
  "topic": "phoenix",
  "event": "uar_heartbeat",
  "payload": {}
}
```

Server echoes:
```json
{
  "v": 1,
  "join_ref": null,
  "ref": "hb-1",
  "topic": "phoenix",
  "event": "uar_reply",
  "payload": { "status": "ok", "response": {} }
}
```

---

## 5. Presence Protocol

Presence tracks which clients are connected to a topic and their associated metadata.

### 5.1 Presence Payload Format

```json
{
  "user_id": "usr_xyz",
  "meta": {
    "online_at": "2026-02-21T14:00:00Z",
    "display_name": "Alice",
    "client_type": "browser"
  }
}
```

### 5.2 Presence Events

| Event | Direction | Description |
|---|---|---|
| `user:presence:sync` | Server → Client | Full state snapshot after join |
| `user:presence:joined` | Server → Client | A new client joined the channel |
| `user:presence:left` | Server → Client | A client left the channel |

**Sync payload:**
```json
{
  "v": 1,
  "join_ref": "1",
  "ref": null,
  "topic": "session:sess_abc",
  "event": "user:presence:sync",
  "payload": {
    "presences": {
      "usr_xyz": {
        "metas": [
          { "phx_ref": "abc", "online_at": "...", "display_name": "Alice" }
        ]
      }
    }
  }
}
```

---

## 6. Broadcast Protocol

Broadcast messages are low-latency, ephemeral, and not persisted.

**Client pushes a broadcast:**
```json
{
  "v": 1,
  "join_ref": "1",
  "ref": "5",
  "topic": "session:sess_abc",
  "event": "broadcast",
  "payload": {
    "event": "cursor:moved",
    "payload": { "x": 123, "y": 456 }
  }
}
```

The server fans this out to all other subscribers of `session:sess_abc`.

---

## 7. Error Codes

| Code | Name | Description |
|---|---|---|
| 4000 | `invalid_message` | Malformed envelope (missing fields, wrong types) |
| 4001 | `unauthorized` | JWT missing, invalid, or lacks permission for this topic |
| 4002 | `topic_not_found` | No handler registered for the requested topic pattern |
| 4003 | `rate_limited` | Too many messages in window |
| 4004 | `channel_full` | Channel has reached max subscriber limit |
| 4005 | `protocol_version_unsupported` | `v` field specifies an unsupported version |
| 4006 | `payload_too_large` | Message payload exceeds configured limit (default 1 MiB) |
| 4007 | `buffer_warning` | Client receive buffer is filling up |
| 4008 | `messages_dropped` | Messages dropped due to full buffer |
| 4009 | `unknown_operator` | Filter predicate contains an unrecognized `$operator` |

---

## 8. Filter Subscriptions

The full subscription filter DSL — covering multi-channel subscriptions, server-side payload predicates, logical combinators (`$and`/`$or`/`$not`), runtime filter updates, custom plugin operators, and SDK examples — is documented in **[subscriptions.md](./subscriptions.md)**.

Quick reference for single-channel joins via `uar_join`, pass a `filter` object in the payload:

```json
"payload": {
  "config": { "presence": false },
  "filter": {
    "events": ["agent:run:*", "agent:tool:called"],
    "where": {
      "payload.level": { "$in": ["error", "critical"] }
    }
  }
}
```

See [subscriptions.md §3](./subscriptions.md) for the full grammar.

---

## 9. Version Negotiation

The server advertises supported protocol versions in the HTTP upgrade response:

```
Sec-WebSocket-Protocol: uar.realtime.v1
X-UAR-Realtime-Versions: 1
```

Future versions will negotiate via the subprotocol header.

---

## 10. Transport-Agnostic Notes (WebRTC Path)

When WebRTC data channels are enabled, the identical JSON envelope is used. The only difference is the transport framing:

| Property | WebSocket | WebRTC Data Channel |
|---|---|---|
| Framing | WS text/binary frames | SCTP messages |
| Ordering | Ordered by default | Configurable (ordered or unordered) |
| Reliability | Reliable | Configurable |
| Address | `wss://host/api/realtime` | Negotiated via signaling |

The application-layer protocol (envelope, lifecycle, presence, broadcast) is unchanged.
