## ADDED Requirements

### Requirement: Output reservation follows the request output cap
A non-team model call SHALL reserve output tokens equal to the smaller of the request's output cap and the model's catalog maximum output when a cap is present. Input SHALL keep the full catalog context window as its reserve.

#### Scenario: Budgeted agent on a large-context model
- **WHEN** an agent's per-turn token limit is 450000, the model's context window is 400000 with 128000 maximum output, and the request sets `max_tokens` to 1000
- **THEN** the call is admitted and the agent answers.

### Requirement: A catch-all binding failure logs its cause
When a run's model binding fails in the catch-all `client_capture_failed` category, UAR SHALL log the failure's cause, passed through the run's secret scrubber, together with the diagnostic reference. The run-stream message SHALL stay generic.

#### Scenario: Operator follows a diagnostic reference
- **WHEN** a run is refused with `Run binding refused during client_construction; diagnostic reference <id>`
- **THEN** the ERROR log entry with that reference carries a scrubbed `cause` field.
