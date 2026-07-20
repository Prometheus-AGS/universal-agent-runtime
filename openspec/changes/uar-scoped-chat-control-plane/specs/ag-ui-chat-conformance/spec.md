## MODIFIED Requirements

### Requirement: Declared AG-UI conformance
UAR SHALL publish a versioned mapping from normalized runtime events to AG-UI lifecycle, message, thinking, skill activation, MCP/tool, retrieval/citation, policy, model lifecycle, state, raw, and custom events, and SHALL retain them in replay order without silently discarding unknown provider events.

#### Scenario: Resume without duplication
- **WHEN** a client reconnects with a valid replay cursor
- **THEN** it reconstructs the same run state in order without duplicate logical events

#### Scenario: State divergence
- **WHEN** a state delta cannot be applied
- **THEN** the client requests or consumes a fresh snapshot instead of silently diverging

#### Scenario: Effective policy is observable
- **WHEN** UAR resolves a run policy before execution
- **THEN** the stream contains a secret-free policy event identifying the selected provider, model, skills, MCP servers, knowledge bases, context strategy, and provenance

#### Scenario: Unknown event preservation
- **WHEN** a provider emits an event without a dedicated normalized variant
- **THEN** UAR preserves it as a typed raw or artifact event with provider and event-name metadata
