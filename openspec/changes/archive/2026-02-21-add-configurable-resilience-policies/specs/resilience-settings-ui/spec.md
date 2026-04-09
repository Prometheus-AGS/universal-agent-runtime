## ADDED Requirements

### Requirement: Resilience Settings UI MUST Explain Operational Tradeoffs

The settings UI MUST provide clear helper text and grouping so users can understand the purpose and impact of each resilience parameter.

#### Scenario: Global settings present clear grouping
- **WHEN** a user opens the global Resilience settings panel
- **THEN** controls MUST be grouped into logical sections (rate limiting, timeouts, retries)
- **AND** each control MUST include concise explanatory helper text.

#### Scenario: Advanced options are progressively disclosed
- **WHEN** a user opens the global Resilience settings panel
- **THEN** advanced retry controls MUST be available
- **AND** they MUST be hidden behind an explicit expand action by default.

### Requirement: UI MUST Support Global and Per-Agent Policy Workflows

The settings UI MUST support both global defaults and per-agent overrides with explicit inheritance state.

#### Scenario: Agent inherits global policy in UI
- **WHEN** an agent is set to `Inherit Global`
- **THEN** override inputs MUST be disabled or hidden
- **AND** the UI MUST show that global policy values are in effect.

#### Scenario: Agent override shows effective values
- **WHEN** an agent is set to `Override`
- **THEN** override inputs MUST be editable
- **AND** the UI MUST show an effective policy preview that combines global + override values.

### Requirement: UI Validation MUST Prevent Invalid Saves

The settings UI MUST validate user input inline and prevent saving invalid resilience configurations.

#### Scenario: Inline validation blocks save
- **WHEN** a user enters an invalid resilience value (for example negative timeout or invalid status code)
- **THEN** the UI MUST display an inline actionable validation message
- **AND** the save action MUST remain disabled until all validation errors are resolved.
