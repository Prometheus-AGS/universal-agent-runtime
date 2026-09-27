# service-instance-placement Specification

## ADDED Requirements

### Requirement: UAR describes one stable configured service instance

UAR MUST expose the same stable logical instance identity through service
discovery, A2A contracts, collaboration bindings, and run admission. The
descriptor MUST include a versioned profile, local or remote workspace location,
lifecycle ownership posture, supported endpoint roles, capabilities, and only
opaque host-store references.

#### Scenario: Host discovers a configured runtime

- **WHEN** an authenticated host reads UAR capabilities
- **THEN** the response identifies the configured instance and profile
- **AND** separates runtime, administration, model, and console endpoint roles
- **AND** contains no credential value.

#### Scenario: Identity aliases disagree

- **WHEN** service-instance and A2A identities are both configured with different values
- **THEN** UAR refuses startup configuration instead of advertising two identities.

### Requirement: Compatibility is evaluated before executable admission

UAR MUST compare an expected identity, profile, workspace location, endpoint
roles, and required capabilities with its live descriptor before credentials or
executable work are admitted. Refusal MUST be structured and MUST NOT start
fallback work.

#### Scenario: Selected instance is compatible

- **WHEN** the expected descriptor matches the live instance and every required capability is supported
- **THEN** UAR returns an admitted effective binding for that instance.

#### Scenario: Selected instance is incompatible

- **WHEN** identity, profile, locality, endpoint role, or a required capability differs
- **THEN** UAR refuses admission with a diagnostic for every mismatch
- **AND** does not execute the run.

### Requirement: Placement intent is explicit

UAR MUST distinguish new-run placement from reattachment and migration. New-run
admission MUST accept only `new`; resume admission MUST accept only `reattach` to
the source run's effective instance; migration MUST be explicitly unsupported.

#### Scenario: Run resumes on its owning instance

- **WHEN** a resume request identifies the source run's effective instance and binding
- **THEN** UAR admits the reattachment and preserves that effective service binding.

#### Scenario: Migration is requested

- **WHEN** a client submits placement intent `migrate`
- **THEN** UAR refuses it as unsupported
- **AND** does not create replacement work.

### Requirement: Run inspection exposes effective service binding

UAR MUST retain the admitted instance identity, profile, locality, endpoint
roles, capabilities, placement intent, and optional deployment-binding identity
in run context and expose it through run inspection.

#### Scenario: Operator inspects a placed run

- **WHEN** a placed run is read or listed
- **THEN** its effective service binding identifies the exact runtime and intent admitted for that run.

