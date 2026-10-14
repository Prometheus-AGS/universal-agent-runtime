# Spec Delta

## ADDED Requirements

### Requirement: Provider credentials use verified complete owner identity

Credential ownership SHALL use host-verified issuer, tenant and subject in a namespace disjoint from legacy identifiers. Caller-supplied identity SHALL NOT confer ownership.

#### Scenario: Identical subjects

- **WHEN** two authenticated principals share a subject but differ by tenant or issuer
- **THEN** their credential create, read, list, delete and use operations are isolated

#### Scenario: Legacy ownership

- **WHEN** a legacy credential lacks sufficient owner provenance
- **THEN** it is not granted to a matching subject and only an authenticated audited mapping can migrate it without key collisions


