## 1. Contract tests first (must fail before implementation, pass after)

Shared fixtures: a scripted `MockLlmDriver` turn set — step 1 streams reasoning `R1`, text `T1a`, then two tool calls (`ok_tool` succeeds, `bad_tool` fails); step 2 streams reasoning `R2` and text `T2`. A helper that reads a full SSE body from `GET /api/uar/runs/{id}/stream?stream_mode=agui_spec` into parsed frames with their SSE ids.

D2 falsifier map: F3 (ordering and attribution) → 1.1, 1.2, 1.6; F4 (no silent loss: every event type accounted for; slow consumer resyncs) → 1.3, 1.4, 1.5.

- [ ] 1.1 `tests/agui_runs_stream_fidelity.rs::recorded_trace_translates_in_order_with_step_attribution` — run the scripted turns, capture the stream, and compare the frame sequence (type, `messageId`, `toolCallId`, `toolCallName`, `isError`, `stepName`; run ids normalized) with a checked-in golden `tests/fixtures/agui/runs-stream-trace.json`. Assert: every CONTENT frame lies between a START and END for its `messageId`; the reasoning order is `REASONING_START`, `REASONING_MESSAGE_START`, content, `REASONING_MESSAGE_END`, `REASONING_END`; `T1a` and `T2` have different ids naming steps 1 and 2; each tool call has START, ARGS, END, RESULT in that order; `bad_tool`'s RESULT has `isError: true`, `ok_tool`'s `false`, both carry `toolCallName`; sorting frames by (`sequence`, ordinal) reproduces arrival order; `eventId`s are unique; `RUN_STARTED` has `profileRevision: 2`. Verify: fails today (no START/END, fixed ids, no `isError`).
- [ ] 1.2 `agui_runs_stream_fidelity.rs::replay_reproduces_live_boundaries_and_ids` — after the run finishes, attach again with `last_event_id` at a point inside `T2`. Assert `MESSAGES_SNAPSHOT` holds `T1a` complete and the partial `T2` under the live ids, the following frames contain `T2`'s remaining content and its `TEXT_MESSAGE_END` with no second `TEXT_MESSAGE_START`, and the union equals the live capture.
- [ ] 1.3 `agui_runs_stream_fidelity.rs::every_runtime_event_variant_is_accounted_for` — construct one value of every `NormalizedEvent` variant inside an exhaustive `match` with no wildcard arm (a new variant fails compilation). For each, encode through the agui_spec projector and the legacy `to_agui_event`. Assert each yields at least one frame whose `type` is in `schemas/uar-agui-v1.schema.json` and, for `CUSTOM`, whose `name` appears in the custom-event table of `docs/protocols/ag-ui-profile.md`, or the variant is listed in an explicit `NO_FRAME` table with a reason. If some variant cannot be built from outside the crate, the same assertion runs as a `#[cfg(test)]` module in `src/uar/api/sse.rs` and this test asserts that module's variant list is complete.
- [ ] 1.4 `agui_runs_stream_fidelity.rs::slow_consumer_gets_a_lag_signal_not_a_gap` — scripted driver streams 2000 text deltas; the client opens the stream and does not read until the run finishes. Assert: the frames received before the signal cover every runtime event id up to `lastDeliveredEventId` (checked against `history_since`, allowing ids that yield no frame), exactly one `CUSTOM uar.stream.lagged` with that id and `skippedEvents > 0` follows, and the stream ends. Reconnect 1 s later (above the 250 ms guard, below the resync grace) with `last_event_id = lastDeliveredEventId`; assert the concatenated text equals the scripted text exactly and the run finished rather than being cancelled. Verify: fails today (events silently skipped, text shorter than scripted).
- [ ] 1.5 `agui_runs_stream_fidelity.rs::cursor_older_than_history_gets_resync_required_and_exact_snapshots` — a run that emits more than 512 events including state patches; attach with `last_event_id = 1` after it ends. Assert `uar.stream.resync_required` comes first with `requestedCursor: 1` and `snapshotCursor` equal to the last evicted id; `STATE_SNAPSHOT` equals the state computed by applying every patch in order; `MESSAGES_SNAPSHOT` text equals all text up to `snapshotCursor`; the retained events follow without a gap. Verify: fails today (partial snapshot, no signal).
- [ ] 1.6 `agui_runs_stream_fidelity.rs::cancel_closes_open_segments_before_terminal` — cancel during a text segment; assert `TEXT_MESSAGE_END` for it precedes `RUN_ERROR` with code `CANCELLED`.
- [ ] 1.7 Extend `tests/agui_golden.rs::shared_agui_golden_fixture_matches_the_profile_schema` — the schema requires `toolCallName` and `isError` on `TOOL_CALL_RESULT`, and the golden fixture includes START/END, reasoning boundaries, `uar.stream.lagged` and `uar.stream.resync_required` frames.
- [ ] 1.8 Frontend (vitest, `frontend/src/platform/agui/agui-adapter.test.ts`): `TOOL_CALL_RESULT` with `isError: true` maps to `success: false`; absent `isError` maps to `success: true`; `uar.stream.lagged` is surfaced as a resync request carrying `lastDeliveredEventId`, not as a generic `agui.custom`.
- [ ] 1.9 Record the failing output of 1.1–1.8 against the unmodified branch before implementation starts.

## 2. Runtime events

- [ ] 2.1 Add `MessageStarted`/`MessageEnded` variants and the `message_id` field on delta variants; verify the crate builds with `cargo check`.
- [ ] 2.2 Track step and open segment in `EventHistory::publish`; publish boundaries and stamp ids; verify 1.1, 1.6.
- [ ] 2.3 Add the eviction base (state, synchronization flag, text and tool-result accumulators); verify 1.5.

## 3. Encoding and stream

- [ ] 3.1 Map boundary events, tool result fields and `profileRevision` in `adapters.rs`; update the schema and golden fixture; verify 1.1, 1.3, 1.7.
- [ ] 3.2 Build snapshots from base plus retained events and emit `uar.stream.resync_required`; verify 1.2, 1.5.
- [ ] 3.3 Replace `filter_map(Result::ok)` with lag detection and the final signal; give lag-ended streams the resync grace in `RunDisconnectGuard`; verify 1.4.

## 4. Consumers and documents

- [ ] 4.1 `agui-adapter.ts`, `runtime-chunk-projection.ts`, `chat-stream-api.ts`: read `isError` and `toolCallName`, reconnect on `uar.stream.lagged` with its cursor, show `uar.stream.resync_required` as a notice; verify 1.8 and a manual run in the UAR console with a failing tool.
- [ ] 4.2 Update `docs/protocols/ag-ui-profile.md` with the ordering contract, reserved identifiers, per-step ids and the two custom events; verify 1.3 reads the new table.
- [ ] 4.3 Advertise the `agui_stream_fidelity` flag and AG-UI profile revision 2 in `GET /api/uar/capabilities` (added by `sidecar-launch-security`) once this change ships; verify the endpoint reports revision 2 only with the new frames live.

## 5. Phase-boundary verification

- [ ] 5.1 Run `cargo test --test agui_runs_stream_fidelity --test agui_golden` and the frontend adapter tests once after sections 2–4 are complete, then the broader integration gate once; record outputs.
- [ ] 5.2 Run `openspec validate agui-runs-stream-fidelity --strict`; verify it passes.
