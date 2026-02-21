## ADDED Requirements

### Requirement: Global Resilience Policy MUST Be Configurable

The system MUST expose a global resilience policy that configures rate limiting, timeout, and retry behavior through runtime settings.

#### Scenario: Global policy is seeded with validated defaults
- **WHEN** the settings system initializes without explicit resilience overrides
- **THEN** it MUST seed all required resilience policy fields with defaults
- **AND** each field MUST satisfy schema validation constraints.

#### Scenario: Invalid resilience values are rejected
- **WHEN** a settings write attempts to persist an out-of-range resilience value
- **THEN** the write MUST fail validation
- **AND** the persisted configuration MUST remain unchanged.

### Requirement: Per-Agent Resilience Overrides MUST Support Inheritance

The system MUST allow each agent to either inherit the global resilience policy or override selected fields.

#### Scenario: Agent inherits global policy
- **WHEN** an agent has `resilience.mode=inherit`
- **THEN** the effective policy for that agent MUST equal the global policy.

#### Scenario: Agent overrides selected fields
- **WHEN** an agent has `resilience.mode=override` and only a subset of override fields is defined
- **THEN** defined override fields MUST take precedence
- **AND** undefined fields MUST inherit from global policy.

### Requirement: Effective Policy Resolution MUST Be Deterministic

The system MUST resolve effective resilience policy in a deterministic order: global defaults first, then agent overrides.

#### Scenario: Resolution order is stable
- **WHEN** effective policy is computed for the same global and agent inputs multiple times
- **THEN** the resolved output MUST be identical for each computation.
