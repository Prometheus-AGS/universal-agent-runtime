# product-capability-support-matrix Specification

## Purpose
TBD - created by archiving change publish-capability-support-matrix. Update Purpose after archive.

## Requirements

### Requirement: Support claims are evidence backed
Every advertised capability, provider, feature combination and platform SHALL have a maturity classification and verification reference.

#### Scenario: Catalog-only provider
- **WHEN** a provider exists in the catalog but lacks integration evidence
- **THEN** it is labeled catalog/community support and not production-certified

#### Scenario: Mobile claim
- **WHEN** mobile packaging and platform tests are absent
- **THEN** mobile is labeled experimental rather than equivalent to web/desktop

### Requirement: Assistant capability discovery matches reachable operations

Assistant discovery SHALL identify supported operations by backend, deployment profile and entry point, backed by reachable governed behavior. Unsupported operations SHALL fail explicitly without silent ephemeral substitution.

#### Scenario: Advertised operation

- **WHEN** a client discovers an assistant operation for its backend and entry point
- **THEN** the operation is reachable under the same host authority and durable semantics claimed by discovery

#### Scenario: Unsupported profile

- **WHEN** a client uses an operation unavailable on its backend or direct embedded entry point
- **THEN** discovery marks it unsupported and invocation fails explicitly without an ephemeral substitute
