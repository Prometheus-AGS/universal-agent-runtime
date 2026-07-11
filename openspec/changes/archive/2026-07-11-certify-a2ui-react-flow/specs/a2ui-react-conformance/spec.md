## ADDED Requirements

### Requirement: Safe React A2UI rendering
UAR SHALL render the declared A2UI profile as validated data using approved React components and SHALL never execute agent-supplied code.

#### Scenario: Unknown component
- **WHEN** a surface references a component outside the approved catalog
- **THEN** rendering fails safely with a visible diagnostic and no code execution

#### Scenario: Interactive round trip
- **WHEN** a user acts on a rendered surface
- **THEN** a typed action response is correlated to the originating run and the next surface update is rendered
