## Why

The AG-UI client adapter currently reduces official frames only into legacy
chat events, forcing message rendering and Runtime Console ingestion to
reinterpret the same payload independently and leaving no phase-timing output.
The stale `complete-agui-event-parity` change also records replay snapshot and
tool-start gaps that C-06 explicitly absorbs.

## What Changes

- Add a typed AG-UI normalizer under `platform/agui/` that produces message
  chunks, phase attribution/timings, and event rows from one validated frame.
- Extend the per-stream adapter to publish those three projections while
  preserving event-id deduplication, replay ordering, and state-patch recovery.
- Wire chat reduction to normalized message chunks, Runtime Console ingestion
  to normalized event rows, and terminal run entities to computed phase timings.
- Complete the absorbed parity scope: attach/replay emits state and message
  snapshots before deltas, official tool-start semantics remain faithful, and
  RAW frames pass through the frontend normalizer without reinterpretation.
- Supersede the unstarted `complete-agui-event-parity` proposal with this
  reviewed change rather than creating its obsolete `agui-spec-parity`
  capability.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `ag-ui-chat-conformance`: extend the declared mapping contract with typed
  three-consumer normalization, terminal phase timing, and attach/replay
  snapshot parity.

## Impact

- **Runtime UX:** streamed message content remains behavior-compatible while
  trace/event consumers receive a single normalized row and completed runs
  expose phase timing data.
- **Provider compatibility:** provider/model APIs and dependencies do not
  change; all providers continue through the normalized runtime event stream.
- **Realtime state:** AG-UI events are normalized once before updating chat and
  entity-graph consumers; replay remains event-id idempotent and ordered.
- **Backend/API:** the official `agui_spec` replay stream gains initial
  snapshot frames but preserves its endpoint and existing event vocabulary.
- **Workflow:** C-06 completion must be written to canonical KBD state; the
  absorbed unstarted change is archived as superseded without applying its
  obsolete capability delta.
