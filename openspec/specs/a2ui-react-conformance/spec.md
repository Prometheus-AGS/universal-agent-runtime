# a2ui-react-conformance Specification

## Purpose
TBD - created by archiving change certify-a2ui-react-flow. Update Purpose after archive.
## Requirements
### Requirement: Safe React A2UI rendering
UAR SHALL render the declared A2UI profile as validated data using approved React components, SHALL never execute agent-supplied code, and SHALL contain malformed or unsupported rendering within a visible recoverable surface boundary.

#### Scenario: Unknown component
- **WHEN** a surface references a component outside the approved catalog
- **THEN** rendering fails safely inside the surface with a visible localized diagnostic, optional retry, and no code execution

#### Scenario: Interactive round trip
- **WHEN** a user acts on a rendered surface
- **THEN** a typed action response is correlated to the originating run and the next surface update is rendered

#### Scenario: Host-independent UX semantics
- **WHEN** the same certified surface is embedded in different hosts
- **THEN** its theme, locale, direction, focus, validation, recovery, and reduced-motion contracts remain deterministic

