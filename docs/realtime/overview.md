# UAR Realtime — Overview & Design Rationale

_Last updated: 2026-02-21_

---

## 1. Goals

The UAR Realtime system provides a **single, standardized, generic mechanism** for pushing events from the server to any connected client — whether that client is:

- The first-party React frontend
- An external API caller (developer, partner)
- A Tauri desktop shell
- A future mobile app
- A plugin running inside the UAR plugin sandbox

It must be:

| Requirement | Details |
|---|---|
| **Generic** | Not tied to any specific domain. Any subsystem may emit events. |
| **Multiplexed** | Many logical "channels" over a single connection |
| **Multi-transport** | Same protocol over WebSocket, WebRTC data channels, SSE, and future WebTransport |
| **Media-capable** | Supports WebRTC audio/video tracks for conferencing and agent voice |
| **Presence-aware** | Clients can track who else is online / subscribed to a channel |
| **Extensible** | Plugin developers register custom channels and events without touching core |
| **Documented** | Machine-readable spec (AsyncAPI 3.0) ships alongside the runtime |
| **Independently scalable** | A standalone `uar-realtime` service handles all connections, independent of UAR |
| **Secure** | Every subscription is authorized against JWT claims / RBAC |

---

## 2. Prior Art Survey

### 2.1 Phoenix Channels (Elixir/Erlang)

The most battle-tested generic real-time framework in production. Supabase Realtime is itself built on Phoenix Channels.

**Key ideas borrowed:**
- Single WebSocket → many named topics
- Compact array message envelope: `[join_ref, msg_ref, topic, event, payload]`
- Lifecycle events: `phx_join`, `phx_leave`, `phx_reply`, `phx_heartbeat`, `phx_error`
- Server-side `PubSub` bus for horizontal scale

**Why not adopt verbatim:** UAR is Rust/Axum, not Elixir. We adopt the _mental model_ and _message format_ but implement natively.

---

### 2.2 Supabase Realtime

A hosted service built on Phoenix Channels that adds:
- **Broadcast** — low-latency ephemeral messages (no persistence)
- **Presence** — CRDT-based shared state tracking (who is online, user metadata)
- **Postgres Changes** — subscribing to DB-level row events

**Key ideas borrowed:**
- Three event categories: Broadcast / Presence / Data Changes
- `topic` as the primary routing key (e.g. `realtime:agent:run:abc123`)
- Version 2.0 binary frame optimization (noted for future adoption)

**Why not adopt verbatim:** We do not depend on PostgreSQL LISTEN/NOTIFY. Our "data changes" events come from internal Rust event streams.

---

### 2.3 WAMP (Web Application Messaging Protocol)

An IETF-tracked open standard WebSocket subprotocol providing both PubSub and routed RPC in a single protocol.
- Registered WebSocket subprotocol: `wamp.2.json`, `wamp.2.msgpack`
- Internet-Draft for WAMP Basic Profile published July 2024
- Clean separation: **broker** (PubSub) + **dealer** (RPC)

**Key ideas borrowed:**
- URI-based structured topic naming: `io.prometheus.uar.agent.run.started`
- Routed RPC pattern for future command/response patterns
- Formal registration with IANA

**Why not adopt verbatim:** WAMP requires a separate router process; UAR wants an embedded broker. We adopt the naming convention and conceptual patterns.

---

### 2.4 AsyncAPI 3.0

The industry-standard machine-readable specification for event-driven APIs (analogous to OpenAPI for REST), officially released December 2023.
- Supports WebSocket, Kafka, MQTT, AMQP, and more
- In v3.0: channels and operations are decoupled for better reuse
- `send` / `receive` semantics replace the old ambiguous `publish`/`subscribe`
- JSON Schema for all message payloads

**Decision:** UAR ships an `asyncapi.yaml` file at `docs/realtime/asyncapi.yaml` (and serves it at `/api/realtime/spec`).

---

### 2.5 What does NOT exist as a universal standard

There is no single IETF or W3C standard that covers the _application-layer_ messaging pattern (channels, topics, presence, typed events, lifecycle) for WebSocket. RFC 6455 only specifies the transport. Every successful framework defines its own application-layer protocol:

| Project | Wire format | Channel model | Presence |
|---|---|---|---|
| Phoenix Channels | JSON array | topic string | via `Phoenix.Presence` |
| Supabase Realtime | JSON object | topic string | built-in CRDT |
| Socket.io | JSON array + namespace | namespace + room | via adapters |
| WAMP | JSON/msgpack array | URI | via meta-events |
| Ably | JSON object | channel name | built-in |
| Pusher | JSON object | channel name | built-in |

**Conclusion:** We must define our own UAR application-layer protocol, informed by these patterns. We choose to closely follow the Phoenix/Supabase design as it is the most widely deployed and elegantly designed open-source model.

---

## 3. Architectural Decisions

### ADR-001: Single endpoint, topic-routed

**Decision:** One WebSocket endpoint at `/api/realtime`. All channels live in the same connection.

**Rationale:** Reduces connection overhead; mirrors Phoenix Channels and Supabase.

---

### ADR-002: JSON envelope (v1), binary optimization later (v2)

**Decision:** V1 uses JSON text frames. V2 will add MessagePack binary frames for high-throughput channels.

**Rationale:** Matches Supabase v1/v2 evolution path. JSON is debuggable and universal.

---

### ADR-003: Topic naming — colon-delimited hierarchy

**Decision:** Topics follow the pattern `{namespace}:{resource}:{id}` using colon as delimiter.

```
system:notifications
agent:run:{run_id}
session:{session_id}
session:{session_id}:agent
user:{user_id}:activity
plugin:{plugin_name}:{scope}
```

**Rationale:** Colon-delimited topics are established practice (Phoenix, Supabase, Ably). WAMP uses dot-delimited URIs but colons are more readable for human-facing topics.

---

### ADR-004: Three channel types — Broadcast, Presence, Data

Directly mirrors Supabase's proven three-way split:

| Type | Description | Persisted |
|---|---|---|
| **Broadcast** | Ephemeral messages, fire-and-forget | No |
| **Presence** | Real-time shared state (who is here) | In-memory only |
| **Data** | System-level typed events (agent lifecycle, tool calls, etc.) | Yes (in event store) |

---

### ADR-005: WebRTC parity via same message format

**Decision:** When WebRTC data channels are activated, they carry the identical message envelope. The transport layer is swapped; the protocol is identical.

**Rationale:** Enables seamless upgrade from WebSocket to WebRTC without client-side rewrite.

---

### ADR-007: `uar-realtime` — standalone service, not embedded

**Decision:** Realtime transport is implemented as a **dedicated `uar-realtime` Rust binary** — a separate process (and eventually a separate Docker service). UAR publishes events to it via an internal HTTP API. Clients connect to `uar-realtime` directly for all WebSocket and WebRTC connections.

**Rationale:**
- UAR is CPU-heavy (LLM inference); realtime is I/O-bound (connection fan-out). They have antagonistic resource profiles.
- UAR restarts/deploys must not disconnect clients. Separation gives independent lifecycles.
- Multiple UAR instances (horizontal scale) can all publish to a shared `uar-realtime` cluster.
- `uar-realtime` can be scaled independently — WebSocket/WebRTC fan-out has very different scaling characteristics from LLM inference.

**See:** [uar-realtime-service.md](./uar-realtime-service.md)

---

### ADR-008: `uar-realtime` owns ALL transport mediums

**Decision:** `uar-realtime` handles every realtime transport in the Prometheus ecosystem:
- **WebSocket** — subscriptions, events, presence, plugin channels
- **WebRTC signaling** — SDP offer/answer, ICE candidate exchange
- **WebRTC data channels** — identical `uar.realtime.v1` envelope over SCTP
- **WebRTC media (SFU)** — audio/video track routing for conferencing and agent voice
- **SSE** — server-push fallback for restricted environments
- _(future)_ **WebTransport** — HTTP/3-based low-latency upgrade path

**Rationale:** Centralising all transports in one service means:
- A single auth layer and JWT verifier
- A single `RealtimeBroker` that fans out to all transports simultaneously
- Clients on WebSocket and clients on WebRTC data channels receive the same events from the same broker with no duplication
- Video conferencing (SFU) can be co-located with the event fan-out infrastructure and share the same presence system

---

## 4. Comparison with the Current SSE Streaming

The existing `/api/chat/completion` SSE stream is a **one-way, short-lived, request-scoped** transport. It delivers `chat.completion.chunk` tokens during a single LLM run and closes.

The new Realtime system is:
- **Bi-directional** (client can send messages to server)
- **Persistent** (connection stays open across multiple runs, sessions)
- **Multiplexed** (many channels on one connection)
- **Presence-aware**

The SSE stream remains for OpenAI API compatibility. Realtime is additive, not a replacement.
