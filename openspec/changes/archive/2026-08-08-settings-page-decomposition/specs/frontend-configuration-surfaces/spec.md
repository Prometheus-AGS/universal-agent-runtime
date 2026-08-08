## ADDED Requirements

### Requirement: Settings composition is decomposed by stable responsibility
The settings feature SHALL keep its route-level page focused on responsive navigation and panel composition. Shared controls, schema-driven rendering, the panel registry, and domain panel implementations MUST reside in named internal settings UI modules, and no resulting settings page or panel module SHALL exceed approximately 600 source lines.

#### Scenario: Settings source structure is inspected
- **WHEN** C-14b completes
- **THEN** the route-level settings page contains navigation and composition rather than every panel implementation
- **AND** each settings page or panel module remains at or below the established size ceiling

#### Scenario: A domain panel is maintained
- **WHEN** a provider, file-processing, resilience, governance, agent, memory, caching, or user-settings panel changes
- **THEN** its implementation resolves from a named domain module without widening the public settings feature root

### Requirement: Settings behavior remains compatible through decomposition
The decomposition SHALL preserve the existing navigation categories and order, namespace keys, default active panel, metadata-based availability, generic namespace fallback, responsive layout, visible controls and copy, loading/error/saved states, validation, save/reload actions, JWT gating, provider/model semantics, and realtime settings updates. It MUST NOT change REST payloads, persistence, authentication, AG-UI/A2UI, entity schemas, or backend contracts.

#### Scenario: Operator selects a settings namespace
- **WHEN** the operator selects any available custom or schema-driven settings item
- **THEN** the same panel, controls, values, and action semantics render as before decomposition

#### Scenario: Settings metadata is unavailable or incomplete
- **WHEN** type metadata is empty or an available namespace has no custom panel
- **THEN** the existing availability and generic-schema fallback behavior remains intact

#### Scenario: User settings authentication is evaluated
- **WHEN** the configured frontend key is or is not a JWT
- **THEN** the user-settings panel preserves its existing gated state and remote save/reload behavior

### Requirement: Settings decomposition has focused structural and composition evidence
C-14b SHALL include automated evidence for the internal module-size contract and focused React evidence for route composition, navigation, availability, and panel resolution. The completed change MUST pass TypeScript, lint, frontend architecture, Flat 2.0, token, full frontend, bundle-budget, strict OpenSpec, and scoped diff-integrity checks without modifying protected backend, submodule, or operator-staged paths.

#### Scenario: A decomposition regression is introduced
- **WHEN** a settings UI module exceeds the size ceiling or composition loses a required navigation/panel contract
- **THEN** a deterministic validation or focused test fails before archive

#### Scenario: C-14b reaches closeout
- **WHEN** all settings UI modules are wired and focused tests pass
- **THEN** consolidated frontend and bundle validation passes and protected paths remain unchanged by C-14b
