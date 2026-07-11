# ag-ui-chat-conformance Specification

## Purpose
TBD - created by archiving change certify-agui-chat-flow. Update Purpose after archive.
## Requirements
### Requirement: Declared AG-UI conformance
UAR SHALL publish a versioned mapping from normalized runtime events to AG-UI lifecycle, message, tool, state, raw, and custom events.

#### Scenario: Resume without duplication
- **WHEN** a client reconnects with a valid replay cursor
- **THEN** it reconstructs the same run state in order without duplicate logical events

#### Scenario: State divergence
- **WHEN** a state delta cannot be applied
- **THEN** the client requests or consumes a fresh snapshot instead of silently diverging

