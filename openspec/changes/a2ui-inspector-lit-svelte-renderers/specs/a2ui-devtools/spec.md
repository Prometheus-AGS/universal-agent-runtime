## ADDED Requirements

### Requirement: Inspector correlates live messages with preview and source
The dev-only Inspector SHALL consume an injected A2UI event stream and present a selectable message timeline synchronized with rendered preview and formatted source JSON.

#### Scenario: Selecting a message synchronizes panes
- **WHEN** a developer selects a valid captured message
- **THEN** the timeline selection, source JSON, and rendered preview SHALL describe the same surface state

### Requirement: Freeze preview is explicit and lossless
The Inspector SHALL distinguish preview freezing from stream ingestion and SHALL retain bounded messages received while frozen.

#### Scenario: Messages arrive while preview is frozen
- **WHEN** messages arrive after the developer activates Freeze preview
- **THEN** the displayed preview SHALL remain fixed, connection status SHALL remain current, and the Inspector SHALL show the queued-message count until Resume

### Requirement: Diagnostic failures preserve evidence
The Inspector SHALL preserve the last-good preview and display actionable empty, malformed, unknown-component, disconnected, and retry states.

#### Scenario: Malformed message arrives
- **WHEN** a captured payload fails protocol validation
- **THEN** the Inspector SHALL retain the last-good preview and expose the failing source payload and validation path without crashing the tool

### Requirement: Storybook can host the Inspector
The Inspector package SHALL export an addon descriptor and panel entrypoint without requiring Storybook in production runtime bundles.

#### Scenario: Storybook host registers the addon
- **WHEN** a compatible Storybook host imports the Inspector addon entrypoint
- **THEN** it SHALL receive a stable addon identifier and panel component suitable for registration
