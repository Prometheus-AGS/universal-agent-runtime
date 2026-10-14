# Spec Delta

## ADDED Requirements

### Requirement: Assistant capability discovery matches reachable operations

Assistant discovery SHALL identify supported operations by backend, deployment profile and entry point, backed by reachable governed behavior. Unsupported operations SHALL fail explicitly without silent ephemeral substitution.

#### Scenario: Advertised operation

- **WHEN** a client discovers an assistant operation for its backend and entry point
- **THEN** the operation is reachable under the same host authority and durable semantics claimed by discovery

#### Scenario: Unsupported profile

- **WHEN** a client uses an operation unavailable on its backend or direct embedded entry point
- **THEN** discovery marks it unsupported and invocation fails explicitly without an ephemeral substitute


