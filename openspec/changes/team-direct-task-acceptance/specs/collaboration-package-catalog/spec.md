## ADDED Requirements

### Requirement: Mode-specific team workflow allowlist

A draft.2 TeamDefinition SHALL require an immutable-reference `allowedWorkflows` array. `coordinator-within-binding` SHALL permit an empty array for direct bounded task coordination. `operator` SHALL require at least one workflow reference. An empty array SHALL authorize no workflow launch and SHALL NOT grant execution or protected authority.

#### Scenario: Direct coordinator package

- **WHEN** an otherwise valid coordinator-within-binding TeamDefinition declares `allowedWorkflows: []`
- **THEN** package preflight does not reject it for an empty workflow list
- **AND** current binding, role, policy, budget and execution admission remain required

#### Scenario: Operator package without a workflow

- **WHEN** an operator-mode TeamDefinition declares an empty workflow list
- **THEN** package preflight rejects its structure

#### Scenario: Workflow outside the exact allowlist

- **WHEN** a workflow launch targets a team whose allowlist is empty or excludes that exact immutable workflow reference
- **THEN** workflow launch remains denied
