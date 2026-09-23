## ADDED Requirements

### Requirement: Runs stream delimits text and reasoning per step
The `agui_spec` runs stream SHALL wrap each contiguous assistant text segment in `TEXT_MESSAGE_START` (with `role: "assistant"`) and `TEXT_MESSAGE_END`, and each contiguous reasoning segment in `REASONING_START`, `REASONING_MESSAGE_START`, `REASONING_MESSAGE_END` and `REASONING_END`, all carrying the segment's `messageId`. A segment SHALL end when a tool call starts, when the other kind of segment starts, when the step finishes, or when the run terminates. Each segment SHALL have a `messageId` unique within the run that identifies its step, in the form `{runId}:step-{n}:text-{k}` or `{runId}:step-{n}:reasoning-{k}`, where `n` is the step named by the enclosing `STEP_STARTED` and `k` counts segments of that kind within the step from 0. No content frame SHALL be emitted outside an open segment, and every opened segment SHALL be closed before `RUN_FINISHED` or `RUN_ERROR`. Replay SHALL reproduce the same boundaries and identifiers as live delivery.

#### Scenario: Text in two steps
- **WHEN** a run streams text in step 1, calls a tool, and streams text in step 2
- **THEN** the stream contains two text segments with different message ids naming steps 1 and 2, each opened by `TEXT_MESSAGE_START` and closed by `TEXT_MESSAGE_END`

#### Scenario: Reasoning then text in one step
- **WHEN** a model streams reasoning and then text within one step
- **THEN** the reasoning segment is closed with `REASONING_MESSAGE_END` and `REASONING_END` before `TEXT_MESSAGE_START` of the text segment

#### Scenario: Cancelled mid-segment
- **WHEN** a run is cancelled while a text segment is open
- **THEN** `TEXT_MESSAGE_END` for that segment precedes the terminal `RUN_ERROR`

### Requirement: Tool results carry their outcome and tool name
Every `TOOL_CALL_RESULT` frame on the runs stream SHALL carry `toolCallName` and a boolean `isError` that is `true` exactly when the runtime recorded the tool call as failed.

#### Scenario: Failing tool
- **WHEN** a tool call fails
- **THEN** its `TOOL_CALL_RESULT` has `isError: true` and `toolCallName` equal to the called tool

#### Scenario: Succeeding tool
- **WHEN** a tool call succeeds
- **THEN** its `TOOL_CALL_RESULT` has `isError: false`

### Requirement: A lagging subscriber is told, not skipped
When a runs-stream subscriber falls far enough behind that events for it are dropped, the server SHALL NOT continue that stream past the gap. It SHALL send one final `CUSTOM` frame named `uar.stream.lagged` whose value carries `lastDeliveredEventId` (the SSE id of the last event delivered on that stream), `skippedEvents`, and `oldestRetainedEventId`, and SHALL then end the stream. The frame SHALL carry the `sequence` of the last delivered event so a consumer that discards regressing sequences accepts it. A stream ended this way SHALL NOT cause the run to be cancelled as abandoned unless no subscriber has attached when a resync grace period, longer than the ordinary disconnect grace, expires. The same signal SHALL be sent as `agui.stream.lagged` on the legacy stream mode.

#### Scenario: Forced slow consumer
- **WHEN** a subscriber stops reading while the run emits more events than the per-run buffer holds
- **THEN** the subscriber receives every event up to some id without a gap, then one `uar.stream.lagged` frame naming that id, and then the stream ends

#### Scenario: Resync after lag
- **WHEN** the subscriber reconnects with `last_event_id` equal to `lastDeliveredEventId` within the resync grace period
- **THEN** it receives every later event in order with no gap and the run was not cancelled

### Requirement: Runs stream frames follow a documented ordering contract
For every frame on the `agui_spec` runs stream, the SSE `id` SHALL be the id of the runtime event it was derived from. Runtime event ids SHALL increase by one per event within a run; a runtime event may yield no frame of its own, so a skipped id on the stream SHALL NOT mean loss, and loss SHALL be signaled only by `uar.stream.lagged` or `uar.stream.resync_required`. `sequence` SHALL equal that id multiplied by 16 and SHALL be shared by every frame derived from the same runtime event, so it never decreases along the stream. `eventId` SHALL be unique within the run and SHALL be `{id}:{ordinal}`, where `ordinal` orders frames derived from one runtime event; snapshot, lag and resync frames SHALL use the reserved ordinals and suffixes listed in the profile document. Consumers SHALL be able to reconstruct emission order by sorting on (`sequence`, `ordinal`). `RUN_STARTED` SHALL carry `profileRevision: 2`.

#### Scenario: Frames from one runtime event
- **WHEN** one runtime tool event yields `TOOL_CALL_START`, `TOOL_CALL_ARGS` and `TOOL_CALL_END`
- **THEN** the three frames share `id` and `sequence` and have ordinals 0, 1 and 2 in emission order

#### Scenario: Profile revision
- **WHEN** a client attaches to a run started after this change
- **THEN** `RUN_STARTED` carries `profileRevision: 2`

## MODIFIED Requirements

### Requirement: Cursor-consistent attach and replay snapshots
The `agui_spec` run stream SHALL emit a complete state snapshot and message
snapshot at the selected cursor before emitting any later state or message
deltas. Snapshots SHALL be exact even after older events have been evicted from
retained history: the server SHALL keep, for each run, the state and message
content produced by evicted events. The message snapshot SHALL contain one
message per text segment and per completed tool result, with the same message
ids the live stream used.

#### Scenario: New stream attachment
- **WHEN** a client attaches without a replay cursor
- **THEN** the server snapshots retained state and assistant messages through the newest retained event
- **AND** begins delivery with distinct `STATE_SNAPSHOT` and `MESSAGES_SNAPSHOT` frames before subsequent live deltas

#### Scenario: Cursor resume
- **WHEN** a client resumes with a valid last-event cursor
- **THEN** the server reconstructs state and assistant messages through that cursor
- **AND** emits those snapshots before replaying only events after the cursor
- **AND** the resumed transcript contains no duplicate logical message content

#### Scenario: Attach during an open segment
- **WHEN** a client attaches while a text segment is open
- **THEN** the message snapshot contains that segment's content so far under its message id
- **AND** later `TEXT_MESSAGE_CONTENT` frames for that id follow without a second `TEXT_MESSAGE_START`, and its `TEXT_MESSAGE_END` follows

#### Scenario: Cursor older than retained history
- **WHEN** a client resumes with a cursor older than the oldest event the server can still replay
- **THEN** the server first sends a `CUSTOM` frame `uar.stream.resync_required` carrying the requested cursor, the snapshot cursor it will use, and the number of events that can no longer be replayed
- **AND** sends state and message snapshots that are exact at that snapshot cursor, then replays every retained event after it

#### Scenario: State patch cannot be reconstructed
- **WHEN** retained history contains a state patch that cannot be applied to the UAR initial state at the selected cursor
- **THEN** the server does not emit a state snapshot that falsely claims synchronization
