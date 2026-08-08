# ag-ui-chat-conformance Specification

## Purpose
TBD - created by archiving change certify-agui-chat-flow. Update Purpose after archive.
## Requirements
### Requirement: Declared AG-UI conformance
UAR SHALL publish a versioned mapping from normalized runtime events to AG-UI lifecycle, message, tool, state, raw, and custom events.

#### Scenario: Resume without duplication
- **WHEN** a client reconnects with a valid replay cursor
- **THEN** it reconstructs the same run state in order without duplicate logical events

#### Scenario: State divergence
- **WHEN** a state delta cannot be applied
- **THEN** the client requests or consumes a fresh snapshot instead of silently diverging

### Requirement: Single-pass AG-UI consumer normalization
The frontend SHALL normalize each validated `uar.agui/1` frame once into a
typed event row and, when applicable, a message chunk and terminal phase timing
projection without reinterpreting the wire payload in individual consumers.

#### Scenario: Text and reasoning message chunks
- **WHEN** the adapter accepts a `TEXT_MESSAGE_CONTENT` or `REASONING_MESSAGE_CONTENT` frame
- **THEN** it emits one typed message chunk with the original message identifier and delta
- **AND** chat rendering consumes that chunk rather than extracting the delta again from the compatibility payload

#### Scenario: Runtime event row
- **WHEN** the adapter accepts any official profile frame, including a high-frequency content frame
- **THEN** it emits one event row preserving the profile event identity, sequence, official type, phase attribution, and payload
- **AND** Runtime Console ingestion consumes that row

#### Scenario: Duplicate replay frame
- **WHEN** an official frame repeats an event identity already accepted by the per-stream adapter
- **THEN** the adapter emits no compatibility event, message chunk, timing update, or event row for the duplicate

### Requirement: Terminal run phase timings
The frontend SHALL map observed official events into `context`, `skill`,
`memory`, `retrieval`, `reasoning`, `tool`, and `generate` phases and SHALL emit
the completed, run-window-clamped timing map exactly once at terminal run
completion.

#### Scenario: Phase mapping
- **WHEN** context-update, skill-activation, memory, citation/RAG, reasoning, tool, and text-generation frames are accepted
- **THEN** each frame is attributed to `context`, `skill`, `memory`, `retrieval`, `reasoning`, `tool`, and `generate` respectively

#### Scenario: Unattributed run time
- **WHEN** a run finishes with time inside its observed run window that is not attributed to a non-generation phase
- **THEN** the unassigned duration is included in the `generate` timing
- **AND** no phase duration extends outside the observed run window

#### Scenario: Run timing persistence
- **WHEN** `RUN_FINISHED` or `RUN_ERROR` terminates an observed run
- **THEN** the normalizer emits one complete timing map
- **AND** the RuntimeRun entity is upserted with that map as `phase_timings`

### Requirement: Cursor-consistent attach and replay snapshots
The `agui_spec` run stream SHALL emit a complete state snapshot and message
snapshot at the selected cursor before emitting any later state or message
deltas.

#### Scenario: New stream attachment
- **WHEN** a client attaches without a replay cursor
- **THEN** the server snapshots retained state and assistant messages through the newest retained event
- **AND** begins delivery with distinct `STATE_SNAPSHOT` and `MESSAGES_SNAPSHOT` frames before subsequent live deltas

#### Scenario: Cursor resume
- **WHEN** a client resumes with a valid last-event cursor
- **THEN** the server reconstructs state and assistant messages through that cursor
- **AND** emits those snapshots before replaying only events after the cursor
- **AND** the resumed transcript contains no duplicate logical message content

#### Scenario: State patch cannot be reconstructed
- **WHEN** retained history contains a state patch that cannot be applied to the UAR initial state at the selected cursor
- **THEN** the server does not emit a state snapshot that falsely claims synchronization

### Requirement: Official tool and RAW parity
The AG-UI transport and frontend adapter SHALL preserve official tool-call
lifecycle ordering and SHALL expose RAW external payloads without interpreting
their contents as UAR domain state.

#### Scenario: Tool lifecycle in run replay
- **WHEN** retained history contains a normalized tool call
- **THEN** `agui_spec` replay emits exactly one `TOOL_CALL_START` for its tool-call identifier before the corresponding args or end frame
- **AND** later emits `TOOL_CALL_RESULT` when the normalized tool result arrives

#### Scenario: RAW passthrough
- **WHEN** the frontend accepts a validated RAW profile frame
- **THEN** the adapter emits an `agui.raw` compatibility event and an official RAW event row
- **AND** preserves the raw payload without deriving a message chunk or domain-state mutation from it

