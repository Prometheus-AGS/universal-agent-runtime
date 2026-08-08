# runtime-console Specification

## Purpose
TBD - created by archiving change runtime-console-entity-workflow. Update Purpose after archive.
## Requirements
### Requirement: Shared Workflow State
UAR SHALL coordinate runtime-console planning state through OpenSpec and KBD
across Codex, Claude Code, Cursor, and OpenCode.

#### Scenario: Tool starts work on the runtime console
- **WHEN** an agent tool begins work on the runtime console
- **THEN** it MUST read `openspec/config.yaml`, the active OpenSpec change, and
  `.kbd-orchestrator/current-waypoint.json` before planning or implementation.

#### Scenario: Phase progress changes
- **WHEN** an agent completes a runtime-console task
- **THEN** it MUST update `.kbd-orchestrator/phases/runtime-console-ux/progress.json`
  with the task status and source tool.

### Requirement: Surreal Memory Workflow Mirror
UAR SHALL expose Surreal Memory MCP as a secondary mirror for workflow state.

#### Scenario: Workflow state is persisted
- **WHEN** KBD workflow state changes
- **THEN** the state SHOULD be mirrored to the `surreal_memory` MCP server with
  project, phase, task, timestamp, and `source_tool` metadata.

#### Scenario: Workflow state conflicts
- **WHEN** file state and memory mirror state disagree
- **THEN** `.kbd-orchestrator/` MUST remain authoritative unless a human
  explicitly promotes the memory state.

### Requirement: Runtime Entity Graph
The frontend SHALL model live runtime activity as normalized graph entities.

#### Scenario: Runtime event arrives
- **WHEN** SSE, AG-UI, A2UI, provider health, route decision, approval, artifact,
  or memory events arrive
- **THEN** the frontend MUST normalize them into runtime entity types before UI
  components render them.

#### Scenario: Runtime entity updates
- **WHEN** an existing runtime entity is updated
- **THEN** every visible console surface that reads that entity MUST update
  without manual refresh.

### Requirement: Runtime Console UX
The frontend SHALL provide a compact runtime operations console inspired by
librefang's registry/detail structure.

#### Scenario: Operator opens admin console
- **WHEN** the operator opens `/admin`
- **THEN** the default surface MUST be the runtime cockpit rather than a static
  provider onboarding page.

#### Scenario: Operator searches console surfaces
- **WHEN** the operator presses Cmd+K or Ctrl+K
- **THEN** the console MUST open a command search dialog that navigates to
  runtime, provider, protocol, memory, skills, tools, and settings surfaces.

### Requirement: Protocol Compatibility Console
The frontend SHALL expose protocol and provider compatibility status in the
runtime console.

#### Scenario: Operator inspects protocols
- **WHEN** the operator opens the protocols surface
- **THEN** it MUST show Anthropic REST, OpenAI-compatible REST, AG-UI, A2UI, MCP,
  and `liter-llm` routing status areas.

#### Scenario: Model routing decision is available
- **WHEN** a model routing decision is present in the entity graph
- **THEN** the console MUST show the selected provider/model and the routing
  reason when available.

### Requirement: Persisted Run Trace Source
The runtime console SHALL derive run traces from the selected run and its ordered `run_event` records in browser PGlite without maintaining a second event ledger in component state.

#### Scenario: Operator selects a persisted run
- **WHEN** an operator selects a run that has persisted phase timings and events
- **THEN** the console MUST load that run's phase timings and every `run_event` in ascending sequence order
- **AND** each event MUST retain its stable event ID, wire sequence, type, normalized kind, timestamp, and payload.

#### Scenario: Selected run receives another persisted event
- **WHEN** PGlite commits a new `run_event` for the selected run
- **THEN** the visible trace MUST update through a live persistence subscription without polling or manual refresh
- **AND** an event selected by stable event ID MUST remain selected when it still exists.

#### Scenario: Operator switches selected runs
- **WHEN** the operator selects a different run
- **THEN** the console MUST unsubscribe from the previous run before subscribing to the new run
- **AND** events from the previous run MUST NOT appear in the new trace.

### Requirement: Phase-Proportional Trace Bar
The runtime console SHALL summarize persisted run phase timings as an accessible phase-proportional trace bar.

#### Scenario: Terminal run has phase timings
- **WHEN** the selected run contains one or more positive phase durations
- **THEN** the trace bar MUST render each present phase in the canonical phase order using its `--color-phase-*` token
- **AND** each segment's flex weight MUST be proportional to its duration with a 3 percent visual minimum
- **AND** its accessible label MUST report the exact phase name, duration, and percentage so the minimum width is not mistaken for quantitative precision.

#### Scenario: Operator navigates the trace bar by keyboard
- **WHEN** focus is within the horizontal phase listbox
- **THEN** Left, Right, Home, and End MUST move its active option according to listbox order
- **AND** activating a phase MUST select and scroll to the first visible event in that phase.

#### Scenario: Phase has no recorded duration
- **WHEN** a phase timing is zero or absent
- **THEN** the trace bar MUST omit its visual segment
- **AND** the surrounding UI MUST NOT imply that the phase executed.

### Requirement: Hierarchical Run Event Timeline
The runtime console SHALL represent the selected run as a deterministic `run → phase → event` hierarchy in which every persisted event appears exactly once.

#### Scenario: Event hierarchy is projected
- **WHEN** persisted events are loaded
- **THEN** the console MUST group attributed events beneath their canonical phase and unattributed events beneath `lifecycle`
- **AND** phase nodes MUST be ordered by their first event
- **AND** event leaves MUST remain ordered by persisted sequence.

#### Scenario: Operator filters event kinds
- **WHEN** the operator toggles one or more event-kind filter chips
- **THEN** only matching event leaves and phase ancestors with matching descendants MUST remain visible
- **AND** each chip MUST expose its text label, event count, and pressed state.

#### Scenario: Operator expands and navigates the tree
- **WHEN** focus is in the event tree
- **THEN** Up, Down, Left, Right, Home, End, Enter, and Space MUST support roving focus, expansion, collapse, and inspection across the visible hierarchy
- **AND** every tree item MUST expose its level, position, sibling count, expanded state when applicable, and selected state to assistive technology.

#### Scenario: Operator activates an event
- **WHEN** the operator activates an event row with pointer or keyboard input
- **THEN** that event MUST become the inspector selection
- **AND** the selected state MUST be communicated with text or an icon in addition to surface color.

#### Scenario: Message event has conversation identity
- **WHEN** a selected message or reasoning event identifies its persisted thread and message
- **THEN** the console MUST offer an explicit `Open in conversation` action that navigates to and focuses the corresponding message anchor.

### Requirement: Run Event Inspector
The runtime console SHALL inspect one selected persisted event through Payload, Timing, and Raw AG-UI tabs without interpreting protocol data as executable content.

#### Scenario: Operator inspects event payload
- **WHEN** an event is selected and the Payload tab is active
- **THEN** the console MUST render deterministic pretty JSON plus the event summary
- **AND** it MUST render JSON as text rather than HTML or markdown.

#### Scenario: Operator inspects event timing
- **WHEN** the Timing tab is active
- **THEN** the console MUST show the persisted start timestamp, preceding-event gap, sequence, and wire sequence
- **AND** it MUST show an explicit payload duration or a matched start/end duration when one exists
- **AND** it MUST label an event with no factual duration as `instant` rather than deriving duration from the next event's arrival.

#### Scenario: Operator inspects raw AG-UI
- **WHEN** the Raw AG-UI tab is active
- **THEN** the console MUST show the verbatim persisted event representation in a copyable text block
- **AND** an explicit copy action MUST announce success or failure through a polite live region.

#### Scenario: Persisted payload contains executable-looking text
- **WHEN** a payload contains HTML, a script URL, an event-handler attribute, or another executable-looking string
- **THEN** the inspector MUST display those bytes only as escaped text
- **AND** it MUST NOT send them to `innerHTML`, the markdown pipeline, navigation, or dynamic code execution.

### Requirement: Offline Trace and Independent Network Actions
The runtime console SHALL keep persisted trace inspection available independently from network-backed checkpoint, resume, and replay actions.

#### Scenario: Runtime API is unavailable
- **WHEN** checkpoint discovery or A2UI replay fails while the selected run exists in PGlite
- **THEN** the phase bar, event timeline, filters, and event inspector MUST remain usable
- **AND** the failed network capability MUST expose its own actionable error state without replacing the local trace.

#### Scenario: Realtime event arrives while network actions are failing
- **WHEN** PGlite receives another event during a checkpoint or replay error
- **THEN** the trace MUST still update from local persistence
- **AND** the network error MUST remain scoped to its affected action.

### Requirement: Checkpoint Discovery and Run Resume
The runtime console SHALL consume the existing checkpoint-list and latest-resume contracts without reconstructing runtime execution state in the browser.

#### Scenario: Selected run has checkpoints
- **WHEN** `GET /api/uar/runs/{run_id}/checkpoints` succeeds
- **THEN** the console MUST expose the returned checkpoints in creation order with ID, node, iteration, and timestamp
- **AND** checkpoint state and messages MUST be inspectable only as inert JSON.

#### Scenario: Selected run has a complete runtime agent artifact
- **WHEN** the selected run identifies an agent whose complete artifact is available and the operator activates Resume
- **THEN** the console MUST POST that artifact and selected session context to `/api/uar/runs/{run_id}/resume`
- **AND** it MUST select or navigate to the new run ID returned by the server.

#### Scenario: Selected run cannot supply a complete agent artifact
- **WHEN** the selected run lacks an agent ID or its complete runtime artifact cannot be loaded
- **THEN** Resume MUST be disabled
- **AND** the console MUST explain the missing prerequisite in text rather than synthesizing a partial artifact.

#### Scenario: Resume request fails
- **WHEN** the resume endpoint rejects or cannot complete the request
- **THEN** the original selected run and its offline trace MUST remain unchanged
- **AND** the action MUST expose an error without claiming that runtime state was restored.

### Requirement: Validated A2UI Surface Replay
The runtime console SHALL reconstruct late-join A2UI surface state from ordered replay patches using the existing A2UI validator and reducer.

#### Scenario: A2UI replay succeeds
- **WHEN** `GET /api/uar/runs/{run_id}/a2ui/surface-replay` returns valid ordered state-patch operations
- **THEN** the console MUST reconstruct their A2UI v0.9.1 message envelopes in publish order
- **AND** it MUST pass every message through the existing A2UI validation and reduction path before exposing replayed surface metadata.

#### Scenario: Replay contains an invalid patch
- **WHEN** a replay item has an unknown operation, invalid surface path, invalid component graph, or executable content
- **THEN** the console MUST reject that item from rendered surface state
- **AND** it MUST expose an inspector error that identifies the rejected replay position without executing the payload.

#### Scenario: Replay and live events overlap
- **WHEN** a late-joining console loads replay history and then receives newer persisted state events
- **THEN** replay operations MUST retain publish order
- **AND** stable surface and event identities MUST prevent duplicate visible records.

### Requirement: Responsive Flat 2.0 Trace Composition
The runtime console SHALL present the run registry, trace timeline, and inspector as one responsive semantic tree that conforms to the repository's Flat 2.0 and focus contracts.

#### Scenario: Wide runtime console
- **WHEN** the available layout width supports three panes
- **THEN** the run registry, trace/timeline, and inspector MUST occupy distinct grid areas while preserving one selected-run and selected-event state.

#### Scenario: Narrow runtime console
- **WHEN** the layout no longer supports three panes
- **THEN** the same semantic registry, trace/timeline, and inspector nodes MUST reflow into a compact sequence without duplicating an alternate mobile interaction tree
- **AND** interactive targets MUST remain at least 44 CSS pixels in their compact presentation.

#### Scenario: Trace control receives keyboard focus
- **WHEN** a trace, filter, tree, tab, copy, replay, or resume control receives visible focus
- **THEN** it MUST use the repository's 3 pixel ember focus treatment
- **AND** focus visibility MUST NOT rely on a border, shadow, gradient, or color alone.

#### Scenario: Trace surfaces separate content
- **WHEN** the trace composition separates registry, timeline, inspector, groups, rows, or selected state
- **THEN** it MUST use surface fills, spacing, typography, and state tokens
- **AND** it MUST NOT add visible borders, shadows, blur, gradients, or outline-style variants.

### Requirement: Bounded Trace Rendering
The runtime console SHALL bound work for large persisted traces and meet the phase's 500-event interaction budget.

#### Scenario: Trace has at most 200 visible rows
- **WHEN** filtering and expansion produce 200 or fewer visible rows
- **THEN** the timeline MAY render the complete visible row set without virtualization.

#### Scenario: Trace exceeds 200 visible rows
- **WHEN** filtering and expansion produce more than 200 visible rows
- **THEN** the timeline MUST virtualize the flattened visible projection using stable row keys, dynamic measurement, and bounded overscan
- **AND** the number of mounted event rows MUST remain bounded by the viewport and overscan rather than total event count.

#### Scenario: Operator opens a 500-event trace
- **WHEN** the deterministic 500-event fixture is projected and mounted in the supported browser profile
- **THEN** the trace MUST become interactive within 100 milliseconds
- **AND** the projection portion MUST complete within 20 milliseconds so rendering retains an explicit budget.

#### Scenario: Provider or protocol varies
- **WHEN** persisted events originated from any supported OpenAI-compatible, Anthropic-compatible, `liter-llm`, AG-UI, A2UI, or MCP path
- **THEN** the trace MUST consume the same normalized persisted event contract
- **AND** this feature MUST NOT change provider selection, routing, or protocol wire behavior.

