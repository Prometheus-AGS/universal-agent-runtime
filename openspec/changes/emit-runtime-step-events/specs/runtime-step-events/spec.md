## ADDED Requirements

### Requirement: Runtime emits a step event per orchestrator iteration

The runtime SHALL emit a step event for each tool-loop iteration of a run: a
`started` event when the iteration begins and a `finished` event when it ends.
Each step event SHALL carry the run id and a monotonic per-run step index.

#### Scenario: Steps emitted across a multi-iteration run
- **WHEN** a run executes N tool-loop iterations
- **THEN** the runtime emits a `started` and a `finished` step event for each iteration, with the step index increasing monotonically from the first iteration

#### Scenario: Step carries run id and index
- **WHEN** a step event is emitted
- **THEN** it includes the run id and the iteration's step index

### Requirement: Step events are delivered on the runtime entity bus

Step events SHALL be delivered as `runtime.step` entity-bus events in the shape
consumed by the Runtime Console ingest (`step_started` / `step_finished` →
`RuntimeRunStep`), routed through the existing run event emitter (broadcast +
replay buffer) so late-joining subscribers receive prior steps.

#### Scenario: Console-ingestible shape
- **WHEN** a step `started` event is emitted
- **THEN** a `runtime.step` event with `type: "step_started"`, the run id, and the step index is produced (and `type: "step_finished"` for the finished event)

#### Scenario: Replay to a late subscriber
- **WHEN** a client subscribes to a run's stream after some steps have already occurred
- **THEN** the replayed history includes the prior step events (subject to the existing replay-buffer bound)

### Requirement: Step emission is additive and non-breaking

Step events SHALL NOT alter existing events or run behavior. Consumers that do
not handle `runtime.step` SHALL be unaffected.

#### Scenario: Existing clients unaffected
- **WHEN** step events are emitted during a run
- **THEN** existing `agui.*` and other `runtime.*` events are unchanged and a client that ignores `runtime.step` behaves exactly as before
