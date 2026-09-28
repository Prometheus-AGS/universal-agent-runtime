# Local scoped observers

## ADDED Requirements

### Requirement: Committed source occurrence

A durable source occurrence MUST have stable identity and provenance, and MUST be committed atomically with its source state. Ephemeral token streams MUST NOT be presented as durable occurrences.

#### Scenario: Crash after source commit

- **WHEN** the process exits after an instance transition commits and before observer admission
- **THEN** the occurrence remains discoverable from the outbox with the same identity after restart.

### Requirement: Independent scoped subscriptions

Each observer MUST retain its own revision, source/conversation intersection, projection grant, cursor, inbox admission and acknowledgement. Delivery MUST recheck current authority before disclosure or execution.

#### Scenario: Two observers with different grants

- **WHEN** two observers watch overlapping sources but one lacks a grant for a conversation
- **THEN** each advances independently and the restricted observer never sees that conversation's projection.

### Requirement: Bounded delivery and honest recovery

The runtime MUST expose pause, backlog, retention gap and dead-letter state. Replayed occurrences MUST NOT re-execute source effects, and uncertain observer effects MUST obey the existing C02/C06 reconciliation boundary.

#### Scenario: Restart at admission or acknowledgement

- **WHEN** the process exits at either boundary and later resumes
- **THEN** a stable admission identity recovers the same observer command receipt or reports a visible gap/failure without silently dropping accepted work.
