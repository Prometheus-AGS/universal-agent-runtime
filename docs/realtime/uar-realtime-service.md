# `uar-realtime` — Standalone Realtime Service Design

_Last updated: 2026-02-21 | Status: Planned_

---

## 1. What `uar-realtime` Is

`uar-realtime` is a **standalone Rust binary** — a dedicated, independently deployable service that owns 100% of realtime transport for the Prometheus AGS ecosystem.

It handles **all notification mediums in one place**:

| Transport | Purpose |
|---|---|
| **WebSocket** | Standard bidirectional realtime events, subscriptions, presence |
| **WebRTC Signaling** | SDP offer/answer, ICE candidate exchange (enables peer connections) |
| **WebRTC Data Channels** | Same `uar.realtime.v1` events over SCTP — same protocol, different wire |
| **WebRTC Media** | Audio/video tracks — screenshare, video conference, agent voice output |
| **Server-Sent Events** | Fallback for environments that can't upgrade to WebSocket |
| _(future)_ **QUIC/WebTransport** | Low-latency, multiplexed, connection-migration-safe transport |

UAR (and any other Prometheus service) publishes events to `uar-realtime` via a lightweight internal API. `uar-realtime` fans those events out over whichever transport each client has negotiated.

---

## 2. Why a Separate Process is Correct

### Resource profiles are fundamentally different

| Concern | UAR | `uar-realtime` |
|---|---|---|
| Primary work | LLM inference, tool execution, DB queries | I/O fan-out, media relay |
| CPU profile | Bursty, compute-heavy | Near-zero for events; CPU for media encode/decode |
| Memory profile | Large (model context, embeddings) | ~4–16 KB per connection |
| Concurrency unit | Request/response | 10K–1M persistent connections |
| Crash impact | Ends a run | Drops connections (bad) |
| Restart frequency | More frequent (new deployments) | Infrequent (must preserve connections) |

Separating them means **UAR can restart, redeploy, or crash without disconnecting a single client**.

### Clients connect directly

```
Browser / Mobile / Desktop
        │
        ├── WebSocket ──────────────────────► uar-realtime :4001
        │     (subscriptions, events, presence)
        │
        ├── WebRTC (via signaling) ──────────► uar-realtime :4001
        │     (data channels = same protocol)
        │     (media tracks = audio/video)
        │
        └── SSE fallback ───────────────────► uar-realtime :4001
              (server-push only, no backpressure)

UAR Process
        └── Internal publish API ────────────► uar-realtime :4002 (internal only)
```

---

## 3. Service Structure

```
uar-realtime/                    ← standalone Rust workspace member (or separate repo)
  src/
    main.rs                       ← tokio entrypoint, loads config, starts listeners
    config.rs                     ← AppConfig for uar-realtime (ports, TURN, auth, etc.)
    broker/
      mod.rs                      ← RealtimeBroker (tokio::broadcast in-process)
      nats.rs                     ← Optional NATS back-plane for clustered mode
    transport/
      mod.rs                      ← Transport trait
      websocket.rs                ← WebSocket listener + connection lifecycle
      sse.rs                      ← SSE fallback endpoint
      webrtc/
        mod.rs                    ← WebRTC transport orchestrator
        signaling.rs              ← SDP/ICE signaling over WebSocket
        data_channel.rs           ← DataChannel → Envelope bridge
        media/
          mod.rs                  ← Media session management
          sfu.rs                  ← Selective Forwarding Unit (route tracks between peers)
          recording.rs            ← Optional media recording
    internal/
      publish.rs                  ← Internal HTTP API for UAR → uar-realtime publish
    presence.rs                   ← CRDT presence state
    filter.rs                     ← Subscription filter DSL evaluator (shared with UAR)
    auth.rs                       ← JWT verification (shared secret with UAR)
```

---

## 4. Transport Architecture

### 4.1 The `Transport` Trait

Every transport implements a common interface:

```rust
// src/uar-realtime/transport/mod.rs

#[async_trait]
pub trait Transport: Send + Sync {
    /// Send a message envelope to this specific connection.
    async fn send(&self, envelope: &Envelope) -> Result<(), TransportError>;
    /// Receive the next message from the client (returns None on disconnect).
    async fn recv(&mut self) -> Option<Envelope>;
    /// Unique connection ID.
    fn connection_id(&self) -> &str;
    /// The transport type (for logging / metrics)
    fn transport_type(&self) -> TransportType;
}

#[derive(Debug, Clone, Copy)]
pub enum TransportType {
    WebSocket,
    WebRtcDataChannel,
    Sse,
    WebTransport, // future
}
```

The `RealtimeConnection` session manager is **transport-agnostic** — it works identically whether the underlying wire is a WebSocket frame or a WebRTC SCTP message.

---

### 4.2 WebSocket Transport

Standard Axum WebSocket handler — same as documented in `rust-implementation.md`. Clients send JSON text frames using the `uar.realtime.v1` envelope.

**Port:** `4001` (public, TLS-terminated at load balancer)
**Endpoint:** `wss://realtime.example.com/realtime`

---

### 4.3 WebRTC Signaling + Data Channels

WebRTC requires a **signaling** phase before the data channel can be established. `uar-realtime` handles this entirely:

#### Signaling Flow

```
Client                              uar-realtime
  │                                     │
  │── WS connect (/realtime) ──────────►│  (initial connection always WS)
  │                                     │
  │── uar_join "uar:webrtc:session" ───►│
  │                                     │
  │◄── webrtc:offer (SDP) ─────────────│  server creates RTCPeerConnection
  │                                     │  generates offer
  │── webrtc:answer (SDP) ────────────►│  client accepts
  │                                     │
  │◄── webrtc:ice_candidate ───────────│  (trickling ICE candidates)
  │── webrtc:ice_candidate ────────────►│
  │                                     │
  │  [ WebRTC DataChannel established ] │
  │                                     │
  │── [ disconnect WS, use DataChannel ]│  optional — can keep both
```

#### Signaling Events (on `uar:webrtc:{peer_id}` channel)

| Event | Direction | Payload |
|---|---|---|
| `webrtc:offer` | Server → Client | `{ sdp: "..." }` |
| `webrtc:answer` | Client → Server | `{ sdp: "..." }` |
| `webrtc:ice_candidate` | Both | `{ candidate, sdpMid, sdpMLineIndex }` |
| `webrtc:connected` | Server → Client | `{ peer_id, data_channel_label }` |
| `webrtc:disconnected` | Server → Client | `{ reason }` |

Once the DataChannel is open, the client sends the standard `uar_join` / `uar_subscribe` messages **over the DataChannel** — same protocol, different wire.

#### Rust WebRTC Library

```toml
# uar-realtime/Cargo.toml
webrtc = "0.11"  # webrtc-rs — pure Rust WebRTC implementation
```

---

### 4.4 WebRTC Media — Video Conferencing

`uar-realtime` can act as an **SFU (Selective Forwarding Unit)** for multi-party audio/video:

```
Participant A  ──[audio+video tracks]──► uar-realtime SFU
Participant B  ──[audio+video tracks]──► uar-realtime SFU
Participant C  ──[audio+video tracks]──► uar-realtime SFU

uar-realtime SFU routes:
  A's tracks ──► B, C
  B's tracks ──► A, C
  C's tracks ──► A, C
```

An SFU **does not decode or transcode** — it only selects which encoded media packets to forward to each subscriber, making it extremely efficient (far less CPU than an MCU).

#### Use Cases Enabled

| Use Case | Description |
|---|---|
| AI agent voice output | Agent speaks; audio track delivered to client |
| Screen share to AI | Client shares screen; frames sent to vision model |
| Video conference | Multi-party meetings with participant management |
| Collaborative agent sessions | Multiple users watching the same agent run live |
| Agent observability | Developer watches live tool calls + token stream |

#### Media Channel Events (on `session:{id}:media`)

| Event | Direction | Payload |
|---|---|---|
| `media:participant:joined` | Server → Client | `{ participant_id, tracks }` |
| `media:participant:left` | Server → Client | `{ participant_id }` |
| `media:track:started` | Server → Client | `{ track_id, kind, participant_id }` |
| `media:track:ended` | Server → Client | `{ track_id }` |
| `media:recording:started` | Server → Client | `{ recording_id }` |
| `media:recording:stopped` | Server → Client | `{ recording_id, url }` |

---

## 5. Internal Publish API (UAR → uar-realtime)

UAR sends events to `uar-realtime` over a lightweight internal HTTP API (LAN-only, not public):

**Port:** `4002` (internal only, not exposed outside the pod/VM)

### `POST /internal/v1/publish`

```http
POST http://uar-realtime:4002/internal/v1/publish
Authorization: Bearer <shared-internal-secret>
Content-Type: application/json

{
  "topic": "agent:run:abc123",
  "event": "agent:token:delta",
  "payload": { "delta": "Hello", "token_index": 42 }
}
```

Response:
```json
{ "delivered_to": 7, "topic": "agent:run:abc123" }
```

### `POST /internal/v1/publish/batch`

For high-throughput scenarios (e.g., token streaming), batch multiple events in one HTTP call:

```json
{
  "events": [
    { "topic": "agent:run:abc123", "event": "agent:token:delta", "payload": { "delta": "He" } },
    { "topic": "agent:run:abc123", "event": "agent:token:delta", "payload": { "delta": "llo" } }
  ]
}
```

### UAR's `EventEmitter` — transport is invisible

```rust
// EventEmitter when uar-realtime is configured as external service:
pub async fn emit(&self, topic: &str, event: &str, payload: Value) {
    // fire-and-forget — UAR does not wait for delivery confirmation
    self.http_client
        .post(&format!("{}/internal/v1/publish", self.realtime_base_url))
        .bearer_auth(&self.internal_secret)
        .json(&json!({ "topic": topic, "event": event, "payload": payload }))
        .send()
        .await
        .ok();
}
```

The call-site (`state.emit().agent_token_delta(...)`) is **identical** regardless of whether the broker is in-process or remote.

---

## 6. Deployment

### Docker Compose (Development)

```yaml
# docker-compose.dev.yaml
services:
  uar:
    image: prometheus-ags/universal-agent-runtime:latest
    environment:
      UAR_REALTIME_MODE: external
      UAR_REALTIME_URL: http://uar-realtime:4002
      UAR_REALTIME_SECRET: ${INTERNAL_SECRET}
    depends_on:
      - uar-realtime

  uar-realtime:
    image: prometheus-ags/uar-realtime:latest
    ports:
      - "4001:4001"   # Public WebSocket / WebRTC signaling port
    environment:
      URT_INTERNAL_PORT: 4002
      URT_PUBLIC_PORT: 4001
      URT_JWT_SECRET: ${JWT_SECRET}
      URT_INTERNAL_SECRET: ${INTERNAL_SECRET}
      URT_STUN_SERVERS: "stun:stun.l.google.com:19302"
      URT_TURN_URL: ${TURN_URL}
      URT_TURN_USERNAME: ${TURN_USERNAME}
      URT_TURN_CREDENTIAL: ${TURN_CREDENTIAL}
```

### Kubernetes (Production)

```yaml
# k8s/uar-realtime/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: uar-realtime
spec:
  replicas: 3          # Scale independently from UAR
  template:
    spec:
      containers:
        - name: uar-realtime
          image: prometheus-ags/uar-realtime:latest
          ports:
            - containerPort: 4001  # WebSocket (public)
            - containerPort: 4002  # Publish API (internal)
          resources:
            requests:
              cpu: 100m        # very low CPU for pure event fan-out
              memory: 256Mi
            limits:
              cpu: 2000m       # spikes during media relay
              memory: 2Gi
```

When running multiple `uar-realtime` replicas, add NATS as the back-plane so events published to one node reach subscribers on all nodes (see `rust-implementation.md §10`).

---

## 7. Configuration Reference

```yaml
# uar-realtime config.yaml
server:
  public_port: 4001
  internal_port: 4002
  host: "0.0.0.0"

auth:
  jwt_secret: "${JWT_SECRET}"
  internal_secret: "${INTERNAL_SECRET}"

websocket:
  heartbeat_interval_seconds: 30
  max_connections: 100_000
  max_channels_per_connection: 100
  client_buffer_size: 1_000

webrtc:
  enabled: true
  stun_servers:
    - "stun:stun.l.google.com:19302"
  turn:
    enabled: false
    url: "${TURN_URL}"
    username: "${TURN_USERNAME}"
    credential: "${TURN_CREDENTIAL}"
  media:
    enabled: false       # SFU — enable for video conference support
    max_participants: 50
    recording:
      enabled: false
      storage_path: "/data/recordings"

cluster:
  enabled: false
  nats_url: "${NATS_URL}"   # Enable for multi-node deployments
```

---

## 8. Roadmap

| Phase | Feature | Status |
|---|---|---|
| **P1** | WebSocket transport + subscription DSL | Planned |
| **P1** | Internal publish API (UAR → uar-realtime) | Planned |
| **P1** | Presence tracking | Planned |
| **P2** | WebRTC signaling (SDP/ICE) | Planned |
| **P2** | WebRTC data channels (uar.realtime.v1 over SCTP) | Planned |
| **P3** | WebRTC SFU (audio/video media relay) | Future |
| **P3** | Agent voice output (TTS → audio track) | Future |
| **P3** | Screen-to-AI (video frame → vision model) | Future |
| **P4** | Multi-node clustering via NATS | Future |
| **P4** | WebTransport (HTTP/3-based) | Future |
