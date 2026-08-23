## Purpose

Prevent known React and entity-state failure shapes with scoped architecture checks and short, local functional evidence after code completion.

## ADDED Requirements

### Requirement: React and entity work starts from the approved guidance
Repository instructions SHALL require contributors changing React/entity-state
code to consult Vercel React Best Practices, Vercel Composition Patterns, and the
applicable Prometheus Entity Management skill before implementation.

#### Scenario: React entity-state code is changed
- **WHEN** an agent or contributor prepares to modify a React component, entity hook, entity transport, or graph-backed feature state
- **THEN** the named guidance is part of the implementation context
- **AND** the resulting change records the task-specific rules it applied

### Requirement: Architecture validation rejects the observed failure shapes
The local frontend architecture gate SHALL reject synchronous state setters in a
component render body, feature-owned loops that issue one graph mutation per row,
direct entity-package imports outside the platform facade, and duplicate feature
caches for the graph-owned configured Provider, Model, AgentSession, or
AgentSessionDraft records. It SHALL allow UI-local state and event-driven updates
that do not duplicate business entities.

#### Scenario: A forbidden fixture is checked
- **WHEN** a fixture contains any named forbidden pattern
- **THEN** the local gate exits non-zero with a deterministic rule identifier and source location

#### Scenario: A valid narrow subscriber is checked
- **WHEN** a component obtains one field through a platform domain hook and mutates it only from an event handler
- **THEN** the local gate accepts the component

#### Scenario: The gate's negative controls run
- **WHEN** the validation fixtures intentionally contain each forbidden pattern
- **THEN** every negative fixture is observed to fail before the gate is accepted

### Requirement: Session Configuration has bounded local functional evidence
After the phase is code-complete, one short local installed-service browser
scenario SHALL verify the user-visible repair against the release build. Product
testing MUST NOT run in GitHub Actions and MUST NOT become a soak test.

#### Scenario: The repaired workflow is certified locally
- **WHEN** the installed release service and production UI at `http://localhost:1906` are exercised through a real browser
- **THEN** the sheet opens within two seconds and remains interactive
- **AND** no `/api/models` request occurs
- **AND** configured models are selectable
- **AND** save/reopen and cancel isolation behave as specified
- **AND** a genuine inference turn uses the saved session model
- **AND** computed styles at 320, 768, 1024, and 1440 pixels satisfy the spacing contract
- **AND** browser console, browser network, and matching server log evidence are recorded

#### Scenario: A functional assertion fails
- **WHEN** any required interaction, inference result, request bound, or spacing assertion is not observed
- **THEN** the phase remains incomplete and reports the exact failed requirement
