# UAR Realtime System

> Standardized, generic, bi-directional realtime messaging for the Universal Agent Runtime — available to views, API callers, and plugin developers over WebSocket (and later WebRTC data channels).

## Documents in this directory

| Document | Description |
|---|---|
| [overview.md](./overview.md) | Design goals, prior art survey, architectural decisions |
| [protocol.md](./protocol.md) | Wire protocol — message envelope, lifecycle, built-in events |
| [channels.md](./channels.md) | Standard channel taxonomy (system, agent, session, user, plugin) |
| [subscriptions.md](./subscriptions.md) | Subscription DSL — multi-channel subscriptions, server-side payload filters, extensible criteria |
| [rust-implementation.md](./rust-implementation.md) | Rust implementation — module layout, broker, emitter, handler, filter engine, scaling path |
| [uar-realtime-service.md](./uar-realtime-service.md) | **`uar-realtime` standalone service** — multi-transport hub (WS, WebRTC, SFU, SSE), internal publish API, deployment |
| [workspace.md](./workspace.md) | **Cargo workspace structure** — monorepo layout, shared crate design, Docker targets, migration steps |
| [api-reference.md](./api-reference.md) | HTTP upgrade endpoint, REST management API, SDK reference |
| [integration.md](./integration.md) | How to use from frontend, Rust backend, and 3rd-party plugins |
| [asyncapi.yaml](./asyncapi.yaml) | Machine-readable AsyncAPI 3.0 spec for the realtime endpoint |

## TL;DR

The UAR Realtime system is a **topic-routed, pub/sub + presence WebSocket channel protocol** inspired by Phoenix Channels and Supabase Realtime, formally described using AsyncAPI 3.0. A single WebSocket connection multiplexes unlimited named channels. Every message uses a compact JSON envelope. The transport layer is transport-agnostic: currently WebSocket, later WebRTC data channels via the same message format.

```
ws://host/api/realtime
  └─ channel  "agent:run:{run_id}"
  └─ channel  "session:{session_id}"
  └─ channel  "system:notifications"
  └─ channel  "plugin:{name}:{scope}"
```
