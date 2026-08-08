## Why

UAR now persists normalized run events and phase timings, but operators still cannot read that record as a coherent execution trace, inspect an event without leaving the runtime console, or resume and replay a run from the same surface. C-11 turns the C-07 event record into an accessible operations workflow while meeting the 500-event render budget.

## What Changes

- Add a phase-proportional run trace bar with keyboard navigation, explicit phase labels, and timing details that do not rely on color alone.
- Add a filterable hierarchical event timeline that projects the persisted event tree into visible rows and virtualizes traces beyond approximately 200 rows.
- Resolve `cand-010` by adopting `@tanstack/react-virtual` 3.14.9 for the flattened visible-row projection. UAR retains ownership of tree expansion, stable event identity, semantics, and Flat 2.0 markup.
- Add an event inspector with Payload, Timing, and Raw AG-UI views; raw event data remains inert, verbatim, and copyable rather than executable markup.
- Wire checkpoint discovery, run resume, and A2UI surface replay through the frontend service/store/hook boundary using `/runs/{id}/checkpoints`, `/runs/{id}/resume`, and `/runs/{id}/a2ui/surface-replay`.
- Keep selected-run, filter, expansion, checkpoint, resume, replay, loading, and error state in the feature store so realtime persistence updates can be projected without component-owned business state or manual refresh.
- Add focused contract, projection, interaction, accessibility, endpoint, and 500-event performance coverage.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `runtime-console`: add persisted run-trace visualization, hierarchical event inspection, checkpoint resume, A2UI surface replay, accessibility, and bounded 500-event rendering requirements.

## Impact

- Affected frontend areas: the target `frontend/src/features/chat/` API/model/UI layers, focused tests, and the existing runtime runs surface that hosts the feature until the C-14 admin-page migration.
- APIs: consumes the existing checkpoint list, latest resume, and A2UI surface replay routes. No backend route or wire-format change is proposed.
- Dependencies: adds `@tanstack/react-virtual` 3.14.9. Its published peer range includes React 19, and its headless model leaves UAR in control of accessible tree markup and styling.
- Runtime UX and realtime state: persisted `run_event` rows and phase timings become the source for the trace, timeline, and offline inspector; live additions refresh the feature store projection while preserving the selected event when possible.
- Provider compatibility: unchanged. The feature reads normalized AG-UI/A2UI run data after provider adaptation and does not alter OpenAI-compatible, Anthropic-compatible, `liter-llm`, AG-UI, A2UI, or MCP contracts.
- Security boundary: persisted payloads and raw AG-UI data are rendered as text/JSON only and never interpreted as HTML or executable content.
- KBD workflow state: canonical C-11 execution state is updated through the existing KBD change transition commands; this change does not alter product-visible KBD behavior or schemas.
