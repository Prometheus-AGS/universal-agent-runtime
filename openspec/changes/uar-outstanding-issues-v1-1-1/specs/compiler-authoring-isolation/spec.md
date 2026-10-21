# Spec Delta

## ADDED Requirements

### Requirement: Ownerless compiler REST is unavailable in shared profiles

The host SHALL reject every operation under both `/api/uar/compiler` and
`/api/compiler` whenever ingress is remote or
`security.provider_credentials.mode` is `shared`. This SHALL include spec
listing, reads and mutations, reports, sessions, inline compilation and
compile-and-register. The decision SHALL use trusted host configuration, not a
caller-provided header or route alias. The release SHALL disclose this profile
limit and SHALL NOT claim tenant-isolated compiler authoring. Local
single-owner behavior SHALL remain available under its existing authority.

#### Scenario: Two admitted remote principals

- **WHEN** two valid remote principals have different verified tenant mappings
- **THEN** neither can use either compiler REST alias to read, write, list,
  compile, register or operate on the other's ownerless compiler resources
- **AND** rejection occurs before compiler storage or session effects

#### Scenario: Shared credentials on local ingress

- **WHEN** local ingress uses shared provider credential mode
- **THEN** both compiler REST aliases reject every registered operation
- **AND** a request header cannot restore access

#### Scenario: Existing resources and local compatibility

- **WHEN** a restricted profile starts with existing compiler specs or sessions
- **THEN** denied requests do not alter them, including after restart
- **AND** a trusted local single-owner profile retains its existing operations

#### Scenario: Full authoring remains unresolved

- **WHEN** the restricted release is published
- **THEN** owner-scoped compiler storage, session records and migration are
  identified as future work under #387 rather than reported as implemented
