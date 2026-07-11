## ADDED Requirements

### Requirement: Stable admin actions are executable
Every stable administrative action SHALL persist or execute through its owning API, surface errors, and reconcile state reactively.

#### Scenario: Advertised action fails
- **WHEN** the backing service rejects a stable action
- **THEN** the UI displays an actionable error and does not present an optimistic false success

#### Scenario: Uncertified action
- **WHEN** an action lacks a passing contract
- **THEN** it is removed from stable UI or visibly labeled experimental before GA
