---
sidebar_position: 4
title: Understand Realtime State
description: Distinguish run event streams from the shared live-entity transport.
source_records:
  - openspec/specs/runtime-event-replay-entity-sync/spec.md
  - openspec/specs/entity-surreal-live-adapter/spec.md
current_authority: /docs/operations/realtime
---

# Understand Realtime State

UAR has two distinct realtime paths: a per-run execution stream and a multiplexed entity-change stream. They serve different state owners and have different replay guarantees.

:::warning Boundary statement
Reconnection is transport recovery. It is not durable replay across process restart, history eviction, browser storage loss, or a source that never retained the missed event.
:::

## Run stream and AG-UI replay

`GET /api/uar/runs/{run_id}/stream` subscribes to an owned run. A `last_event_id` query value or `Last-Event-ID` header requests events retained after that cursor. With `stream_mode=agui_spec`, the server also builds an AG-UI replay snapshot from the current run manager's retained history.

This AG-UI replay boundary is process-scoped. It can bridge a short disconnect while the run and its history remain available; it is not a durable event log after server restart.

## Shared live entity stream

`GET /api/live` merges enrolled entity topics into one shared EventSource per browser origin. Each event includes its topic, identifier, operation, and data. The frontend performs topic demultiplexing so separate entity adapters receive only their rows.

The single connection avoids exhausting the browser's HTTP/1.1 connection budget. The older `/api/live/{topic}` endpoint still exposes one known topic, while the packaged React application uses the multiplexed endpoint for applicable remote Surreal and Postgres notification paths.

Embedded Surreal uses `/api/uar/sync/stream` as its SSE bridge. If persistence discovery or transport setup fails, the frontend can fall back to REST reads and stale-time behavior rather than claiming a live subscription.

## Reconnect behavior

The shared adapter starts at a one-second delay, applies exponential reconnect, and caps the delay at 30 seconds. The default maximum attempt count is infinite while at least one listener remains. A successful open resets the attempt count. Removing the last listener closes the EventSource and cancels a pending timer.

Events dropped because a broadcast receiver lagged are skipped by the server stream. The `/api/live` payload does not carry a durable cursor. Some entity-sync paths reconcile an authoritative snapshot after reconnect, but consumers must not assume every topic can recover every missed transition.

## Offline and embedded behavior

Browser PGlite can retain local rows and support local-first UI behavior. That does not mean the server processed offline mutations, nor that another device sees them. The `embedded-mobile` profile relies on its host to own connectivity, persistence, retry limits, foreground/background transitions, and conflict handling.

## State ownership and durability

The run manager owns retained run history for the active process. Server persistence and live buses own authoritative entity rows and notifications according to the configured backend. Browser PGlite and the entity graph own local projections. EventSource owns only the connection; it does not make an event durable.

## Profile limits

Run and live HTTP streams apply to `server-full` and relevant `minimal` server builds. The branded shared adapter is part of the `server-full` React application. `embedded-mobile` has a separate host-controlled lifecycle. Realtime availability makes no cross-profile, cross-tenant, or backup/recovery claim.

See [Runs](/docs/operations/runs) for cursored execution streams and [Recovery and Shutdown](/docs/operations/recovery-and-shutdown) for process boundaries.
