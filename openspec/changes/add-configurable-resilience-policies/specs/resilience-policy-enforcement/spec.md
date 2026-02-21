## ADDED Requirements

### Requirement: Runtime Components MUST Enforce Effective Resilience Policy

Runtime components MUST apply the effective resilience policy to timeout and retry decisions in agent-scoped operations.

#### Scenario: Agent-scoped timeout uses effective policy
- **WHEN** an agent-scoped operation executes
- **THEN** timeout budgeting MUST use the agent’s effective timeout settings.

#### Scenario: Agent-scoped retries use effective policy
- **WHEN** an agent-scoped outbound call fails with a retryable condition
- **THEN** retries MUST follow configured attempt limits, backoff, jitter, and retry budget.

### Requirement: Retry Eligibility MUST Be Explicit and Safe

The system MUST retry only explicitly retryable error classes and MUST honor server retry hints when provided.

#### Scenario: Retry-After is honored
- **WHEN** a retryable HTTP response includes `Retry-After`
- **THEN** the next retry delay MUST respect that value within configured delay caps.

#### Scenario: Non-retryable error is not retried
- **WHEN** an error class is not included in configured retry eligibility
- **THEN** the operation MUST fail without additional retry attempts.

### Requirement: Retry Behavior MUST Be Observable

The system MUST emit telemetry for retry outcomes and policy source.

#### Scenario: Retry telemetry includes policy context
- **WHEN** a retried operation completes (success or terminal failure)
- **THEN** telemetry MUST include attempt count and whether policy values came from global or agent override.
