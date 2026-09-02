## ADDED Requirements

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
The runtime SHALL fail an established model stream that emits no data for the configured idle timeout as a retryable error.

#### Scenario: Stalled stream
- **WHEN** a provider opens a stream and stops emitting
- **THEN** the run does not hang; the stream fails after the idle timeout and retry policy applies

### Requirement: Interrupted turns are persisted as interrupted
When a model stream fails or is cancelled after partial output, the runtime SHALL persist the partial assistant content with an interrupted-turn marker visible to the model on the next turn, and SHALL NOT persist it as a complete assistant message.

#### Scenario: Mid-stream provider failure
- **WHEN** a stream fails after emitting some text
- **THEN** session history contains the partial text followed by an interrupted-turn marker fragment

### Requirement: Chat streams resume from a cursor
The primary chat stream endpoint SHALL accept `Last-Event-ID` and SHALL replay only events after that cursor.

#### Scenario: Client reconnects mid-run
- **WHEN** a chat client reconnects with the id of the last event it rendered
- **THEN** it receives subsequent events once and no earlier event is repeated
