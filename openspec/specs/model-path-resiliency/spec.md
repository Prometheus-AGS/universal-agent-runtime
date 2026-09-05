# model-path-resiliency Specification

## Purpose

Define policy-driven model retry, health-aware failover, interruption persistence and authorized stream replay.

## Requirements

### Requirement: Retry mechanics honor the configured resilience policy
The runtime SHALL apply the configured jitter mode, maximum delay, total delay budget, and maximum attempts to every model-call retry, and SHALL honor a provider's `Retry-After` value in preference to the computed backoff when the policy enables it.

#### Scenario: Jittered backoff
- **WHEN** `retry_jitter_mode` is not `none` and a retryable failure repeats
- **THEN** successive delays vary within the jitter bounds and never exceed the maximum delay or the total budget

#### Scenario: Provider supplies Retry-After
- **WHEN** a provider responds with a retryable status and a `Retry-After` value and `retry_respect_retry_after` is enabled
- **THEN** the next attempt waits the provider's value

### Requirement: Retryability is decided from a typed provider error
The runtime SHALL classify provider failures into a typed error carrying status, kind, and optional retry delay at the driver boundary, and SHALL NOT decide retryability by matching substrings of an error message.

#### Scenario: Non-retryable request error
- **WHEN** the provider rejects the request as invalid
- **THEN** the run fails without retrying regardless of the message text

### Requirement: Model selection and failover respect provider health
The runtime SHALL consult provider health when selecting the run model and each failover candidate, SHALL skip providers in cooldown, and SHALL try every configured fallback in order.

#### Scenario: Primary in cooldown
- **WHEN** the policy-resolved provider is in cooldown and a healthy fallback is configured
- **THEN** the run uses the fallback without attempting the primary

### Requirement: Established streams have an idle timeout
The runtime SHALL fail an established model stream that emits no data for the configured idle timeout as a retryable error. Before semantic output, lifecycle metadata, cumulative usage, and empty text deltas SHALL NOT close the retry boundary or reset the first-output deadline. Once nonempty text, reasoning, tool-call data, or another semantic event is committed, a failure SHALL interrupt the turn rather than replay model execution.

#### Scenario: Stalled stream
- **WHEN** a provider opens a stream and stops emitting
- **THEN** the run does not hang; the stream fails after the idle timeout and retry policy applies if no semantic output was committed

#### Scenario: Metadata-only stall
- **WHEN** a provider emits stream-start or usage metadata and then stalls before semantic output
- **THEN** the configured retry policy applies and metadata from the failed attempt is not emitted as a successful response

#### Scenario: Idle timeout after partial text
- **WHEN** a provider stalls after nonempty assistant text
- **THEN** the partial text is preserved as interrupted and the model call is not replayed

### Requirement: Interrupted turns are persisted as interrupted
When a model stream fails or is cancelled after partial output, the runtime SHALL persist the partial assistant content with an interrupted-turn marker visible to the model on the next turn, and SHALL NOT persist it as a complete assistant message.

#### Scenario: Mid-stream provider failure
- **WHEN** a stream fails after emitting some text
- **THEN** session history contains the partial text followed by an interrupted-turn marker fragment

### Requirement: Chat streams resume from a cursor
The primary chat stream endpoint SHALL accept `Last-Event-ID` together with the original `x-uar-run-id`, SHALL authorize the exact stored user and tenant, and SHALL replay only frames after that cursor without starting another model execution. New cursors SHALL identify the runtime event, projected frame ordinal, and stream format; numeric legacy cursors acknowledge the entire runtime event. Reconnects SHALL use the same format. If retained history cannot reconstruct the projection state, the endpoint SHALL report expiration rather than silently duplicate or omit frames. Replay SHALL NOT repeat memory capture or post-response model calls.

#### Scenario: Client reconnects mid-run
- **WHEN** a chat client reconnects with the id of the last event it rendered
- **THEN** it receives subsequent events once and no earlier event is repeated

#### Scenario: Reconnect between projected frames
- **WHEN** a runtime event expands into several SSE frames and the client reconnects after one of them
- **THEN** only the remaining frames of that source event and subsequent events are emitted

#### Scenario: Foreign run or expired projection history
- **WHEN** a reconnect targets another principal's run or lacks the retained prefix needed to rebuild projection state
- **THEN** the endpoint returns not-found or history-expired respectively, and never starts a replacement run
