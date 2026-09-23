## Context

See proposal.md for why. The pieces involved:

- **Publish path.** Every runtime event for a run goes through `RunEventEmitter` into `EventHistory::publish`, which assigns `next_id`, pushes into a ring of 512 (`EVENT_HISTORY_LIMIT`, `src/uar/runtime/manager.rs:46, 59-90`), and sends on a `broadcast::channel(256)` (`manager.rs:2121`). `EventHistory` already keeps one presentation projection beyond the ring (`latest_presentation`, `manager.rs:59-64, 5632-5640`) — a precedent for state that must survive eviction.
- **Steps.** The orchestrator emits `RuntimeStep { step, kind }` at iteration boundaries (`src/llm/orchestrator.rs:1525, 2002, 2040, 2283, 2589`), mapped at `manager.rs:4790-4795`; the graph engine emits its own (`src/uar/runtime/graph/engine.rs:104`). The encoder turns them into `STEP_STARTED`/`STEP_FINISHED` with `stepName: "step-{n}"` (`src/uar/api/adapters.rs:269-282`).
- **Encoding.** `to_agui_spec_event` maps one runtime event to one frame (`adapters.rs:67-400`); `AguiSpecProjector` synthesizes `TOOL_CALL_START` and buffers early `TOOL_CALL_ARGS` (`src/uar/api/sse.rs:175-255`); `enrich_agui_spec_payload` sets `eventId = "{id}:{ordinal}"` and `sequence = id * 16` (`adapters.rs:463-484`). Replay snapshots are rebuilt from retained events only (`sse.rs:122-165`).
- **Stream route.** `stream_run` subscribes, reads `history_since`, builds the snapshot, chains replay with `BroadcastStream::new(rx).filter_map(Result::ok)` and wraps it in `RunDisconnectGuard` (`src/uar/api/routes.rs:88-168`). The guard cancels the run 250 ms after the last subscriber leaves (`manager.rs:636-676`).
- **Frontend.** The adapter drops frames whose `sequence` is lower than the last seen and dedupes by `eventId` (`frontend/src/platform/agui/agui-adapter.ts:212-216`). It already accepts all START/END types as lifecycle frames (`agui-adapter.ts:163-172`, `runtime-chunk-projection.ts:92-97`) and hard-codes `success: true` for tool results (`agui-adapter.ts:148-152`).

## Goals / Non-Goals

**Goals:**
- Every frame the-boss needs to build message parts is on the wire, with ids that attribute it to a step.
- A consumer can always tell that it missed something, and can always get back to an exact state.

**Non-Goals:**
- The chat-completions AG-UI path (`src/server.rs:5590-5720`). It keeps its per-run ids.
- A new profile id. The changes are additive frames and fields plus finer message ids, which the profile already treats as opaque (`docs/protocols/ag-ui-profile.md:27`). `profileRevision` on `RUN_STARTED` lets a host gate on the new behavior.
- Replaying evicted trace events (skills, memory, citations). They are counted in `resync_required`, not recovered.

## Decisions

### D1 — Boundaries are runtime events assigned at publish time
`EventHistory::publish` tracks, per run, the current step (from `RuntimeStep` started/finished), the open segment (none, text or reasoning) and per-step segment counters. Before a `ChatDelta` when no text segment is open it publishes a `MessageStarted { message_id, kind: Text }` event; before a reasoning delta, the reasoning equivalent; it publishes `MessageEnded` when a tool event, the other kind, a step finish or a terminal event arrives. It stamps `message_id` onto the delta events it publishes (a new field with a serde default, `None` at construction).
- **Why here:** `publish` is the one place every run path passes through, and what it records is what replay reads. Live frames, replayed frames and snapshots then carry the same ids by construction.
- **Alternative rejected:** inferring boundaries in `AguiSpecProjector`. The projector is per stream; a client that reconnects after a `START` would need the projector seeded from the snapshot, and the embedded encoder would need the same logic again. Two derivations of one fact drift.

### D2 — Frame mapping
`MessageStarted/Ended` for text map to `TEXT_MESSAGE_START` (`role: "assistant"`) and `TEXT_MESSAGE_END`. For reasoning they map to two frames each (`REASONING_START` + `REASONING_MESSAGE_START`, and `REASONING_MESSAGE_END` + `REASONING_END`) with ordinals 0 and 1, matching what the chat path already emits (`server.rs:5618-5665`). `TOOL_CALL_RESULT` adds `toolCallName: tool` and `isError: !ok` from `ToolEnd` (`events.rs:67-74`). `RUN_STARTED` adds `profileRevision: 2`. The legacy `to_agui_event` mode maps the boundary events to `agui.lifecycle` so it does not change shape.

### D3 — Lag ends the stream with a signal
`stream_run` replaces `filter_map(Result::ok)` with a scan that tracks the last delivered source id. On `BroadcastStreamRecvError::Lagged(n)` it yields one synthetic event rendered as `CUSTOM uar.stream.lagged { lastDeliveredEventId, skippedEvents: n, oldestRetainedEventId }` with SSE id and `sequence` of the last delivered event and `eventId "{id}:lagged"`, and then ends.
- **Why end instead of continuing:** after a gap the live stream is no longer a faithful suffix. A reconnect with `last_event_id` goes through the replay path, which is already correct.
- **Why the last delivered sequence:** the frontend discards frames whose `sequence` is lower than the last seen (`agui-adapter.ts:215`); an equal one is accepted.
- **Disconnect guard:** a lag-ended stream drops its `RunDisconnectGuard` with a `RESYNC_GRACE` (proposed 5 s) instead of 250 ms, so a single subscriber that reconnects promptly does not cancel its own run.

### D4 — Exact snapshots after eviction
When the ring evicts an event, `publish` folds it into an eviction base: the state value after applying its patch (and whether state is still synchronized), and message accumulators keyed by message id (text, and tool results by tool-call id). A snapshot at cursor `c`:
- if `c ≥ base_id` (the last evicted id): base plus retained events up to `c`. Exact.
- if `c < base_id`: emit `CUSTOM uar.stream.resync_required { requestedCursor: c, snapshotCursor: base_id, unreplayableEvents: base_id - c }` first, then snapshots at `base_id`, then all retained events.
Reasoning text is not placed in `MESSAGES_SNAPSHOT`; it is delivered live and in replay. After eviction it is part of what `unreplayableEvents` counts.
- **Why:** a snapshot built from the ring alone (`sse.rs:122-165`) is wrong for long runs but claims to be complete. The base makes it right with memory bounded by the run's own output, and it matches the `latest_presentation` precedent.
- **Alternative rejected:** raising the ring size. It moves the cliff; it does not remove it.

### D5 — Ordering contract written down
`docs/protocols/ag-ui-profile.md` gains the contract in the spec (id per runtime event, `sequence = id × 16` shared per runtime event, `eventId = {id}:{ordinal}`, sort by (`sequence`, ordinal)) and the reserved identifiers: snapshot frames use the cursor id with ordinals 1 (`STATE_SNAPSHOT`), 2 (`MESSAGES_SNAPSHOT`) and 3 (presentation snapshot) as today (`sse.rs:269-318`), plus suffixes `:lagged` and `:resync`. Line 15's "monotonic" becomes "non-decreasing".

## Risks / Trade-offs

- [Consumers that keyed on `{run_id}:assistant` break] → UAR's frontend does not: the legacy mapping drops `messageId` (`agui-adapter.ts:141-144`). The-boss is built against revision 2. Other external consumers are unknown; `profileRevision` lets them detect it.
- [Resync grace keeps an abandoned run alive for 5 s longer after a lag] → Accepted; the alternative is a slow single consumer cancelling its own run.
- [Eviction base memory] → Bounded by total text and tool-result size of the run, which the tool-output bound (`conversation-history-integrity`) already caps per result.
- [Run-record eviction from `sidecar-session-principal`] → The retention window after a terminal event must exceed `RESYNC_GRACE`, or a lagged host could reconnect to a 404. That change's rule accounts for it.
- [Boundary events add ring entries] → Roughly two per segment. Negligible against 512, but the eviction base makes the count irrelevant for correctness.

## Migration Plan

Deploy UAR and the UAR frontend adapter change together (the frontend change is tolerant of old streams: `isError` absent means success). Rollback: revert both.

## Open Questions

- `RESYNC_GRACE` value (proposed 5 s). Tuning only.
