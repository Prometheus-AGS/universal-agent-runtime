## ADDED Requirements

### Requirement: Prompt-caching settings render authoritative state
The Prompt Caching settings panel SHALL render editable controls only after the server value loads successfully. An initial load failure MUST render an actionable blocking alert and Retry action without editable fallback controls. Save SHALL be disabled while state is clean, loading, saving, or unavailable, and failed refresh or save operations MUST preserve unsaved edits.

#### Scenario: Prompt-caching settings load succeeds
- **WHEN** the settings endpoint returns an authoritative value
- **THEN** the panel renders that value with its complete label, description, effective status, and keyboard-operable control

#### Scenario: Initial prompt-caching load fails
- **WHEN** the first settings request fails
- **THEN** the panel shows a blocking alert and Retry action and renders no editable setting

#### Scenario: A clean setting is displayed
- **WHEN** the rendered value matches the last authoritative value
- **THEN** Save is disabled

#### Scenario: Refresh or save fails after an edit
- **WHEN** an operator has changed the setting and refresh or save fails
- **THEN** the draft remains visible and Save remains available after the operation ends

### Requirement: Prompt-caching settings avoid unsupported controls
The global Prompt Caching panel SHALL NOT display a configurable cache-control type, agent precedence, or toolbar precedence that the runtime does not support. A deprecated preferred-scope response field MAY remain for compatibility but MUST NOT render as a control.

#### Scenario: The global panel is inspected
- **WHEN** an operator opens Prompt Caching settings
- **THEN** only supported global-default behavior and provider differences are described
- **AND** no cache-control type or preferred-scope control is displayed
