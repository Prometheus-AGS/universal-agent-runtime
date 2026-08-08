## Context

The frontend currently validates official `uar.agui/1` frames and then reduces
them to legacy chat event names. Chat rendering and Runtime Console ingestion
interpret that reduced payload independently, while run entities have no phase
timing projection. The `/api/uar/runs/{id}/stream` attach/replay path also maps
one normalized event to one wire frame, so it cannot prepend state/message
snapshots or synthesize the `TOOL_CALL_START` frame already emitted by the main
chat stream.

AG-UI defines snapshots as complete replacement state, state deltas as ordered
RFC 6902 patches, RAW as uninterpreted external data, and tool calls as
`TOOL_CALL_START` -> `TOOL_CALL_ARGS` -> `TOOL_CALL_END`. The implementation
must preserve those meanings, the existing `uar.agui/1` profile, monotonic
event identities, reconnect deduplication, and the repository's
store-to-service frontend layering.

## Goals / Non-Goals

**Goals:**

- Normalize each validated official frame once into a chat message chunk, a
  typed Runtime Console event row, and terminal phase timings where applicable.
- Attribute context, skill, memory, retrieval, reasoning, tool, and generation
  phases according to the migration plan and persist the completed timing map
  on the runtime run entity.
- Make attach/resume replay self-synchronizing by emitting state and message
  snapshots for the selected replay cursor before later deltas.
- Give the `/api/uar/runs/{id}/stream` AG-UI path the same official tool-start
  lifecycle behavior as the main chat stream.
- Preserve RAW payloads without interpreting or reshaping their external data.

**Non-Goals:**

- Replacing the existing chat message store or Runtime Console entity graph.
- Adding an AG-UI SDK dependency or changing the `uar.agui/1` wire profile.
- Reconstructing user messages that are not present in `RunManager` history.
- Persisting server-side phase timings; C-06 stores the client projection on
  the existing runtime run entity and leaves durable database work to its
  owning data-foundation change.
- Redesigning trace/timeline UI; later changes consume the typed projections.

## Decisions

### Normalize official frames before legacy reduction

`platform/agui/agui-normalizer.ts` will accept only a validated
`UarAguiEvent`. It will return a canonical event row for every frame, an
optional text/reasoning message chunk, and an optional completed timing map.
`UarAguiAdapter` remains the per-stream owner of deduplication, ordering, and
state-patch recovery, but its result will include these projections alongside
the compatibility `event` and `payload` fields.

This keeps one parse/validation boundary and lets existing chat cases migrate
incrementally. Replacing all legacy event reduction in this change was rejected
because the store still handles citations, artifacts, approvals, and memory
content that are not message chunks.

### Derive timings from one per-stream clock

The normalizer records the first and last observed timestamp for each mapped
phase and the run window. On `RUN_FINISHED` or `RUN_ERROR`, each span is clamped
to that window. Time inside the run window not attributed to a non-generation
phase is assigned to `generate`; explicit text-generation span and gap time are
combined. The result is emitted once and upserted as `phase_timings` on the
`RuntimeRun` entity.

The clock is injected, defaulting to `Date.now`, so deterministic tests can
prove mapping, clamping, gap attribution, and terminal-only emission. Computing
timings continuously was rejected because consumers could observe totals that
change meaning as late events arrive.

### Snapshot exactly at the replay boundary

The run stream will read the retained history once and choose a snapshot
cursor. For a new attach without a cursor, the cursor is the newest retained
event, so snapshots represent current state/messages and the client continues
with live events. For resume, the supplied cursor is used, snapshots are
reconstructed from retained events through that cursor, and only later events
are replayed.

State reconstruction applies the supported `add`, `replace`, and `remove`
operations to UAR's initial state shape. Message reconstruction concatenates
assistant `ChatDelta` content through the cursor. Snapshot frames use distinct
profile event IDs at the cursor and precede replay/live deltas. Replaying all
history after a current-state snapshot was rejected because conforming clients
would append message content twice.

### Keep transport lifecycle synthesis at the stream boundary

`to_agui_spec_event` keeps `ToolStart` -> `TOOL_CALL_END` because UAR's
normalized `ToolStart` means the tool name and complete arguments are known;
changing it to `TOOL_CALL_START` would remove the required end frame. The run
stream instead tracks tool-call IDs and emits `TOOL_CALL_START` before the first
mapped args/end frame, mirroring the main chat stream. Tests cover start before
args/end and exactly-once synthesis.

A larger rename/split of normalized tool events was rejected as a cross-domain
runtime migration beyond C-06.

### RAW is observable but opaque

The frontend compatibility mapping will expose official RAW as `agui.raw`, and
the event row will retain the validated wire payload unchanged. It will not
derive chat content or phase-specific fields from the raw body. This satisfies
passthrough without creating a second protocol parser or trusting external raw
data as UAR domain state.

## Risks / Trade-offs

- **Retained history can begin after an evicted state mutation** -> snapshots
  are explicitly limited to `RunManager`'s retained replay window; patch
  reconstruction starts from the documented UAR initial shape and tests fail
  closed on an invalid patch rather than emitting a misleading state.
- **Client receipt time is not server execution time** -> C-06 records a
  deterministic transport-observation timing projection; event-schema
  timestamps can replace the injected clock later without changing consumers.
- **Phase spans may overlap** -> every span is clamped to the run window and
  unattributed time is never negative; consumers receive the observed spans
  rather than fabricated ordering.
- **Snapshot event IDs share an SSE cursor** -> profile `eventId` ordinals remain
  distinct, and adapter deduplication continues to key on `eventId`, not the SSE
  cursor.
- **Legacy frames lack the official typed source** -> existing legacy reduction
  remains compatible, but the three new projections are guaranteed only for
  validated `uar.agui/1` frames.

## Migration Plan

1. Add normalizer types/tests and attach projections to the existing adapter.
2. Route message chunks, event rows, and terminal timings to their current
   stores/entities without changing component APIs.
3. Add replay snapshot reconstruction and tool-start synthesis with focused
   Rust tests and the existing live seam.
4. Run frontend cheap gates during implementation, then the Wave 2 full
   frontend test/build boundary after C-06 verification.
5. Roll back by removing the projection fields and snapshot prefix; existing
   legacy reduction remains intact throughout the change.

## Open Questions

None. Durable server-side timing persistence and replay beyond the retained
history window are explicitly assigned outside C-06.
