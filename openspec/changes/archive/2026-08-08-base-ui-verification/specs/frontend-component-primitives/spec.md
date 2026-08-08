## ADDED Requirements

### Requirement: Application command search is Base UI owned
UAR SHALL implement the stable local `Command*` facade with Base UI primitives and SHALL NOT retain `cmdk` or another Radix-backed application command implementation.

#### Scenario: A feature filters an action list
- **WHEN** an operator types in an agent, model, skill, tool, or knowledge-base command search
- **THEN** matching items remain keyboard and pointer activatable through the unchanged local wrapper API

#### Scenario: A repeated-add command remains open
- **WHEN** an operator selects an item from a command search whose host remains open
- **THEN** the search remains an action filter rather than persisting the selected item as a form value

### Requirement: Third-party primitive ownership is auditable
UAR SHALL document Radix packages retained through supported third-party dependencies and SHALL distinguish them from application-owned source imports and direct dependency declarations.

#### Scenario: The dependency graph is audited
- **WHEN** pnpm explains a retained Radix package
- **THEN** the receipt identifies its supported third-party owner and application source remains free of direct Radix imports

#### Scenario: Entity-management ownership is audited
- **WHEN** the Prometheus Entity Management package metadata and Radix graph are inspected
- **THEN** the receipt records whether that package introduces a Radix dependency without inferring ownership from unrelated packages
