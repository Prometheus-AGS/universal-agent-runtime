## ADDED Requirements

### Requirement: A2UI axe-core validation gate
The frontend validation gate SHALL run package-local axe-core and interaction tests when A2UI renderer source, styles, resources, stories, tests, or dependencies change.

#### Scenario: A2UI accessibility checks pass
- **WHEN** a pull request changes `frontend/packages/a2ui-uar/**`
- **THEN** CI runs the package typecheck, lint, tests, and axe-core fixtures successfully before acceptance

#### Scenario: Existing frontend gates remain required
- **WHEN** the A2UI accessibility gate is added
- **THEN** frontend workspace typecheck, lint, and build remain required for Change 21 completion
