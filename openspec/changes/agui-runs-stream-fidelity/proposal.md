## Why

the-boss will translate UAR's AG-UI runs stream into its own message parts and resync from history when it falls behind (decision D2, revision 3). The runs stream (`GET /api/uar/runs/{id}/stream?stream_mode=agui_spec`) cannot support that today:

- It emits `TEXT_MESSAGE_CONTENT` and `REASONING_MESSAGE_CONTENT` with no `START`/`END` frames (`src/uar/api/adapters.rs:92-108`). Only the chat-completions path emits them (`src/server.rs:5590-5670`).
- The message id is fixed per run (`"{run_id}:assistant"`, `"{run_id}:reasoning"`), so text from different steps is one message and cannot be attributed to a step.
- `TOOL_CALL_RESULT` has no tool name and no error flag (`adapters.rs:160-172`), although the runtime event has both (`ToolEnd { tool, ok }`, `src/uar/domain/events.rs:67-74`). UAR's own frontend therefore marks every tool result successful (`frontend/src/platform/agui/agui-adapter.ts:148-152`).
- A subscriber that falls more than 256 events behind loses events silently: the per-run broadcast channel holds 256 (`src/uar/runtime/manager.rs:2121`) and lag errors are dropped by `BroadcastStream::new(rx).filter_map(Result::ok)` (`src/uar/api/routes.rs:134-136`).
- A reconnect with a cursor older than the 512-event history (`manager.rs:46`) silently replays what remains (`manager.rs:5618-5642`), and the replay snapshot is rebuilt from the retained events only (`src/uar/api/sse.rs:122-165`), so message text and state from evicted events are missing from a snapshot that claims to be complete.

## What Changes

- The runs stream emits `TEXT_MESSAGE_START`/`END` around each assistant text segment and `REASONING_START`, `REASONING_MESSAGE_START`, `REASONING_MESSAGE_END`, `REASONING_END` around each reasoning segment. Boundaries are recorded as runtime events, so live delivery, replay and snapshots agree.
- Each text and reasoning segment gets its own message id that names its step: `{run_id}:step-{n}:text-{k}` and `{run_id}:step-{n}:reasoning-{k}`. **BREAKING** for any consumer that relied on the fixed `{run_id}:assistant` id on the runs stream.
- `TOOL_CALL_RESULT` carries `toolCallName` and `isError`.
- A subscriber that lags receives a final `CUSTOM` frame `uar.stream.lagged` with the last event id it was delivered, and the stream ends. It reconnects with that id. A lag-ended stream does not count as the last subscriber leaving until a resync grace period passes.
- A reconnect whose cursor is older than the retained history receives `CUSTOM uar.stream.resync_required`, then exact state and message snapshots at the oldest point UAR can still prove, then the retained events. Snapshots stay exact after eviction because the history keeps a base state and message accumulators for evicted events.
- The frame ordering contract (`id`, `sequence`, `eventId`) is written down in the profile document, and `RUN_STARTED` declares `profileRevision: 2`.

## Capabilities

### New Capabilities
(none)

### Modified Capabilities
- `ag-ui-chat-conformance`: adds requirements for message boundaries and per-step ids on the runs stream, tool result outcome and name, the lag signal, and the ordering contract; modifies "Cursor-consistent attach and replay snapshots" to cover cursors older than retained history.

## Impact

- **Code:** `src/uar/runtime/manager.rs` (`EventHistory` publish path: boundary events, message ids, eviction base), `src/uar/domain/events.rs` (boundary variants, message id on delta events), `src/uar/api/adapters.rs` (frames, tool result fields, profile revision), `src/uar/api/sse.rs` (snapshot from base plus retained events, resync frame), `src/uar/api/routes.rs` (lag handling in `stream_run`), `RunDisconnectGuard` (`manager.rs:636-676`, resync grace).
- **Consumers that must change:** `frontend/src/platform/agui/agui-adapter.ts` (read `isError` instead of `success: true`; handle `uar.stream.lagged` and `uar.stream.resync_required`), `frontend/src/services/chat-stream-api.ts` (reconnect with the lagged cursor), `frontend/src/platform/agui/runtime-chunk-projection.ts` (tool result name and failed status). `frontend/src/stores/chat-stream-store.ts:945-950` already honors `success`. The Python SDK (`sdks/python/.../client.py:162-166`) passes frames through and needs only documentation. The chat-completions AG-UI path (`server.rs:5590-5720`) keeps its per-run ids; it is not in scope and now differs from the runs stream.
- **Contracts:** `schemas/uar-agui-v1.schema.json` (TOOL_CALL_RESULT fields), `tests/fixtures/agui/uar-agui-v1.json` (golden), `docs/protocols/ag-ui-profile.md` (ordering contract, new custom events, per-step ids; its line 15 calls `sequence` "monotonic", but frames from one source event share it).
- **Realtime state:** retained history per run grows by the base accumulators (bounded by the run's own output). Replay after eviction is exact instead of partial.
- **Runtime UX:** tool failures render as failed in the UAR console; a lagging tab reconnects instead of showing a gap.
- **Provider compatibility:** unaffected.
- **KBD workflow state:** yes. This change belongs to the-boss child phase `the-boss-universal-agent-runtime` (step 2); that phase's `progress.json` in `prometheus-skills-mini` must list it. This repository's `.kbd-orchestrator/` is not updated by this authoring pass.
