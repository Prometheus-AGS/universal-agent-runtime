# UAR Realtime — API Reference

_Last updated: 2026-02-21_

---

## 1. WebSocket Endpoint

### `GET /api/realtime`

Upgrades the HTTP connection to a WebSocket. This is the primary entry point for all realtime subscriptions.

#### Request Headers

| Header | Required | Description |
|---|---|---|
| `Upgrade: websocket` | Yes | Standard WebSocket upgrade |
| `Connection: Upgrade` | Yes | Standard WebSocket upgrade |
| `Sec-WebSocket-Protocol: uar.realtime.v1` | Recommended | Protocol version negotiation |
| `Authorization: Bearer <jwt>` | One of | JWT authentication |

#### Query Parameters

| Parameter | Description |
|---|---|
| `token` | Alternative to `Authorization` header for environments where headers cannot be set (e.g., browser `WebSocket` API without custom headers) |
| `vsn` | Protocol version override (e.g., `vsn=1`). Defaults to latest. |

#### Response (Success)

```
HTTP/1.1 101 Switching Protocols
Upgrade: websocket
Connection: Upgrade
Sec-WebSocket-Protocol: uar.realtime.v1
X-UAR-Realtime-Versions: 1
X-UAR-Connection-Id: conn_7f3a9b2c
```

#### Response (Error)

| Status | Reason |
|---|---|
| `401 Unauthorized` | Missing or invalid JWT |
| `403 Forbidden` | Valid JWT but account is suspended |
| `400 Bad Request` | Missing required WebSocket headers |
| `503 Service Unavailable` | Realtime broker is not available |

---

## 2. REST Management API

These HTTP endpoints provide non-WebSocket access to realtime primitives (useful for server-to-server publishing, monitoring, and plugin management).

### 2.1 Publish to a Channel

**`POST /api/realtime/publish`**

Push an event to a channel from the server side (no WebSocket connection required). Requires `realtime:publish` permission.

```http
POST /api/realtime/publish
Authorization: Bearer <api_key>
Content-Type: application/json

{
  "topic": "system:notifications",
  "event": "system:notification",
  "payload": {
    "level": "warning",
    "title": "Scheduled maintenance",
    "body": "Maintenance starts in 30 minutes"
  }
}
```

**Response:**
```json
{
  "ok": true,
  "delivered_to": 42
}
```

---

### 2.2 List Active Channels

**`GET /api/realtime/channels`**

Returns currently active channels and subscriber counts. Requires `admin` permission.

```http
GET /api/realtime/channels
Authorization: Bearer <api_key>
```

**Response:**
```json
{
  "channels": [
    {
      "topic": "agent:run:abc123",
      "subscriber_count": 3,
      "created_at": "2026-02-21T14:00:00Z"
    }
  ],
  "total": 1
}
```

---

### 2.3 Channel Presence

**`GET /api/realtime/channels/{topic}/presence`**

Returns the current presence state for a channel (URL-encoded topic).

```http
GET /api/realtime/channels/session%3Asess_abc/presence
Authorization: Bearer <api_key>
```

**Response:**
```json
{
  "topic": "session:sess_abc",
  "presences": {
    "usr_xyz": {
      "metas": [
        {
          "phx_ref": "abc",
          "online_at": "2026-02-21T14:10:00Z",
          "display_name": "Alice"
        }
      ]
    }
  }
}
```

---

### 2.4 Spec Endpoint

**`GET /api/realtime/spec`**

Returns the AsyncAPI 3.0 YAML specification for the realtime endpoint.

```http
GET /api/realtime/spec
Accept: application/yaml
```

---

## 3. Client SDKs

### 3.1 JavaScript / TypeScript (Browser & Node)

Located at `sdks/js/realtime/` (planned).

```typescript
import { UARRealtime } from '@uar/realtime';

const rt = new UARRealtime({
  url: 'wss://api.example.com/api/realtime',
  token: 'eyJ...',
});

// Subscribe to a channel
const channel = rt.channel('agent:run:abc123', { presence: false });

channel
  .on('agent:token:delta', (payload) => {
    console.log('Token:', payload.delta);
  })
  .on('agent:run:completed', (payload) => {
    console.log('Run complete:', payload);
  })
  .subscribe((status) => {
    if (status === 'SUBSCRIBED') {
      console.log('Connected!');
    }
  });

// Publish a broadcast (on presence channels)
channel.send({
  type: 'broadcast',
  event: 'cursor:moved',
  payload: { x: 100, y: 200 },
});

// Unsubscribe
await channel.unsubscribe();
await rt.disconnect();
```

---

### 3.2 Rust (Server-side / Tauri)

Located at `sdks/rust/realtime/` (planned).

```rust
use uar_realtime::{RealtimeClient, ChannelConfig};

let client = RealtimeClient::new("wss://localhost:8080/api/realtime")
    .with_token("eyJ...")
    .connect()
    .await?;

let channel = client
    .channel("agent:run:abc123")
    .with_config(ChannelConfig { presence: false, ..Default::default() })
    .subscribe()
    .await?;

while let Some(event) = channel.next().await {
    match event.event.as_str() {
        "agent:token:delta" => { /* handle token */ }
        "agent:run:completed" => { /* handle completion */ }
        _ => {}
    }
}
```

---

## 4. Subscription DSL Reference

When joining a channel, the `config` object in the `uar_join` payload supports:

| Field | Type | Default | Description |
|---|---|---|---|
| `presence` | boolean | `false` | Enable presence tracking for this channel |
| `filter` | string[] | `[]` | Event name filters (empty = all events) |
| `broadcast_self` | boolean | `false` | Echo broadcast events back to the sender |
| `ack_timeout_ms` | integer | `10000` | Timeout for join acknowledgment |

### Filter Examples

```json
"filter": ["agent:run:*"]           // all agent run events
"filter": ["agent:token:delta"]     // only token delta events
"filter": ["agent:**"]              // all agent events (any depth)
"filter": []                        // all events (default)
```

---

## 5. Rate Limits

| Limit | Default | Configurable |
|---|---|---|
| Max connections per user | 10 | Yes (`realtime.max_connections_per_user`) |
| Max channels per connection | 100 | Yes (`realtime.max_channels_per_connection`) |
| Max messages per second (inbound) | 50 | Yes (`realtime.inbound_rate_limit`) |
| Max message size | 1 MiB | Yes (`realtime.max_message_size_bytes`) |
| Heartbeat interval | 30s | Yes (`realtime.heartbeat_interval_seconds`) |

When rate limits are exceeded, the server sends a `uar_error` event with code `4003`.

---

## 6. Backpressure & Slow Consumers

If a client is not reading fast enough, the server applies backpressure:

1. **Buffer phase** — messages queued up to `realtime.client_buffer_size` (default 1000 messages)
2. **Warning phase** — `uar_error` with code `4007` (buffer_warning) is sent
3. **Drop phase** — oldest messages are dropped from the buffer, client receives `uar_error` code `4008` (messages_dropped)
4. **Disconnect** — if buffer remains full for `realtime.slow_consumer_timeout_seconds` (default 60s), connection is closed with `uar_close`
