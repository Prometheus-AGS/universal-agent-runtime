## ADDED Requirements

### Requirement: Each run owns a cancellation token

Every run started by `RunManager` SHALL be associated with a `CancellationToken` created at run start and retained in the run's stream state for the lifetime of the run. The token SHALL be a child of a process-level root token so that cancelling the root cancels all in-flight runs.

#### Scenario: Token created at run start
- **WHEN** `RunManager::start_run` spawns the agent task for a new run
- **THEN** a `CancellationToken` is created, stored in the run's `RunStreamState`, and a child handle is passed into the orchestrator for that run

#### Scenario: Token removed on terminal state
- **WHEN** a run reaches any terminal state (`done`, `error`, or `cancelled`)
- **THEN** the run's cancellation token entry is released so it no longer accumulates in `RunManager` state

### Requirement: A run can be cancelled explicitly via the API

The system SHALL expose `POST /api/uar/runs/{id}/cancel` that cancels the identified run's token and causes the run to terminate promptly with a `cancelled` outcome.

#### Scenario: Cancel an in-flight run
- **WHEN** a client sends `POST /api/uar/runs/{id}/cancel` for a run that is actively streaming or executing a tool
- **THEN** the run's `CancellationToken` is cancelled, the run stops issuing further LLM or tool calls, and a terminal `cancelled` event is emitted on the run's event stream

#### Scenario: Cancel an unknown or already-finished run
- **WHEN** `POST /api/uar/runs/{id}/cancel` targets a run id that does not exist or has already reached a terminal state
- **THEN** the endpoint responds without error and does not emit a duplicate terminal event

### Requirement: Cancellation propagates through LLM and tool execution

A cancelled run SHALL abort the in-flight LLM driver call, the stream-consumption loop, and any in-flight or pending tool execution (MCP, native, or sandbox) at the next await point, rather than running to completion.

#### Scenario: Cancellation during LLM streaming
- **WHEN** a run is cancelled while the orchestrator is awaiting or consuming the LLM driver stream
- **THEN** the stream await is abandoned via cancellation and no further assistant tokens are processed for that run

#### Scenario: Cancellation during tool execution
- **WHEN** a run is cancelled while a tool call is awaiting (MCP, native, or sandbox)
- **THEN** the tool await is abandoned, no subsequent tools in the iteration are dispatched, and the orchestrator loop exits with a cancelled outcome

#### Scenario: Cancellation checked before each tool iteration
- **WHEN** the orchestrator reaches the top of its tool-execution loop and the run's token is already cancelled
- **THEN** the loop exits immediately without dispatching the next tool

### Requirement: Client disconnect cancels only on last-subscriber drop

The run event stream supports multiple concurrent subscribers and late joiners reconnecting via history replay. A run SHALL be auto-cancelled on client disconnect ONLY when the last remaining subscriber to that run's event stream drops; while any subscriber remains attached, the run SHALL continue.

#### Scenario: One of several viewers disconnects
- **WHEN** a run has more than one active subscriber and one of them disconnects
- **THEN** the run is NOT cancelled and continues streaming to the remaining subscribers

#### Scenario: Last viewer disconnects
- **WHEN** the last active subscriber to a run's event stream disconnects and no reconnection occurs
- **THEN** the run's cancellation token is cancelled and the run terminates with a `cancelled` outcome

#### Scenario: Reconnect before last-drop cancellation
- **WHEN** a subscriber reconnects (via history replay) while the run still has at least one attached subscriber
- **THEN** the run is unaffected and continues normally

### Requirement: Graceful shutdown cancels in-flight runs

On server shutdown signal, the process-level root cancellation token SHALL be cancelled so that all in-flight runs abort within the configured shutdown drain window instead of being killed at process teardown.

#### Scenario: Shutdown with active runs
- **WHEN** the server receives a shutdown signal while runs are in flight
- **THEN** the root token is cancelled, every run's child token is cancelled, in-flight LLM and tool awaits abort, and the server completes graceful shutdown within the drain window

### Requirement: Cancellation emits a distinct terminal event

A cancelled run SHALL emit a terminal `cancelled` event on the normalized event stream that is distinct from `done` and `error`, and this event SHALL be recorded in the run's replay history so reconnecting clients observe the cancelled terminal state.

#### Scenario: Cancelled terminal event on the stream
- **WHEN** a run is cancelled (explicitly, by last-subscriber drop, or by shutdown)
- **THEN** a terminal `cancelled` event is emitted, distinguishable from `done`/`error`, and the frontend renders the run as cancelled

#### Scenario: Cancelled state observable via replay
- **WHEN** a client requests the run's event history after the run was cancelled
- **THEN** the replayed history includes the terminal `cancelled` event

### Requirement: Cancelling a run resolves a pending tool approval

When a run that is paused awaiting tool approval is cancelled, the pending approval SHALL be resolved as aborted so the orchestrator does not deadlock waiting on an approval that will never arrive.

#### Scenario: Cancel while awaiting approval
- **WHEN** a run is paused on a tool-approval gate and the run is cancelled
- **THEN** the pending approval is resolved as aborted, the orchestrator loop unblocks, and the run terminates with a `cancelled` outcome
