---
sidebar_position: 2
title: Inspect and Cancel Runs
description: Follow a run stream, inspect its trace, and cancel owned work.
source_records:
  - openspec/specs/runtime-event-replay-entity-sync/spec.md
  - docs/product-surface-inventory.md
current_authority: /docs/operations/runs
---

# Inspect and Cancel Runs

A run is the server-owned execution of an agent artifact and input. UAR assigns the run ID, emits normalized lifecycle events, and checks the authenticated user before stream, approval, checkpoint, resume, or cancel operations.

:::warning Boundary statement
Run event replay is scoped to history retained by the active run manager. Browser PGlite is a local inspection store. Neither is a durable, cross-process execution ledger.
:::

## Packaged UI workflow

Start a run from the chat surface. The response stream carries the server-assigned run ID; the chat Stop action calls the server cancel endpoint. Open `/admin/runs` to select a run and inspect its messages, reasoning, tools, state changes, artifacts, checkpoints, and related conversation. Pending approval is resolved from the chat dialog or `/admin/approvals`.

The Runs page is primarily an inspection surface. Cancellation is initiated by the active chat Stop control or the API described below; do not infer a cancellation control from the run list alone.

## API workflow

| Method | Endpoint | Purpose |
|---|---|---|
| `POST` | `/api/uar/runs` | Create a run and receive `run_id` plus `stream_url` |
| `GET` | `/api/uar/runs/{run_id}/stream` | Subscribe and replay retained events after an optional cursor |
| `POST` | `/api/uar/runs/{run_id}/cancel` | Cancel an owned in-flight run |
| `POST` | `/api/uar/sessions/{session_id}/cancel` | Cancel the active run projected through a session |
| `POST` | `/api/uar/runs/{run_id}/approval` | Resolve the packaged UI approval channel |
| `GET` | `/api/uar/runs/{run_id}/checkpoints` | List checkpoints when persistence is configured |
| `POST` | `/api/uar/runs/{run_id}/resume` | Resume through the supported checkpoint path |

Run cancellation is idempotent for an owned known run: the body reports whether live work was cancelled. An unknown or other-user run is not exposed as cancellable. The stream supports `last_event_id` and the `Last-Event-ID` header; `stream_mode=agui_spec` also emits an AG-UI replay snapshot.

## Terminal outcome

The terminal event tells the client whether a run completed, failed, or was cancelled. Cancellation propagates through the run token and emits a cancelled terminal event when the runtime observes it. Repeated cancellation must not create a second terminal outcome.

Disconnecting the last stream subscriber starts a short guard path that can cancel the run when no subscriber returns. Explicit cancel remains the clearest operator intent.

## Pending approval

A pending approval parks a tool call inside the run. Approving resumes that call; rejecting, timeout, channel close, or run cancellation prevents execution. See [Resolve Tool Approvals](/docs/governance/approvals) for the exact policy order.

## State ownership and durability

The server run manager owns live runs, event subscriptions, approval channels, and in-process replay history. Configured persistence can own checkpoints. The browser stores normalized run and event rows in its own PGlite database for local trace rendering. A reload can recover those local rows, but another browser or a restarted server does not automatically share that same history.

## Profile limits

The HTTP run API is available in `server-full` and `minimal`. The packaged Runs, chat, and approval surfaces are `server-full`. `embedded-mobile` must map host lifecycle and persistence onto its own run surface. These docs do not claim durable replay across process restart, cross-browser history, or blanket tenant isolation.

See [Runtime Console](/docs/operations/runtime-console) for the broader projection and [Realtime State](/docs/operations/realtime) for the two streaming boundaries.
