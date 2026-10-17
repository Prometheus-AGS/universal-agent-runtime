# Spec Delta

## ADDED Requirements

### Requirement: Bounded admission uses captured durable-store eligibility

**Stable task:** S1. The trusted host SHALL capture one safe durable-store eligibility state at provider construction: `local_durable`, `remote_attested`, `remote_not_attested`, or `unsupported_backend`. A run with any explicit token, cost, remote, or scoped limit SHALL require `local_durable` or `remote_attested` before provider or tool effects. The captured state SHALL contain no endpoint, namespace, credential, or source error text.

#### Scenario: Unattested remote Surreal rejects a bounded run

- **WHEN** a remote SurrealDB provider was constructed without the trusted-host durability attestation and a run has an explicit budget limit
- **THEN** admission refuses before provider construction or tool execution using the captured `remote_not_attested` reason
- **AND** the configured budget is neither removed nor converted to process-local accounting

#### Scenario: Eligible durable accounting resumes after restart

- **WHEN** a local durable or attested remote provider has reserved accounting capacity and the process restarts before final settlement
- **THEN** the exact owner-scoped reservation and unresolved liability remain recoverable from the same durable store
- **AND** admission does not reset available budget or release unknown liability

#### Scenario: Captured eligibility does not follow mutable environment changes

- **WHEN** the process environment changes after a persistence provider is constructed
- **THEN** accounting admission uses the eligibility captured by that provider
- **AND** it does not re-read attestation state per request

